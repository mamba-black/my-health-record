//! Reservar (5.3, 5.4), ver (5.5), cancelar y reprogramar (5.6) citas.

use crate::application::integration::{ClinicInfo, NewPatientData, PatientRef};
use crate::application::state::{SchedulingState, allow, is_member, own_agenda_only};
use crate::domain::appointment::Appointment;
use crate::domain::calendar::Instant;
use crate::domain::error::SchedulingError;
use crate::domain::reminder::{Reminder, ReminderStatus, due_at};
use crate::domain::schedule::Availability;
use app_core::domain::access::{AccessDenied, ProtectedAction, RequestContext};
use tracing::{info, warn};
use uuid::Uuid;

/// Paciente de una reserva hecha por la clínica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatientChoice {
    /// Un registro de la clínica.
    Existing(Uuid),
    /// Un paciente nuevo, aunque no tenga cuenta.
    New(NewPatientData),
}

/// Cita con los nombres que se muestran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppointmentView {
    pub appointment: Appointment,
    pub clinic_name: String,
    pub clinic_subdomain: String,
    pub practitioner_name: String,
    pub patient_name: Option<String>,
}

/// Casos de uso de las citas.
pub struct BookingUseCases {
    state: SchedulingState,
}

impl BookingUseCases {
    pub fn new(state: SchedulingState) -> Self {
        Self { state }
    }

