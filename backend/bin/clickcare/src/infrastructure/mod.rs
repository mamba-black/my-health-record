use crate::infrastructure::grpc::FILE_DESCRIPTOR_SET;
use crate::infrastructure::grpc::access::{AccessContextBuilder, Authenticator};
use crate::infrastructure::grpc::clinic_api_impl::ClinicApiImpl;
use crate::infrastructure::grpc::clinic_api_server::ClinicApiServer;
use crate::infrastructure::grpc::patient_api_impl::PatientApiImpl;
use crate::infrastructure::grpc::patient_api_server::PatientApiServer;
use crate::infrastructure::grpc::public_scheduling_api_server::PublicSchedulingApiServer;
use crate::infrastructure::grpc::scheduling_api_impl::{
    PublicSchedulingApiImpl, SchedulingApiImpl,
};
use crate::infrastructure::grpc::scheduling_api_server::SchedulingApiServer;
use crate::infrastructure::grpc::subdomain_resolver::SubdomainResolver;
use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
use crate::infrastructure::grpc::user_api_server::UserApiServer;
use crate::infrastructure::scheduling as scheduling_adapters;
use administration::infrastructure::config as administration_di;
use apalis_board::axum::framework::{ApiBuilder, RegisterRoute};
use apalis_board::axum::ui::ServeUI;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;
use tonic_web::GrpcWebLayer;
use tracing::info;

pub mod cli;
pub mod grpc;
pub mod log;
pub mod migration;
pub mod scheduling;

/// Inicia el servidor gRPC y gRPC-Web de ClickCare junto con los workers de eventos en segundo plano.
///
/// Antes de construir las dependencias aplica las migraciones pendientes del esquema
/// (ver [`migration`]), así que la base queda al día en cada arranque.
///
/// # Parámetros
/// - `database_url`: Cadena de conexión a la base de datos PostgreSQL (ej. `"postgres://user:pass@localhost:5432"`).
/// - `enable_administration_worker`: Si es `true`, ejecuta el worker de administración para procesar eventos en segundo plano.
pub async fn start_server(
    database_url: Option<String>,
    enable_administration_worker: bool,
) -> Result<(), ClickCareError> {
    let reflection_server_v1alpha = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .expect("Could not build server");

    let reflection_server_v1 = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("Could not build server");

    match database_url
        .clone()
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .or_else(|| std::env::var("PG_URL").ok())
    {
        Some(url) => migration::run(&url).await?,
        None => tracing::warn!("Sin DATABASE_URL ni PG_URL: se omiten las migraciones del esquema"),
    }

    let services = build_services(
        database_url,
        user::infrastructure::config::DIOverrides::default(),
    )
    .await?;
    let administration = services.administration;
    let authenticator = services.authenticator;
    let patient_service_server =
        PatientApiServer::with_interceptor(services.patient_api, authenticator.interceptor());
    let user_service_server = UserApiServer::new(services.user_api);
    let clinic_service_server =
        ClinicApiServer::with_interceptor(services.clinic_api, authenticator.interceptor());
    let scheduling_service_server =
        SchedulingApiServer::with_interceptor(services.scheduling.api, authenticator.interceptor());
    let public_scheduling_server = PublicSchedulingApiServer::new(services.scheduling.public_api);
    let reminders = services.scheduling.reminders;
    let reminder_storage = services.scheduling.reminder_storage;

    let apalis_api = ApiBuilder::new(axum::Router::<()>::new())
        .register(administration.storage())
        .register(reminder_storage.clone())
        .build();
    let apalis_board_router: axum::Router = axum::Router::new().nest("/api/v1", apalis_api);

    let grpc_router: axum::Router = tonic::service::Routes::default()
        .add_service(patient_service_server)
        .add_service(user_service_server)
        .add_service(clinic_service_server)
        .add_service(scheduling_service_server)
        .add_service(public_scheduling_server)
        .add_service(reflection_server_v1alpha)
        .add_service(reflection_server_v1)
        .into_axum_router()
        .layer(GrpcWebLayer::new());

    let app = grpc_router
        .merge(apalis_board_router)
        .fallback_service(ServeUI::new())
        .layer(axum::middleware::from_fn(grpc_web_cors_middleware));

    let listener = {
        let port = std::env::var("CLICKCARE_PORT").unwrap_or_else(|_| "50051".to_string());
        let addr: std::net::SocketAddr = format!("0.0.0.0:{port}").parse().map_err(|e| {
            ClickCareError::generic(format!("Error al parsear la direccion del servidor: {}", e))
        })?;

        tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            ClickCareError::generic(format!("Error al enlazar la direccion del servidor: {}", e))
        })?
    };

    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    if enable_administration_worker {
        info!("Iniciando servidor gRPC/Web y worker de administración...");
        tokio::select! {
            result = server => result
                .map_err(|e| ClickCareError::generic(format!("Error al iniciar el servidor: {}", e)))?,
            result = administration.run_worker() => result?,
            result = SchedulingServices::run_reminders(reminders, reminder_storage) => result?,
        }
    } else {
        info!("Iniciando servidor gRPC/Web (worker de administración deshabilitado)...");
        server
            .await
            .map_err(|e| ClickCareError::generic(format!("Error al iniciar el servidor: {}", e)))?;
    }

    Ok(())
}

