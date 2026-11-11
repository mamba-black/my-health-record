//! `Person.link` persistido en `identity.person_link`.

use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::PersonLinkTarget;
use app_core::domain::person_link::PersonLinkRegistry;
use async_trait::async_trait;
use toasty::Db;
use tracing::error;
use uuid::Uuid;

pub(crate) struct PersonLinkRepositoryImpl {
    pub(crate) db: Db,
}

fn type_code(target: &PersonLinkTarget) -> (&'static str, Uuid) {
    match target {
        PersonLinkTarget::Organization(id) => ("ORGANIZATION", *id),
        PersonLinkTarget::Patient(id) => ("PATIENT", *id),
        PersonLinkTarget::Practitioner(id) => ("PRACTITIONER", *id),
        PersonLinkTarget::RelatedPerson(id) => ("RELATED_PERSON", *id),
    }
}

/// Fila de `identity.person_link` (clave primaria compuesta, V7).
#[derive(Debug, Clone, toasty::Model)]
#[table = "person_link"]
#[key(user_id, target_type, target_id)]
pub struct PersonLinkRecord {
    #[index]
    pub user_id: uuid::Uuid,
    pub target_type: String,
    pub target_id: uuid::Uuid,
    #[auto]
    pub created_at: jiff::Timestamp,
}

impl From<PersonLinkRecord> for PersonLinkTarget {
    fn from(record: PersonLinkRecord) -> Self {
        let id = record.target_id;
        match record.target_type.as_str() {
            "PATIENT" => PersonLinkTarget::Patient(id),
            "PRACTITIONER" => PersonLinkTarget::Practitioner(id),
            "RELATED_PERSON" => PersonLinkTarget::RelatedPerson(id),
            _ => PersonLinkTarget::Organization(id),
        }
    }
}

/// Convierte un error de toasty en `ClickCareError`, dejando el detalle en el log.
fn db_error(context: String) -> impl FnOnce(toasty::Error) -> ClickCareError {
    move |error| {
        error!("{context}: {error}");
        ClickCareError::generic(format!("{context} ({error})"))
    }
}

#[async_trait]
impl PersonLinkRegistry for PersonLinkRepositoryImpl {
    async fn link(&self, user_id: &Uuid, target: PersonLinkTarget) -> Result<(), ClickCareError> {
        let (target_type, target_id) = type_code(&target);
        // SQL crudo por necesidad: `on conflict do nothing` hace el vínculo idempotente
        // en una sola sentencia, sin carrera entre buscar y crear.
        toasty::sql::statement(
            "insert into identity.person_link (user_id, target_type, target_id) \
             values ($1, $2, $3) on conflict do nothing",
        )
        .bind(*user_id)
        .bind(target_type.to_string())
        .bind(target_id)
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al vincular user_id={user_id} con {target_type} {target_id}"
        )))?;
        Ok(())
    }

    async fn links_of(&self, user_id: &Uuid) -> Result<Vec<PersonLinkTarget>, ClickCareError> {
        let links = PersonLinkRecord::filter_by_user_id(*user_id)
            .order_by(PersonLinkRecord::fields().created_at().asc())
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al leer los vínculos de user_id={user_id}"
            )))?;
        Ok(links.into_iter().map(PersonLinkTarget::from).collect())
    }
}

/// Vínculos en memoria, para el DI sin base de datos (`DBType::Mock`).
#[derive(Default)]
pub(crate) struct InMemoryPersonLinkRegistry {
    links: tokio::sync::Mutex<Vec<(Uuid, PersonLinkTarget)>>,
}

#[async_trait]
impl PersonLinkRegistry for InMemoryPersonLinkRegistry {
    async fn link(&self, user_id: &Uuid, target: PersonLinkTarget) -> Result<(), ClickCareError> {
        let mut links = self.links.lock().await;
        if !links.iter().any(|(u, t)| u == user_id && *t == target) {
            links.push((*user_id, target));
        }
        Ok(())
    }

    async fn links_of(&self, user_id: &Uuid) -> Result<Vec<PersonLinkTarget>, ClickCareError> {
        Ok(self
            .links
            .lock()
            .await
            .iter()
            .filter(|(u, _)| u == user_id)
            .map(|(_, t)| t.clone())
            .collect())
    }
}
