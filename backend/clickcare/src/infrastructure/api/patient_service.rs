use tonic::*;
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
                Patient::default(),
                Patient::default(),
                Patient::default(),
                Patient::default(),
                Patient::default(),
                Patient::default(),
                Patient::default(),
            ],
        };

        Ok(Response::new(search_patient_response))
    }

    async fn get_patient_by_id(
        &self,
        request: Request<PatientIdRequest>,
    ) -> Result<Response<Patient>, Status> {
        let patient_information = Patient::default();
        Ok(Response::new(patient_information))
    }

    async fn save(
        &self,
        request: Request<Patient>,
    ) -> Result<Response<Patient>, Status> {
        Ok(Response::new(request.into_inner()))
    }
}


#[cfg(test)]
mod test {
}
