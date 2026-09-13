use crate::infrastructure::grpc::FILE_DESCRIPTOR_SET;
use crate::infrastructure::grpc::clinic_api_impl::ClinicApiImpl;
use crate::infrastructure::grpc::clinic_api_server::ClinicApiServer;
use crate::infrastructure::grpc::patient_api_impl::PatientApiImpl;
use crate::infrastructure::grpc::patient_api_server::PatientApiServer;
use crate::infrastructure::grpc::subdomain_resolver::SubdomainResolver;
use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
use crate::infrastructure::grpc::user_api_server::UserApiServer;
use administration::infrastructure::di as administration_di;
use apalis_board::axum::framework::ApiBuilder;
use apalis_board::axum::ui::ServeUI;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;
use tonic_web::GrpcWebLayer;
use tracing::info;

pub mod cli;
pub mod grpc;
pub mod log;

/// Inicia el servidor gRPC y gRPC-Web de ClickCare junto con los workers de eventos en segundo plano.
///
/// # Parámetros
/// - `database_url`: Cadena de conexión a la base de datos PostgreSQL (ej. `"postgres://user:pass@localhost:5432"`).
/// - `enable_administration_worker`: Si es `true`, ejecuta el worker de administración para procesar eventos en segundo plano.
pub async fn start_server(
    database_url: Option<String>,
    enable_administration_worker: bool,
) -> Result<(), ClickCareError> {
    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .expect("Could not build server");

    let administration =
        administration_di::new(administration_di::DBType::Postgres(database_url.clone())).await?;
    let subdomain_resolver = Arc::new(SubdomainResolver::new(Arc::clone(
        &administration.state.organization_repository,
    )));

    let patient_service_server = PatientApiServer::new(PatientApiImpl::default());
    let user_service_server = UserApiServer::new(
        UserApiImpl::new(database_url, Arc::clone(&subdomain_resolver)).await?,
    );
    let clinic_service_server = ClinicApiServer::new(ClinicApiImpl::new(Arc::clone(
        &administration.create_clinic_use_case,
    )));

    let apalis_board_router: axum::Router = ApiBuilder::new(axum::Router::<()>::new()).build();

    let grpc_router: axum::Router = tonic::service::Routes::default()
        .add_service(patient_service_server)
        .add_service(user_service_server)
        .add_service(clinic_service_server)
        .add_service(reflection_server)
        .into_axum_router()
        .layer(GrpcWebLayer::new());

    let app = grpc_router
        .merge(apalis_board_router)
        .fallback_service(ServeUI::new());

    let listener = {
        let addr: std::net::SocketAddr = "[::1]:50051".parse().map_err(|e| {
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
        }
    } else {
        info!("Iniciando servidor gRPC/Web (worker de administración deshabilitado)...");
        server
            .await
            .map_err(|e| ClickCareError::generic(format!("Error al iniciar el servidor: {}", e)))?;
    }

    Ok(())
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
        let patient_service_server = PatientApiServer::new(PatientApiImpl::default());
        let apalis_board_router: axum::Router = ApiBuilder::new(axum::Router::<()>::new()).build();
        let grpc_router: axum::Router = tonic::service::Routes::default()
            .add_service(patient_service_server)
            .into_axum_router()
            .layer(GrpcWebLayer::new());

        let _app = grpc_router
            .merge(apalis_board_router)
            .fallback_service(ServeUI::new());
    }
}
