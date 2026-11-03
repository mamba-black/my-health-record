use crate::domain::error::ClickCareError;
use crate::domain::event::{FounderRegistered, UserCreatedEvent};
use async_trait::async_trait;

/// Puerto de salida para publicar eventos de integración hacia otros microservicios.
///
/// Es el publicador externo de `docs/arquitectura_cebolla.md`: el contrato vive en
/// `application/integration/event`, y su implementación técnica (Apalis, Kafka, log) en `infrastructure`.
/// El publicador interno de dominio no pasa por aquí.
#[async_trait]
pub trait IntegrationEventPublisher: Send + Sync {
    /// Publica el evento de creación de usuario hacia la cola de integración.
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), ClickCareError>;

    /// Publica el evento de registro de fundador hacia la cola de integración.
    async fn publish_founder_registered(
        &self,
        event: FounderRegistered,
    ) -> Result<(), ClickCareError> {
        let _ = event;
        Ok(())
    }
}
