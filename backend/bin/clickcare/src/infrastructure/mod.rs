use crate::infrastructure::grpc::FILE_DESCRIPTOR_SET;
use crate::infrastructure::grpc::clinic_api_impl::ClinicApiImpl;
use crate::infrastructure::grpc::clinic_api_server::ClinicApiServer;
use crate::infrastructure::grpc::patient_api_impl::PatientApiImpl;
use crate::infrastructure::grpc::patient_api_server::PatientApiServer;
use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
use crate::infrastructure::grpc::user_api_server::UserApiServer;
use administration::infrastructure::di as administration_di;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tracing::info;

pub mod grpc;
pub mod log;

pub async fn start_server(url: Option<String>) -> Result<(), ClickCareError> {
    let addr = "[::1]:50051".parse().map_err(|e| {
        ClickCareError::generic(format!("Error al parsear la direccion del servidor: {}", e))
    })?;

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .expect("Could not build server");

    let patient_service_server = PatientApiServer::new(PatientApiImpl::default());
    let user_service_server = UserApiServer::new(UserApiImpl::new(url.clone()).await?);
    let administration = administration_di::new(administration_di::DBType::Postgres(url)).await?;
    let clinic_service_server = ClinicApiServer::new(ClinicApiImpl::new(Arc::clone(
        &administration.create_clinic_use_case,
    )));

    let server = Server::builder()
        .layer(GrpcWebLayer::new())
        .accept_http1(true)
        .add_service(patient_service_server)
        .add_service(user_service_server)
        .add_service(clinic_service_server)
        .add_service(reflection_server)
        .serve_with_shutdown(addr, shutdown_signal());

    // Si cualquiera de los dos termina, el proceso completo baja de forma ordenada
    // en lugar de quedar sirviendo gRPC sin worker (o al revés).
    tokio::select! {
        result = server => result
            .map_err(|e| ClickCareError::generic(format!("Error al iniciar el servidor: {}", e)))?,
        result = administration.run_worker() => result?,
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
