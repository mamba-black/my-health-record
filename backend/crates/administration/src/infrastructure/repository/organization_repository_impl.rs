use crate::domain::organization::Organization;
use crate::domain::repository::organization_repository::OrganizationRepository;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.organization`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "organization"]
pub struct OrganizationRecord {
    #[key]
    pub id: uuid::Uuid,
    pub name: String,
    pub tax_id: Option<String>,
    pub owner_user_id: uuid::Uuid,
    pub active: bool,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct OrganizationRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl OrganizationRepository for OrganizationRepositoryImpl {
    async fn exists_by_owner_user_id(&self, owner_user_id: &Uuid) -> Result<bool, ClickCareError> {
        let rows = toasty::sql::query(
            "select 1 from administration.organization where owner_user_id = $1 limit 1",
        )
        .bind(*owner_user_id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar la organización de owner_user_id={owner_user_id}: {error}");
            ClickCareError::generic(format!(
                "Error al consultar la organización de owner_user_id={owner_user_id} ({error})"
            ))
        })?;

        Ok(!rows.is_empty())
    }

    async fn save(&self, organization: &Organization) -> Result<(), ClickCareError> {
        toasty::create!(OrganizationRecord {
            id: organization.id,
            name: organization.name.clone(),
            tax_id: organization.tax_id.clone(),
            owner_user_id: organization.owner_user_id,
            active: organization.active,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!(
                "Error al guardar la organización id={}: {error}",
                organization.id
            );
            ClickCareError::generic(format!(
                "Error al guardar la organización id={} ({error})",
                organization.id
            ))
        })?;

        Ok(())
    }
}
