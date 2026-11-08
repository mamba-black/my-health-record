//! Adaptadores gRPC de la agenda: `PublicSchedulingApi` (sin sesión) y `SchedulingApi`
//! (con el interceptor). Traducen mensajes y errores; las reglas están en `scheduling`.

use crate::infrastructure::grpc;
use crate::infrastructure::grpc::access::{AccessContextBuilder, claims_of, denied_to_status};
use crate::infrastructure::grpc::public_scheduling_api_server::PublicSchedulingApi;
use crate::infrastructure::grpc::scheduling_api_server::SchedulingApi;
use crate::infrastructure::grpc::user_api_impl::clinic_status_to_proto;
use app_core::domain::access::RequestContext;
use scheduling::application::booking_usecase::{AppointmentView, BookingUseCases, PatientChoice};
use scheduling::application::integration::{LinkSuggestion, NewPatientData};
use scheduling::application::reception_usecase::ReceptionUseCases;
use scheduling::application::schedule_usecase::ScheduleUseCases;
use scheduling::domain::appointment::{AppointmentStatus, DocumentCheck};
use scheduling::domain::calendar::{Instant, format_local_time, parse_local_time};
use scheduling::domain::error::SchedulingError;
use scheduling::domain::schedule::{Schedule, ScheduleException, TimeBlock};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// ─── Traducción ─────────────────────────────────────────────────────────────

/// Error de la agenda a estado gRPC (tarea 8.2).
fn to_status(error: SchedulingError, organization_id: Option<Uuid>) -> Status {
    use SchedulingError as E;
    match error {
        E::Denied(denied) => denied_to_status(denied, organization_id),
        E::NotFound(message) => Status::not_found(message),
        E::SlotUnavailable | E::Conflict => Status::aborted(error.to_string()),
        E::Invalid(_) | E::MissingPatientData(_) | E::SlotNotOffered | E::DifferentPractitioner => {
            Status::invalid_argument(error.to_string())
        }
        E::CrossClinicOverlap { .. }
        | E::ExceptionHasAppointments(_)
        | E::AlreadyStarted
        | E::NotStarted
        | E::NotToday
        | E::InvalidState => Status::failed_precondition(error.to_string()),
        E::Unknown(error) => Status::internal(error.to_string()),
    }
}

fn parse_id(raw: &str, label: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(raw.trim())
        .map_err(|_| Status::invalid_argument(format!("{label} no es válido")))
}

fn parse_instant(raw: &str, label: &str) -> Result<Instant, Status> {
    raw.trim()
        .parse::<Instant>()
        .map_err(|_| Status::invalid_argument(format!("{label} debe ser una fecha RFC 3339")))
}

fn blocks_from(blocks: Vec<grpc::TimeBlockInfo>) -> Result<Vec<TimeBlock>, Status> {
    blocks
        .into_iter()
        .map(|block| {
            let parse = |text: &str| {
                parse_local_time(text)
                    .ok_or_else(|| Status::invalid_argument("Las horas deben ser HH:MM"))
            };
            Ok(TimeBlock {
                weekday: u8::try_from(block.weekday)
                    .map_err(|_| Status::invalid_argument("Día de la semana inválido"))?,
                start: parse(&block.start)?,
                end: parse(&block.end)?,
            })
        })
        .collect()
}

