//! Migraciones versionadas del esquema con [refinery](https://docs.rs/refinery).
//!
//! Los archivos `V{n}__descripcion.sql` de `backend/migrations` se embeben en el binario
//! al compilar y se aplican en orden al arrancar el servidor. Cada migración corre en
//! su propia transacción, y las ya aplicadas se registran en [`MIGRATION_TABLE`].
//!
//! La tabla de control es propia y está calificada con `public`: `apalis-postgres`
//! guarda sus migraciones en `_sqlx_migrations`, y sin el esquema explícito la tabla
//! caería en `identity`, el primero del `search_path` de la base.

use app_core::domain::error::ClickCareError;
use std::str::FromStr;
use std::sync::Arc;
use tokio_postgres_rustls::MakeRustlsConnect;
use tracing::{info, warn};

refinery::embed_migrations!("../../migrations");

/// Tabla donde refinery registra las migraciones aplicadas.
pub const MIGRATION_TABLE: &str = "public.refinery_schema_history";

/// Aplica las migraciones pendientes sobre la base de `database_url`.
///
/// Respeta el `sslmode` de la URL: con `require` (Neon) negocia TLS con los
/// certificados raíz del sistema, y sin él (contenedor local) lo intenta y, si el
/// servidor no lo ofrece, sigue sin cifrar.
///
/// # Errors
/// Devuelve [`ClickCareError`] si la URL no es válida, no hay conexión o alguna
/// migración falla. En ese caso la migración fallida se revierte entera.
pub async fn run(database_url: &str) -> Result<(), ClickCareError> {
    run_until(database_url, None).await
}

/// Igual que [`run`], pero se detiene en la versión `target` (inclusive) si se indica.
/// Sirve a las pruebas que verifican el relleno de datos de una migración.
pub async fn run_until(database_url: &str, target: Option<i32>) -> Result<(), ClickCareError> {
    let config = tokio_postgres::Config::from_str(database_url).map_err(|e| {
        ClickCareError::generic(format!("URL de base de datos inválida para migrar: {e}"))
    })?;

    let (mut client, connection) = config
        .connect(tls_connector()?)
        .await
        .map_err(|e| ClickCareError::generic(format!("No se pudo conectar para migrar: {e}")))?;

    let connection_task = tokio::spawn(async move {
        if let Err(e) = connection.await {
            warn!("La conexión de migraciones terminó con error: {e}");
        }
    });

    let mut runner = migrations::runner();
    runner.set_migration_table_name(MIGRATION_TABLE);
    let runner = match target {
        Some(version) => runner.set_target(refinery::Target::Version(version)),
        None => runner,
    };
    let result = runner.run_async(&mut client).await;

    drop(client);
    let _ = connection_task.await;

    let report = result.map_err(|e| ClickCareError::generic(format!("Falló la migración: {e}")))?;

    let applied = report.applied_migrations();
    if applied.is_empty() {
        info!("Esquema al día: no hay migraciones pendientes");
    }
    for migration in applied {
        info!("Migración aplicada: {migration}");
    }

    Ok(())
}

/// Conector TLS con los certificados raíz del sistema.
///
/// El proveedor criptográfico se fija explícitamente porque en el árbol de
/// dependencias conviven `ring` y `aws-lc-rs`, y `ClientConfig::builder()` entra en
/// pánico si no puede elegir uno por defecto.
fn tls_connector() -> Result<MakeRustlsConnect, ClickCareError> {
    let mut roots = rustls::RootCertStore::empty();
    let native = rustls_native_certs::load_native_certs();
    for error in &native.errors {
        warn!("No se pudo cargar un certificado raíz del sistema: {error}");
    }
    let (added, _ignored) = roots.add_parsable_certificates(native.certs);
    if added == 0 {
        warn!("No se cargó ningún certificado raíz: las conexiones TLS fallarán");
    }

    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| ClickCareError::generic(format!("Configuración TLS inválida: {e}")))?
    .with_root_certificates(roots)
    .with_no_client_auth();

    Ok(MakeRustlsConnect::new(config))
}
