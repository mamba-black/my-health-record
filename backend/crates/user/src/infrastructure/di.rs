use crate::application::{CreateUserUseCase, CreateUserUseCaseImpl};
use crate::domain::user::User;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::emitter_impl::EmitterImpl;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;
use app_core::domain::error::ClickCareError;
use sqlx::PgPool;
use std::env::var;
use async_trait::async_trait;
use tokio::sync::broadcast::channel;
use crate::domain::repository::user_repository::UserRepository;

pub async fn new(dbtype: DBType) -> Result<Box<dyn CreateUserUseCase>, ClickCareError> {
    let (sender, _receiver) = channel::<User>(100);

    let user_repository: Box<dyn UserRepository + Send + Sync> = match dbtype {
        DBType::Postgres => {
            let url = var("PG_URL").unwrap_or("postgres://user:password@localhost:5432".to_string());
            let pool = PgPool::connect(url.as_str()).await.map_err(|e| {
                ClickCareError::generic(format!("Error en la conexion a la DB [{}] ({})", url, e))
            })?;
            Box::new(UserRepositoryImpl { pool })
        }
        DBType::Mock => {
            Box::new(MockUserRepositoryImpl{})
        }
    };


    let create_user_use_case = CreateUserUseCaseImpl {
        user_repository,
        clinic_repository: Box::new(ClinicRepositoryImpl {
            user_emitter: Box::new(EmitterImpl { sender }),
        }),
    };

    Ok(Box::new(create_user_use_case))
}

pub enum DBType {
    Postgres,
    Mock
}

struct MockUserRepositoryImpl;

#[async_trait]
impl UserRepository for MockUserRepositoryImpl {
    async fn exist_user(&self, _user_id: &str) -> Result<bool, ClickCareError> {
        Ok(false)
    }

    async fn find_user_by_id(&self, _user_id: &str) -> Result<User, ClickCareError> {
        Err(ClickCareError::generic("Mock error".to_string()))
    }

    async fn save_user(&self, _user: &User) -> Result<(), ClickCareError> {
        Ok(())
    }
}
