use crate::domain::practitioner::Practitioner;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia de la ficha `Practitioner`.
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait PractitionerRepository: Send + Sync {
    /// Indica si el usuario indicado ya tiene una ficha de profesional de salud.
    ///
    /// Existe para que el consumo del evento sea idempotente: la entrega es
    /// *at-least-once*, de modo que el mismo `UserCreatedEvent` puede llegar más
    /// de una vez y no debe producir fichas duplicadas.
    async fn exists_by_user_id(&self, user_id: &Uuid) -> Result<bool, ClickCareError>;

    /// Persiste la ficha del profesional junto a su recurso FHIR `Person`.
    async fn save(&self, practitioner: &Practitioner) -> Result<(), ClickCareError>;
}
