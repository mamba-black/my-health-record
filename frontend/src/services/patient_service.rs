use leptos::{create_rw_signal, spawn_local, ReadSignal, RwSignal, SignalSet, SignalUpdate};
use std::time::Duration;
// use leptos::tracing::debug;
use log::{debug, error, info};
use tonic_web_wasm_client::Client;

use crate::domain::error::AppError;
use crate::domain::patient::Patient;
use crate::services::api::patient_service_client::*;
use crate::services::api::*;

pub trait PatientService: Send + Sync {
    fn get_loading(&self) -> ReadSignal<Load>;
    fn set_loading(&self, load: Load);
    fn get_app_status(&self) -> ReadSignal<Option<Patient>>;
    fn update_app_status(&self, patient: Patient);
    async fn find_and_update_app_status(&self, id: String);
    async fn search_patient(&self, patient: String) -> Result<Vec<Patient>, AppError>;
    async fn save(&self, patient: Patient);
}

#[derive(Debug)]
pub(crate) enum Load {
    None,
    Loading,
}

pub struct PatientServiceImpl {
    pub app_state: RwSignal<Option<Patient>>,
    pub loading: RwSignal<Load>,
    pub client: PatientServiceClient<Client>,
}

impl PatientServiceImpl {
    pub(crate) fn new() -> PatientServiceImpl {
        let app_state = create_rw_signal(Option::<Patient>::None);
        let loading = create_rw_signal(Load::None);
        // provide_context(app_state);
        // provide_context(loading);
        Self {
            app_state,
            loading,
            client: build_client(),
        }
    }
}

impl PatientService for PatientServiceImpl {
    fn get_loading(&self) -> ReadSignal<Load> {
        self.loading.read_only()
    }

    fn set_loading(&self, load: Load) {
        self.loading.update(move |mut value| *value = load);
    }

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

    async fn find_and_update_app_status(&self, id: String) {
        debug!("find_and_update_app_status: {}", id);
        let mut client = self.client.clone();
        let app_state = self.app_state;
        spawn_local(async move {
            // FIXME: Buscar en GRPC el paciente por id
            let _patient = client
                .get_patient_by_id(PatientIdRequest {
                    id: Some(id.clone()),
                })
                .await;
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
        let mut client = self.client.clone();
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

    async fn save(&self, patient: Patient) {
        self.set_loading(Load::Loading);
        let mut client = self.client.clone();
        // FIXME: Guardar el paciente usando GRPC
        client
            .save(PatientInformation {
                id: "123".to_string(),
                first_name: patient.name.clone(),
                last_name: patient.name.clone(),
                second_last_name: None,
                email: Some(patient.email.clone()),
                phone_number: None,
                icon: None,
                note: None,
            })
            .await;
        async_std::task::sleep(Duration::from_millis(3000)).await;
        // self.app_state(Some(patient));
        self.set_loading(Load::None);
    }
}

fn build_client() -> PatientServiceClient<Client> {
    let wasm_client = Client::new("http://localhost:9000".to_string());
    PatientServiceClient::new(wasm_client)
}
