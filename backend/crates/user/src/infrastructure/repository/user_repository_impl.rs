use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use log::{debug, error};
use sqlx::postgres::PgPool;
use toasty::Db;

#[derive(Debug, Clone, toasty::Model)]
#[table = "user_account"]
pub struct UserAccount {
    #[key]
    pub id: uuid::Uuid,

    pub active: bool,

    pub is_owner: bool,

    pub provider_info: String,

    pub given_name: String,

    pub family_name: Option<String>,

    pub second_family_name: Option<String>,

    pub document_type: Option<String>,

    pub document_value: Option<String>,

    pub email: Option<String>,

    pub phone: Option<String>,

    #[auto]
    pub created_at: jiff::Timestamp,

    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct UserRepositoryImpl {
    #[allow(dead_code)]
    pub(crate) pool: PgPool,
    pub(crate) db: Db,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn exist_user_by_document(
        &self,
        document_type: &str,
        document_value: &str,
    ) -> Result<bool, ClickCareError> {
        debug!("user_id: {}", document_value);
        // FIXME: Corregir esto para buscar por documento
        let exist = toasty::sql::query(
            "select 1 from user_account where document_type = $1 and document_value = $2",
        )
        .bind(document_type)
        .bind(document_value)
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| {
            error!(
                "Error desconocido al tratar de buscar el user_account con id:{} ({})",
                document_value, e
            );
            ClickCareError::generic(e.to_string())
        })?;
        error!("Resultado: {:?}", exist);
        Ok(!exist.is_empty())
    }
    async fn find_user_by_id(&self, _user_id: &str) -> Result<User, ClickCareError> {
        // Aquí iría la lógica para buscar un usuario en la base de datos
        // let a = query_as!(User, "SELECT * FROM user WHERE id = $1", user_id)
        //     .fetch_one(self.pool)
        //     .await?;
        // Ok(a)
        todo!();
    }

    async fn save_user(&self, user: &User) -> Result<(), ClickCareError> {
        let given_name = user.person.name().given().join(" ");
        let family_name = user.person.name().family().clone();
        let second_family_name = user.person.name().second_family().clone();

        let (document_type, document_value) = match user.person.identifier() {
            Some(crate::domain::user::Identifier::DNI(val)) => {
                (Some("DNI".to_string()), Some(val.clone()))
            }
            None => (None, None),
        };

        let email = user
            .person
            .telecom()
            .iter()
            .find(|c| matches!(c.system, crate::domain::user::ContactPointSystem::Email))
            .map(|c| c.value.clone());

        let phone = user
            .person
            .telecom()
            .iter()
            .find(|c| matches!(c.system, crate::domain::user::ContactPointSystem::Phone))
            .map(|c| c.value.clone());

        let _user_account = toasty::create!(UserAccount {
            id: user.id,
            active: user.active,
            is_owner: user.is_owner,
            provider_info: "Google".to_string(),
            given_name,
            family_name,
            second_family_name,
            document_type,
            document_value,
            email,
            phone,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| {
            error!("Error al guardar user_account con id {}: {e}", user.id);
            ClickCareError::generic(format!("Error al guardar usuario en la base de datos: {e}"))
        })?;

        Ok(())
    }
}
