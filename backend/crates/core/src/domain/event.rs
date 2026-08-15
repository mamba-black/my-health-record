use crate::domain::error::ClickCareError;
use crate::domain::fhir::Person;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Trait marcador para todos los eventos de dominio en el sistema.
pub trait DomainEvent: Send + Sync + Debug {
    /// Nombre o identificador único del tipo de evento (ej: "user.created").
    fn event_name(&self) -> &'static str;
}

/// Evento de dominio emitido cuando una nueva cuenta de usuario y persona física es registrada (`crates/user`).
///
/// Este evento es consumido por otros Bounded Contexts (ej. `crates/administration`) de forma asíncrona y reactiva
/// para inicializar la réplica demográfica local de `Patient` o `Practitioner` / `Organization`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserCreatedEvent {
    /// Identificador único de la cuenta de usuario (UUID v7).
    pub user_id: Uuid,
    /// Recurso de identidad y demografía de la persona física (FHIR R4 Person).
    pub person: Person,
    /// Flag que indica si se debe inicializar una clínica u organización para el usuario.
    pub create_clinic: bool,
}

impl UserCreatedEvent {
    /// Cola donde viaja el evento. El productor (`crates/user`) y el consumidor
    /// (`crates/administration`) referencian esta constante en lugar de un literal,
    /// de modo que el compilador garantiza que ambos hablan de la misma cola.
    pub const QUEUE: &'static str = "user.created";
}

impl DomainEvent for UserCreatedEvent {
    fn event_name(&self) -> &'static str {
        Self::QUEUE
    }
}

/// Puerto de dominio para la publicación de eventos de dominio en el sistema.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publica el evento de creación de usuario hacia Apalis / Event Bus.
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), ClickCareError>;
}

/// Implementación por defecto de infraestructura que registra la emisión de eventos en el sistema log/tracing.
#[derive(Debug, Default)]
pub struct LoggingEventPublisher;

#[async_trait]
impl EventPublisher for LoggingEventPublisher {
    async fn publish_user_created(&self, event: UserCreatedEvent) -> Result<(), ClickCareError> {
        tracing::info!(
            "[LoggingEventPublisher] Publicando evento {}: user_id={}",
            event.event_name(),
            event.user_id
        );
        Ok(())
    }
}
