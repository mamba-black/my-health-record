use crate::domain::membership::Membership;
use crate::domain::membership::membership_repository::MembershipRepository;
use crate::infrastructure::repository::db_error;
use app_core::domain::access::{MembershipRole, MembershipStatus};
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use uuid::Uuid;

/// Fila de la tabla `administration.membership`.
///
/// Los atributos `#[unique]`/`#[index]` solo generan los buscadores (`filter_by_*`):
/// los índices los crean las migraciones de `backend/migrations/` y deben coincidir
/// (`uq_membership_org_user_role` e `idx_membership_user_id` en V4,
/// `idx_membership_org_id` en V6).
#[derive(Debug, Clone, toasty::Model)]
#[table = "membership"]
#[unique(organization_id, user_id, role)]
pub struct MembershipRecord {
    #[key]
    pub id: uuid::Uuid,
    #[index]
    pub organization_id: uuid::Uuid,
    #[index]
    pub user_id: uuid::Uuid,
    pub role: String,
    pub status: String,
    pub license_confirmed_at: Option<jiff::Timestamp>,
    pub license_confirmed_by: Option<uuid::Uuid>,
    pub activation_requested: bool,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}

impl TryFrom<MembershipRecord> for Membership {
    type Error = ClickCareError;

    fn try_from(record: MembershipRecord) -> Result<Self, Self::Error> {
        Ok(Membership {
            id: record.id,
            organization_id: record.organization_id,
            user_id: record.user_id,
            role: MembershipRole::from_code(&record.role).ok_or_else(|| {
                ClickCareError::generic(format!("Rol desconocido: {}", record.role))
            })?,
            status: MembershipStatus::from_code(&record.status).ok_or_else(|| {
                ClickCareError::generic(format!("Estado desconocido: {}", record.status))
            })?,
            license_confirmed: record.license_confirmed_at.is_some(),
            activation_requested: record.activation_requested,
        })
    }
}

fn to_memberships(records: Vec<MembershipRecord>) -> Result<Vec<Membership>, ClickCareError> {
    records.into_iter().map(Membership::try_from).collect()
}

pub(crate) struct MembershipRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl MembershipRepository for MembershipRepositoryImpl {
    async fn find_by_user(&self, user_id: &Uuid) -> Result<Vec<Membership>, ClickCareError> {
        MembershipRecord::filter_by_user_id(*user_id)
            .order_by(MembershipRecord::fields().created_at().asc())
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al consultar las membresías de user_id={user_id}"
            )))
            .and_then(to_memberships)
    }

    async fn find_by_organization(
        &self,
        organization_id: &Uuid,
    ) -> Result<Vec<Membership>, ClickCareError> {
        MembershipRecord::filter_by_organization_id(*organization_id)
            .order_by(MembershipRecord::fields().created_at().asc())
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al consultar las membresías de la clínica {organization_id}"
            )))
            .and_then(to_memberships)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Membership>, ClickCareError> {
        MembershipRecord::filter_by_id(*id)
            .first()
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!("Error al consultar la membresía id={id}")))?
            .map(Membership::try_from)
            .transpose()
    }

    async fn find(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        role: MembershipRole,
    ) -> Result<Option<Membership>, ClickCareError> {
        MembershipRecord::filter_by_organization_id_and_user_id_and_role(
            *organization_id,
            *user_id,
            role.code().to_string(),
        )
        .first()
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al consultar la membresía {} de user_id={user_id} en la clínica {organization_id}",
            role.code()
        )))?
        .map(Membership::try_from)
        .transpose()
    }

    async fn count_active_owners(&self, organization_id: &Uuid) -> Result<usize, ClickCareError> {
        let fields = MembershipRecord::fields();
        let owners = MembershipRecord::all()
            .filter(fields.organization_id().eq(*organization_id))
            .filter(fields.role().eq(MembershipRole::Owner.code().to_string()))
            .filter(
                fields
                    .status()
                    .eq(MembershipStatus::Active.code().to_string()),
            )
            .count()
            .exec(&mut self.db.clone())
            .await
            .map_err(db_error(format!(
                "Error al contar los dueños activos de la clínica {organization_id}"
            )))?;
        usize::try_from(owners).map_err(|error| ClickCareError::generic(error.to_string()))
    }

    async fn insert(&self, membership: &Membership) -> Result<(), ClickCareError> {
        toasty::create!(MembershipRecord {
            id: membership.id,
            organization_id: membership.organization_id,
            user_id: membership.user_id,
            role: membership.role.code().to_string(),
            status: membership.status.code().to_string(),
            activation_requested: membership.activation_requested,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(db_error(format!(
            "Error al crear la membresía id={}",
            membership.id
        )))?;
        Ok(())
    }

    async fn update(
        &self,
        membership: &Membership,
        confirmed_by: Option<Uuid>,
    ) -> Result<(), ClickCareError> {
        let mut db = self.db.clone();
        let mut record = MembershipRecord::filter_by_id(membership.id)
            .first()
            .exec(&mut db)
            .await
            .map_err(db_error(format!(
                "Error al consultar la membresía id={}",
                membership.id
            )))?
            .ok_or_else(|| {
                ClickCareError::generic(format!("No existe la membresía id={}", membership.id))
            })?;

        let first_confirmation =
            membership.license_confirmed && record.license_confirmed_at.is_none();
        let mut update = record.update();
        update.set_status(membership.status.code().to_string());
        update.set_activation_requested(membership.activation_requested);
        // La primera confirmación de la colegiatura fija la fecha; las siguientes no la mueven.
        if first_confirmation {
            update.set_license_confirmed_at(Some(jiff::Timestamp::now()));
        }
        if let Some(confirmed_by) = confirmed_by {
            update.set_license_confirmed_by(Some(confirmed_by));
        }
        update.exec(&mut db).await.map_err(db_error(format!(
            "Error al actualizar la membresía id={}",
            membership.id
        )))?;
        Ok(())
    }
}
