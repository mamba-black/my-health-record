use crate::application::dto::sign_in::SignInError;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use uuid::Uuid;

/// Parámetros de entrada del caso de uso `CompleteProfileUseCase`.
#[derive(Debug, Clone)]
pub struct CompleteProfileCommand {
    /// Red médica resuelta a partir del subdominio de la solicitud.
    pub network_id: Uuid,
    /// Token que identifica al titular de la cuenta.
    pub id_token: String,
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub identifier: Option<Identifier>,
    pub phone: String,
    /// Fecha en formato `YYYY-MM-DD`.
    pub birth_date: String,
    pub address: Option<String>,
}

/// Errores tipados de `CompleteProfileUseCase`.
#[derive(Debug)]
pub enum CompleteProfileError {
    /// Un dato está vacío o excede el tamaño permitido.
    InvalidInput(String),
    /// El documento de identidad ya pertenece a otra cuenta de la red médica.
    DocumentAlreadyRegistered(ClickCareError),
    /// El token es inválido, malformado o expiró.
    InvalidToken(ClickCareError),
    /// El titular del token no tiene cuenta en la red médica.
    UserNotFound(ClickCareError),
    /// Error técnico o de infraestructura.
    UnknownError(ClickCareError),
}

impl From<ClickCareError> for CompleteProfileError {
    fn from(error: ClickCareError) -> Self {
        CompleteProfileError::UnknownError(error)
    }
}

impl From<SignInError> for CompleteProfileError {
    fn from(error: SignInError) -> Self {
        match error {
            SignInError::InvalidToken(error) => CompleteProfileError::InvalidToken(error),
            SignInError::UserNotFound(error) => CompleteProfileError::UserNotFound(error),
            SignInError::UnknownError(error) => CompleteProfileError::UnknownError(error),
        }
    }
}
