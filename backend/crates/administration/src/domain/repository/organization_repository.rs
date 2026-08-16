use crate::domain::organization::Organization;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia de `Organization` (la clínica).
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Devuelve el identificador de la organización que el usuario ya posee, si la hay.
    ///
    /// Devuelve el id en lugar de un booleano porque quien procesa el evento necesita
    /// la clínica para colgar de ella las entidades locales (`Practitioner`, `Patient`),
    /// tanto si acaba de crearla como si ya existía.
    ///
    /// Existe además para que el consumo del evento sea idempotente: la entrega es
    /// *at-least-once*, de modo que el mismo `UserCreatedEvent` puede llegar más
    /// de una vez y no debe producir clínicas duplicadas.
    async fn find_id_by_owner_user_id(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError>;

    /// Persiste la organización.
    async fn save(&self, organization: &Organization) -> Result<(), ClickCareError>;
}