fn schedule_response(
    schedule: Option<&Schedule>,
    exceptions: &[ScheduleException],
    timezone: String,
) -> grpc::ScheduleResponse {
    grpc::ScheduleResponse {
        defined: schedule.is_some(),
        slot_minutes: schedule.map_or(30, |s| i32::from(s.slot_minutes)),
        blocks: schedule
            .map(|s| {
                s.blocks
                    .iter()
                    .map(|b| grpc::TimeBlockInfo {
                        weekday: i32::from(b.weekday),
                        start: format_local_time(b.start),
                        end: format_local_time(b.end),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        exceptions: exceptions
            .iter()
            .map(|e| grpc::ScheduleExceptionInfo {
                id: e.id.to_string(),
                start: e.start.to_string(),
                end: e.end.to_string(),
                reason: e.reason.clone(),
            })
            .collect(),
        timezone,
    }
}

fn status_to_proto(status: AppointmentStatus) -> grpc::AppointmentStatus {
    match status {
        AppointmentStatus::Booked => grpc::AppointmentStatus::Booked,
        AppointmentStatus::Arrived => grpc::AppointmentStatus::Arrived,
        AppointmentStatus::Cancelled => grpc::AppointmentStatus::Cancelled,
        AppointmentStatus::NoShow => grpc::AppointmentStatus::Noshow,
    }
}

fn appointment_info(view: AppointmentView) -> grpc::AppointmentInfo {
    let a = view.appointment;
    grpc::AppointmentInfo {
        id: a.id.to_string(),
        organization_id: a.organization_id.to_string(),
        clinic_name: view.clinic_name,
        clinic_subdomain: view.clinic_subdomain,
        practitioner_user_id: a.practitioner_user_id.to_string(),
        practitioner_name: view.practitioner_name,
        patient_id: a.patient_id.to_string(),
        patient_name: view.patient_name,
        start: a.start.to_string(),
        end: a.end.to_string(),
        status: status_to_proto(a.status) as i32,
        document_check: match a.document_check {
            Some(DocumentCheck::Match) => grpc::DocumentCheck::Match,
            Some(DocumentCheck::Mismatch) => grpc::DocumentCheck::Mismatch,
            None => grpc::DocumentCheck::Unspecified,
        } as i32,
    }
}

fn suggestion_info(s: LinkSuggestion) -> grpc::LinkSuggestionInfo {
    grpc::LinkSuggestionInfo {
        user_id: s.user_id.to_string(),
        name: s.name,
        email: s.email,
    }
}

// ─── Público ────────────────────────────────────────────────────────────────

pub struct PublicSchedulingApiImpl {
    schedules: Arc<ScheduleUseCases>,
}

impl PublicSchedulingApiImpl {
    pub fn new(schedules: Arc<ScheduleUseCases>) -> Self {
        Self { schedules }
    }
}

#[tonic::async_trait]
impl PublicSchedulingApi for PublicSchedulingApiImpl {
    async fn get_clinic(
        &self,
        request: Request<grpc::ClinicLinkRequest>,
    ) -> Result<Response<grpc::PublicClinic>, Status> {
        let clinic = self
            .schedules
            .public_clinic(&request.into_inner().subdomain)
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(grpc::PublicClinic {
            organization_id: clinic.id.to_string(),
            name: clinic.name,
            status: clinic_status_to_proto(clinic.status) as i32,
            timezone: clinic
                .zone
                .iana_name()
                .unwrap_or("America/Lima")
                .to_string(),
            address: clinic.address,
        }))
    }

    async fn list_practitioners(
        &self,
        request: Request<grpc::ClinicLinkRequest>,
    ) -> Result<Response<grpc::PublicPractitioners>, Status> {
        let practitioners = self
            .schedules
            .public_practitioners(&request.into_inner().subdomain)
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(grpc::PublicPractitioners {
            practitioners: practitioners
                .into_iter()
                .map(|p| grpc::PublicPractitioner {
                    user_id: p.user_id.to_string(),
                    name: p.name,
                    specialty: p.specialty,
                })
                .collect(),
        }))
    }

    async fn get_availability(
        &self,
        request: Request<grpc::AvailabilityRequest>,
    ) -> Result<Response<grpc::Availability>, Status> {
        let request = request.into_inner();
        let slots = self
            .schedules
            .public_availability(
                &request.subdomain,
                parse_id(&request.practitioner_user_id, "El médico")?,
                parse_instant(&request.from, "Desde")?,
                parse_instant(&request.to, "Hasta")?,
            )
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(grpc::Availability {
            slots: slots
                .into_iter()
                .map(|slot| grpc::SlotInfo {
                    id: slot.id(),
                    start: slot.start.to_string(),
                    end: slot.end.to_string(),
                })
                .collect(),
        }))
    }
}

// ─── Con sesión ─────────────────────────────────────────────────────────────

pub struct SchedulingApiImpl {
    schedules: Arc<ScheduleUseCases>,
    booking: Arc<BookingUseCases>,
    reception: Arc<ReceptionUseCases>,
    access: Arc<AccessContextBuilder>,
}

impl SchedulingApiImpl {
    pub fn new(
        schedules: Arc<ScheduleUseCases>,
        booking: Arc<BookingUseCases>,
        reception: Arc<ReceptionUseCases>,
        access: Arc<AccessContextBuilder>,
    ) -> Self {
        Self {
            schedules,
            booking,
            reception,
            access,
        }
    }

    async fn ctx<T>(&self, request: &Request<T>) -> Result<RequestContext, Status> {
        let claims = claims_of(request)?;
        Ok(self.access.build(&claims).await?.1)
    }

    /// La cita con sus nombres para responder.
    async fn view(
        &self,
        appointment: scheduling::domain::appointment::Appointment,
    ) -> Result<grpc::AppointmentInfo, Status> {
        self.booking
            .describe(appointment)
            .await
            .map(appointment_info)
            .map_err(|e| to_status(e, None))
    }

    async fn schedule_of(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
    ) -> Result<Response<grpc::ScheduleResponse>, Status> {
        let (schedule, exceptions) = self
            .schedules
            .get_schedule(ctx, organization_id)
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        let timezone = self
            .schedules
            .clinic_timezone(organization_id)
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        Ok(Response::new(schedule_response(
            schedule.as_ref(),
            &exceptions,
            timezone,
        )))
    }
}

#[tonic::async_trait]
impl SchedulingApi for SchedulingApiImpl {
    async fn set_weekly_schedule(
        &self,
        request: Request<grpc::SetWeeklyScheduleRequest>,
    ) -> Result<Response<grpc::ScheduleResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let slot_minutes = u16::try_from(request.slot_minutes)
            .map_err(|_| Status::invalid_argument("Duración inválida"))?;
        self.schedules
            .set_weekly_schedule(
                &ctx,
                organization_id,
                slot_minutes,
                blocks_from(request.blocks)?,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        self.schedule_of(&ctx, organization_id).await
    }

    async fn get_schedule(
        &self,
        request: Request<grpc::ClinicIdRequest>,
    ) -> Result<Response<grpc::ScheduleResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let organization_id = parse_id(&request.get_ref().organization_id, "La clínica")?;
        self.schedule_of(&ctx, organization_id).await
    }

    async fn add_schedule_exception(
        &self,
        request: Request<grpc::AddScheduleExceptionRequest>,
    ) -> Result<Response<grpc::ScheduleResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        self.schedules
            .add_exception(
                &ctx,
                organization_id,
                parse_instant(&request.start, "El inicio")?,
                parse_instant(&request.end, "El fin")?,
                request.reason,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        self.schedule_of(&ctx, organization_id).await
    }

    async fn remove_schedule_exception(
        &self,
        request: Request<grpc::RemoveScheduleExceptionRequest>,
    ) -> Result<Response<grpc::ScheduleResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        self.schedules
            .remove_exception(
                &ctx,
                organization_id,
                parse_id(&request.exception_id, "La excepción")?,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        self.schedule_of(&ctx, organization_id).await
    }

    async fn book_appointment(
        &self,
        request: Request<grpc::BookAppointmentRequest>,
    ) -> Result<Response<grpc::AppointmentInfo>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let appointment = self
            .booking
            .book(
                &ctx,
                organization_id,
                parse_id(&request.practitioner_user_id, "El médico")?,
                parse_instant(&request.start, "El inicio")?,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        Ok(Response::new(self.view(appointment).await?))
    }

    async fn book_for_patient(
        &self,
        request: Request<grpc::BookForPatientRequest>,
    ) -> Result<Response<grpc::AppointmentInfo>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let patient = match request.patient {
            Some(grpc::book_for_patient_request::Patient::PatientId(id)) => {
                PatientChoice::Existing(parse_id(&id, "El paciente")?)
            }
            Some(grpc::book_for_patient_request::Patient::NewPatient(p)) => {
                let (document_type, document_value) = match p
                    .identifier
                    .and_then(|identifier| identifier.identifier_type)
                {
                    Some(grpc::identifier::IdentifierType::Dni(v)) => ("DNI", v),
                    Some(grpc::identifier::IdentifierType::Ce(v)) => ("CE", v),
                    Some(grpc::identifier::IdentifierType::Passport(v)) => ("PASSPORT", v),
                    None => ("DNI", String::new()),
                };
                PatientChoice::New(NewPatientData {
                    given_name: p.given_name,
                    family_name: p.family_name,
                    second_family_name: p.second_family_name,
                    document_type: document_type.to_string(),
                    document_value,
                    phone: p.phone,
                    birth_date: p.birth_date,
                })
            }
            None => return Err(Status::invalid_argument("Indica el paciente")),
        };
        let appointment = self
            .booking
            .book_for_patient(
                &ctx,
                organization_id,
                parse_id(&request.practitioner_user_id, "El médico")?,
                parse_instant(&request.start, "El inicio")?,
                patient,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        Ok(Response::new(self.view(appointment).await?))
    }

    async fn list_my_appointments(
        &self,
        request: Request<grpc::ListMyAppointmentsRequest>,
    ) -> Result<Response<grpc::AppointmentList>, Status> {
        let ctx = self.ctx(&request).await?;
        let views = self
            .booking
            .my_appointments(&ctx)
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(grpc::AppointmentList {
            appointments: views.into_iter().map(appointment_info).collect(),
        }))
    }

    async fn list_clinic_appointments(
        &self,
        request: Request<grpc::ListClinicAppointmentsRequest>,
    ) -> Result<Response<grpc::AppointmentList>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let views = self
            .booking
            .clinic_appointments(
                &ctx,
                organization_id,
                parse_instant(&request.from, "Desde")?,
                parse_instant(&request.to, "Hasta")?,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        Ok(Response::new(grpc::AppointmentList {
            appointments: views.into_iter().map(appointment_info).collect(),
        }))
    }

    async fn cancel_appointment(
        &self,
        request: Request<grpc::AppointmentIdRequest>,
    ) -> Result<Response<grpc::AppointmentInfo>, Status> {
        let ctx = self.ctx(&request).await?;
        let id = parse_id(&request.get_ref().appointment_id, "La cita")?;
        let appointment = self
            .booking
            .cancel(&ctx, id)
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(self.view(appointment).await?))
    }

    async fn reschedule_appointment(
        &self,
        request: Request<grpc::RescheduleAppointmentRequest>,
    ) -> Result<Response<grpc::AppointmentInfo>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let appointment = self
            .booking
            .reschedule(
                &ctx,
                parse_id(&request.appointment_id, "La cita")?,
                parse_id(&request.practitioner_user_id, "El médico")?,
                parse_instant(&request.start, "El inicio")?,
            )
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(self.view(appointment).await?))
    }

    async fn check_in(
        &self,
        request: Request<grpc::CheckInRequest>,
    ) -> Result<Response<grpc::CheckInResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let check = match grpc::DocumentCheck::try_from(request.document_check) {
            Ok(grpc::DocumentCheck::Match) => Some(DocumentCheck::Match),
            Ok(grpc::DocumentCheck::Mismatch) => Some(DocumentCheck::Mismatch),
            _ => None,
        };
        let result = self
            .reception
            .check_in(&ctx, parse_id(&request.appointment_id, "La cita")?, check)
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(grpc::CheckInResponse {
            appointment: Some(self.view(result.appointment).await?),
            account_verified: result.account_verified,
            link_suggestions: result
                .link_suggestions
                .into_iter()
                .map(suggestion_info)
                .collect(),
        }))
    }

    async fn mark_no_show(
        &self,
        request: Request<grpc::AppointmentIdRequest>,
    ) -> Result<Response<grpc::AppointmentInfo>, Status> {
        let ctx = self.ctx(&request).await?;
        let appointment = self
            .reception
            .mark_no_show(
                &ctx,
                parse_id(&request.get_ref().appointment_id, "La cita")?,
            )
            .await
            .map_err(|e| to_status(e, None))?;
        Ok(Response::new(self.view(appointment).await?))
    }

    async fn suggest_patient_links(
        &self,
        request: Request<grpc::PatientLinkRequest>,
    ) -> Result<Response<grpc::LinkSuggestions>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let suggestions = self
            .reception
            .suggest_links(
                &ctx,
                organization_id,
                parse_id(&request.patient_id, "El paciente")?,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        Ok(Response::new(grpc::LinkSuggestions {
            suggestions: suggestions.into_iter().map(suggestion_info).collect(),
        }))
    }

    async fn link_patient_account(
        &self,
        request: Request<grpc::LinkPatientAccountRequest>,
    ) -> Result<Response<grpc::PatientLinkResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let user_id = parse_id(&request.user_id, "La cuenta")?;
        let link_id = self
            .reception
            .link_account(
                &ctx,
                organization_id,
                parse_id(&request.patient_id, "El paciente")?,
                user_id,
                request.document_seen,
            )
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        self.access.invalidate(&user_id);
        Ok(Response::new(grpc::PatientLinkResponse {
            link_id: link_id.to_string(),
        }))
    }

    async fn unlink_patient_account(
        &self,
        request: Request<grpc::UnlinkPatientAccountRequest>,
    ) -> Result<Response<grpc::PatientLinkResponse>, Status> {
        let ctx = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let link_id = parse_id(&request.link_id, "El vínculo")?;
        self.reception
            .unlink_account(&ctx, organization_id, link_id, &request.reason)
            .await
            .map_err(|e| to_status(e, Some(organization_id)))?;
        Ok(Response::new(grpc::PatientLinkResponse {
            link_id: link_id.to_string(),
        }))
    }
}
