//! Envío de recordatorios (7.1): antes repetido que perdido.

use crate::application::state::SchedulingState;
use crate::domain::appointment::Appointment;
use crate::domain::calendar::{Instant, plus_minutes};
use crate::domain::error::SchedulingError;
use crate::domain::reminder::{ReminderMessage, ReminderStatus, retry_delay_minutes};
use tracing::{info, warn};
use uuid::Uuid;

/// Qué pasó con una entrega de la cola.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    Sent,
    NoChannel,
    /// Falló y se reprogramó para otro intento.
    Retrying,
    Failed,
    /// El aviso estaba obsoleto (cancelado, reprogramado) o ya lo tomó otra entrega.
    Skipped,
}

pub struct ReminderUseCases {
    state: SchedulingState,
}

impl ReminderUseCases {
    pub fn new(state: SchedulingState) -> Self {
        Self { state }
    }

    /// Procesa la entrega de la cola para `(reminder_id, due_at)`.
    pub async fn deliver(
        &self,
        reminder_id: Uuid,
        due_at: Instant,
    ) -> Result<Delivery, SchedulingError> {
        let now = self.state.clock.now();
        let Some(claimed) = self
            .state
            .reminders
            .claim(&reminder_id, due_at, now)
            .await?
        else {
            return Ok(Delivery::Skipped);
        };
        let (reminder, appointment) = (claimed.reminder, claimed.appointment);

        let Some(email) = self.email_for(&appointment).await? else {
            self.state
                .reminders
                .finish(
                    &reminder.id,
                    ReminderStatus::NoChannel,
                    reminder.due_at,
                    None,
                    now,
                )
                .await?;
            return Ok(Delivery::NoChannel);
        };
        let message = self.message_for(&appointment).await?;

        match self.state.notifier.send(&email, &message).await {
            Ok(()) => {
                self.state
                    .reminders
                    .finish(
                        &reminder.id,
                        ReminderStatus::Sent,
                        reminder.due_at,
                        None,
                        now,
                    )
                    .await?;
                info!("Recordatorio enviado: id={}", reminder.id);
                Ok(Delivery::Sent)
            }
            Err(error) if reminder.attempts >= self.state.config.reminder_max_attempts => {
                warn!(
                    "Recordatorio {} fallido tras {} intentos",
                    reminder.id, reminder.attempts
                );
                self.state
                    .reminders
                    .finish(
                        &reminder.id,
                        ReminderStatus::Failed,
                        reminder.due_at,
                        Some(error),
                        now,
                    )
                    .await?;
                Ok(Delivery::Failed)
            }
            Err(error) => {
                let retry_at = plus_minutes(now, retry_delay_minutes(reminder.attempts));
                self.state
                    .reminders
                    .finish(
                        &reminder.id,
                        ReminderStatus::Pending,
                        retry_at,
                        Some(error),
                        now,
                    )
                    .await?;
                self.state
                    .scheduler
                    .schedule(&reminder.id, retry_at)
                    .await?;
                Ok(Delivery::Retrying)
            }
        }
    }

    /// Correo de la cuenta del paciente (propia o vinculada), si tiene.
    async fn email_for(
        &self,
        appointment: &Appointment,
    ) -> Result<Option<String>, SchedulingError> {
        let account = self
            .state
            .patients
            .account_of(&appointment.patient_id)
            .await?;
        let email = match account {
            Some(user_id) => self.state.identity.email_of(&user_id).await?,
            None => None,
        };
        Ok(email.filter(|email| !email.trim().is_empty()))
    }

    /// Datos del aviso: sin motivo de consulta ni datos clínicos.
    async fn message_for(
        &self,
        appointment: &Appointment,
    ) -> Result<ReminderMessage, SchedulingError> {
        let clinic = self
            .state
            .clinics
            .by_id(&appointment.organization_id)
            .await?
            .ok_or_else(|| SchedulingError::NotFound("La clínica no existe".to_string()))?;
        let practitioner_name = self
            .state
            .clinics
            .person_name(&appointment.practitioner_user_id)
            .await?
            .unwrap_or_default();
        Ok(ReminderMessage {
            clinic_name: clinic.name,
            clinic_address: clinic.address,
            practitioner_name,
            start: appointment.start,
            zone: clinic.zone,
        })
    }

    /// Barrido al arrancar y periódico: devuelve a `PENDING` los envíos que se
    /// cayeron a mitad (se reenvían) y vuelve a encolar los vencidos.
    pub async fn sweep(&self) -> Result<usize, SchedulingError> {
        let now = self.state.clock.now();
        let stale_before = plus_minutes(now, -self.state.config.reminder_sending_timeout_minutes);
        let released = self.state.reminders.release_stale(stale_before).await?;
        if released > 0 {
            warn!("{released} recordatorios en envío quedaron colgados: se reenviarán");
        }
        let pending = self.state.reminders.pending_due(now).await?;
        for reminder in &pending {
            self.state
                .scheduler
                .schedule(&reminder.id, reminder.due_at)
                .await?;
        }
        Ok(pending.len())
    }
}
