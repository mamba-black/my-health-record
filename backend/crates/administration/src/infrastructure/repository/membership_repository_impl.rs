use crate::domain::membership::Membership;
use crate::domain::membership::membership_repository::MembershipRepository;
use app_core::domain::access::{MembershipRole, MembershipStatus};
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::{Type, Value};
use tracing::error;
use uuid::Uuid;

/// Fila de la tabla `administration.membership`.
#[derive(Debug, Clone, toasty::Model)]
#[table = "membership"]
pub struct MembershipRecord {
    #[key]
    pub id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub role: String,
    pub status: String,
    pub activation_requested: bool,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct MembershipRepositoryImpl {
    pub(crate) db: Db,
}

const COLUMNS: &str = "select id, organization_id, user_id, role, status, \
     license_confirmed_at is not null, activation_requested from administration.membership";

fn column_types() -> [Type; 7] {
    [
        Type::Uuid,
        Type::Uuid,
        Type::Uuid,
        Type::String,
        Type::String,
        Type::Bool,
        Type::Bool,
    ]
}

fn map_row(row: Value) -> Result<Membership, ClickCareError> {
    let mut fields = row.into_record().fields.into_iter();
    let mut next = |name: &str| {
        fields.next().ok_or_else(|| {
            ClickCareError::generic(format!("Falta la columna {name} de membership"))
        })
    };
    let uuid =
        |value: Value| Uuid::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()));
    let text =
        |value: Value| String::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()));
    let boolean =
        |value: Value| bool::try_from(value).map_err(|e| ClickCareError::generic(e.to_string()));

    let id = uuid(next("id")?)?;
    let organization_id = uuid(next("organization_id")?)?;
    let user_id = uuid(next("user_id")?)?;
    let role_code = text(next("role")?)?;
    let status_code = text(next("status")?)?;
    Ok(Membership {
        id,
        organization_id,
        user_id,
        role: MembershipRole::from_code(&role_code)
            .ok_or_else(|| ClickCareError::generic(format!("Rol desconocido: {role_code}")))?,
        status: MembershipStatus::from_code(&status_code)
            .ok_or_else(|| ClickCareError::generic(format!("Estado desconocido: {status_code}")))?,
        license_confirmed: boolean(next("license_confirmed")?)?,
        activation_requested: boolean(next("activation_requested")?)?,
    })
}

impl MembershipRepositoryImpl {
    async fn query(
        &self,
        sql: String,
        binds: Vec<Value>,
    ) -> Result<Vec<Membership>, ClickCareError> {
        let mut query = toasty::sql::query(&sql);
        for value in binds {
            query = query.bind(value);
        }
        let rows = query
            .column_types(column_types())
            .exec(&mut self.db.clone())
            .await
            .map_err(|error| {
                error!("Error al consultar membresías: {error}");
                ClickCareError::generic(format!("Error al consultar membresías ({error})"))
            })?;
        rows.into_iter().map(map_row).collect()
    }
}

#[async_trait]
impl MembershipRepository for MembershipRepositoryImpl {
    async fn find_by_user(&self, user_id: &Uuid) -> Result<Vec<Membership>, ClickCareError> {
        self.query(
            format!("{COLUMNS} where user_id = $1 order by created_at"),
            vec![Value::from(*user_id)],
        )
        .await
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Membership>, ClickCareError> {
        Ok(self
            .query(format!("{COLUMNS} where id = $1"), vec![Value::from(*id)])
            .await?
            .into_iter()
            .next())
    }

    async fn find(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        role: MembershipRole,
    ) -> Result<Option<Membership>, ClickCareError> {
        Ok(self
            .query(
                format!("{COLUMNS} where organization_id = $1 and user_id = $2 and role = $3"),
                vec![
                    Value::from(*organization_id),
                    Value::from(*user_id),
                    Value::from(role.code().to_string()),
                ],
            )
            .await?
            .into_iter()
            .next())
    }

    async fn count_active_owners(&self, organization_id: &Uuid) -> Result<usize, ClickCareError> {
        Ok(self
            .query(
                format!(
                    "{COLUMNS} where organization_id = $1 and role = 'OWNER' and status = 'ACTIVE'"
                ),
                vec![Value::from(*organization_id)],
            )
            .await?
            .len())
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
        .map_err(|error| {
            error!("Error al crear la membresía id={}: {error}", membership.id);
            ClickCareError::generic(format!(
                "Error al crear la membresía id={} ({error})",
                membership.id
            ))
        })?;
        Ok(())
    }

    async fn update(
        &self,
        membership: &Membership,
        confirmed_by: Option<Uuid>,
    ) -> Result<(), ClickCareError> {
        toasty::sql::query(
            "update administration.membership set status = $1, activation_requested = $2, \
             license_confirmed_at = case when $3 then coalesce(license_confirmed_at, current_timestamp) \
             else license_confirmed_at end, \
             license_confirmed_by = coalesce((nullif($4::text, ''))::uuid, license_confirmed_by), \
             updated_at = current_timestamp where id = $5",
        )
        .bind(membership.status.code().to_string())
        .bind(membership.activation_requested)
        .bind(membership.license_confirmed)
        // Toasty no infiere el tipo de un `NULL` en SQL crudo: viaja como texto vacío.
        .bind(confirmed_by.map(|id| id.to_string()).unwrap_or_default())
        .bind(membership.id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al actualizar la membresía id={}: {error}", membership.id);
            ClickCareError::generic(format!(
                "Error al actualizar la membresía id={} ({error})",
                membership.id
            ))
        })?;
        Ok(())
    }
}
