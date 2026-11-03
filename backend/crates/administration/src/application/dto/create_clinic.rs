use app_core::domain::error::ClickCareError;
use thiserror::Error;
use uuid::Uuid;

/// Datos de entrada, planos, tal como llegan desde la API.
///
/// La demografía del propietario viaja aquí porque este contexto acotado **no
/// consulta** al de identidad: construye su propia réplica local del profesional.
#[derive(Debug, Clone)]
pub struct CreateClinicCommand {
    pub owner_user_id: String,
    pub name: String,
    pub subdomain: String,
    pub network_id: Option<Uuid>,
    pub tax_id: Option<String>,
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub medical_license_number: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateClinicResponse {
    pub organization_id: Uuid,
    pub network_id: Uuid,
    pub practitioner_id: Uuid,
    /// La clínica ya existía y se devolvió sin recrearla.
    pub already_existed: bool,
}

#[derive(Debug, Error)]
pub enum CreateClinicError {
    #[error("El identificador del propietario debe ser un UUID v7: {0}")]
    InvalidOwnerUserId(String),

    #[error("El nombre de la clínica no puede estar vacío")]
    EmptyName,

    #[error("El subdominio no puede estar vacío")]
    EmptySubdomain,

    #[error("El subdominio '{0}' está reservado por la plataforma")]
    ReservedSubdomain(String),

    #[error(
        "El subdominio '{0}' es inválido (solo debe contener letras minúsculas, números y guiones)"
    )]
    InvalidSubdomainFormat(String),

    #[error("El subdominio '{0}' ya está en uso")]
    SubdomainAlreadyExists(String),

    #[error("La ficha de profesional del propietario no pudo resolverse")]
    MissingPractitioner,

    #[error(transparent)]
    Unknown(#[from] ClickCareError),
}