/// Agenda: sus APIs, el worker de recordatorios y su cola.
pub struct SchedulingServices {
    pub public_api: PublicSchedulingApiImpl,
    pub api: SchedulingApiImpl,
    pub reminders: Arc<::scheduling::application::reminder_usecase::ReminderUseCases>,
    pub reminder_storage:
        apalis_postgres::PostgresStorage<::scheduling::infrastructure::worker::ReminderDue>,
}

impl SchedulingServices {
    /// Corre el worker de recordatorios y, cada minuto, el barrido que reencola los
    /// vencidos y reenvía los que quedaron colgados (antes repetido que perdido).
    pub async fn run_reminders(
        reminders: Arc<::scheduling::application::reminder_usecase::ReminderUseCases>,
        storage: apalis_postgres::PostgresStorage<
            ::scheduling::infrastructure::worker::ReminderDue,
        >,
    ) -> Result<(), ClickCareError> {
        let sweeper = Arc::clone(&reminders);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(error) = sweeper.sweep().await {
                    tracing::warn!("Barrido de recordatorios fallido: {error}");
                }
            }
        });
        ::scheduling::infrastructure::worker::run_reminder_worker(storage, reminders).await
    }
}

/// Parámetros de la agenda desde el entorno.
fn scheduling_config() -> ::scheduling::application::state::SchedulingConfig {
    let mut config = ::scheduling::application::state::SchedulingConfig::default();
    let read = |name: &str| {
        std::env::var(name)
            .ok()
            .and_then(|v| v.trim().parse::<i64>().ok())
    };
    if let Some(hours) = read("REMINDER_LEAD_HOURS") {
        config.reminder_lead_hours = hours.max(0);
    }
    if let Some(attempts) = read("REMINDER_MAX_ATTEMPTS") {
        config.reminder_max_attempts = i16::try_from(attempts.max(1)).unwrap_or(3);
    }
    if let Some(minutes) = read("REMINDER_SENDING_TIMEOUT") {
        config.reminder_sending_timeout_minutes = minutes.max(1);
    }
    config
}

/// Servicios gRPC ya cableados, compartidos por el servidor y el arnés de pruebas.
pub struct Services {
    pub scheduling: SchedulingServices,
    pub administration: administration_di::DI,
    pub authenticator: Authenticator,
    pub user_api: UserApiImpl,
    pub clinic_api: ClinicApiImpl,
    pub patient_api: PatientApiImpl,
}

/// Arma las dependencias de los contextos acotados y las APIs gRPC.
///
/// `user_overrides` permite a las pruebas sustituir el verificador de Google y la clave
/// de sesión.
pub async fn build_services(
    database_url: Option<String>,
    user_overrides: user::infrastructure::config::DIOverrides,
) -> Result<Services, ClickCareError> {
    let resolved_url = database_url
        .clone()
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .or_else(|| std::env::var("PG_URL").ok())
        .unwrap_or_else(|| "postgres://user:password@localhost:5432".to_string());
    let user_di = user::infrastructure::config::new_with_overrides(
        user::infrastructure::config::DBType::Postgres(database_url.clone()),
        user_overrides,
    )
    .await?;
    // `Person.link` es del contexto `user`; administración lo usa por su puerto.
    let administration = administration_di::new(
        administration_di::DBType::Postgres(database_url),
        Arc::clone(&user_di.person_links),
    )
    .await?;
    let subdomain_resolver = Arc::new(SubdomainResolver::new(Arc::clone(
        &administration.state.organization_repository,
    )));

    let authenticator = Authenticator::new(Arc::clone(&user_di.session_service));
    let access = Arc::new(AccessContextBuilder::new(
        Arc::clone(&user_di.user_repository),
        Arc::clone(&administration.membership_use_cases),
    ));

    let user_api = UserApiImpl::new(
        &user_di,
        authenticator.clone(),
        Arc::clone(&access),
        Arc::clone(&administration.membership_use_cases),
        Arc::clone(&administration.root_organization_use_case),
        Arc::clone(&subdomain_resolver),
    );
    let clinic_api = ClinicApiImpl::new(
        Arc::clone(&administration.create_clinic_use_case),
        Arc::clone(&administration.membership_use_cases),
        Arc::clone(&access),
        Arc::clone(&user_di.user_repository),
    );
    let patient_api = PatientApiImpl::new(
        Arc::clone(&administration.state.patient_repository),
        Arc::new(
            administration::application::patient_usecase::PatientUseCases::new(
                administration.state.clone(),
            ),
        ),
        Arc::clone(&administration.membership_use_cases),
        Arc::clone(&access),
        subdomain_resolver,
    );

    let scheduling = build_scheduling(&resolved_url, &administration, &user_di, &access).await?;

    Ok(Services {
        scheduling,
        administration,
        authenticator,
        user_api,
        clinic_api,
        patient_api,
    })
}

