//! Cola de recordatorios sobre apalis: el despertador de cada aviso y su worker.
//!
//! La tabla `scheduling.reminder` es la fuente de verdad; la cola solo despierta al
//! worker en `due_at`. Una entrega repetida no envía dos veces (reclamo en la tabla).

use crate::application::integration::ReminderScheduler;
use crate::application::reminder_usecase::ReminderUseCases;
use crate::domain::calendar::Instant;
use crate::domain::error::SchedulingError;
use apalis::layers::WorkerBuilderExt;
use apalis::layers::retry::RetryPolicy;
use apalis::prelude::{Data, TaskBuilder, TaskSink, WorkerBuilder};
use apalis_postgres::{Config, PgPool, PostgresStorage};
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::info;
use uuid::Uuid;

/// Cola de los recordatorios.
pub const REMINDER_QUEUE: &str = "scheduling-reminders";
/// Identificador del worker en `apalis.workers`.
const WORKER_NAME: &str = "scheduling-reminders-worker";

/// Mensaje de la cola: qué recordatorio y para qué `due_at` se programó.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderDue {
    pub reminder_id: Uuid,
    /// `due_at` en segundos y nanosegundos Unix: si cambió, el aviso es obsoleto.
    pub due_at_seconds: i64,
    pub due_at_nanos: i32,
}

impl ReminderDue {
    fn due_at(&self) -> Result<Instant, SchedulingError> {
        Instant::new(self.due_at_seconds, self.due_at_nanos).map_err(|error| {
            SchedulingError::Unknown(ClickCareError::generic(format!(
                "due_at inválido en la cola ({error})"
            )))
        })
    }
}

fn queue_error(error: impl std::fmt::Display) -> SchedulingError {
    SchedulingError::Unknown(ClickCareError::generic(format!(
        "Error en la cola de recordatorios ({error})"
    )))
}

/// Abre el almacenamiento de la cola (con su propio pool).
pub async fn reminder_storage(
    database_url: &str,
) -> Result<PostgresStorage<ReminderDue>, SchedulingError> {
    let pool = PgPool::connect(database_url).await.map_err(queue_error)?;
    PostgresStorage::setup(&pool).await.map_err(queue_error)?;
    Ok(PostgresStorage::new_with_config(
        &pool,
        &Config::new(REMINDER_QUEUE),
    ))
}

/// Despertador de recordatorios con `run_at_time`.
pub struct ApalisReminderScheduler {
    pub storage: PostgresStorage<ReminderDue>,
}

#[async_trait]
impl ReminderScheduler for ApalisReminderScheduler {
    async fn schedule(&self, reminder_id: &Uuid, due_at: Instant) -> Result<(), SchedulingError> {
        let at = SystemTime::UNIX_EPOCH
            + Duration::from_secs(u64::try_from(due_at.as_second()).unwrap_or(0));
        let task = TaskBuilder::new(ReminderDue {
            reminder_id: *reminder_id,
            due_at_seconds: due_at.as_second(),
            due_at_nanos: due_at.subsec_nanosecond(),
        })
        .run_at_time(at)
        .build();
        self.storage
            .clone()
            .push_task(task)
            .await
            .map_err(queue_error)
    }
}

/// Corre el worker de recordatorios hasta que el proceso termine.
pub async fn run_reminder_worker(
    storage: PostgresStorage<ReminderDue>,
    use_cases: Arc<ReminderUseCases>,
) -> Result<(), ClickCareError> {
    info!("Iniciando '{WORKER_NAME}' sobre la cola '{REMINDER_QUEUE}'");
    WorkerBuilder::new(WORKER_NAME)
        .backend(storage)
        .data(use_cases)
        .retry(RetryPolicy::retries(3))
        .build(
            |message: ReminderDue, use_cases: Data<Arc<ReminderUseCases>>| async move {
                let due_at = message.due_at()?;
                use_cases
                    .deliver(message.reminder_id, due_at)
                    .await
                    .map(|_| ())
            },
        )
        .run()
        .await
        .map_err(|error| {
            ClickCareError::generic(format!("Error al ejecutar '{WORKER_NAME}' ({error})"))
        })
}
