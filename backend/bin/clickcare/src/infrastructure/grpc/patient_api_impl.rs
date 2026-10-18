use crate::infrastructure::grpc::patient_api_server::*;
use crate::infrastructure::grpc::*;
use tonic::*;

#[derive(Default)]
pub struct PatientApiImpl {}

#[async_trait]
impl PatientApi for PatientApiImpl {
    async fn search_patient(
        &self,
        request: Request<SearchPatientRequest>,
    ) -> Result<Response<SearchPatientResponse>, Status> {
        let search_patient_request = request.into_inner();
        let query_name = search_patient_request.name.clone().unwrap_or_default();

        let mock_patients = vec![
            Patient {
                id: "018f6f71-6c12-7801-b5cb-112233445566".to_string(),
                identifier: vec![],
                active: true,
                name: vec![HumanName {
                    r#use: "official".to_string(),
                    family: vec!["Malpica".to_string(), "Gallegos".to_string()],
                    given: vec![
                        if query_name.is_empty() {
                            "Hector".to_string()
                        } else {
                            query_name.clone()
                        },
                        "Miuler".to_string(),
                    ],
                }],
                telecom: vec![
                    ContactPoint {
                        system: contact_point::System::Email as i32,
                        value: "miuler@gmail.com".to_string(),
                        r#use: contact_point::Use::Home as i32,
                    },
                    ContactPoint {
                        system: contact_point::System::Phone as i32,
                        value: "+51999888777".to_string(),
                        r#use: contact_point::Use::Mobile as i32,
                    },
                ],
                gender: patient::Gender::Male as i32,
                birth_date: "1980-05-15".to_string(),
                deceased_status: None,
                managing_organization_reference: String::new(),
                photo: vec![],
            },
            Patient {
                id: "018f6f71-6c12-7802-b5cb-223344556677".to_string(),
                identifier: vec![],
                active: true,
                name: vec![HumanName {
                    r#use: "official".to_string(),
                    family: vec!["Picard".to_string()],
                    given: vec!["Jan".to_string(), "Luc".to_string()],
                }],
                telecom: vec![ContactPoint {
                    system: contact_point::System::Email as i32,
                    value: "jan.picard@starfleet.org".to_string(),
                    r#use: contact_point::Use::Work as i32,
                }],
                gender: patient::Gender::Male as i32,
                birth_date: "1940-07-13".to_string(),
                deceased_status: None,
                managing_organization_reference: String::new(),
                photo: vec![],
            },
            Patient {
                id: "018f6f71-6c12-7803-b5cb-334455667788".to_string(),
                identifier: vec![],
                active: true,
                name: vec![HumanName {
                    r#use: "official".to_string(),
                    family: vec!["Crusher".to_string()],
                    given: vec!["Beverly".to_string()],
                }],
                telecom: vec![ContactPoint {
                    system: contact_point::System::Email as i32,
                    value: "beverly.crusher@starfleet.org".to_string(),
                    r#use: contact_point::Use::Work as i32,
                }],
                gender: patient::Gender::Female as i32,
                birth_date: "1953-10-13".to_string(),
                deceased_status: None,
                managing_organization_reference: String::new(),
                photo: vec![],
            },
        ];

        let search_patient_response = SearchPatientResponse {
            first_name: search_patient_request.name,
            last_name: Some("Malpica".to_string()),
            second_last_name: Some("Gallegos".to_string()),
            email: Some("miuler@gmail.com".to_string()),
            patients: mock_patients,
        };

        Ok(Response::new(search_patient_response))
    }

    async fn get_patient_by_id(
        &self,
        _request: Request<PatientIdRequest>,
    ) -> Result<Response<Patient>, Status> {
        let patient_information = Patient::default();
        Ok(Response::new(patient_information))
    }

    async fn save(&self, request: Request<Patient>) -> Result<Response<Patient>, Status> {
        Ok(Response::new(request.into_inner()))
    }
}

#[cfg(test)]
mod test {}
