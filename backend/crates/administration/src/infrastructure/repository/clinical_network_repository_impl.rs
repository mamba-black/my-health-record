use crate::domain::clinical_network::ClinicalNetwork;
use crate::domain::clinical_network::clinical_network_repository::ClinicalNetworkRepository;
use crate::infrastructure::repository::db_error;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use uuid::Uuid;

/// Fila de la tabla `administration.clinical_network`.
///
/// Los atributos `#[unique]`/`#[index]` solo generan los buscadores (`filter_by_*`):
/// los índices los crean las migraciones de `backend/migrations/` y deben coincidir.
#[derive(Debug, Clone, toasty::Model)]
#[table = "clinical_network"]
pub struct ClinicalNetworkRecord {
    #[key]
    pub id: uuid::Uuid,
    pub name: String,
    pub is_default: bool,
    pub active: bool,
    /// Único cuando no es nulo (`uq_clinical_network_owner`, V7).
    #[unique]
    pub owner_user_id: Option<uuid::Uuid>,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct ClinicalNetworkRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl ClinicalNetworkRepository for ClinicalNetworkRepositoryImpl {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<ClinicalNetwork>, ClickCareError> {
        ClinicalNetworkRecord::filter_by_id(*id)
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(ClinicalNetwork::from))
            .map_err(db_error(format!(
                "Error al consultar la Organization raíz id={id}"
            )))
    }

    async fn find_by_owner(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<ClinicalNetwork>, ClickCareError> {
        ClinicalNetworkRecord::filter_by_owner_user_id(Some(*owner_user_id))
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(ClinicalNetwork::from))
            .map_err(db_error(format!(
                "Error al consultar la Organization raíz del dueño {owner_user_id}"
            )))
    }

    async fn save(&self, network: &ClinicalNetwork) -> Result<(), ClickCareError> {
        toasty::create!(ClinicalNetworkRecord {
            id: network.id,
            name: network.name.clone(),
            is_default: network.is_default,
            active: network.active,
            owner_user_id: network.owner_user_id,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al guardar la red clínica id={}",
            network.id
        )))?;

        Ok(())
    }
}

impl From<ClinicalNetworkRecord> for ClinicalNetwork {
    fn from(record: ClinicalNetworkRecord) -> Self {
        ClinicalNetwork {
            id: record.id,
            name: record.name,
            is_default: record.is_default,
            active: record.active,
            owner_user_id: record.owner_user_id,
        }
    }
}
