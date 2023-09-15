use crate::domain::patient::Patient;
use leptos::{expect_context, RwSignal, SignalUpdate};
use log::info;

pub trait PatientService: Send + Sync {
    fn search_patient(&self, patient: Patient);
}

pub struct PatientServiceImpl {
    pub(crate) patient: Box<dyn PatientRepository>,
}

impl PatientService for PatientServiceImpl {
    fn search_patient(&self, patient: Patient) {
        let app_state = expect_context::<RwSignal<Option<Patient>>>();
        let mut a = patient.name.clone();
        info!("nombre del paciente: {}", a);
        app_state.update(move |mut value| *value = Some(patient.clone()));
        // app_state.patient_set.set(Some(patient.clone()));
        // signal.set(AppState { patient: Some(patient.clone()) });
    }
}

pub trait PatientRepository: Send + Sync {}

pub struct PatientRepositoryImpl {}

impl PatientRepository for PatientRepositoryImpl {}
