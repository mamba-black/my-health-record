use crate::domain::patient::Patient;
use crate::domain::patient::patient_repository::PatientRepository;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use app_core::infrastructure::transport::PersonDto;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::{Type, Value};
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
    /// Discriminador de inquilino: la clínica dueña del expediente.
    pub organization_id: uuid::Uuid,
    /// Vacío cuando recepción registró al paciente sin cuenta.
    pub user_id: Option<uuid::Uuid>,
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
    async fn exists_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, ClickCareError> {
        let rows = toasty::sql::query(
            "select 1 from administration.patient \
             where organization_id = $1 and user_id = $2 limit 1",
        )
        .bind(*organization_id)
        .bind(*user_id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!(
                "Error al consultar el expediente de user_id={user_id} en la clínica {organization_id}: {error}"
            );
            ClickCareError::generic(format!(
                "Error al consultar el expediente de user_id={user_id} en la clínica {organization_id} ({error})"
            ))
        })?;

        Ok(!rows.is_empty())
    }

    async fn save(&self, patient: &Patient) -> Result<(), ClickCareError> {
        let person =
            serde_json::to_string(&PersonDto::from(patient.person.clone())).map_err(|error| {
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
            organization_id: patient.organization_id,
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

    async fn search(
        &self,
        organization_id: &Uuid,
        name: &str,
    ) -> Result<Vec<Patient>, ClickCareError> {
        // Búsqueda simple sobre el JSON de la `Person`: suficiente para la escala actual.
        let pattern = format!("%{}%", name.trim().to_lowercase());
        let rows = toasty::sql::query(
            "select id, organization_id, user_id, active, person from administration.patient \
             where organization_id = $1 and active and lower(person) like $2 \
               and not exists ( \
                 select 1 from administration.patient_link l \
                   join administration.patient own \
                     on own.organization_id = l.organization_id and own.user_id = l.user_id \
                 where l.patient_id = patient.id and l.unlinked_at is null) \
             order by created_at limit 100",
        )
        .bind(*organization_id)
        .bind(pattern)
        .column_types(patient_column_types())
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al buscar pacientes de la clínica {organization_id}: {error}");
            ClickCareError::generic(format!(
                "Error al buscar pacientes de la clínica {organization_id} ({error})"
            ))
        })?;
        rows.into_iter().map(map_patient_row).collect()
    }

    async fn find_by_document(
        &self,
        organization_id: &Uuid,
        identifier: &Identifier,
    ) -> Result<Option<Patient>, ClickCareError> {
        // La `Person` se guarda con la forma de `PersonDto`: el documento es un objeto
        // con una sola clave, el nombre de la variante (`{"DNI": "12345678"}`).
        let key = match identifier {
            Identifier::DNI(_) => "DNI",
            Identifier::CE(_) => "CE",
            Identifier::Passport(_) => "Passport",
        };
        let rows = toasty::sql::query(
            "select id, organization_id, user_id, active, person from administration.patient \
             where organization_id = $1 and (person::jsonb -> 'identifier' ->> $2) = $3 \
             order by created_at limit 1",
        )
        .bind(*organization_id)
        .bind(key.to_string())
        .bind(identifier.value().to_string())
        .column_types(patient_column_types())
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al buscar por documento en la clínica {organization_id}: {error}");
            ClickCareError::generic(format!(
                "Error al buscar por documento en la clínica {organization_id} ({error})"
            ))
        })?;
        rows.into_iter().next().map(map_patient_row).transpose()
    }

    async fn find_by_id(
        &self,
        organization_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<Patient>, ClickCareError> {
        let rows = toasty::sql::query(
            "select id, organization_id, user_id, active, person from administration.patient \
             where organization_id = $1 and id = $2 limit 1",
        )
        .bind(*organization_id)
        .bind(*id)
        .column_types(patient_column_types())
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar el expediente {id}: {error}");
            ClickCareError::generic(format!("Error al consultar el expediente {id} ({error})"))
        })?;
        rows.into_iter().next().map(map_patient_row).transpose()
    }
}

fn patient_column_types() -> [Type; 5] {
    [Type::Uuid, Type::Uuid, Type::Uuid, Type::Bool, Type::String]
}

fn map_patient_row(row: Value) -> Result<Patient, ClickCareError> {
    let mut fields = row.into_record().fields.into_iter();
    let mut next = |name: &str| {
        fields
            .next()
            .ok_or_else(|| ClickCareError::generic(format!("Falta la columna {name} de patient")))
    };
    let uuid =
        |value: Value| Uuid::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()));
    let id = uuid(next("id")?)?;
    let organization_id = uuid(next("organization_id")?)?;
    let user_id = match next("user_id")? {
        Value::Null => None,
        value => Some(uuid(value)?),
    };
    let active =
        bool::try_from(next("active")?).map_err(|e| ClickCareError::generic(e.to_string()))?;
    let person_json =
        String::try_from(next("person")?).map_err(|e| ClickCareError::generic(e.to_string()))?;
    let person: PersonDto = serde_json::from_str(&person_json).map_err(|error| {
        ClickCareError::generic(format!(
            "La Person del expediente {id} no es válida ({error})"
        ))
    })?;
    let mut patient = Patient::without_account(id, organization_id, person.into());
    patient.user_id = user_id;
    patient.active = active;
    Ok(patient)
}
