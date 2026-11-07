use app_core::domain::access::Role;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use uuid::Uuid;

use crate::domain::user::Sex;

/// Parámetros de entrada del caso de uso `CompleteProfileUseCase`.
///
/// Ningún dato es obligatorio: un dato en `None` o vacío conserva el valor actual.
/// Los datos se exigen solo al intentar una acción que los requiere.
#[derive(Debug, Clone, Default)]
pub struct CompleteProfileCommand {
    /// Red médica de la sesión.
    pub network_id: Uuid,
    /// Cuenta de la sesión: nunca un dato enviado por el cliente.
    pub user_id: Uuid,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub identifier: Option<Identifier>,
    pub phone: Option<String>,
    /// Fecha en formato `YYYY-MM-DD`.
    pub birth_date: Option<String>,
    pub address: Option<String>,
    pub sex: Option<Sex>,
    pub professional_body: Option<String>,
    pub license_number: Option<String>,
    pub specialty: Option<String>,
    pub rne: Option<String>,
    pub preferred_role: Option<Role>,
}

/// Errores tipados de `CompleteProfileUseCase`.
#[derive(Debug)]
pub enum CompleteProfileError {
    /// Un dato tiene un formato inválido o excede el tamaño permitido.
    InvalidInput(String),
    /// El dato está verificado y el usuario ya no puede modificarlo.
    ReadOnly(String),
    /// El documento de identidad ya pertenece a otra cuenta de la red médica.
    DocumentAlreadyRegistered(ClickCareError),
    /// La cuenta de la sesión ya no existe.
    UserNotFound(ClickCareError),
    /// Error técnico o de infraestructura.
    UnknownError(ClickCareError),
}

impl From<ClickCareError> for CompleteProfileError {
    fn from(error: ClickCareError) -> Self {
        CompleteProfileError::UnknownError(error)
    }
}
