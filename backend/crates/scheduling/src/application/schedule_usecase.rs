//! Horario del médico (5.1) y disponibilidad pública de una clínica (5.2).

use crate::application::integration::{ClinicInfo, PractitionerInfo};
use crate::application::state::{SchedulingState, allow};
use crate::domain::calendar::{Instant, format_local_time};
use crate::domain::error::SchedulingError;
use crate::domain::schedule::{
    Availability, Schedule, ScheduleException, Slot, TimeBlock, weekday_name,
};
use app_core::domain::access::{ClinicStatus, ProtectedAction, RequestContext};
use tracing::info;
use uuid::Uuid;

/// Casos de uso del horario y de la disponibilidad.
pub struct ScheduleUseCases {
    state: SchedulingState,
}

impl ScheduleUseCases {
    pub fn new(state: SchedulingState) -> Self {
        Self { state }
    }

    /// Guarda el horario semanal del médico de la sesión en la clínica. Rechaza
    /// franjas que se cruzan con su horario en otra clínica de la misma red.
    pub async fn set_weekly_schedule(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        slot_minutes: u16,
        blocks: Vec<TimeBlock>,
    ) -> Result<Schedule, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::ManageSchedule, ctx, &clinic)?;
        let id = self
            .state
            .schedules
            .find(&organization_id, &ctx.user_id)
            .await?
            .map_or_else(Uuid::now_v7, |existing| existing.id);
        let schedule = Schedule::new(
            id,
            clinic.network_id,
            organization_id,
            ctx.user_id,
            slot_minutes,
            blocks,
        )?;
        let check = |others: &[Schedule]| {
            for other in others {
                if let Some((weekday, start, end)) = schedule.first_overlap_with(other) {
                    return Err(SchedulingError::CrossClinicOverlap {
                        weekday: weekday_name(weekday).to_string(),
                        start: format_local_time(start),
                        end: format_local_time(end),
                    });
                }
            }
            Ok(())
        };
        self.state.schedules.save(&schedule, &check).await?;
        info!(
            "Horario guardado: médico={} clínica={organization_id} franjas={}",
            ctx.user_id,
            schedule.blocks.len()
        );
        Ok(schedule)
    }

    /// Horario y excepciones del médico de la sesión en la clínica.
    pub async fn get_schedule(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
    ) -> Result<(Option<Schedule>, Vec<ScheduleException>), SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::ViewClinicAgenda, ctx, &clinic)?;
        let schedule = self
            .state
            .schedules
            .find(&organization_id, &ctx.user_id)
            .await?;
        let exceptions = match &schedule {
            Some(schedule) => self.state.schedules.exceptions(&schedule.id).await?,
            None => Vec::new(),
        };
        Ok((schedule, exceptions))
    }

    /// Registra una ausencia. Se rechaza si se cruza con citas activas.
    pub async fn add_exception(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        start: Instant,
        end: Instant,
        reason: Option<String>,
    ) -> Result<ScheduleException, SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::ManageSchedule, ctx, &clinic)?;
        let schedule = self.own_schedule(ctx, &organization_id).await?;
        let exception = ScheduleException::new(Uuid::now_v7(), schedule.id, start, end, reason)?;
        let busy = self
            .state
            .appointments
            .count_active(&organization_id, &ctx.user_id, start, end)
            .await?;
        if busy > 0 {
            return Err(SchedulingError::ExceptionHasAppointments(busy));
        }
        self.state.schedules.add_exception(&exception).await?;
        Ok(exception)
    }

    pub async fn remove_exception(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        exception_id: Uuid,
    ) -> Result<(), SchedulingError> {
        let clinic = self.state.clinic_in_network(ctx, &organization_id).await?;
        allow(ProtectedAction::ManageSchedule, ctx, &clinic)?;
        let schedule = self.own_schedule(ctx, &organization_id).await?;
        if self
            .state
            .schedules
            .remove_exception(&schedule.id, &exception_id)
            .await?
        {
            Ok(())
        } else {
            Err(SchedulingError::NotFound(
                "La excepción no existe".to_string(),
            ))
        }
    }

    async fn own_schedule(
        &self,
        ctx: &RequestContext,
        organization_id: &Uuid,
    ) -> Result<Schedule, SchedulingError> {
        self.state
            .schedules
            .find(organization_id, &ctx.user_id)
            .await?
            .ok_or_else(|| SchedulingError::NotFound("Aún no defines tu horario".to_string()))
    }

    /// Zona horaria IANA de la clínica.
    pub async fn clinic_timezone(&self, organization_id: Uuid) -> Result<String, SchedulingError> {
        let clinic = self
            .state
            .clinics
            .by_id(&organization_id)
            .await?
            .ok_or_else(|| SchedulingError::NotFound("La clínica no existe".to_string()))?;
        Ok(clinic
            .zone
            .iana_name()
            .unwrap_or("America/Lima")
            .to_string())
    }

    // ─── Público ───────────────────────────────────────────────────────────

    /// Clínica por su enlace, en cualquier estado (la página dice si aún no atiende).
    pub async fn public_clinic(&self, subdomain: &str) -> Result<ClinicInfo, SchedulingError> {
        self.state
            .clinics
            .by_subdomain(subdomain)
            .await?
            .ok_or_else(|| SchedulingError::NotFound("La clínica no existe".to_string()))
    }

    /// Médicos activos con horario de una clínica activa. Una clínica pendiente o
    /// suspendida no muestra a nadie.
    pub async fn public_practitioners(
        &self,
        subdomain: &str,
    ) -> Result<Vec<PractitionerInfo>, SchedulingError> {
        let clinic = self.public_clinic(subdomain).await?;
        if clinic.status != ClinicStatus::Active {
            return Ok(Vec::new());
        }
        let with_schedule: Vec<Uuid> = self
            .state
            .schedules
            .by_clinic(&clinic.id)
            .await?
            .into_iter()
            .filter(|schedule| !schedule.blocks.is_empty())
            .map(|schedule| schedule.practitioner_user_id)
            .collect();
        Ok(self
            .state
            .clinics
            .active_practitioners(&clinic.id)
            .await?
            .into_iter()
            .filter(|practitioner| with_schedule.contains(&practitioner.user_id))
            .collect())
    }

    /// Cupos libres de un médico activo de una clínica activa.
    pub async fn public_availability(
        &self,
        subdomain: &str,
        practitioner_user_id: Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<Slot>, SchedulingError> {
        let clinic = self.public_clinic(subdomain).await?;
        if clinic.status != ClinicStatus::Active {
            return Ok(Vec::new());
        }
        let active = self
            .state
            .clinics
            .active_practitioners(&clinic.id)
            .await?
            .iter()
            .any(|p| p.user_id == practitioner_user_id);
        if !active {
            return Ok(Vec::new());
        }
        let Some(schedule) = self
            .state
            .schedules
            .find(&clinic.id, &practitioner_user_id)
            .await?
        else {
            return Ok(Vec::new());
        };
        let exceptions = self.state.schedules.exceptions(&schedule.id).await?;
        let busy = self
            .state
            .appointments
            .busy(&clinic.network_id, &practitioner_user_id, from, to)
            .await?;
        Ok(Availability {
            schedule: &schedule,
            zone: &clinic.zone,
            exceptions: &exceptions,
            busy: &busy,
            now: self.state.clock.now(),
            window_days: self.state.config.booking_window_days,
        }
        .free_slots(from, to))
    }
}
