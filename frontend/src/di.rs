use crate::services::patient_service::{PatientRepositoryImpl, PatientService, PatientServiceImpl};
use parse_display::helpers::once_cell::sync::Lazy;

pub struct DIPrd {
    pub(crate) patient_service: Box<dyn PatientService>,
}

pub static DI: Lazy<DIPrd> = Lazy::new(|| DIPrd {
    patient_service: Box::new(PatientServiceImpl {
        patient: Box::new(PatientRepositoryImpl {}),
    }),
});
