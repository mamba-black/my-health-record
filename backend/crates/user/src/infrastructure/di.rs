use std::env::var;
use crate::application::CreateUserUseCase;
use crate::domain::user::User;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::emitter_impl::EmitterImpl;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;
use sqlx::PgPool;
use tokio::sync::broadcast::channel;
use app_core::domain::error::ClickCareError;
use app_core::domain::error::ClickCareError::GenericError;

impl CreateUserUseCase {
    pub async fn new() -> Result<Self, ClickCareError> {
        let (sender, _receiver) = channel::<User>(100);

        let url = var("PG_URL")
            .unwrap_or("postgres://user:password@localhost:5432".to_string());
        let pool = PgPool::connect(url.as_str())
            .await
            .map_err(|e| ClickCareError::generic(format!("Error en la conexion a la DB [{}] ({})", url, e)))?;

        Ok(Self {
            user_repository: Box::new(UserRepositoryImpl{ pool }),
            clinic_repository: Box::new(ClinicRepositoryImpl {
                user_emitter: Box::new(EmitterImpl { sender }),
            }),
        })
    }
}

