use crate::domain::patient::Patient;
use crate::domain::repository::patient_repository::PatientRepository;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.patient`.
///
/// El recurso FHIR `Person` se guarda serializado como JSON en una sola columna en
/// lugar de aplanarse en columnas sueltas: así el expediente conserva el recurso
/// completo tal como llegó en el evento, sin perder campos ni duplicar el modelo.
#[derive(Debug, Clone, toasty::Model)]
#[table = "patient"]
pub struct PatientRecord {
    #[key]
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub active: bool,
    pub person: String,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct PatientRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl PatientRepository for PatientRepositoryImpl {
    async fn exists_by_user_id(&self, user_id: &Uuid) -> Result<bool, ClickCareError> {
        let rows =
            toasty::sql::query("select 1 from administration.patient where user_id = $1 limit 1")
                .bind(*user_id)
                .exec(&mut self.db.clone())
                .await
                .map_err(|error| {
                    error!("Error al consultar el expediente de user_id={user_id}: {error}");
                    ClickCareError::generic(format!(
                        "Error al consultar el expediente de user_id={user_id} ({error})"
                    ))
                })?;

        Ok(!rows.is_empty())
    }

    async fn save(&self, patient: &Patient) -> Result<(), ClickCareError> {
        let person = serde_json::to_string(&patient.person).map_err(|error| {
            error!(
                "Error al serializar la Person del expediente id={}: {error}",
                patient.id
            );
            ClickCareError::generic(format!(
                "Error al serializar la Person del expediente id={} ({error})",
                patient.id
            ))
        })?;

        toasty::create!(PatientRecord {
            id: patient.id,
            user_id: patient.user_id,
            active: patient.active,
            person: person,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al guardar el expediente id={}: {error}", patient.id);
            ClickCareError::generic(format!(
                "Error al guardar el expediente id={} ({error})",
                patient.id
            ))
        })?;

        Ok(())
    }
}