    /// El paciente reserva para sí mismo. Exige sus datos (`ConfirmAppointment`): si
    /// faltan, la regla responde cuáles y el frontend abre «Completa tus datos».
    pub async fn book(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        practitioner_user_id: Uuid,
        start: Instant,
    ) -> Result<Appointment, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::ConfirmAppointment, ctx, &clinic)?;
        let slot = self
            .offered_slot(&clinic, practitioner_user_id, start)
            .await?;
        let patient = self
            .state
            .patients
            .ensure_for_account(&organization_id, &ctx.user_id)
            .await?;
        self.create(&clinic, practitioner_user_id, &patient, slot, ctx.user_id)
            .await
    }

    /// La clínica reserva en nombre de un paciente. El médico solo en su agenda.
    pub async fn book_for_patient(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        practitioner_user_id: Uuid,
        start: Instant,
        patient: PatientChoice,
    ) -> Result<Appointment, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::BookForPatient, ctx, &clinic)?;
        own_agenda_only(ctx, &organization_id, &practitioner_user_id)?;
        let slot = self
            .offered_slot(&clinic, practitioner_user_id, start)
            .await?;
        let patient = match patient {
            PatientChoice::Existing(id) => {
                self.state
                    .patients
                    .find(&organization_id, &id)
                    .await?
                    .ok_or_else(|| SchedulingError::NotFound("El paciente no existe".to_string()))?
            }
            PatientChoice::New(data) => {
                self.state.patients.register(&organization_id, data).await?
            }
        };
        self.create(&clinic, practitioner_user_id, &patient, slot, ctx.user_id)
            .await
    }

    async fn offered_slot(
        &self,
        clinic: &ClinicInfo,
        practitioner_user_id: Uuid,
        start: Instant,
    ) -> Result<(Instant, Instant), SchedulingError> {
        let active = self
            .state
            .clinics
            .active_practitioners(&clinic.id)
            .await?
            .iter()
            .any(|p| p.user_id == practitioner_user_id);
        let schedule = self
            .state
            .schedules
            .find(&clinic.id, &practitioner_user_id)
            .await?
            .filter(|_| active)
            .ok_or_else(|| {
                SchedulingError::NotFound("El médico no atiende en esta clínica".to_string())
            })?;
        let exceptions = self.state.schedules.exceptions(&schedule.id).await?;
        Availability {
            schedule: &schedule,
            zone: &clinic.zone,
            exceptions: &exceptions,
            busy: &[],
            now: self.state.clock.now(),
            window_days: self.state.config.booking_window_days,
        }
        .offered_slot(start)
    }

    async fn create(
        &self,
        clinic: &ClinicInfo,
        practitioner_user_id: Uuid,
        patient: &PatientRef,
        slot: (Instant, Instant),
        booked_by: Uuid,
    ) -> Result<Appointment, SchedulingError> {
        let now = self.state.clock.now();
        let appointment = Appointment::book(
            clinic.network_id,
            clinic.id,
            practitioner_user_id,
            patient.id,
            slot,
            booked_by,
        );
        let reminder = Reminder::for_appointment(
            clinic.network_id,
            appointment.id,
            appointment.start,
            self.state.config.reminder_lead_hours,
            now,
        );
        self.state
            .appointments
            .insert(&appointment, &reminder)
            .await?;
        self.wake(&reminder.id, reminder.due_at).await;
        info!(
            "Cita reservada: id={} clínica={} médico={practitioner_user_id}",
            appointment.id, clinic.id
        );
        Ok(appointment)
    }

    /// Programa el recordatorio. Si falla, la cita sigue en pie: el barrido del
    /// worker lo vuelve a encolar.
    async fn wake(&self, reminder_id: &Uuid, due_at: Instant) {
        if let Err(error) = self.state.scheduler.schedule(reminder_id, due_at).await {
            warn!("No se pudo programar el recordatorio {reminder_id}: {error}");
        }
    }

    // ─── Consultas ─────────────────────────────────────────────────────────

    /// Citas del paciente de la sesión en todas las clínicas de su red.
    pub async fn my_appointments(
        &self,
        ctx: &RequestContext,
    ) -> Result<Vec<AppointmentView>, SchedulingError> {
        let records = self
            .state
            .patients
            .records_of_account(&ctx.network_id, &ctx.user_id)
            .await?;
        let appointments = self
            .state
            .appointments
            .by_patients(&ctx.network_id, &records)
            .await?;
        self.views(appointments, false).await
    }

    /// Agenda de la clínica en un rango. El médico sin rol de recepción ve solo la suya.
    pub async fn clinic_appointments(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<AppointmentView>, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::ViewClinicAgenda, ctx, &clinic)?;
        let only = (!crate::application::state::is_reception(ctx, &organization_id))
            .then_some(ctx.user_id);
        let appointments = self
            .state
            .appointments
            .by_clinic(&organization_id, only.as_ref(), from, to)
            .await?;
        self.views(appointments, true).await
    }

    /// La cita con sus nombres (clínica, médico y paciente).
    pub async fn describe(
        &self,
        appointment: Appointment,
    ) -> Result<AppointmentView, SchedulingError> {
        self.views(vec![appointment], true)
            .await?
            .pop()
            .ok_or_else(|| SchedulingError::NotFound("La cita no existe".to_string()))
    }

    async fn views(
        &self,
        appointments: Vec<Appointment>,
        with_patient: bool,
    ) -> Result<Vec<AppointmentView>, SchedulingError> {
        let mut views = Vec::with_capacity(appointments.len());
        for appointment in appointments {
            let clinic = self
                .state
                .clinics
                .by_id(&appointment.organization_id)
                .await?;
            let practitioner_name = self
                .state
                .clinics
                .person_name(&appointment.practitioner_user_id)
                .await?
                .unwrap_or_default();
            let patient_name = if with_patient {
                self.state
                    .patients
                    .find(&appointment.organization_id, &appointment.patient_id)
                    .await?
                    .map(|patient| patient.name)
            } else {
                None
            };
            views.push(AppointmentView {
                clinic_name: clinic.as_ref().map(|c| c.name.clone()).unwrap_or_default(),
                clinic_subdomain: clinic.map(|c| c.subdomain).unwrap_or_default(),
                practitioner_name,
                patient_name,
                appointment,
            });
        }
        Ok(views)
    }

    // ─── Cambios ───────────────────────────────────────────────────────────

    /// Cita sobre la que actúa quien llama: suya como paciente, o de su clínica con
    /// `ManageAppointment` (el médico, solo de su agenda). Lo ajeno no existe.
    pub(crate) async fn actionable(
        &self,
        ctx: &RequestContext,
        appointment_id: Uuid,
    ) -> Result<(Appointment, ClinicInfo), SchedulingError> {
        let not_found = || SchedulingError::NotFound("La cita no existe".to_string());
        let appointment = self
            .state
            .appointments
            .find(&ctx.network_id, &appointment_id)
            .await?
            .ok_or_else(not_found)?;
        let clinic = self
            .state
            .clinic_in_network(ctx, &appointment.organization_id)
            .await?;
        let mine = self
            .state
            .patients
            .records_of_account(&ctx.network_id, &ctx.user_id)
            .await?
            .contains(&appointment.patient_id);
        if mine {
            return Ok((appointment, clinic));
        }
        if !is_member(ctx, &clinic.id) {
            return Err(not_found());
        }
        allow(ProtectedAction::ManageAppointment, ctx, &clinic)?;
        own_agenda_only(ctx, &clinic.id, &appointment.practitioner_user_id)?;
        Ok((appointment, clinic))
    }

    /// Cancela la cita y anula su recordatorio. El cupo vuelve a ofrecerse.
    pub async fn cancel(
        &self,
        ctx: &RequestContext,
        appointment_id: Uuid,
    ) -> Result<Appointment, SchedulingError> {
        let (mut appointment, _) = self.actionable(ctx, appointment_id).await?;
        let now = self.state.clock.now();
        appointment.cancel(ctx.user_id, now)?;
        self.state.appointments.update(&appointment).await?;
        appointment.version += 1;
        self.state
            .reminders
            .reset(&appointment.id, now, ReminderStatus::Cancelled)
            .await?;
        Ok(appointment)
    }

    /// Mueve la cita a otro cupo libre del mismo médico, en un solo paso: si el cupo
    /// nuevo ya está tomado, la cita conserva el suyo.
    pub async fn reschedule(
        &self,
        ctx: &RequestContext,
        appointment_id: Uuid,
        practitioner_user_id: Uuid,
        start: Instant,
    ) -> Result<Appointment, SchedulingError> {
        let (mut appointment, clinic) = self.actionable(ctx, appointment_id).await?;
        if practitioner_user_id != appointment.practitioner_user_id {
            return Err(SchedulingError::DifferentPractitioner);
        }
        let slot = self
            .offered_slot(&clinic, practitioner_user_id, start)
            .await?;
        let now = self.state.clock.now();
        appointment.reschedule(practitioner_user_id, clinic.id, slot, now)?;
        self.state.appointments.update(&appointment).await?;
        appointment.version += 1;
        let due = due_at(
            appointment.start,
            self.state.config.reminder_lead_hours,
            now,
        );
        if let Some(reminder) = self
            .state
            .reminders
            .reset(&appointment.id, due, ReminderStatus::Pending)
            .await?
        {
            self.wake(&reminder.id, reminder.due_at).await;
        }
        Ok(appointment)
    }
}

/// Un rechazo por permisos sobre lo ajeno se informa como inexistente.
pub fn hide_forbidden(error: SchedulingError, message: &str) -> SchedulingError {
    match error {
        SchedulingError::Denied(AccessDenied::Forbidden) => {
            SchedulingError::NotFound(message.to_string())
        }
        other => other,
    }
}
