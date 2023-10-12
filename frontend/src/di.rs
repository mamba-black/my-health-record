use leptos::{create_rw_signal, provide_context, RwSignal};
use parse_display::helpers::once_cell::sync::Lazy;

use crate::domain::patient::Patient;
use crate::services::patient_service::{Load, PatientServiceImpl};

pub struct DIPrd {
    // pub(crate) patient_service: Box<dyn PatientService>,
    pub(crate) patient_service: PatientServiceImpl,
}

pub static DI: Lazy<DIPrd> = Lazy::new(|| {
    DIPrd {
        // patient_service: Box::new(PatientServiceImpl {
        //     patient: Box::new(PatientRepositoryImpl {}),
        // }),
        patient_service: PatientServiceImpl::new(),
    }
});
