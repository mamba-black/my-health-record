use crate::application::event_handlers::handle_user_created_event;
use crate::application::state::AdministrationState;
use crate::domain::repository::organization_repository::OrganizationRepository;
use crate::domain::repository::patient_repository::PatientRepository;
use crate::domain::repository::practitioner_repository::PractitionerRepository;
use crate::infrastructure::repository::organization_repository_impl::{
    OrganizationRecord, OrganizationRepositoryImpl,
};
use crate::infrastructure::repository::patient_repository_impl::{
    PatientRecord, PatientRepositoryImpl,
};
use crate::infrastructure::repository::practitioner_repository_impl::{
    PractitionerRecord, PractitionerRepositoryImpl,
};
use apalis::layers::WorkerBuilderExt;
use apalis::layers::retry::RetryPolicy;
use apalis::prelude::{Data, WorkerBuilder};
use apalis_postgres::{Config, PgPool, PostgresStorage};
use app_core::domain::error::ClickCareError;
use app_core::domain::event::UserCreatedEvent;
use std::env::var;
use std::sync::Arc;
use toasty::{Db, models};
use tracing::{debug, info};

/// Número de reintentos ante un fallo del handler antes de marcar la tarea como fallida.
const MAX_RETRIES: usize = 3;

/// Identificador del worker, visible en la tabla `apalis.workers`.
const WORKER_NAME: &str = "administration-worker";

/// Origen de datos del contexto acotado.
pub enum DBType {
    /// `None` toma la URL de la variable de entorno `PG_URL`.
    Postgres(Option<String>),
}

// ─── DI container ────────────────────────────────────────────────────────────

pub struct DI {
    storage: PostgresStorage<UserCreatedEvent>,
    state: AdministrationState,
}

// ─── Constructores ───────────────────────────────────────────────────────────

/// Construye el DI del contexto acotado con implementaciones reales.
///
/// Abre **dos conexiones independientes** a la misma base de datos: una exclusiva
/// de la cola de eventos y otra para los repositorios de entidades. Encolar o
/// consumir un evento nunca comparte pool ni transacción con la persistencia del
/// agregado.
pub async fn new(dbtype: DBType) -> Result<DI, ClickCareError> {
    let url = resolve_db_url(dbtype);
    debug!("URL de la base de datos: {url}");

    let storage = build_event_storage(&url).await?;
    let state = build_state(&url).await?;

    Ok(DI { storage, state })
}

impl DI {
    /// Corre el worker que consume [`UserCreatedEvent`] hasta que el proceso termine.
    ///
    /// El worker se construye aquí dentro a propósito: así el tipo genérico que
    /// produce [`WorkerBuilder`] nunca cruza la frontera del crate y quien llama
    /// solo necesita conocer `DI`.
    ///
    /// El handler se envuelve en un cierre para que Apalis extraiga el estado y se
    /// lo entregue por referencia. De ese modo `handle_user_created_event` sigue
    /// siendo una función de aplicación pura, sin conocer [`Data`] ni Apalis.
    pub async fn run_worker(self) -> Result<(), ClickCareError> {
        info!(
            "Iniciando '{WORKER_NAME}' sobre la cola '{}'",
            UserCreatedEvent::QUEUE
        );

        WorkerBuilder::new(WORKER_NAME)
            .backend(self.storage)
            .data(self.state)
            .retry(RetryPolicy::retries(MAX_RETRIES))
            .build(
                |event: UserCreatedEvent, state: Data<AdministrationState>| async move {
                    handle_user_created_event(event, &state).await
                },
            )
            .run()
            .await
            .map_err(|error| {
                ClickCareError::generic(format!("Error al ejecutar '{WORKER_NAME}' ({error})"))
            })
    }
}

// ─── Helpers privados ────────────────────────────────────────────────────────

/// Resuelve la URL de Postgres del `DBType`.
fn resolve_db_url(dbtype: DBType) -> String {
    match dbtype {
        DBType::Postgres(Some(url)) => url,
        DBType::Postgres(None) => {
            var("PG_URL").unwrap_or("postgres://user:password@localhost:5432".to_string())
        }
    }
}

/// Abre el pool exclusivo de la cola y prepara el schema `apalis`.
async fn build_event_storage(
    url: &str,
) -> Result<PostgresStorage<UserCreatedEvent>, ClickCareError> {
    let pool = PgPool::connect(url).await.map_err(|error| {
        ClickCareError::generic(format!(
            "Error en la conexion a la DB de la cola de eventos ({error})"
        ))
    })?;

    // Crea el schema `apalis` si aún no existe (migraciones embebidas del crate).
    PostgresStorage::setup(&pool).await.map_err(|error| {
        ClickCareError::generic(format!("Error al preparar el schema de Apalis ({error})"))
    })?;

    let config = Config::new(UserCreatedEvent::QUEUE);
    Ok(PostgresStorage::new_with_config(&pool, &config))
}

/// Abre la conexión de los repositorios de entidades y arma el estado del worker.
async fn build_state(url: &str) -> Result<AdministrationState, ClickCareError> {
    let db: Db = toasty::Db::builder()
        .models(models!(
            OrganizationRecord,
            PatientRecord,
            PractitionerRecord
        ))
        .connect(url)
        .await
        .map_err(|error| {
            ClickCareError::generic(format!(
                "Error en la conexion a la Toasty DB [{url}] ({error})"
            ))
        })?;

    let organization_repository: Arc<dyn OrganizationRepository> =
        Arc::new(OrganizationRepositoryImpl { db: db.clone() });
    let patient_repository: Arc<dyn PatientRepository> =
        Arc::new(PatientRepositoryImpl { db: db.clone() });
    let practitioner_repository: Arc<dyn PractitionerRepository> =
        Arc::new(PractitionerRepositoryImpl { db });

    Ok(AdministrationState {
        organization_repository,
        patient_repository,
        practitioner_repository,
    })
}
