use crate::infrastructure::grpc::patient_api_impl::PatientApiImpl;
use crate::infrastructure::grpc::patient_api_server::PatientApiServer;
use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
use crate::infrastructure::grpc::user_api_server::UserApiServer;
use crate::infrastructure::grpc::FILE_DESCRIPTOR_SET;
use app_core::domain::error::ClickCareError;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

pub(crate) mod log;
pub mod grpc;

pub async fn start_server(url: Option<String>) -> Result<(), ClickCareError> {
    let addr = "[::1]:50051"
        .parse()
        .map_err(|e| ClickCareError::generic(format!("Error al parsear la direccion del servidor: {}", e)))?;

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .expect("Could not build server");

    let patient_service_server = PatientApiServer::new(PatientApiImpl::default());
    let user_service_server = UserApiServer::new(UserApiImpl::new(url).await?);

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

