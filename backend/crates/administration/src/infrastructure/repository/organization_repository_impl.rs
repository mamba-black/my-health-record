use crate::domain::organization::Organization;
use crate::domain::organization::organization_repository::OrganizationRepository;
use crate::infrastructure::repository::first_uuid_column;
use app_core::domain::access::ClinicStatus;
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
    pub status: String,
    pub legal_name: Option<String>,
    pub trade_name: Option<String>,
    pub ipress_code: Option<String>,
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

    async fn find_organization_and_network_by_subdomain(
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
        let organization_id_value = fields.next().ok_or_else(|| {
            ClickCareError::generic("Falta columna id en la consulta de organización".to_string())
        })?;
        let network_id_value = fields.next().ok_or_else(|| {
            ClickCareError::generic(
                "Falta columna network_id en la consulta de organización".to_string(),
            )
        })?;

        let organization_id = Uuid::try_from(organization_id_value).map_err(|error| {
            ClickCareError::generic(format!("Columna id no es un UUID válido: {error}"))
        })?;
        let network_id = Uuid::try_from(network_id_value).map_err(|error| {
            ClickCareError::generic(format!("Columna network_id no es un UUID válido: {error}"))
        })?;

        Ok(Some((organization_id, network_id)))
    }

    async fn save(&self, organization: &Organization) -> Result<(), ClickCareError> {
        toasty::create!(OrganizationRecord {
            id: organization.id,
            network_id: organization.network_id,
            subdomain: organization.subdomain.clone(),
            name: organization.name.clone(),
            tax_id: organization.tax_id.clone(),
            owner_user_id: organization.owner_user_id,
            active: organization.status == ClinicStatus::Active,
            status: organization.status.code().to_string(),
            legal_name: organization.legal_name.clone(),
            trade_name: organization.trade_name.clone(),
            ipress_code: organization.ipress_code.clone(),
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

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Organization>, ClickCareError> {
        let rows = toasty::sql::query(
            "select id, network_id, subdomain, name, coalesce(tax_id, ''), owner_user_id, status, \
             coalesce(legal_name, ''), coalesce(trade_name, ''), coalesce(ipress_code, '') \
             from administration.organization where id = $1 limit 1",
        )
        .bind(*id)
        .column_types([
            Type::Uuid,
            Type::Uuid,
            Type::String,
            Type::String,
            Type::String,
            Type::Uuid,
            Type::String,
            Type::String,
            Type::String,
            Type::String,
        ])
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar la organización id={id}: {error}");
            ClickCareError::generic(format!(
                "Error al consultar la organización id={id} ({error})"
            ))
        })?;

        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let mut fields = row.into_record().fields.into_iter();
        let mut next = |name: &str| {
            fields.next().ok_or_else(|| {
                ClickCareError::generic(format!("Falta la columna {name} de organization"))
            })
        };
        let uuid = |value: toasty::stmt::Value| {
            Uuid::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()))
        };
        let text = |value: toasty::stmt::Value| {
            String::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()))
        };
        let optional = |value: String| (!value.is_empty()).then_some(value);

        let id = uuid(next("id")?)?;
        let network_id = uuid(next("network_id")?)?;
        let subdomain = text(next("subdomain")?)?;
        let name = text(next("name")?)?;
        let tax_id = optional(text(next("tax_id")?)?);
        let owner_user_id = uuid(next("owner_user_id")?)?;
        let status_code = text(next("status")?)?;
        let status = ClinicStatus::from_code(&status_code).ok_or_else(|| {
            ClickCareError::generic(format!("Estado de clínica desconocido: {status_code}"))
        })?;
        let legal_name = optional(text(next("legal_name")?)?);
        let trade_name = optional(text(next("trade_name")?)?);
        let ipress_code = optional(text(next("ipress_code")?)?);

        Ok(Some(Organization {
            id,
            network_id,
            subdomain,
            name,
            tax_id,
            owner_user_id,
            status,
            legal_name,
            trade_name,
            ipress_code,
        }))
    }

    async fn update_setup(&self, organization: &Organization) -> Result<(), ClickCareError> {
        toasty::sql::query(
            "update administration.organization set status = $1, active = $2, \
             tax_id = nullif($3::text, ''), legal_name = nullif($4::text, ''), \
             trade_name = nullif($5::text, ''), ipress_code = nullif($6::text, ''), \
             updated_at = current_timestamp where id = $7",
        )
        .bind(organization.status.code().to_string())
        .bind(organization.status == ClinicStatus::Active)
        // Toasty no infiere el tipo de un `NULL` en SQL crudo: viaja como texto vacío.
        .bind(organization.tax_id.clone().unwrap_or_default())
        .bind(organization.legal_name.clone().unwrap_or_default())
        .bind(organization.trade_name.clone().unwrap_or_default())
        .bind(organization.ipress_code.clone().unwrap_or_default())
        .bind(organization.id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!(
                "Error al actualizar la organización id={}: {error}",
                organization.id
            );
            ClickCareError::generic(format!(
                "Error al actualizar la organización id={} ({error})",
                organization.id
            ))
        })?;
        Ok(())
    }
}
