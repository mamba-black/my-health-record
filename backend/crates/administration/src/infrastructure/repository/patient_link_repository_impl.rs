use crate::domain::patient::patient_link_repository::{PatientLink, PatientLinkRepository};
use crate::infrastructure::repository::first_uuid_column;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::{Type, Value};
use tracing::error;
use uuid::Uuid;

pub(crate) struct PatientLinkRepositoryImpl {
    pub(crate) db: Db,
}

fn failure(context: &str, error: impl std::fmt::Display) -> ClickCareError {
    error!("{context}: {error}");
    ClickCareError::generic(format!("{context} ({error})"))
}

fn uuids(rows: Vec<Value>) -> Result<Vec<Uuid>, ClickCareError> {
    rows.into_iter()
        .map(|row| {
            let value = row
                .into_record()
                .fields
                .into_iter()
                .next()
                .ok_or_else(|| ClickCareError::generic("Sin columnas"))?;
            Uuid::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()))
        })
        .collect()
}

#[async_trait]
impl PatientLinkRepository for PatientLinkRepositoryImpl {
    async fn record_of_account(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError> {
        let rows = toasty::sql::query(
            "select id from administration.patient \
             where organization_id = $1 and user_id = $2 limit 1",
        )
        .bind(*organization_id)
        .bind(*user_id)
        .column_types([Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al buscar el registro de la cuenta", e))?;
        first_uuid_column(rows)
    }

    async fn records_of_account(
        &self,
        network_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<Uuid>, ClickCareError> {
        let rows = toasty::sql::query(
            "select p.id from administration.patient p \
               join administration.organization o on o.id = p.organization_id \
             where o.network_id = $1 and p.user_id = $2 \
             union \
             select l.patient_id from administration.patient_link l \
             where l.network_id = $1 and l.user_id = $2 and l.unlinked_at is null",
        )
        .bind(*network_id)
        .bind(*user_id)
        .column_types([Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al buscar los registros de la cuenta", e))?;
        uuids(rows)
    }

    async fn account_of(&self, patient_id: &Uuid) -> Result<Option<Uuid>, ClickCareError> {
        let rows = toasty::sql::query(
            "select coalesce(p.user_id, l.user_id) from administration.patient p \
               left join administration.patient_link l \
                 on l.patient_id = p.id and l.unlinked_at is null \
             where p.id = $1 and coalesce(p.user_id, l.user_id) is not null",
        )
        .bind(*patient_id)
        .column_types([Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al buscar la cuenta del registro", e))?;
        first_uuid_column(rows)
    }

    async fn active_link(&self, patient_id: &Uuid) -> Result<Option<PatientLink>, ClickCareError> {
        let rows = toasty::sql::query(
            "select id, network_id, organization_id, patient_id, user_id, linked_by \
             from administration.patient_link where patient_id = $1 and unlinked_at is null",
        )
        .bind(*patient_id)
        .column_types([const { Type::Uuid }; 6])
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al leer el vínculo del registro", e))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let ids = row
            .into_record()
            .fields
            .into_iter()
            .map(|v| Uuid::try_from(v).map_err(|e| ClickCareError::generic(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        let [
            id,
            network_id,
            organization_id,
            patient_id,
            user_id,
            linked_by,
        ] = ids[..]
        else {
            return Err(ClickCareError::generic("Vínculo incompleto"));
        };
        Ok(Some(PatientLink {
            id,
            network_id,
            organization_id,
            patient_id,
            user_id,
            linked_by,
        }))
    }

    async fn insert(&self, link: &PatientLink) -> Result<(), ClickCareError> {
        toasty::sql::statement(
            "insert into administration.patient_link \
               (id, network_id, organization_id, patient_id, user_id, linked_by) \
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(link.id)
        .bind(link.network_id)
        .bind(link.organization_id)
        .bind(link.patient_id)
        .bind(link.user_id)
        .bind(link.linked_by)
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al vincular el registro", e))?;
        Ok(())
    }

    async fn close(
        &self,
        organization_id: &Uuid,
        link_id: &Uuid,
        unlinked_by: &Uuid,
        reason: &str,
    ) -> Result<Option<Uuid>, ClickCareError> {
        let rows = toasty::sql::query(
            "update administration.patient_link \
               set unlinked_by = $3, unlinked_at = now(), unlink_reason = $4 \
             where id = $1 and organization_id = $2 and unlinked_at is null \
             returning user_id",
        )
        .bind(*link_id)
        .bind(*organization_id)
        .bind(*unlinked_by)
        .bind(reason.to_string())
        .column_types([Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al deshacer el vínculo", e))?;
        first_uuid_column(rows)
    }

    async fn mark_document_verified(
        &self,
        patient_id: &Uuid,
        verified_by: &Uuid,
    ) -> Result<(), ClickCareError> {
        toasty::sql::statement(
            "update administration.patient \
               set document_verified_at = now(), document_verified_by = $2 where id = $1",
        )
        .bind(*patient_id)
        .bind(*verified_by)
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| failure("Error al verificar el documento del paciente", e))?;
        Ok(())
    }
}
