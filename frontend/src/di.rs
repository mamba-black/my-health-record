use crate::domain::patient::Patient;
use crate::services::patient_service::{PatientService, PatientServiceImpl};
use leptos::{expect_context, RwSignal};
use parse_display::helpers::once_cell::sync::Lazy;

pub struct DIPrd {
    // pub(crate) patient_service: Box<dyn PatientService>,
    pub(crate) patient_service: PatientServiceImpl,
}

pub static DI: Lazy<DIPrd> = Lazy::new(|| DIPrd {
    // patient_service: Box::new(PatientServiceImpl {
    //     patient: Box::new(PatientRepositoryImpl {}),
    // }),
    patient_service: PatientServiceImpl {
        app_state: expect_context::<RwSignal<Option<Patient>>>(),
    },
});
