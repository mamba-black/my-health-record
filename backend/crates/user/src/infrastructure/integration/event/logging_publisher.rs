use app_core::application::integration::event_publisher::IntegrationEventPublisher;
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{DomainEvent, FounderRegistered, UserCreatedEvent};
use async_trait::async_trait;

/// Publicador de integración sin cola: solo registra la emisión en log/tracing.
///
/// Se usa cuando no hay base de datos (`DBType::Mock`) y la cola no existe.
#[derive(Debug, Default)]
pub(crate) struct LoggingIntegrationEventPublisherImpl;

#[async_trait]
impl IntegrationEventPublisher for LoggingIntegrationEventPublisherImpl {
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), ClickCareError> {
        tracing::info!(
            "[LoggingIntegrationEventPublisherImpl] Publicando evento {}: user_id={}",
            event.event_name(),
            event.user_id
        );
        Ok(())
    }

    async fn publish_founder_registered(
        &self,
        event: FounderRegistered,
    ) -> Result<(), ClickCareError> {
        tracing::info!(
            "[LoggingIntegrationEventPublisherImpl] Publicando evento {}: user_id={}, person_id={}, organization_id={}",
            event.event_name(),
            event.user_id,
            event.person_id,
            event.organization_id
        );
        Ok(())
    }
}
