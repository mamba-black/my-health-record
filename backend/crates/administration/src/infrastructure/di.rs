use crate::application::event_handlers::handle_user_created_event;
use apalis::layers::WorkerBuilderExt;
use apalis::layers::retry::RetryPolicy;
use apalis::prelude::WorkerBuilder;
use apalis_postgres::{Config, PgPool, PostgresStorage};
use app_core::domain::error::ClickCareError;
use app_core::domain::event::UserCreatedEvent;
use std::env::var;
use tracing::info;

/// Número de reintentos ante un fallo del handler antes de marcar la tarea como fallida.
const MAX_RETRIES: usize = 3;

/// Identificador del worker, visible en la tabla `apalis.workers`.
const WORKER_NAME: &str = "administration-worker";

/// Origen de datos del contexto acotado.
pub enum DBType {
    /// `None` toma la URL de la variable de entorno `PG_URL`.
    Postgres(Option<String>),
}

// ─── DI container ────────────────────────────────────────────────────────────

pub struct DI {
    storage: PostgresStorage<UserCreatedEvent>,
}

// ─── Constructores ───────────────────────────────────────────────────────────

/// Construye el DI del contexto acotado con implementaciones reales.
///
/// Abre un pool propio y exclusivo para la cola de eventos: no comparte conexión
/// ni transacción con los repositorios de entidades de ningún contexto.
pub async fn new(dbtype: DBType) -> Result<DI, ClickCareError> {
    let url = match dbtype {
        DBType::Postgres(Some(url)) => url,
        DBType::Postgres(None) => {
            var("PG_URL").unwrap_or("postgres://user:password@localhost:5432".to_string())
        }
    };

    let pool = PgPool::connect(url.as_str()).await.map_err(|e| {
        ClickCareError::generic(format!(
            "Error en la conexion a la DB de la cola de eventos ({e})"
        ))
    })?;

    // Crea el schema `apalis` si aún no existe (migraciones embebidas del crate).
    PostgresStorage::setup(&pool).await.map_err(|e| {
        ClickCareError::generic(format!("Error al preparar el schema de Apalis ({e})"))
    })?;

    let config = Config::new(UserCreatedEvent::QUEUE);
    Ok(DI {
        storage: PostgresStorage::new_with_config(&pool, &config),
    })
}

impl DI {
    /// Corre el worker que consume [`UserCreatedEvent`] hasta que el proceso termine.
    ///
    /// El worker se construye aquí dentro a propósito: así el tipo genérico que
    /// produce [`WorkerBuilder`] nunca cruza la frontera del crate y quien llama
    /// solo necesita conocer `DI`.
    pub async fn run_worker(self) -> Result<(), ClickCareError> {
        info!(
            "Iniciando '{WORKER_NAME}' sobre la cola '{}'",
            UserCreatedEvent::QUEUE
        );

        WorkerBuilder::new(WORKER_NAME)
            .backend(self.storage)
            .retry(RetryPolicy::retries(MAX_RETRIES))
            .build(handle_user_created_event)
            .run()
            .await
            .map_err(|e| {
                ClickCareError::generic(format!("Error al ejecutar '{WORKER_NAME}' ({e})"))
            })
    }
}
