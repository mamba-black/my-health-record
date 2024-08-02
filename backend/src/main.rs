use fake::faker::chrono::raw::*;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::raw::*;
use fake::faker::phone_number::raw::PhoneNumber;
use fake::locales::EN;
use fake::Fake;
use tonic::transport::Server;
use tonic::*;
use ulid::Ulid;

use medical_back::infraestructure::api::patient_service_server::*;
use medical_back::infraestructure::api::*;

// use tonic_web::GrpcWebLayer;

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
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Co-Founder / CEO".to_string()),
                },
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Co-Founder / CEO".to_string()),
                },
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1519244703995-f4e0f30006d5?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Co-Founder / CEO".to_string()),
                },
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1506794778202-cad84cf45f1d?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Business Relations".to_string()),
                },
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1517841905240-472988babdf9?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Designer".to_string()),
                },
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("Front-end Developer".to_string()),
                },
                PatientInformation {
                    id: Ulid::new().to_string(),
                    first_name: FirstName(EN).fake(),
                    last_name: LastName(EN).fake(),
                    second_last_name: Some(LastName(EN).fake()),
                    birthdate: DateTime(EN).fake(),
                    email: Some(FreeEmail(EN).fake()),
                    phone_number: Some(PhoneNumber(EN).fake()),
                    icon: Some("https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
                    note: Some("empresa".to_string()),
                },
            ],
        };

        Ok(Response::new(search_patient_response))
    }

    async fn get_patient_by_id(
        &self,
        request: Request<PatientIdRequest>,
    ) -> Result<Response<PatientInformation>, Status> {
        let patient_information = PatientInformation {
            id: "1".to_string(),
            first_name: "Hector Miuler".to_string(),
            last_name: "Malpica".to_string(),
            second_last_name: Some("Gallegos".to_string()),
            birthdate: "".to_string(),
            email: Some("miuler@gmail.com".to_string()),
            phone_number: None,
            icon: Some("https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            note: Some("Co-Founder / CEO".to_string()),
        };
        Ok(Response::new(patient_information))
    }

    async fn save(
        &self,
        request: Request<PatientInformation>,
    ) -> Result<Response<PatientInformation>, Status> {
        Ok(Response::new(request.into_inner()))
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

#[cfg(test)]
mod test {
    use fluvio_jolt::{transform, TransformSpec};
    use serde_json::{json, Value};

    #[test]
    fn test() {
        let input: Value = serde_json::from_str(
            r#"
        {
            "id": 1,
            "name": "John Smith",
            "accounts": [
                {
                    "id": 1000,
                    "type": "Checking"
                }
            ]
        }
        "#,
        )
        .unwrap();

        let out = r#"[
            {
                "operation": "shift",
                "spec": {
                    "accounts": {
                        "*": {
                            "id": "accounts[&(1)].id",
                            "type": "accounts[&(1)].type",
                            "@(2,name)": "accounts[&(1)].name"
                        }
                    }
                }
            }
        ]"#;

        let spec: TransformSpec = serde_json::from_str(out).unwrap();

        let output = transform(input, &spec).unwrap();

        assert_eq!(
            output,
            json!({
                  "accounts": [{
                    "id": 1000,
                    "type": "Checking",
                    "name": "John Smith"
                  }]
            })
        );
    }
}
