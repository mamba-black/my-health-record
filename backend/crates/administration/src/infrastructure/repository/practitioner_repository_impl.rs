use crate::domain::practitioner::Practitioner;
use crate::domain::repository::practitioner_repository::PractitionerRepository;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.practitioner`.
///
/// Igual que en el expediente del paciente, el recurso FHIR `Person` se guarda
/// serializado como JSON en una sola columna en lugar de aplanarse.
#[derive(Debug, Clone, toasty::Model)]
#[table = "practitioner"]
pub struct PractitionerRecord {
    #[key]
    pub id: uuid::Uuid,
    /// Discriminador de inquilino: la clínica en la que ejerce el profesional.
    pub organization_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub active: bool,
    pub medical_license_number: String,
    pub specialty: Option<String>,
    pub person: String,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct PractitionerRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl PractitionerRepository for PractitionerRepositoryImpl {
    async fn exists_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, ClickCareError> {
        let rows = toasty::sql::query(
            "select 1 from administration.practitioner \
             where organization_id = $1 and user_id = $2 limit 1",
        )
        .bind(*organization_id)
        .bind(*user_id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!(
                "Error al consultar la ficha de profesional de user_id={user_id} en la clínica {organization_id}: {error}"
            );
            ClickCareError::generic(format!(
                "Error al consultar la ficha de profesional de user_id={user_id} en la clínica {organization_id} ({error})"
            ))
        })?;

        Ok(!rows.is_empty())
    }

    async fn save(&self, practitioner: &Practitioner) -> Result<(), ClickCareError> {
        let person = serde_json::to_string(&practitioner.person).map_err(|error| {
            error!(
                "Error al serializar la Person de la ficha id={}: {error}",
                practitioner.id
            );
            ClickCareError::generic(format!(
                "Error al serializar la Person de la ficha id={} ({error})",
                practitioner.id
            ))
        })?;

        toasty::create!(PractitionerRecord {
            id: practitioner.id,
            organization_id: practitioner.organization_id,
            user_id: practitioner.user_id,
            active: practitioner.active,
            medical_license_number: practitioner.medical_license_number.clone(),
            specialty: practitioner.specialty.clone(),
            person: person,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!(
                "Error al guardar la ficha de profesional id={}: {error}",
                practitioner.id
            );
            ClickCareError::generic(format!(
                "Error al guardar la ficha de profesional id={} ({error})",
                practitioner.id
            ))
        })?;

        Ok(())
    }
}
