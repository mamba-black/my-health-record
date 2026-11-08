//! Puertos de persistencia de la agenda.

use crate::domain::appointment::Appointment;
use crate::domain::calendar::Instant;
use crate::domain::error::SchedulingError;
use crate::domain::reminder::{Reminder, ReminderStatus};
use crate::domain::schedule::{Schedule, ScheduleException};
use async_trait::async_trait;
use uuid::Uuid;

/// Comprueba el horario nuevo contra los del mismo médico en otras clínicas de su red.
pub type OverlapCheck<'a> = &'a (dyn Fn(&[Schedule]) -> Result<(), SchedulingError> + Sync);

#[async_trait]
pub trait ScheduleRepository: Send + Sync {
    /// Horario del médico en la clínica.
    async fn find(
        &self,
        organization_id: &Uuid,
        practitioner_user_id: &Uuid,
    ) -> Result<Option<Schedule>, SchedulingError>;

    /// Horarios de una clínica.
    async fn by_clinic(&self, organization_id: &Uuid) -> Result<Vec<Schedule>, SchedulingError>;

    /// Guarda el horario. Bajo un bloqueo por red y médico, entrega a `check` los
    /// horarios del médico en las demás clínicas de su red (nunca de otra red) y solo
    /// guarda si `check` lo acepta.
    async fn save(
        &self,
        schedule: &Schedule,
        check: OverlapCheck<'_>,
    ) -> Result<(), SchedulingError>;

    async fn exceptions(
        &self,
        schedule_id: &Uuid,
    ) -> Result<Vec<ScheduleException>, SchedulingError>;

    async fn add_exception(&self, exception: &ScheduleException) -> Result<(), SchedulingError>;

    /// Borra la excepción si es de ese horario.
    async fn remove_exception(
        &self,
        schedule_id: &Uuid,
        exception_id: &Uuid,
    ) -> Result<bool, SchedulingError>;
}

#[async_trait]
pub trait AppointmentRepository: Send + Sync {
    /// Inserta la cita y su recordatorio juntos. Si el horario del médico ya está
    /// ocupado en su red, responde [`SchedulingError::SlotUnavailable`].
    async fn insert(
        &self,
        appointment: &Appointment,
        reminder: &Reminder,
    ) -> Result<(), SchedulingError>;

    /// Cita por id, solo dentro de la red.
    async fn find(
        &self,
        network_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<Appointment>, SchedulingError>;

    /// Guarda el cambio si nadie la cambió antes (`version`). Si el nuevo horario ya
    /// está ocupado, responde [`SchedulingError::SlotUnavailable`] y nada cambia.
    async fn update(&self, appointment: &Appointment) -> Result<(), SchedulingError>;

    /// Horarios ocupados del médico en toda su red entre `from` y `to`.
    async fn busy(
        &self,
        network_id: &Uuid,
        practitioner_user_id: &Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<(Instant, Instant)>, SchedulingError>;

    /// Citas activas del médico en la clínica que se cruzan con el rango.
    async fn count_active(
        &self,
        organization_id: &Uuid,
        practitioner_user_id: &Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<usize, SchedulingError>;

    /// Citas de esos registros de paciente, solo dentro de la red.
    async fn by_patients(
        &self,
        network_id: &Uuid,
        patient_ids: &[Uuid],
    ) -> Result<Vec<Appointment>, SchedulingError>;

    /// Citas de una clínica en el rango, opcionalmente de un solo médico.
    async fn by_clinic(
        &self,
        organization_id: &Uuid,
        practitioner_user_id: Option<&Uuid>,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<Appointment>, SchedulingError>;
}

/// Recordatorio reclamado para enviarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedReminder {
    pub reminder: Reminder,
    pub appointment: Appointment,
}

#[async_trait]
pub trait ReminderRepository: Send + Sync {
    /// Recordatorio de la cita.
    async fn of_appointment(
        &self,
        appointment_id: &Uuid,
    ) -> Result<Option<Reminder>, SchedulingError>;

    /// Reprograma (nuevo `due_at`, vuelve a `PENDING`) o anula (`CANCELLED`) el
    /// recordatorio de la cita, salvo que ya se haya enviado.
    async fn reset(
        &self,
        appointment_id: &Uuid,
        due_at: Instant,
        status: ReminderStatus,
    ) -> Result<Option<Reminder>, SchedulingError>;

    /// Reclama el envío: pasa a `SENDING` solo si sigue `PENDING` con ese `due_at`.
    /// `None` si el aviso quedó obsoleto o ya lo tomó otra entrega.
    async fn claim(
        &self,
        id: &Uuid,
        due_at: Instant,
        now: Instant,
    ) -> Result<Option<ClaimedReminder>, SchedulingError>;

    /// Registra el resultado de un envío reclamado.
    async fn finish(
        &self,
        id: &Uuid,
        status: ReminderStatus,
        due_at: Instant,
        error: Option<String>,
        now: Instant,
    ) -> Result<(), SchedulingError>;

    /// Devuelve a `PENDING` los envíos en `SENDING` reclamados antes de `stale_before`
    /// (se cayó el proceso a mitad del envío: antes repetido que perdido).
    async fn release_stale(&self, stale_before: Instant) -> Result<usize, SchedulingError>;

    /// Recordatorios `PENDING` que ya vencieron, para volver a encolarlos.
    async fn pending_due(&self, now: Instant) -> Result<Vec<Reminder>, SchedulingError>;
}
