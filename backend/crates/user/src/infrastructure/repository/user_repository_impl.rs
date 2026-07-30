use crate::domain::repository::user_repository::UserRepository;use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use log::{debug, error};
use sqlx::postgres::PgPool;
use toasty::{Db, Error};

pub(crate) struct UserRepositoryImpl {
    pub(crate) pool: PgPool,
    pub(crate) db: Db,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn exist_user_by_document(&self, document_type: &str, document_value: &str) -> Result<bool, ClickCareError> {
        debug!("user_id: {}", document_value);
        // FIXME: Corregir esto para buscar por documento
        let exist = toasty::sql::query("select 1 from user_account where document_type = ?1 and document_value = ?2")
            .bind(document_type)
            .bind(document_value)
            .exec(&mut self.db.clone())
            .await
            .map_err(|e| {
                error!("Error desconocido al tratar de buscar el user_account con id:{} ({})", document_value, e);
                ClickCareError::generic(e.to_string())
            })?;
        error!("Resultado: {:?}", exist);
        Ok(exist.len() > 0)
    }
    async fn find_user_by_id(&self, _user_id: &str) -> Result<User, ClickCareError> {
        // Aquí iría la lógica para buscar un usuario en la base de datos
        // let a = query_as!(User, "SELECT * FROM user WHERE id = $1", user_id)
        //     .fetch_one(self.pool)
        //     .await?;
        // Ok(a)
        todo!();
    }

    async fn save_user(&self, _user: &User) -> Result<(), ClickCareError> {
        // Aquí iría la lógica para guardar un usuario en la base de datos
        todo!();
    }
}
