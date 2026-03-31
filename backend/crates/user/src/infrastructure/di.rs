use crate::application::{CreateUserUseCase, CreateUserUseCaseImpl};
use crate::domain::user::User;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::emitter_impl::EmitterImpl;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;
use app_core::domain::error::ClickCareError;
use sqlx::PgPool;
use std::env::var;
use tokio::sync::broadcast::channel;

pub async fn new() -> Result<Box<dyn CreateUserUseCase>, ClickCareError> {
    let (sender, _receiver) = channel::<User>(100);

    let url = var("PG_URL").unwrap_or("postgres://user:password@localhost:5432".to_string());
    let pool = PgPool::connect(url.as_str()).await.map_err(|e| {
        ClickCareError::generic(format!("Error en la conexion a la DB [{}] ({})", url, e))
    })?;

    let create_user_use_case = CreateUserUseCaseImpl {
        user_repository: Box::new(UserRepositoryImpl { pool }),
        clinic_repository: Box::new(ClinicRepositoryImpl {
            user_emitter: Box::new(EmitterImpl { sender }),
        }),
    };

    Ok(Box::new(create_user_use_case))
}
