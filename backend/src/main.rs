use tonic::transport::Server;
use tonic::*;
// use tonic_web::GrpcWebLayer;

use medical_back::infraestructure::api::patient_service_server::*;
use medical_back::infraestructure::api::*;

#[derive(Default)]
struct PatientServiceImpl;

#[tonic::async_trait]
impl PatientService for PatientServiceImpl {
    async fn search_patient(
        &self,
        request: Request<SearchPatientRequest>,
    ) -> Result<Response<SearchPatientResponse>, Status> {
        let search_patient_response = SearchPatientResponse {
            first_name: Some("Hector Miuler".to_string()),
            last_name: Some("Malpica".to_string()),
            second_last_name: Some("Gallegos".to_string()),
            email: Some("miuler@gmail.com".to_string()),
            patients: vec![
                PatientResponse {
                    id: "1".to_string(),
                    first_name: "Hector Miuler".to_string(),
                    last_name: "Malpica".to_string(),
                    second_last_name: Some("Gallegos".to_string()),
                    email: Some("miuler@gmail.com".to_string()),
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Co-Founder / CEO".to_string()),
                },
                PatientResponse {
                    id: "2".to_string(),
                    first_name: "Leslie".to_string(),
                    last_name: "Alexander".to_string(),
                    second_last_name: None,
                    email: Some("leslie.alexander@example.com".to_string()),
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Co-Founder / CEO".to_string()),
                },
                PatientResponse {
                    id: "3".to_string(),
                    first_name: "Michael".to_string(),
                    last_name: " Foster".to_string(),
                    second_last_name: None,
                    email: Some("michael.foster@example.com".to_string()),
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1519244703995-f4e0f30006d5?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Co-Founder / CEO".to_string()),
                },
                PatientResponse {
                    id: "4".to_string(),
                    first_name: "Dries".to_string(),
                    last_name: "Vincent".to_string(),
                    second_last_name: None,
                    email: Some("dries.vincent@example.com".to_string()),
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1506794778202-cad84cf45f1d?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Business Relations".to_string()),
                },
                PatientResponse {
                    id: "5".to_string(),
                    first_name: "Lindsay".to_string(),
                    last_name: "Walton".to_string(),
                    second_last_name: None,
                    email: Some("lindsay.walton@example.com".to_string()),
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1517841905240-472988babdf9?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Designer".to_string()),
                },
                PatientResponse {
                    id: "6".to_string(),
                    first_name: "Courtney".to_string(),
                    last_name: "Henry".to_string(),
                    second_last_name: None,
                    email: Some("courtney.henry@example.com".to_string()),
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Front-end Developer".to_string()),
                },
                PatientResponse {
                    id: "7".to_string(),
                    first_name: "Tom".to_string(),
                    last_name: "Cook".to_string(),
                    second_last_name: None,
                    email: None,
                    phone_number: None,
                    icon: Some("https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("empresa".to_string()),
                },
            ],
        };

        Ok(Response::new(search_patient_response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let addr = "0.0.0.0:9000".parse().unwrap();
    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let patient_service_server = PatientServiceServer::new(PatientServiceImpl::default());

    Server::builder()
        .accept_http1(true)
        .add_service(reflection_server)
        .add_service(tonic_web::enable(patient_service_server))
        .serve(addr)
        .await?;

    Ok(())
}
