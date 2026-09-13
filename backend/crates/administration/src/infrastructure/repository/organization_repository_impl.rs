use crate::domain::organization::Organization;
use crate::domain::repository::organization_repository::OrganizationRepository;
use crate::infrastructure::repository::first_uuid_column;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::Type;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.organization`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "organization"]
pub struct OrganizationRecord {
    #[key]
    pub id: uuid::Uuid,
    pub network_id: uuid::Uuid,
    pub subdomain: String,
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
    async fn find_id_by_owner_user_id(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError> {
        let rows = toasty::sql::query(
            "select id from administration.organization where owner_user_id = $1 limit 1",
        )
        .bind(*owner_user_id)
        .column_types([Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar la organización de owner_user_id={owner_user_id}: {error}");
            ClickCareError::generic(format!(
                "Error al consultar la organización de owner_user_id={owner_user_id} ({error})"
            ))
        })?;

        first_uuid_column(rows).map_err(|error| {
            error!(
                "El id de la organización de owner_user_id={owner_user_id} no es válido: {error}"
            );
            error
        })
    }

    async fn find_org_and_network_by_subdomain(
        &self,
        subdomain: &str,
    ) -> Result<Option<(Uuid, Uuid)>, ClickCareError> {
        let rows = toasty::sql::query(
            "select id, network_id from administration.organization where subdomain = $1 limit 1",
        )
        .bind(subdomain.to_string())
        .column_types([Type::Uuid, Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar la organización por subdominio={subdomain}: {error}");
            ClickCareError::generic(format!(
                "Error al consultar la organización por subdominio={subdomain} ({error})"
            ))
        })?;

        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };

        let mut fields = row.into_record().fields.into_iter();
        let org_id_val = fields.next().ok_or_else(|| {
            ClickCareError::generic("Falta columna id en la consulta de organización".to_string())
        })?;
        let network_id_val = fields.next().ok_or_else(|| {
            ClickCareError::generic(
                "Falta columna network_id en la consulta de organización".to_string(),
            )
        })?;

        let org_id = Uuid::try_from(org_id_val).map_err(|error| {
            ClickCareError::generic(format!("Columna id no es un UUID válido: {error}"))
        })?;
        let network_id = Uuid::try_from(network_id_val).map_err(|error| {
            ClickCareError::generic(format!("Columna network_id no es un UUID válido: {error}"))
        })?;

        Ok(Some((org_id, network_id)))
    }

    async fn save(&self, organization: &Organization) -> Result<(), ClickCareError> {
        toasty::create!(OrganizationRecord {
            id: organization.id,
            network_id: organization.network_id,
            subdomain: organization.subdomain.clone(),
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
