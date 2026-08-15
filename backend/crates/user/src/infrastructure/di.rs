use crate::application::{CreateUserUseCase, CreateUserUseCaseImpl};
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use app_core::domain::fhir::Identifier;

use crate::infrastructure::event::apalis_publisher::ApalisEventPublisher;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::emitter_impl::EmitterImpl;
use crate::infrastructure::repository::user_repository_impl::{UserAccount, UserRepositoryImpl};
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{EventPublisher, LoggingEventPublisher};
use async_trait::async_trait;
use log::error;
use std::env::var;
use std::sync::Arc;
use toasty::{Db, models};
use tokio::sync::Mutex;
use tokio::sync::broadcast::channel;
use tracing::debug;

// ─── DI container ────────────────────────────────────────────────────────────

pub struct DI {
    pub create_user_use_case: Arc<dyn CreateUserUseCase>,
    #[allow(dead_code)]
    pub user_repository: Arc<dyn UserRepository>,
    pub clinic_repository: Arc<dyn ClinicRepository>,
    pub event_publisher: Arc<dyn EventPublisher>,
}

// ─── Overrides: solo los repos/servicios que quieres mockear en tests ────────

#[derive(Default)]
pub struct DIOverrides {
    pub user_repository: Option<Arc<dyn UserRepository>>,
    pub clinic_repository: Option<Arc<dyn ClinicRepository>>,
    pub event_publisher: Option<Arc<dyn EventPublisher>>,
}

// ─── Constructores ───────────────────────────────────────────────────────────

/// Construye el DI completo con implementaciones reales.
pub async fn new(dbtype: DBType) -> Result<DI, ClickCareError> {
    new_with_overrides(dbtype, DIOverrides::default()).await
}

/// Construye el DI usando implementaciones reales, pero sustituye
/// solo aquellas dependencias presentes en `overrides`.
pub async fn new_with_overrides(
    dbtype: DBType,
    overrides: DIOverrides,
) -> Result<DI, ClickCareError> {
    // La URL se resuelve una sola vez y se reparte entre los consumidores. Cada uno
    // abre su propia conexión: la cola de eventos nunca comparte pool ni transacción
    // con los repositorios de entidades.
    let db_url = resolve_db_url(&dbtype);

    // ── user_repository ──────────────────────────────────────────────────────
    let user_repository: Arc<dyn UserRepository> = if let Some(repo) = overrides.user_repository {
        repo
    } else {
        build_user_repository(db_url.as_deref()).await?
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

    // ── event_publisher ──────────────────────────────────────────────────────
    let event_publisher: Arc<dyn EventPublisher> = if let Some(publ) = overrides.event_publisher {
        publ
    } else {
        match db_url.as_deref() {
            Some(url) => Arc::new(ApalisEventPublisher::new(url).await?),
            // Sin base de datos (`DBType::Mock`) la cola no existe: se degrada a log.
            None => Arc::new(LoggingEventPublisher),
        }
    };

    // ── use cases ────────────────────────────────────────────────────────────
    let create_user_use_case = Arc::new(CreateUserUseCaseImpl {
        user_repository: Arc::clone(&user_repository),
        clinic_repository: Arc::clone(&clinic_repository),
        event_publisher: Arc::clone(&event_publisher),
    });

    Ok(DI {
        create_user_use_case,
        user_repository,
        clinic_repository,
        event_publisher,
    })
}

// ─── Helpers privados ────────────────────────────────────────────────────────

/// Resuelve la URL de Postgres del `DBType`. `None` significa que no hay base de
/// datos y las dependencias deben degradarse a sus variantes en memoria.
fn resolve_db_url(dbtype: &DBType) -> Option<String> {
    match dbtype {
        DBType::Postgres(Some(url)) => Some(url.clone()),
        DBType::Postgres(None) => {
            Some(var("PG_URL").unwrap_or("postgres://user:password@localhost:5432".to_string()))
        }
        DBType::Mock => None,
    }
}

async fn build_user_repository(
    db_url: Option<&str>,
) -> Result<Arc<dyn UserRepository>, ClickCareError> {
    let Some(url) = db_url else {
        return Ok(Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        }));
    };

    debug!("URL de la base de datos: {url}");
    let db: Db = toasty::Db::builder()
        .models(models!(UserAccount))
        .connect(url)
        .await
        .map_err(|e| {
            error!("Error al crear el Pool para toasty: {e}");
            ClickCareError::generic(format!(
                "Error en la conexion a la Toasty DB [{}] ({})",
                url, e
            ))
        })?;

    Ok(Arc::new(UserRepositoryImpl { db }))
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
    async fn exist_user_by_document(
        &self,
        document_type: &str,
        document_value: &str,
    ) -> Result<bool, ClickCareError> {
        let users = self.saved_users.lock().await;
        let exists = users.iter().any(|u| {
            if let Some(Identifier::DNI(ref val)) = u.person.identifier {
                document_type == "DNI" && val == document_value
            } else {
                false
            }
        });

        Ok(exists)
    }

    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError> {
        let users = self.saved_users.lock().await;
        users
            .iter()
            .find(|u| u.id.to_string() == user_id)
            .cloned()
            .ok_or_else(|| {
                ClickCareError::generic(format!(
                    "User with ID {user_id} not found in mock repository"
                ))
            })
    }

    async fn save_user(&self, user: &User) -> Result<(), ClickCareError> {
        let mut users = self.saved_users.lock().await;
        users.push(user.clone());
        Ok(())
    }
}
