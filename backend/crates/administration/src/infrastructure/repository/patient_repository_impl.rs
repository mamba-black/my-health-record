use crate::domain::patient::Patient;
use crate::domain::patient::patient_repository::PatientRepository;
use crate::infrastructure::repository::{db_error, uuid_column};
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use app_core::infrastructure::transport::PersonDto;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.patient`.
///
/// El recurso FHIR `Person` se guarda serializado como JSON en una sola columna en
/// lugar de aplanarse en columnas sueltas: así el expediente conserva el recurso
/// completo tal como llegó en el evento, sin perder campos ni duplicar el modelo.
///
/// Los índices los crean las migraciones de `backend/migrations/`: `uq_patient_org_user`
/// (V1) no se declara como `#[unique]` porque `user_id` puede ser nulo y toasty no
/// admite columnas nulas en un índice compuesto.
#[derive(Debug, Clone, toasty::Model)]
#[table = "patient"]
pub struct PatientRecord {
    #[key]
    pub id: uuid::Uuid,
    /// Discriminador de inquilino: la clínica dueña del expediente.
    #[index]
    pub organization_id: uuid::Uuid,
    /// Vacío cuando recepción registró al paciente sin cuenta.
    pub user_id: Option<uuid::Uuid>,
    pub active: bool,
    pub person: String,
    pub document_verified_at: Option<jiff::Timestamp>,
    pub document_verified_by: Option<uuid::Uuid>,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct PatientRepositoryImpl {
    pub(crate) db: Db,
}

impl TryFrom<PatientRecord> for Patient {
    type Error = ClickCareError;

    fn try_from(record: PatientRecord) -> Result<Self, Self::Error> {
        let person: PersonDto = serde_json::from_str(&record.person).map_err(|error| {
            ClickCareError::generic(format!(
                "La Person del expediente {} no es válida ({error})",
                record.id
            ))
        })?;
        let mut patient =
            Patient::without_account(record.id, record.organization_id, person.into());
        patient.user_id = record.user_id;
        patient.active = record.active;
        Ok(patient)
    }
}

/// Expedientes de una clínica, a partir de los ids que devolvió una consulta en SQL crudo.
async fn load_patients(
    db: &Db,
    organization_id: Uuid,
    ids: Vec<Uuid>,
) -> Result<Vec<Patient>, ClickCareError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let fields = PatientRecord::fields();
    PatientRecord::filter_by_organization_id(organization_id)
        .filter(fields.id().in_list(ids))
        .order_by(fields.created_at().asc())
        .exec(&mut db.clone())
        .await
        .map_err(db_error(format!(
            "Error al cargar los expedientes de la clínica {organization_id}"
        )))?
        .into_iter()
        .map(Patient::try_from)
        .collect()
}

#[async_trait]
impl PatientRepository for PatientRepositoryImpl {
    async fn exists_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, ClickCareError> {
        let found = PatientRecord::filter_by_organization_id(*organization_id)
            .filter(PatientRecord::fields().user_id().eq(Some(*user_id)))
            .first()
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al consultar el expediente de user_id={user_id} en la clínica {organization_id}"
            )))?;
        Ok(found.is_some())
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
        .map_err(db_error(format!(
            "Error al guardar el expediente id={}",
            patient.id
        )))?;

        Ok(())
    }

    async fn search(
        &self,
        organization_id: &Uuid,
        name: &str,
    ) -> Result<Vec<Patient>, ClickCareError> {
        // SQL crudo por necesidad: busca texto dentro del JSON de la `Person` y descarta
        // con un join los registros ya vinculados a una cuenta. Solo devuelve ids.
        let pattern = format!("%{}%", name.trim().to_lowercase());
        let rows = toasty::sql::query(
            "select id from administration.patient \
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
        .column_types([toasty::stmt::Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al buscar pacientes de la clínica {organization_id}"
        )))?;
        load_patients(&self.db, *organization_id, uuid_column(rows)?).await
    }

    async fn find_by_document(
        &self,
        organization_id: &Uuid,
        identifier: &Identifier,
    ) -> Result<Option<Patient>, ClickCareError> {
        // SQL crudo por necesidad: el documento vive dentro del JSON de la `Person`, con
        // la forma de `PersonDto` (`{"DNI": "12345678"}`). Solo devuelve el id.
        let key = match identifier {
            Identifier::DNI(_) => "DNI",
            Identifier::CE(_) => "CE",
            Identifier::Passport(_) => "Passport",
        };
        let rows = toasty::sql::query(
            "select id from administration.patient \
             where organization_id = $1 and (person::jsonb -> 'identifier' ->> $2) = $3 \
             order by created_at limit 1",
        )
        .bind(*organization_id)
        .bind(key.to_string())
        .bind(identifier.value().to_string())
        .column_types([toasty::stmt::Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al buscar por documento en la clínica {organization_id}"
        )))?;
        let ids = uuid_column(rows)?;
        Ok(load_patients(&self.db, *organization_id, ids)
            .await?
            .into_iter()
            .next())
    }

    async fn find_by_id(
        &self,
        organization_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<Patient>, ClickCareError> {
        PatientRecord::filter_by_id(*id)
            .filter(
                PatientRecord::fields()
                    .organization_id()
                    .eq(*organization_id),
            )
            .first()
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!("Error al consultar el expediente {id}")))?
            .map(Patient::try_from)
            .transpose()
    }
}
