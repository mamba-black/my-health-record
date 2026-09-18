use crate::application::create_user_usecase::{CreateUserUseCase, CreateUserUseCaseImpl};
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use app_core::domain::fhir::Identifier;

use crate::infrastructure::event::apalis_publisher::ApalisEventPublisher;
use crate::infrastructure::repository::user_repository_impl::{UserAccount, UserRepositoryImpl};
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{EventPublisher, LoggingEventPublisher};
use async_trait::async_trait;
use std::env::var;
use std::sync::Arc;
use toasty::{Db, models};
use tokio::sync::Mutex;
use tracing::debug;
use tracing::error;

// ─── DI container ────────────────────────────────────────────────────────────

/// Contenedor de Inyección de Dependencias para el Bounded Context de Usuario e Identidad.
///
/// Expone los casos de uso principales (`CreateUserUseCase`), el repositorio de usuarios
/// y el publicador de eventos de dominio (`EventPublisher`).
pub struct DI {
    /// Caso de uso para registrar y autenticar nuevos usuarios en el sistema.
    pub create_user_use_case: Arc<CreateUserUseCase>,
    #[allow(dead_code)]
    /// Puerto de repositorio para persistencia de cuentas de usuario.
    pub user_repository: Arc<dyn UserRepository>,
    /// Puerto para publicar eventos de dominio (ej. `UserCreatedEvent`).
    pub event_publisher: Arc<dyn EventPublisher>,
}

// ─── Overrides: solo los repos/servicios que quieres mockear en tests ────────

/// Sobrescrituras para sustituir dependencias específicas en pruebas unitarias o de integración.
#[derive(Default)]
pub struct DIOverrides {
    /// Repositorio alternativo o mock para usuarios.
    pub user_repository: Option<Arc<dyn UserRepository>>,
    /// Publicador alternativo o mock para eventos de dominio.
    pub event_publisher: Option<Arc<dyn EventPublisher>>,
}

// ─── Constructores ───────────────────────────────────────────────────────────

/// Construye el contenedor DI de usuarios con implementaciones reales de base de datos.
pub async fn new(database_type: DBType) -> Result<DI, ClickCareError> {
    new_with_overrides(database_type, DIOverrides::default()).await
}

/// Construye el contenedor DI usando implementaciones reales, pero sustituyendo
/// las dependencias especificadas en `overrides`.
pub async fn new_with_overrides(
    database_type: DBType,
    overrides: DIOverrides,
) -> Result<DI, ClickCareError> {
    // La URL se resuelve una sola vez y se reparte entre los consumidores. Cada uno
    // abre su propia conexión: la cola de eventos nunca comparte pool ni transacción
    // con los repositorios de entidades.
    let database_url = resolve_db_url(&database_type);

    // ── user_repository ──────────────────────────────────────────────────────
    let user_repository: Arc<dyn UserRepository> =
        if let Some(repository) = overrides.user_repository {
            repository
        } else {
            build_user_repository(database_url.as_deref()).await?
        };

    // ── event_publisher ──────────────────────────────────────────────────────
    let event_publisher: Arc<dyn EventPublisher> =
        if let Some(publisher) = overrides.event_publisher {
            publisher
        } else {
            match database_url.as_deref() {
                Some(url) => Arc::new(ApalisEventPublisher::new(url).await?),
                // Sin base de datos (`DBType::Mock`) la cola no existe: se degrada a log.
                None => Arc::new(LoggingEventPublisher),
            }
        };

    // ── use cases ────────────────────────────────────────────────────────────
    let create_user_use_case = Arc::new(CreateUserUseCaseImpl {
        user_repository: Arc::clone(&user_repository),
        event_publisher: Arc::clone(&event_publisher),
    });

    Ok(DI {
        create_user_use_case,
        user_repository,
        event_publisher,
    })
}

// ─── Helpers privados ────────────────────────────────────────────────────────

/// Resuelve la URL de Postgres a partir de la configuración `DBType`.
/// `None` significa que no hay base de datos física y las dependencias deben degradarse a memoria.
fn resolve_db_url(database_type: &DBType) -> Option<String> {
    match database_type {
        DBType::Postgres(Some(database_url)) => Some(database_url.clone()),
        DBType::Postgres(None) => Some(
            var("DATABASE_URL")
                .or_else(|_| var("PG_URL"))
                .unwrap_or_else(|_| "postgres://user:password@localhost:5432".to_string()),
        ),
        DBType::Mock => None,
    }
}

/// Construye la instancia de `UserRepository` conectada a Toasty PostgreSQL o una versión en memoria.
async fn build_user_repository(
    database_url: Option<&str>,
) -> Result<Arc<dyn UserRepository>, ClickCareError> {
    let Some(database_url_str) = database_url else {
        return Ok(Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        }));
    };

    debug!("URL de la base de datos: {database_url_str}");
    let db: Db = toasty::Db::builder()
        .models(models!(UserAccount))
        .connect(database_url_str)
        .await
        .map_err(|error| {
            error!("Error al crear el Pool para toasty: {error}");
            ClickCareError::generic(format!(
                "Error en la conexion a la Toasty DB [{database_url_str}] ({error})"
            ))
        })?;

    Ok(Arc::new(UserRepositoryImpl { db }))
}

/// Tipo de backend de persistencia configurado para el contexto de usuario.
pub enum DBType {
    /// Conexión a PostgreSQL mediante cadena de conexión opcional (o variable de entorno `DATABASE_URL` / `PG_URL`).
    Postgres(Option<String>),
    /// Modo mock en memoria para pruebas rápidas sin dependencia de PostgreSQL.
    Mock,
}

/// Implementación en memoria de `UserRepository` para pruebas unitarias.
pub struct MockUserRepositoryImpl {
    /// Lista de usuarios almacenados en memoria durante el test.
    pub saved_users: Mutex<Vec<User>>,
}

#[async_trait]
impl UserRepository for MockUserRepositoryImpl {
    async fn exist_user_by_document(
        &self,
        network_id: &uuid::Uuid,
        identifier: Identifier,
    ) -> Result<bool, ClickCareError> {
        let users = self.saved_users.lock().await;

        let exists = users.iter().any(|user| {
            user.network_id == *network_id && user.person.identifier.as_ref() == Some(&identifier)
        });

        Ok(exists)
    }

    async fn exist_user_by_email(
        &self,
        network_id: &uuid::Uuid,
        email: &str,
    ) -> Result<bool, ClickCareError> {
        let users = self.saved_users.lock().await;

        let exists = users.iter().any(|user| {
            user.network_id == *network_id
                && user.person.telecom.iter().any(|t| {
                    t.system == app_core::domain::fhir::ContactPointSystem::Email
                        && t.value == email
                })
        });

        Ok(exists)
    }

    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError> {
        let users = self.saved_users.lock().await;
        users
            .iter()
            .find(|user| user.id.to_string() == user_id)
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
