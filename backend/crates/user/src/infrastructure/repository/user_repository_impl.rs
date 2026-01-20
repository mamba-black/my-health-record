use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use sqlx::postgres::PgPool;

pub(crate) struct UserRepositoryImpl {
    pub(crate) pool: PgPool,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError> {
        // Aquí iría la lógica para buscar un usuario en la base de datos
        // let a = query_as!(User, "SELECT * FROM user WHERE id = $1", user_id)
        //     .fetch_one(self.pool)
        //     .await?;
        // Ok(a)
        Err(ClickCareError::generic("Prueba".to_string()))
    }

    async fn save_user(&self, _user: &User) -> Result<(), ClickCareError> {
        // Aquí iría la lógica para guardar un usuario en la base de datos
        Ok(())
    }
}
