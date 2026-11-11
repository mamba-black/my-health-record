use crate::domain::practitioner::Practitioner;
use crate::domain::practitioner::practitioner_repository::PractitionerRepository;
use app_core::domain::error::ClickCareError;
use app_core::infrastructure::transport::PersonDto;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.practitioner`.
///
/// Igual que en el expediente del paciente, el recurso FHIR `Person` se guarda
/// serializado como JSON en una sola columna en lugar de aplanarse.
///
/// `#[unique(organization_id, user_id)]` refleja `uq_practitioner_org_user` (V1).
#[derive(Debug, Clone, toasty::Model)]
#[table = "practitioner"]
#[unique(organization_id, user_id)]
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
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct PractitionerRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl PractitionerRepository for PractitionerRepositoryImpl {
    async fn find_id_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError> {
        PractitionerRecord::filter_by_organization_id_and_user_id(*organization_id, *user_id)
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(|record| record.id))
            .map_err(|error| {
                error!(
                    "Error al consultar la ficha de profesional de user_id={user_id} en la clínica {organization_id}: {error}"
                );
                ClickCareError::generic(format!(
                    "Error al consultar la ficha de profesional de user_id={user_id} en la clínica {organization_id} ({error})"
                ))
            })
    }

    async fn save(&self, practitioner: &Practitioner) -> Result<(), ClickCareError> {
        let person = serde_json::to_string(&PersonDto::from(practitioner.person.clone())).map_err(
            |error| {
                error!(
                    "Error al serializar la Person de la ficha id={}: {error}",
                    practitioner.id
                );
                ClickCareError::generic(format!(
                    "Error al serializar la Person de la ficha id={} ({error})",
                    practitioner.id
                ))
            },
        )?;

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
