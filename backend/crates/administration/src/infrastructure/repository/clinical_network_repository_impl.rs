use crate::domain::clinical_network::ClinicalNetwork;
use crate::domain::clinical_network::clinical_network_repository::ClinicalNetworkRepository;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::{Type, Value};
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
    pub owner_user_id: Option<uuid::Uuid>,
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
        self.find_one("id", *id).await
    }

    async fn find_by_owner(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<ClinicalNetwork>, ClickCareError> {
        self.find_one("owner_user_id", *owner_user_id).await
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

impl ClinicalNetworkRepositoryImpl {
    /// Raíz por una columna con valor UUID (`id` u `owner_user_id`).
    async fn find_one(
        &self,
        column: &'static str,
        value: Uuid,
    ) -> Result<Option<ClinicalNetwork>, ClickCareError> {
        let rows = toasty::sql::query(format!(
            "select id, name, is_default, active, owner_user_id \
             from administration.clinical_network where {column} = $1 limit 1"
        ))
        .bind(value)
        .column_types([Type::Uuid, Type::String, Type::Bool, Type::Bool, Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar la Organization raíz ({column}={value}): {error}");
            ClickCareError::generic(format!(
                "Error al consultar la Organization raíz ({column}={value}): {error}"
            ))
        })?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let mut fields = row.into_record().fields.into_iter();
        let mut next = || {
            fields
                .next()
                .ok_or_else(|| ClickCareError::generic("Falta una columna de clinical_network"))
        };
        let id = Uuid::try_from(next()?).map_err(|e| ClickCareError::generic(e.to_string()))?;
        let name = String::try_from(next()?).map_err(|e| ClickCareError::generic(e.to_string()))?;
        let is_default =
            bool::try_from(next()?).map_err(|e| ClickCareError::generic(e.to_string()))?;
        let active = bool::try_from(next()?).map_err(|e| ClickCareError::generic(e.to_string()))?;
        let owner_user_id = match next()? {
            Value::Null => None,
            value => {
                Some(Uuid::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()))?)
            }
        };
        Ok(Some(ClinicalNetwork {
            id,
            name,
            is_default,
            active,
            owner_user_id,
        }))
    }
}
