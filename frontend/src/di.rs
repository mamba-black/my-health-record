// use parse_display::helpers::once_cell::sync::Lazy;
use std::sync::LazyLock;

use crate::services::patient_service::PatientServiceImpl;

pub struct DIPrd {
    // pub(crate) patient_service: Box<dyn PatientService>,
    pub(crate) patient_service: PatientServiceImpl,
}

pub static DI: LazyLock<DIPrd> = LazyLock::new(|| {
    DIPrd {
        // patient_service: Box::new(PatientServiceImpl {
        //     patient: Box::new(PatientRepositoryImpl {}),
        // }),
        patient_service: PatientServiceImpl::new(),
    }
});
