use crate::domain::practitioner::Practitioner;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia de la ficha `Practitioner`.
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait PractitionerRepository: Send + Sync {
    /// Indica si el usuario indicado ya tiene una ficha **en esa clínica**.
    ///
    /// La consulta se acota siempre por `organization_id`: un mismo médico puede
    /// ejercer en varias clínicas, y su ficha en una no debe impedir la de otra.
    ///
    /// Existe para que el consumo del evento sea idempotente: la entrega es
    /// *at-least-once*, de modo que el mismo `UserCreatedEvent` puede llegar más
    /// de una vez y no debe producir fichas duplicadas.
    async fn exists_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, ClickCareError>;

    /// Persiste la ficha del profesional junto a su recurso FHIR `Person`.
    async fn save(&self, practitioner: &Practitioner) -> Result<(), ClickCareError>;
}
