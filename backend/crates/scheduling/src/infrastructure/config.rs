//! Conexión y repositorios del contexto de agenda.

use crate::domain::error::SchedulingError;
use crate::domain::repository::{AppointmentRepository, ReminderRepository, ScheduleRepository};
use crate::infrastructure::repository::{
    PgAppointmentRepository, PgReminderRepository, PgScheduleRepository,
};
use app_core::domain::error::ClickCareError;
use std::sync::Arc;

/// Repositorios de la agenda sobre Postgres.
#[derive(Clone)]
pub struct Repositories {
    pub schedules: Arc<dyn ScheduleRepository>,
    pub appointments: Arc<dyn AppointmentRepository>,
    pub reminders: Arc<dyn ReminderRepository>,
}

/// Abre su propia conexión: la agenda no comparte pool con otros contextos.
pub async fn repositories(database_url: &str) -> Result<Repositories, SchedulingError> {
    let db = toasty::Db::builder()
        .models(toasty::models!())
        .connect(database_url)
        .await
        .map_err(|error| {
            SchedulingError::Unknown(ClickCareError::generic(format!(
                "Error en la conexión de la agenda a la base ({error})"
            )))
        })?;
    Ok(Repositories {
        schedules: Arc::new(PgScheduleRepository { db: db.clone() }),
        appointments: Arc::new(PgAppointmentRepository { db: db.clone() }),
        reminders: Arc::new(PgReminderRepository { db }),
    })
}
