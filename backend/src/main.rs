use tonic::*;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;

use medical_back::infraestructure::api::*;
use medical_back::infraestructure::api::patient_service_server::*;

#[derive(Default)]
struct PatientServiceImpl;

#[tonic::async_trait]
impl PatientService for PatientServiceImpl {
    async fn search_patient(&self, request: Request<PatientRequest>) -> Result<Response<PatientResponse>, Status> {
        let patientResponse = PatientResponse {
            first_name: Some("Hector Miuler".to_string()),
            last_name: Some("Malpica".to_string()),
            second_last_name: Some("Gallegos".to_string()),
            email: Some("miuler@gmail.com".to_string()),
        };
        Ok(Response::new(patientResponse))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let addr = "0.0.0.0:9000".parse().unwrap();
    let reflectionServer = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let patient_service_server = PatientServiceServer::new(PatientServiceImpl::default());

    Server::builder()
        .accept_http1(true)
        .add_service(reflectionServer)
        .add_service(tonic_web::enable(patient_service_server))
        .serve(addr)
        .await?;

    Ok(())
}

