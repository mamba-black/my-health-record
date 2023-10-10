use leptos::{spawn_local, ReadSignal, RwSignal, SignalUpdate};
// use leptos::tracing::debug;
use log::{debug, error, info};
use tonic_web_wasm_client::Client;

use crate::domain::error::AppError;
use crate::domain::patient::Patient;
use crate::services::api::patient_service_client::*;
use crate::services::api::*;

pub trait PatientService: Send + Sync {
    fn get_app_status(&self) -> ReadSignal<Option<Patient>>;
    fn update_app_status(&self, patient: Patient);
    fn find_and_update_app_status(&self, id: String);
    async fn search_patient(&self, patient: String) -> Result<Vec<Patient>, AppError>;
}

pub struct PatientServiceImpl {
    pub app_state: RwSignal<Option<Patient>>,
}

impl PatientService for PatientServiceImpl {
    fn get_app_status(&self) -> ReadSignal<Option<Patient>> {
        self.app_state.read_only()
    }

    fn update_app_status(&self, patient: Patient) {
        let patient_name = patient.name.clone();
        info!("nombre del paciente: {}", patient_name);
        self.app_state
            .update(move |mut value| *value = Some(patient.clone()));
        // app_state.patient_set.set(Some(patient.clone()));
        // signal.set(AppState { patient: Some(patient.clone()) });
    }

    fn find_and_update_app_status(&self, id: String) {
        debug!("find_and_update_app_status: {}", id);
        let app_state = self.app_state;
        spawn_local(async move {
            // FIXME: Buscar en GRPC el paciente por id
            app_state.update(move |mut value| {
                debug!("app_state.update: {}", id);
                *value = Some(Patient::new(
                    id,
                    "Miuler".to_string(),
                    "email".to_string(),
                    "other".to_string(),
                    true,
                    "avatar".to_string(),
                ));
            });
        });
    }

    async fn search_patient(&self, patient: String) -> Result<Vec<Patient>, AppError> {
        let mut client = build_client();
        let patient_response = client
            .search_patient(SearchPatientRequest {
                name: Some("Miuler".to_string()),
            })
            .await?;

        let patients = patient_response.into_inner().patients;

        let patients_vec = patients
            .into_iter()
            .map(|response| {
                Patient::new(
                    response.id,
                    response.first_name,
                    response.email.unwrap_or("".to_string()),
                    response.note.unwrap_or("".to_string()),
                    false,
                    response.icon.unwrap_or("".to_string()),
                )
            })
            .collect::<Vec<_>>();

        Ok(patients_vec)
    }
}

fn build_client() -> PatientServiceClient<Client> {
    let wasm_client = Client::new("http://localhost:9000".to_string());
    PatientServiceClient::new(wasm_client)
}
