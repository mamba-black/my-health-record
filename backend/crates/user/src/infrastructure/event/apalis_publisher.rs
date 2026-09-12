use apalis::prelude::TaskSink;
use apalis_postgres::{Config, PgPool, PostgresStorage};
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{EventPublisher, FounderRegistered, UserCreatedEvent};
use async_trait::async_trait;
use tracing::debug;

/// Adaptador de salida del puerto [`EventPublisher`] respaldado por `apalis-postgres`.
///
/// Encola eventos en colas dedicadas ([`UserCreatedEvent::QUEUE`], [`FounderRegistered::QUEUE`]).
pub(crate) struct ApalisEventPublisher {
    storage: PostgresStorage<UserCreatedEvent>,
    founder_storage: PostgresStorage<FounderRegistered>,
}

impl ApalisEventPublisher {
    /// Abre un pool propio, exclusivo de la cola.
    ///
    /// Deliberadamente **no** reutiliza la conexión de los repositorios de
    /// entidades: encolar un evento nunca debe compartir transacción con la
    /// persistencia del agregado.
    pub(crate) async fn new(url: &str) -> Result<Self, ClickCareError> {
        let pool = PgPool::connect(url).await.map_err(|e| {
            ClickCareError::generic(format!(
                "Error en la conexion a la DB de la cola de eventos ({e})"
            ))
        })?;

        // Crea el schema `apalis` si aún no existe (migraciones embebidas del crate).
        PostgresStorage::setup(&pool).await.map_err(|e| {
            ClickCareError::generic(format!("Error al preparar el schema de Apalis ({e})"))
        })?;

        debug!(
            "ApalisEventPublisher listo sobre las colas '{}' y '{}'",
            UserCreatedEvent::QUEUE,
            FounderRegistered::QUEUE
        );

        let config = Config::new(UserCreatedEvent::QUEUE);
        let founder_config = Config::new(FounderRegistered::QUEUE);
        Ok(Self {
            storage: PostgresStorage::new_with_config(&pool, &config),
            founder_storage: PostgresStorage::new_with_config(&pool, &founder_config),
        })
    }
}

#[async_trait]
impl EventPublisher for ApalisEventPublisher {
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), ClickCareError> {
        let user_id = event.user_id;

        // `push` requiere `&mut self`; `PostgresStorage` es `Clone` y su pool es un
        // `Arc` interno, así que clonar es barato y no abre conexiones nuevas.
        self.storage.clone().push(event).await.map_err(|e| {
            ClickCareError::generic(format!(
                "Error al encolar UserCreatedEvent para user_id={user_id} ({e})"
            ))
        })?;

        Ok(())
    }

    async fn publish_founder_registered(
        &self,
        event: FounderRegistered,
    ) -> Result<(), ClickCareError> {
        let user_id = event.user_id;

        self.founder_storage.clone().push(event).await.map_err(|e| {
            ClickCareError::generic(format!(
                "Error al encolar FounderRegistered para user_id={user_id} ({e})"
            ))
        })?;

        Ok(())
    }
}
