use crate::domain::organization::Organization;
use crate::domain::organization::organization_repository::OrganizationRepository;
use crate::infrastructure::repository::db_error;
use app_core::domain::access::ClinicStatus;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use uuid::Uuid;

/// Fila de la tabla `administration.organization`.
///
/// Los atributos `#[unique]`/`#[index]` solo generan los buscadores (`filter_by_*`):
/// los índices los crean las migraciones de `backend/migrations/` y deben coincidir.
#[derive(Debug, Clone, toasty::Model)]
#[table = "organization"]
pub struct OrganizationRecord {
    #[key]
    pub id: uuid::Uuid,
    #[index]
    pub network_id: uuid::Uuid,
    #[unique]
    pub subdomain: String,
    pub name: String,
    pub tax_id: Option<String>,
    pub owner_user_id: uuid::Uuid,
    pub active: bool,
    pub status: String,
    pub legal_name: Option<String>,
    pub trade_name: Option<String>,
    pub ipress_code: Option<String>,
    pub timezone: String,
    pub address: Option<String>,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}

impl TryFrom<OrganizationRecord> for Organization {
    type Error = ClickCareError;

    fn try_from(record: OrganizationRecord) -> Result<Self, Self::Error> {
        let status = ClinicStatus::from_code(&record.status).ok_or_else(|| {
            ClickCareError::generic(format!("Estado de clínica desconocido: {}", record.status))
        })?;
        Ok(Organization {
            id: record.id,
            network_id: record.network_id,
            subdomain: record.subdomain,
            name: record.name,
            tax_id: non_empty(record.tax_id),
            owner_user_id: record.owner_user_id,
            status,
            legal_name: non_empty(record.legal_name),
            trade_name: non_empty(record.trade_name),
            ipress_code: non_empty(record.ipress_code),
            timezone: record.timezone,
            address: non_empty(record.address),
        })
    }
}

/// Un texto vacío equivale a no tener el dato.
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
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
        OrganizationRecord::all()
            .filter(
                OrganizationRecord::fields()
                    .owner_user_id()
                    .eq(*owner_user_id),
            )
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(|record| record.id))
            .map_err(db_error(format!(
                "Error al consultar la organización de owner_user_id={owner_user_id}"
            )))
    }

    async fn find_organization_and_network_by_subdomain(
        &self,
        subdomain: &str,
    ) -> Result<Option<(Uuid, Uuid)>, ClickCareError> {
        OrganizationRecord::filter_by_subdomain(subdomain.to_string())
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(|record| (record.id, record.network_id)))
            .map_err(db_error(format!(
                "Error al consultar la organización por subdominio={subdomain}"
            )))
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
            timezone: organization.timezone.clone(),
            address: organization.address.clone(),
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al guardar la organización id={}",
            organization.id
        )))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Organization>, ClickCareError> {
        OrganizationRecord::filter_by_id(*id)
            .first()
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al consultar la organización id={id}"
            )))?
            .map(Organization::try_from)
            .transpose()
    }

    async fn update_setup(&self, organization: &Organization) -> Result<(), ClickCareError> {
        OrganizationRecord::filter_by_id(organization.id)
            .update()
            .status(organization.status.code().to_string())
            .active(organization.status == ClinicStatus::Active)
            .tax_id(non_empty(organization.tax_id.clone()))
            .legal_name(non_empty(organization.legal_name.clone()))
            .trade_name(non_empty(organization.trade_name.clone()))
            .ipress_code(non_empty(organization.ipress_code.clone()))
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al actualizar la organización id={}",
                organization.id
            )))?;
        Ok(())
    }
}
