use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use crate::infrastructure::api::FILE_DESCRIPTOR_SET;
use crate::infrastructure::api::patient_service_server::PatientServiceServer;
use crate::infrastructure::patient_service::ClickCareImpl;

pub mod log;
pub mod patient_service;
mod user_service;

pub mod api {
    tonic::include_proto!("api");
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("api_descriptor");
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let clickcare = PatientServiceServer::new(ClickCareImpl::default());
    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .expect("Could not build server");

    Server::builder()
        .layer(GrpcWebLayer::new())
        .accept_http1(true)
        .add_service(clickcare)
        // .add_service(reflection_server_v1)
        .add_service(reflection_server)
        .serve(addr)
        .await?;

    Ok(())
}
