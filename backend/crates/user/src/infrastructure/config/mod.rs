use crate::application::integration::service::token_service::TokenService;
use crate::application::use_case::complete_profile_usecase::CompleteProfileUseCase;
use crate::application::use_case::create_user_usecase::CreateUserUseCase;
use crate::application::use_case::sign_in_usecase::SignInUseCase;
use crate::domain::user::user_repository::UserRepository;
use crate::infrastructure::integration::event::apalis_publisher::ApalisIntegrationEventPublisherImpl;
use crate::infrastructure::integration::event::logging_publisher::LoggingIntegrationEventPublisherImpl;
use crate::infrastructure::integration::service::token_service_impl::TokenServiceImpl;
use crate::infrastructure::repository::user_repository_impl::{UserAccount, UserRepositoryImpl};
use app_core::application::integration::event_publisher::IntegrationEventPublisher;
use app_core::domain::error::ClickCareError;
use std::env::var;
use std::sync::Arc;
use toasty::{Db, models};
use tracing::error;
use tracing::trace;

// ─── DI container ───────────────────────────────────────────────────────────

/// Contenedor de Inyección de Dependencias para el Bounded Context de Usuario e Identidad.
///
/// Expone los casos de uso principales (`CreateUserUseCase`, `SignInUseCase`), el repositorio
/// de usuarios y el publicador de eventos de integración (`IntegrationEventPublisher`).
pub struct DI {
    /// Caso de uso para registrar y autenticar nuevos usuarios en el sistema.
    pub create_user_use_case: Arc<CreateUserUseCase>,
    /// Caso de uso para autenticar usuarios existentes mediante su token OAuth/OIDC.
    pub sign_in_use_case: Arc<SignInUseCase>,
    /// Caso de uso para completar los datos obligatorios de una cuenta existente.
    pub complete_profile_use_case: Arc<CompleteProfileUseCase>,
    #[allow(dead_code)]
    /// Puerto de repositorio para persistencia de cuentas de usuario.
    pub user_repository: Arc<dyn UserRepository>,
    /// Puerto para publicar eventos de dominio (ej. `UserCreatedEvent`).
    pub event_publisher: Arc<dyn IntegrationEventPublisher>,
    pub token_service: Arc<dyn TokenService>,
}

// ─── Overrides: solo los repos/servicios que quieres mockear en tests ────────

/// Sobrescrituras para sustituir dependencias específicas en pruebas unitarias o de integración.
///
/// Con `DBType::Mock` no hay base de datos, así que el repositorio y el servicio de tokens
/// deben venir de aquí. Los dobles de prueba están en `crate::testing` (feature `test-support`).
#[derive(Default)]
pub struct DIOverrides {
    /// Repositorio alternativo o mock para usuarios.
    pub user_repository: Option<Arc<dyn UserRepository>>,
    /// Publicador alternativo o mock para eventos de dominio.
    pub event_publisher: Option<Arc<dyn IntegrationEventPublisher>>,
    /// Servicio alternativo o mock para validación de tokens JWT.
    pub token_service: Option<Arc<dyn TokenService>>,
}

// ─── Constructores ──────────────────────────────────────────────────────────

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

    // ── token_service ──────────────────────────────────────────────────────
    let token_service: Arc<dyn TokenService> =
        match (overrides.token_service, database_url.as_deref()) {
            (Some(token_service), _) => token_service,
            (None, Some(_)) => Arc::new(TokenServiceImpl::new()),
            (None, None) => {
                return Err(ClickCareError::generic(
                    "DBType::Mock requiere un TokenService en DIOverrides".to_string(),
                ));
            }
        };

    // ── user_repository ──────────────────────────────────────────────────────
    let user_repository: Arc<dyn UserRepository> = match overrides.user_repository {
        Some(repository) => repository,
        None => build_user_repository(database_url.as_deref()).await?,
    };

    // ── event_publisher ──────────────────────────────────────────────────────
    let event_publisher: Arc<dyn IntegrationEventPublisher> =
        if let Some(publisher) = overrides.event_publisher {
            publisher
        } else {
            match database_url.as_deref() {
                Some(url) => Arc::new(ApalisIntegrationEventPublisherImpl::new(url).await?),
                // Sin base de datos (`DBType::Mock`) la cola no existe: se degrada a log.
                None => Arc::new(LoggingIntegrationEventPublisherImpl),
            }
        };

    // ── use cases ────────────────────────────────────────────────────────────
    let create_user_use_case = Arc::new(CreateUserUseCase {
        user_repository: Arc::clone(&user_repository),
        event_publisher: Arc::clone(&event_publisher),
    });
    let sign_in_use_case = Arc::new(SignInUseCase {
        user_repository: Arc::clone(&user_repository),
        token_service: Arc::clone(&token_service),
    });
    let complete_profile_use_case = Arc::new(CompleteProfileUseCase {
        user_repository: Arc::clone(&user_repository),
        token_service: Arc::clone(&token_service),
    });

    Ok(DI {
        create_user_use_case,
        sign_in_use_case,
        complete_profile_use_case,
        user_repository,
        event_publisher,
        token_service,
    })
}

// ─── Helpers privados ───────────────────────────────────────────────────────

/// Resuelve la URL de Postgres a partir de la configuración `DBType`.
/// `None` significa que no hay base de datos física y las dependencias deben sustituirse por overrides.
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

/// Construye la instancia de `UserRepository` conectada a Toasty PostgreSQL.
async fn build_user_repository(
    database_url: Option<&str>,
) -> Result<Arc<dyn UserRepository>, ClickCareError> {
    let Some(database_url_str) = database_url else {
        return Err(ClickCareError::generic(
            "DBType::Mock requiere un UserRepository en DIOverrides".to_string(),
        ));
    };

    trace!("URL de la base de datos: {database_url_str}");
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
    /// Sin base de datos: las dependencias de persistencia y de tokens deben llegar por `DIOverrides`.
    Mock,
}
