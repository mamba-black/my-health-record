//! `Person.link` persistido en `identity.person_link`.

use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::PersonLinkTarget;
use app_core::domain::person_link::PersonLinkRegistry;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::Type;
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

#[async_trait]
impl PersonLinkRegistry for PersonLinkRepositoryImpl {
    async fn link(&self, user_id: &Uuid, target: PersonLinkTarget) -> Result<(), ClickCareError> {
        let (target_type, target_id) = type_code(&target);
        toasty::sql::query(
            "insert into identity.person_link (user_id, target_type, target_id) \
             values ($1, $2, $3) on conflict do nothing",
        )
        .bind(*user_id)
        .bind(target_type.to_string())
        .bind(target_id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al vincular user_id={user_id} con {target_type} {target_id}: {error}");
            ClickCareError::generic(format!(
                "Error al vincular la persona con {target_type} {target_id} ({error})"
            ))
        })?;
        Ok(())
    }

    async fn links_of(&self, user_id: &Uuid) -> Result<Vec<PersonLinkTarget>, ClickCareError> {
        let rows = toasty::sql::query(
            "select target_type, target_id from identity.person_link \
             where user_id = $1 order by created_at",
        )
        .bind(*user_id)
        .column_types([Type::String, Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al leer los vínculos de user_id={user_id}: {error}");
            ClickCareError::generic(format!("Error al leer los vínculos ({error})"))
        })?;
        let mut links = Vec::with_capacity(rows.len());
        for row in rows {
            let mut fields = row.into_record().fields.into_iter();
            let (Some(kind), Some(id)) = (fields.next(), fields.next()) else {
                continue;
            };
            let kind =
                String::try_from(kind).map_err(|e| ClickCareError::generic(e.to_string()))?;
            let id = Uuid::try_from(id).map_err(|e| ClickCareError::generic(e.to_string()))?;
            links.push(match kind.as_str() {
                "PATIENT" => PersonLinkTarget::Patient(id),
                "PRACTITIONER" => PersonLinkTarget::Practitioner(id),
                "RELATED_PERSON" => PersonLinkTarget::RelatedPerson(id),
                _ => PersonLinkTarget::Organization(id),
            });
        }
        Ok(links)
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