/// Arma la agenda: sus repositorios, la cola de recordatorios y los adaptadores de sus
/// puertos sobre `administration` y `user`.
async fn build_scheduling(
    resolved_url: &str,
    administration: &administration_di::DI,
    user_di: &user::infrastructure::config::DI,
    access: &Arc<AccessContextBuilder>,
) -> Result<SchedulingServices, ClickCareError> {
    let scheduling_err = |e: ::scheduling::domain::error::SchedulingError| {
        ClickCareError::generic(format!("Error al preparar la agenda ({e})"))
    };
    let repositories = ::scheduling::infrastructure::config::repositories(resolved_url)
        .await
        .map_err(scheduling_err)?;
    let reminder_storage = ::scheduling::infrastructure::worker::reminder_storage(resolved_url)
        .await
        .map_err(scheduling_err)?;
    let patient_use_cases = Arc::new(
        administration::application::patient_usecase::PatientUseCases::new(
            administration.state.clone(),
        ),
    );
    let scheduling_state = ::scheduling::application::state::SchedulingState {
        schedules: repositories.schedules,
        appointments: repositories.appointments,
        reminders: repositories.reminders,
        clinics: Arc::new(scheduling_adapters::ClinicDirectoryAdapter {
            organizations: Arc::clone(&administration.state.organization_repository),
            memberships: Arc::clone(&administration.state.membership_repository),
            users: Arc::clone(&user_di.user_repository),
        }),
        patients: Arc::new(scheduling_adapters::PatientRegistryAdapter {
            patients: Arc::clone(&administration.state.patient_repository),
            links: Arc::clone(&administration.patient_links),
            patient_use_cases,
            users: Arc::clone(&user_di.user_repository),
        }),
        identity: Arc::new(scheduling_adapters::IdentityVerificationAdapter {
            users: Arc::clone(&user_di.user_repository),
            access: Arc::clone(access),
        }),
        notifier: scheduling_adapters::notification_sender()?,
        scheduler: Arc::new(
            ::scheduling::infrastructure::worker::ApalisReminderScheduler {
                storage: reminder_storage.clone(),
            },
        ),
        clock: Arc::new(::scheduling::application::integration::SystemClock),
        config: scheduling_config(),
    };
    let schedule_use_cases = Arc::new(
        ::scheduling::application::schedule_usecase::ScheduleUseCases::new(
            scheduling_state.clone(),
        ),
    );
    let scheduling = SchedulingServices {
        public_api: PublicSchedulingApiImpl::new(Arc::clone(&schedule_use_cases)),
        api: SchedulingApiImpl::new(
            schedule_use_cases,
            Arc::new(
                ::scheduling::application::booking_usecase::BookingUseCases::new(
                    scheduling_state.clone(),
                ),
            ),
            Arc::new(
                ::scheduling::application::reception_usecase::ReceptionUseCases::new(
                    scheduling_state.clone(),
                ),
            ),
            Arc::clone(access),
        ),
        reminders: Arc::new(
            ::scheduling::application::reminder_usecase::ReminderUseCases::new(scheduling_state),
        ),
        reminder_storage,
    };
    Ok(scheduling)
}

/// Middleware CORS para peticiones gRPC-Web desde el navegador (ej. `http://localhost:5173`).
async fn grpc_web_cors_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if request.method() == axum::http::Method::OPTIONS {
        let mut preflight_response = axum::response::Response::new(axum::body::Body::empty());
        *preflight_response.status_mut() = axum::http::StatusCode::NO_CONTENT;
        insert_grpc_web_cors_headers(preflight_response.headers_mut());
        return preflight_response;
    }

    let mut response = next.run(request).await;
    insert_grpc_web_cors_headers(response.headers_mut());
    response
}

fn insert_grpc_web_cors_headers(headers: &mut axum::http::HeaderMap) {
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        axum::http::HeaderValue::from_static("*"),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_METHODS,
        axum::http::HeaderValue::from_static("POST, GET, OPTIONS"),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_HEADERS,
        axum::http::HeaderValue::from_static(
            "content-type, x-grpc-web, x-user-agent, x-subdomain, authorization",
        ),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_EXPOSE_HEADERS,
        axum::http::HeaderValue::from_static(
            "grpc-status, grpc-message, grpc-status-details-bin, grpc-encoding, grpc-accept-encoding",
        ),
    );
}

/// Espera a `Ctrl-C` para permitir un apagado ordenado del servidor gRPC.
async fn shutdown_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("Error al escuchar la señal de apagado: {e}");
        return;
    }
    info!("Señal de apagado recibida, deteniendo el servidor");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_merging_without_fallback_panic() {
        let apalis_board_router: axum::Router = ApiBuilder::new(axum::Router::<()>::new()).build();
        let grpc_router: axum::Router = tonic::service::Routes::default()
            .add_service(
                tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                    .build_v1()
                    .expect("reflexión"),
            )
            .into_axum_router()
            .layer(GrpcWebLayer::new());

        let _app = grpc_router
            .merge(apalis_board_router)
            .fallback_service(ServeUI::new());
    }
}
