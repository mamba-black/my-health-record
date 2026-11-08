//! Dependencias de los casos de uso de la agenda y helpers de autorización.

use crate::application::integration::{
    ClinicDirectory, ClinicInfo, Clock, IdentityVerification, NotificationSender, PatientRegistry,
    ReminderScheduler,
};
use crate::domain::error::SchedulingError;
use crate::domain::reminder::{
    DEFAULT_LEAD_HOURS, DEFAULT_MAX_ATTEMPTS, DEFAULT_SENDING_TIMEOUT_MINUTES,
};
use crate::domain::repository::{AppointmentRepository, ReminderRepository, ScheduleRepository};
use crate::domain::schedule::BOOKING_WINDOW_DAYS;
use app_core::domain::access::{
    AccessDenied, AccessTarget, MembershipRole, MembershipStatus, ProtectedAction, RequestContext,
    authorize,
};
use std::sync::Arc;
use uuid::Uuid;

/// Parámetros de la agenda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedulingConfig {
    /// Anticipación del recordatorio (`REMINDER_LEAD_HOURS`).
    pub reminder_lead_hours: i64,
    /// Intentos de envío (`REMINDER_MAX_ATTEMPTS`).
    pub reminder_max_attempts: i16,
    /// Minutos tras los que un envío en curso se da por caído (`REMINDER_SENDING_TIMEOUT`).
    pub reminder_sending_timeout_minutes: i64,
    /// Ventana de reserva en días.
    pub booking_window_days: i64,
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            reminder_lead_hours: DEFAULT_LEAD_HOURS,
            reminder_max_attempts: DEFAULT_MAX_ATTEMPTS,
            reminder_sending_timeout_minutes: DEFAULT_SENDING_TIMEOUT_MINUTES,
            booking_window_days: BOOKING_WINDOW_DAYS,
        }
    }
}

/// Puertos que necesitan los casos de uso.
#[derive(Clone)]
pub struct SchedulingState {
    pub schedules: Arc<dyn ScheduleRepository>,
    pub appointments: Arc<dyn AppointmentRepository>,
    pub reminders: Arc<dyn ReminderRepository>,
    pub clinics: Arc<dyn ClinicDirectory>,
    pub patients: Arc<dyn PatientRegistry>,
    pub identity: Arc<dyn IdentityVerification>,
    pub notifier: Arc<dyn NotificationSender>,
    pub scheduler: Arc<dyn ReminderScheduler>,
    pub clock: Arc<dyn Clock>,
    pub config: SchedulingConfig,
}

impl SchedulingState {
    /// Clínica por id, solo si es de la red de quien llama (lo de otra red no existe).
    pub(crate) async fn clinic_in_network(
        &self,
        ctx: &RequestContext,
        organization_id: &Uuid,
    ) -> Result<ClinicInfo, SchedulingError> {
        self.clinics
            .by_id(organization_id)
            .await?
            .filter(|clinic| clinic.network_id == ctx.network_id)
            .ok_or_else(|| SchedulingError::NotFound("La clínica no existe".to_string()))
    }
}

/// Objetivo de la regla de acceso para una clínica.
pub(crate) fn target(clinic: &ClinicInfo) -> AccessTarget {
    AccessTarget {
        organization_id: Some(clinic.id),
        clinic_status: Some(clinic.status),
        ..AccessTarget::default()
    }
}

/// Autoriza la acción sobre la clínica.
pub(crate) fn allow(
    action: ProtectedAction,
    ctx: &RequestContext,
    clinic: &ClinicInfo,
) -> Result<(), SchedulingError> {
    authorize(action, ctx, &target(clinic)).map_err(SchedulingError::from)
}

/// Tiene alguna membresía activa en la clínica.
pub(crate) fn is_member(ctx: &RequestContext, organization_id: &Uuid) -> bool {
    ctx.memberships
        .iter()
        .any(|m| m.organization_id == *organization_id && m.status == MembershipStatus::Active)
}

/// Actúa como recepción en la clínica (dueño, administrador o recepcionista activos).
/// Si no, y es médico, solo actúa sobre su propia agenda.
pub(crate) fn is_reception(ctx: &RequestContext, organization_id: &Uuid) -> bool {
    ctx.memberships.iter().any(|m| {
        m.organization_id == *organization_id
            && m.status == MembershipStatus::Active
            && matches!(
                m.role,
                MembershipRole::Owner | MembershipRole::Admin | MembershipRole::Receptionist
            )
    })
}

/// El médico sin rol de recepción solo actúa sobre su propia agenda.
pub(crate) fn own_agenda_only(
    ctx: &RequestContext,
    organization_id: &Uuid,
    practitioner_user_id: &Uuid,
) -> Result<(), SchedulingError> {
    if is_reception(ctx, organization_id) || ctx.user_id == *practitioner_user_id {
        Ok(())
    } else {
        Err(SchedulingError::Denied(AccessDenied::Forbidden))
    }
}
