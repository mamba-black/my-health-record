//! Recepción: llegada y verificación del documento (6.1), inasistencia (6.2) y
//! vínculo de pacientes registrados sin cuenta con su cuenta (6.3 a 6.5).

use crate::application::integration::{ClinicInfo, LinkSuggestion};
use crate::application::state::{SchedulingState, allow, is_member};
use crate::domain::appointment::{Appointment, DocumentCheck};
use crate::domain::error::SchedulingError;
use app_core::domain::access::{ProtectedAction, RequestContext};
use tracing::info;
use uuid::Uuid;

/// Resultado de registrar una llegada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckInResult {
    pub appointment: Appointment,
    /// Se verificó el documento de la cuenta del paciente.
    pub account_verified: bool,
    /// Paciente sin cuenta verificado: cuentas con su documento para vincular.
    pub link_suggestions: Vec<LinkSuggestion>,
}

/// Casos de uso de recepción.
pub struct ReceptionUseCases {
    state: SchedulingState,
}

impl ReceptionUseCases {
    pub fn new(state: SchedulingState) -> Self {
        Self { state }
    }

    /// Cita de la clínica sobre la que actúa recepción. Lo de otra clínica no existe.
    async fn clinic_appointment(
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
        if !is_member(ctx, &appointment.organization_id) {
            return Err(not_found());
        }
        let clinic = self
            .state
            .clinic_in_network(ctx, &appointment.organization_id)
            .await?;
        allow(ProtectedAction::CheckIn, ctx, &clinic)?;
        Ok((appointment, clinic))
    }

    /// Marca la llegada (solo el día de la cita) y, si recepción vio el documento
    /// físico y coincide, lo deja verificado.
    pub async fn check_in(
        &self,
        ctx: &RequestContext,
        appointment_id: Uuid,
        document_check: Option<DocumentCheck>,
    ) -> Result<CheckInResult, SchedulingError> {
        let (mut appointment, clinic) = self.clinic_appointment(ctx, appointment_id).await?;
        let now = self.state.clock.now();
        appointment.check_in(ctx.user_id, now, &clinic.zone, document_check)?;
        self.state.appointments.update(&appointment).await?;
        appointment.version += 1;

        let mut result = CheckInResult {
            appointment,
            account_verified: false,
            link_suggestions: Vec::new(),
        };
        if document_check == Some(DocumentCheck::Match) {
            let patient_id = result.appointment.patient_id;
            self.state
                .patients
                .mark_document_verified(&patient_id, &ctx.user_id)
                .await?;
            match self.state.patients.account_of(&patient_id).await? {
                Some(user_id) => {
                    self.state.identity.verify_document(&user_id).await?;
                    result.account_verified = true;
                }
                None => {
                    result.link_suggestions = self
                        .state
                        .patients
                        .suggest_links(&clinic.network_id, &clinic.id, &patient_id)
                        .await?;
                }
            }
        }
        info!(
            "Llegada registrada: cita={} por user={}",
            result.appointment.id, ctx.user_id
        );
        Ok(result)
    }

    /// Marca «no asistió», solo después de la hora de inicio.
    pub async fn mark_no_show(
        &self,
        ctx: &RequestContext,
        appointment_id: Uuid,
    ) -> Result<Appointment, SchedulingError> {
        let (mut appointment, _) = self.clinic_appointment(ctx, appointment_id).await?;
        appointment.mark_no_show(ctx.user_id, self.state.clock.now())?;
        self.state.appointments.update(&appointment).await?;
        appointment.version += 1;
        Ok(appointment)
    }

    /// Cuentas de la red con el mismo documento que un registro sin cuenta. Solo para
    /// recepción; a nadie más se le revela si existe un registro con ese documento.
    pub async fn suggest_links(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        patient_id: Uuid,
    ) -> Result<Vec<LinkSuggestion>, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::LinkPatientAccount, ctx, &clinic)?;
        let patient = self
            .state
            .patients
            .find(&organization_id, &patient_id)
            .await?
            .ok_or_else(|| SchedulingError::NotFound("El paciente no existe".to_string()))?;
        if patient.user_id.is_some() {
            return Ok(Vec::new());
        }
        self.state
            .patients
            .suggest_links(&clinic.network_id, &organization_id, &patient_id)
            .await
    }

    /// Vincula, en persona y con el documento físico a la vista, un registro sin cuenta
    /// con la cuenta que el paciente creó después. Deja verificado su documento.
    pub async fn link_account(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        patient_id: Uuid,
        user_id: Uuid,
        document_seen: bool,
    ) -> Result<Uuid, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::LinkPatientAccount, ctx, &clinic)?;
        if !document_seen {
            return Err(SchedulingError::Invalid(
                "Confirma que viste el documento físico del paciente".to_string(),
            ));
        }
        let link_id = self
            .state
            .patients
            .link(
                &clinic.network_id,
                &organization_id,
                &patient_id,
                &user_id,
                &ctx.user_id,
            )
            .await?;
        self.state.identity.verify_document(&user_id).await?;
        info!(
            "Paciente vinculado: registro={patient_id} cuenta={user_id} por user={} en {organization_id}",
            ctx.user_id
        );
        Ok(link_id)
    }

    /// El dueño deshace un vínculo equivocado, con su motivo. No mueve citas.
    pub async fn unlink_account(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        link_id: Uuid,
        reason: &str,
    ) -> Result<Uuid, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::UnlinkPatientAccount, ctx, &clinic)?;
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(SchedulingError::Invalid(
                "Indica el motivo para deshacer el vínculo".to_string(),
            ));
        }
        self.state
            .patients
            .unlink(&organization_id, &link_id, &ctx.user_id, reason)
            .await
    }
}
