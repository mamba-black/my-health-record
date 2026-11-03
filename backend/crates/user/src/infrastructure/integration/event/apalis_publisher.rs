use apalis::prelude::TaskSink;
use apalis_postgres::{Config, PgPool, PostgresStorage};
use app_core::application::integration::event_publisher::IntegrationEventPublisher;
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{FounderRegistered, UserCreatedEvent};
use app_core::infrastructure::transport::{FounderRegisteredDto, UserCreatedEventDto};
use async_trait::async_trait;
use tracing::debug;

/// Adaptador de salida del puerto [`IntegrationEventPublisher`] respaldado por `apalis-postgres`.
///
/// Encola eventos en colas dedicadas ([`UserCreatedEvent::QUEUE`], [`FounderRegistered::QUEUE`]).
pub(crate) struct ApalisIntegrationEventPublisherImpl {
    storage: PostgresStorage<UserCreatedEventDto>,
    founder_storage: PostgresStorage<FounderRegisteredDto>,
}

impl ApalisIntegrationEventPublisherImpl {
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
            "ApalisIntegrationEventPublisherImpl listo sobre las colas '{}' y '{}'",
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
impl IntegrationEventPublisher for ApalisIntegrationEventPublisherImpl {
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), ClickCareError> {
        let user_id = event.user_id;

        // `push` requiere `&mut self`; `PostgresStorage` es `Clone` y su pool es un
        // `Arc` interno, así que clonar es barato y no abre conexiones nuevas.
        self.storage
            .clone()
            .push(UserCreatedEventDto::from(event))
            .await
            .map_err(|e| {
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

        self.founder_storage
            .clone()
            .push(FounderRegisteredDto::from(event))
            .await
            .map_err(|e| {
                ClickCareError::generic(format!(
                    "Error al encolar FounderRegistered para user_id={user_id} ({e})"
                ))
            })?;

        Ok(())
    }
}
