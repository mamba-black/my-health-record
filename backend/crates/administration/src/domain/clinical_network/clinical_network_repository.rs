use crate::domain::clinical_network::ClinicalNetwork;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de la Organization raíz (`ClinicalNetwork`).
#[async_trait]
pub trait ClinicalNetworkRepository: Send + Sync {
    /// Devuelve la raíz por su identificador único.
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ClinicalNetwork>, ClickCareError>;

    /// Raíz de un dueño, si ya la tiene.
    async fn find_by_owner(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<ClinicalNetwork>, ClickCareError>;

    /// Persiste la raíz. Falla si el dueño ya tiene otra (índice único por dueño).
    async fn save(&self, network: &ClinicalNetwork) -> Result<(), ClickCareError>;
}
