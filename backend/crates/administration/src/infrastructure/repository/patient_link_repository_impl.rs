use crate::domain::patient::patient_link_repository::{PatientLink, PatientLinkRepository};
use crate::infrastructure::repository::patient_repository_impl::PatientRecord;
use crate::infrastructure::repository::{db_error, uuid_column};
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::Type;
use uuid::Uuid;

/// Fila de la tabla `administration.patient_link`.
///
/// El índice único parcial `uq_patient_link_active` (un vínculo activo por registro)
/// y `idx_patient_link_user` los crea V8; toasty no expresa índices parciales, así que
/// aquí solo se declara el buscador por `patient_id`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "patient_link"]
pub struct PatientLinkRecord {
    #[key]
    pub id: uuid::Uuid,
    pub network_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    #[index]
    pub patient_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub linked_by: uuid::Uuid,
    #[default(jiff::Timestamp::now())]
    pub linked_at: jiff::Timestamp,
    pub unlinked_by: Option<uuid::Uuid>,
    pub unlinked_at: Option<jiff::Timestamp>,
    pub unlink_reason: Option<String>,
}

impl From<PatientLinkRecord> for PatientLink {
    fn from(record: PatientLinkRecord) -> Self {
        PatientLink {
            id: record.id,
            network_id: record.network_id,
            organization_id: record.organization_id,
            patient_id: record.patient_id,
            user_id: record.user_id,
            linked_by: record.linked_by,
        }
    }
}

pub(crate) struct PatientLinkRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl PatientLinkRepository for PatientLinkRepositoryImpl {
    async fn record_of_account(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError> {
        PatientRecord::filter_by_organization_id(*organization_id)
            .filter(PatientRecord::fields().user_id().eq(Some(*user_id)))
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(|record| record.id))
            .map_err(db_error(
                "Error al buscar el registro de la cuenta".to_string(),
            ))
    }

    async fn records_of_account(
        &self,
        network_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<Uuid>, ClickCareError> {
        // SQL crudo por necesidad: join con organization y unión con los vínculos.
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
        .map_err(db_error(
            "Error al buscar los registros de la cuenta".to_string(),
        ))?;
        uuid_column(rows)
    }

    async fn account_of(&self, patient_id: &Uuid) -> Result<Option<Uuid>, ClickCareError> {
        // SQL crudo por necesidad: left join con el vínculo activo y `coalesce`.
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
        .map_err(db_error(
            "Error al buscar la cuenta del registro".to_string(),
        ))?;
        Ok(uuid_column(rows)?.into_iter().next())
    }

    async fn active_link(&self, patient_id: &Uuid) -> Result<Option<PatientLink>, ClickCareError> {
        PatientLinkRecord::filter_by_patient_id(*patient_id)
            .filter(PatientLinkRecord::fields().unlinked_at().is_none())
            .first()
            .exec(&mut self.db.clone())
            .await
            .map(|record| record.map(PatientLink::from))
            .map_err(db_error(
                "Error al leer el vínculo del registro".to_string(),
            ))
    }

    async fn insert(&self, link: &PatientLink) -> Result<(), ClickCareError> {
        toasty::create!(PatientLinkRecord {
            id: link.id,
            network_id: link.network_id,
            organization_id: link.organization_id,
            patient_id: link.patient_id,
            user_id: link.user_id,
            linked_by: link.linked_by,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error("Error al vincular el registro".to_string()))?;
        Ok(())
    }

    async fn close(
        &self,
        organization_id: &Uuid,
        link_id: &Uuid,
        unlinked_by: &Uuid,
        reason: &str,
    ) -> Result<Option<Uuid>, ClickCareError> {
        let fields = PatientLinkRecord::fields();
        let open_link = || {
            PatientLinkRecord::filter_by_id(*link_id)
                .filter(fields.organization_id().eq(*organization_id))
                .filter(fields.unlinked_at().is_none())
        };
        let mut db = self.db.clone();
        let Some(link) = open_link()
            .first()
            .exec(&mut db)
            .await
            .map_err(db_error("Error al leer el vínculo a deshacer".to_string()))?
        else {
            return Ok(None);
        };
        // El filtro repite `unlinked_at is null`: un cierre concurrente no se pisa.
        open_link()
            .update()
            .unlinked_by(Some(*unlinked_by))
            .unlinked_at(Some(jiff::Timestamp::now()))
            .unlink_reason(Some(reason.to_string()))
            .exec(&mut db)
            .await
            .map_err(db_error("Error al deshacer el vínculo".to_string()))?;
        Ok(Some(link.user_id))
    }

    async fn mark_document_verified(
        &self,
        patient_id: &Uuid,
        verified_by: &Uuid,
    ) -> Result<(), ClickCareError> {
        PatientRecord::filter_by_id(*patient_id)
            .update()
            .document_verified_at(Some(jiff::Timestamp::now()))
            .document_verified_by(Some(*verified_by))
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(
                "Error al verificar el documento del paciente".to_string(),
            ))?;
        Ok(())
    }
}
