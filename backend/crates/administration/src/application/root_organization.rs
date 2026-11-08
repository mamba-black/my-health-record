//! Organization raíz del dueño (FHIR R4 `Organization` sin `partOf`).
//!
//! Cada dueño tiene una única raíz, su empresa o grupo. Nace cuando se registra como
//! dueño o, a más tardar, al crear su primera clínica, y no depende de la red del
//! enlace de registro. Todas sus clínicas cuelgan de ella (`partOf`), y la cuenta del
//! dueño se vincula con la raíz y con cada clínica por `Person.link`.

use crate::domain::clinical_network::ClinicalNetwork;
use crate::domain::clinical_network::clinical_network_repository::ClinicalNetworkRepository;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Casos de uso de la Organization raíz.
pub struct RootOrganizationUseCase {
    networks: Arc<dyn ClinicalNetworkRepository>,
}

impl RootOrganizationUseCase {
    pub fn new(networks: Arc<dyn ClinicalNetworkRepository>) -> Self {
        Self { networks }
    }

    /// Devuelve la raíz del dueño y la crea si aún no la tiene. Es idempotente, también
    /// con dos llamadas simultáneas: el índice único por dueño deja una sola raíz.
    pub async fn ensure_for_owner(
        &self,
        owner_user_id: Uuid,
        name: &str,
    ) -> Result<ClinicalNetwork, ClickCareError> {
        if let Some(root) = self.networks.find_by_owner(&owner_user_id).await? {
            return Ok(root);
        }
        let name = match name.trim() {
            "" => "Mi organización".to_string(),
            name => name.to_string(),
        };
        let root = ClinicalNetwork::root(Uuid::now_v7(), name, owner_user_id);
        match self.networks.save(&root).await {
            Ok(()) => {
                info!(
                    "Organization raíz creada: id={} para el dueño user_id={owner_user_id}",
                    root.id
                );
                Ok(root)
            }
            // Otra petición la creó a la vez: se usa la que quedó guardada.
            Err(error) => self
                .networks
                .find_by_owner(&owner_user_id)
                .await?
                .ok_or(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::InMemoryNetworkRepository;

    #[tokio::test]
    async fn cada_dueno_tiene_una_sola_raiz() {
        let networks = Arc::new(InMemoryNetworkRepository::default());
        let use_case = RootOrganizationUseCase::new(networks.clone());
        let owner = Uuid::now_v7();
        let first = use_case
            .ensure_for_owner(owner, "Grupo Salud")
            .await
            .expect("crea la raíz");
        let again = use_case
            .ensure_for_owner(owner, "Otro nombre")
            .await
            .expect("devuelve la raíz");
        assert_eq!(first.id, again.id);
        assert_eq!(networks.networks.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn duenos_distintos_no_comparten_raiz() {
        let use_case = RootOrganizationUseCase::new(Arc::new(InMemoryNetworkRepository::default()));
        let a = use_case
            .ensure_for_owner(Uuid::now_v7(), "A")
            .await
            .expect("raíz A");
        let b = use_case
            .ensure_for_owner(Uuid::now_v7(), "B")
            .await
            .expect("raíz B");
        assert_ne!(a.id, b.id);
    }
}
