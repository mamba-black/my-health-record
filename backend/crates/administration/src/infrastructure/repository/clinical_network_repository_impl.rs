use crate::domain::clinical_network::ClinicalNetwork;
use crate::domain::repository::clinical_network_repository::ClinicalNetworkRepository;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.clinical_network`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "clinical_network"]
pub struct ClinicalNetworkRecord {
    #[key]
    pub id: uuid::Uuid,
    pub name: String,
    pub is_default: bool,
    pub active: bool,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct ClinicalNetworkRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl ClinicalNetworkRepository for ClinicalNetworkRepositoryImpl {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ClinicalNetwork>, ClickCareError> {
        let rows = toasty::sql::query(
            "select 1 from administration.clinical_network where id = $1 limit 1",
        )
        .bind(*id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar red clínica {id}: {error}");
            ClickCareError::generic(format!("Error al consultar red clínica {id}: {error}"))
        })?;

        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ClinicalNetwork::new(*id, String::new(), true)))
        }
    }

    async fn save(&self, network: &ClinicalNetwork) -> Result<(), ClickCareError> {
        toasty::create!(ClinicalNetworkRecord {
            id: network.id,
            name: network.name.clone(),
            is_default: network.is_default,
            active: network.active,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al guardar la red clínica id={}: {error}", network.id);
            ClickCareError::generic(format!(
                "Error al guardar la red clínica id={} ({error})",
                network.id
            ))
        })?;

        Ok(())
    }
}
