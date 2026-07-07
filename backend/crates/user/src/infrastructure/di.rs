use crate::application::{CreateUserUseCase, CreateUserUseCaseImpl};
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::emitter_impl::EmitterImpl;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use sqlx::PgPool;
use std::env::var;
use std::sync::Arc;
use log::error;
use toasty::Db;
use tokio::sync::Mutex;
use tokio::sync::broadcast::channel;
use tracing::debug;
// ─── DI container ────────────────────────────────────────────────────────────

pub struct DI {
    pub create_user_use_case: Arc<dyn CreateUserUseCase>,
    user_repository: Arc<dyn UserRepository>,
}

// ─── Overrides: solo los repos/servicios que quieres mockear ─────────────────

/// Pasa aquí únicamente las dependencias que quieres sustituir.
/// Todo lo que dejes en `None` se construirá con la implementación real.
#[derive(Default)]
pub struct DIOverrides {
    pub user_repository: Option<Arc<dyn UserRepository>>,
    pub clinic_repository: Option<Arc<dyn ClinicRepository>>,
}

// ─── Constructores ───────────────────────────────────────────────────────────

/// Construye el DI completo con implementaciones reales.
pub async fn new(dbtype: DBType) -> Result<DI, ClickCareError> {
    new_with_overrides(dbtype, DIOverrides::default()).await
}

/// Construye el DI usando implementaciones reales, pero sustituye
/// solo aquellas dependencias presentes en `overrides`.
///
/// # Ejemplo en tests
/// ```rust
/// let di = new_with_overrides(
///     DBType::Postgres,
///     DIOverrides {
///         user_repository: Some(Arc::new(MockUserRepository::new())),
///         ..Default::default()
///     },
/// ).await?;
/// ```
pub async fn new_with_overrides(
    dbtype: DBType,
    overrides: DIOverrides,
) -> Result<DI, ClickCareError> {
    // ── user_repository ──────────────────────────────────────────────────────
    let user_repository: Arc<dyn UserRepository> = if let Some(repo) = overrides.user_repository {
        repo
    } else {
        build_user_repository(dbtype).await?
    };

    // ── clinic_repository ────────────────────────────────────────────────────
    let clinic_repository: Arc<dyn ClinicRepository> =
        if let Some(repo) = overrides.clinic_repository {
            repo
        } else {
            let (sender, _receiver) = channel::<User>(100);
            Arc::new(ClinicRepositoryImpl {
                user_emitter: Box::new(EmitterImpl { sender }),
            })
        };

    // ── use cases ────────────────────────────────────────────────────────────
    let create_user_use_case = Arc::new(CreateUserUseCaseImpl {
        user_repository: Arc::clone(&user_repository),
        clinic_repository,
    });

    Ok(DI {
        create_user_use_case,
        user_repository,
    })
}

// ─── Helpers privados ────────────────────────────────────────────────────────

async fn build_user_repository(dbtype: DBType) -> Result<Arc<dyn UserRepository>, ClickCareError> {
    let user_repository: Arc<dyn UserRepository> = match dbtype {
        DBType::Postgres(Some(url)) => {
            debug!("URL de la base de datos: {url}");
            let pool = PgPool::connect(url.as_str()).await.map_err(|e| {
                error!("Error al crear el Pool para sqlx");
                ClickCareError::generic(format!("Error en la conexion a la DB [{}] ({})", url, e))
            })?;
            let db: Db = toasty::Db::builder().connect(url.as_str()).await.map_err(|e| {
                error!("Error al crear el Pool para toasty: {e}");
                ClickCareError::generic(format!("Error en la conexion a la Toasty DB [{}] ({})", url, e))
            })?;
            Arc::new(UserRepositoryImpl { pool, db })
        }
        DBType::Postgres(None) => {
            let url = var("PG_URL").unwrap_or("postgres://user:password@localhost:5432".to_string());
            let pool = PgPool::connect(url.as_str()).await.map_err(|e| {
                ClickCareError::generic(format!("Error en la conexion a la DB [{}] ({})", url, e))
            })?;
            let db: Db = toasty::Db::builder().connect(url.as_str()).await.map_err(|e| {
                ClickCareError::generic(format!("Error en la conexion a la Toasty DB [{}] ({})", url, e))
            })?;
            Arc::new(UserRepositoryImpl { pool, db })
        }
        DBType::Mock => Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        }),
    };

    Ok(user_repository)
}

pub enum DBType {
    Postgres(Option<String>),
    Mock,
}

pub struct MockUserRepositoryImpl {
    pub saved_users: Mutex<Vec<User>>,
}

#[async_trait]
impl UserRepository for MockUserRepositoryImpl {
    async fn exist_user(&self, _user_id: &str) -> Result<bool, ClickCareError> {
        Ok(false)
    }

    async fn find_user_by_id(&self, _user_id: &str) -> Result<User, ClickCareError> {
        Err(ClickCareError::generic("Mock error".to_string()))
    }

    async fn save_user(&self, _user: &User) -> Result<(), ClickCareError> {
        let mut users = self.saved_users.lock().await;
        users.push(_user.clone());
        Ok(())
    }
}
