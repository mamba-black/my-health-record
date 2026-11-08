use crate::domain::clinical_network::clinical_network_repository::ClinicalNetworkRepository;
use crate::domain::membership::membership_repository::MembershipRepository;
use crate::domain::organization::organization_repository::OrganizationRepository;
use crate::domain::patient::patient_repository::PatientRepository;
use crate::domain::practitioner::practitioner_repository::PractitionerRepository;
use app_core::domain::person_link::PersonLinkRegistry;
use std::sync::Arc;

/// Dependencias que el worker inyecta en los handlers de eventos.
///
/// Solo contiene puertos de dominio: la capa de aplicación no conoce Apalis, toasty
/// ni ningún otro tipo de infraestructura. Es el `di` quien la arma con las
/// implementaciones reales y quien la registra en el worker.
#[derive(Clone)]
pub struct AdministrationState {
    pub clinical_network_repository: Arc<dyn ClinicalNetworkRepository>,
    pub organization_repository: Arc<dyn OrganizationRepository>,
    pub patient_repository: Arc<dyn PatientRepository>,
    pub practitioner_repository: Arc<dyn PractitionerRepository>,
    pub membership_repository: Arc<dyn MembershipRepository>,
    /// `Person.link` de las cuentas: lo implementa el contexto `user`.
    pub person_links: Arc<dyn PersonLinkRegistry>,
}
