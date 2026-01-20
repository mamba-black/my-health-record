use crate::infrastructure::api::patient_service::ClickCareImpl;
use crate::infrastructure::api::patient_service_server::PatientServiceServer;
use crate::infrastructure::api::user_service::UserServiceImpl;
use crate::infrastructure::api::user_service_server::UserServiceServer;
use crate::infrastructure::api::FILE_DESCRIPTOR_SET;
use app_core::domain::error::ClickCareError;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

pub(crate) mod log;
pub(crate) mod api;

pub async fn start_server() -> Result<(), ClickCareError> {
    let addr = "[::1]:50051"
        .parse()
        .map_err(|e| ClickCareError::generic(format!("Error al parsear la direccion del servidor: {}", e)))?;

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .expect("Could not build server");

    let patient_service_server = PatientServiceServer::new(ClickCareImpl::default());
    let user_service_server = UserServiceServer::new(UserServiceImpl::new().await?);

    Server::builder()
        .layer(GrpcWebLayer::new())
        .accept_http1(true)
        .add_service(patient_service_server)
        .add_service(user_service_server)
        .add_service(reflection_server)
        .serve(addr)
        .await
        .map_err(|e| ClickCareError::generic(format!("Error al iniciar el servidor: {}", e)))?;

    Ok(())
}

