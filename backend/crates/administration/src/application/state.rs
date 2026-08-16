use crate::domain::repository::organization_repository::OrganizationRepository;
use crate::domain::repository::patient_repository::PatientRepository;
use crate::domain::repository::practitioner_repository::PractitionerRepository;
use std::sync::Arc;

/// Dependencias que el worker inyecta en los handlers de eventos.
///
/// Solo contiene puertos de dominio: la capa de aplicación no conoce Apalis, toasty
/// ni ningún otro tipo de infraestructura. Es el `di` quien la arma con las
/// implementaciones reales y quien la registra en el worker.
#[derive(Clone)]
pub struct AdministrationState {
    pub organization_repository: Arc<dyn OrganizationRepository>,
    pub patient_repository: Arc<dyn PatientRepository>,
    pub practitioner_repository: Arc<dyn PractitionerRepository>,
}
