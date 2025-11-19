use fake::Fake;
use fake::faker::chrono::raw::*;
use fake::faker::internet::raw::*;
use fake::faker::name::raw::*;
use fake::faker::phone_number::raw::PhoneNumber;
use fake::locales::EN;
use tonic::*;
use ulid::Ulid;
use crate::infrastructure::api::patient_service_server::*;
use crate::infrastructure::api::*;

#[derive(Default)]
pub struct ClickCareImpl;

#[async_trait]
impl PatientService for ClickCareImpl {
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
                    birthdate: Date(EN).fake(),
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
                    birthdate: Date(EN).fake(),
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
                    birthdate: Date(EN).fake(),
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
                    birthdate: Date(EN).fake(),
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
                    birthdate: Date(EN).fake(),
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
                    birthdate: Date(EN).fake(),
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
                    birthdate: Date(EN).fake(),
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


#[cfg(test)]
mod test {
}
