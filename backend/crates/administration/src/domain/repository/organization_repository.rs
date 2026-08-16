use crate::domain::organization::Organization;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia de `Organization` (la clínica).
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Indica si el usuario indicado ya es propietario de una organización.
    ///
    /// Existe para que el consumo del evento sea idempotente: la entrega es
    /// *at-least-once*, de modo que el mismo `UserCreatedEvent` puede llegar más
    /// de una vez y no debe producir clínicas duplicadas.
    async fn exists_by_owner_user_id(&self, owner_user_id: &Uuid) -> Result<bool, ClickCareError>;

    /// Persiste la organización.
    async fn save(&self, organization: &Organization) -> Result<(), ClickCareError>;
}
