use crate::domain::clinical_network::ClinicalNetwork;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia de `ClinicalNetwork`.
#[async_trait]
pub trait ClinicalNetworkRepository: Send + Sync {
    /// Devuelve la red clínica por su identificador único.
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ClinicalNetwork>, ClickCareError>;

    /// Persiste la red clínica.
    async fn save(&self, network: &ClinicalNetwork) -> Result<(), ClickCareError>;
}
