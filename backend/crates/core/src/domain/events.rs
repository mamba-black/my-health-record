use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::domain::event::*;

/// Evento de dominio emitido cuando un usuario se registra con intención de ser propietario/fundador de una clínica.
/// Transporta el identificador de la organización clínica generado anticipadamente (UUIDv7).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FounderRegistered {
    /// Identificador del usuario creador (UUID v7).
    pub user_id: Uuid,
    /// Identificador de la persona física asociada (UUID v7).
    pub person_id: Uuid,
    /// Identificador pre-asignado de la organización/clínica (UUID v7).
    pub organization_id: Uuid,
}

impl FounderRegistered {
    /// Cola donde viaja el evento para consumo asíncrono.
    pub const QUEUE: &'static str = "founder.registered";
}

impl DomainEvent for FounderRegistered {
    fn event_name(&self) -> &'static str {
        Self::QUEUE
    }
}
