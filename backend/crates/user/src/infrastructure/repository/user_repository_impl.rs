use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use log::{error, info};
use sqlx::postgres::PgPool;
use toasty::Db;

pub(crate) struct UserRepositoryImpl {
    pub(crate) pool: PgPool,
    pub(crate) db: Db,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn exist_user(&self, _user_id: &str) -> Result<bool, ClickCareError> {
        // FIXME: Agregar la consulta sql
        let _exist = toasty::sql::query("select 1 from user_account limit 1")
            // .bind(user_id)
            .exec(&mut self.db.clone())
            .await.expect("FIXME: CORREGIRME!!!!");
        error!("Resultado: {:?}", _exist);
        todo!();
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
