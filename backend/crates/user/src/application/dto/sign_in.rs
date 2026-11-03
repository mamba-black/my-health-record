use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use uuid::Uuid;

/// Parámetros de entrada para el caso de uso de inicio de sesión (`SignInUseCase`).
#[derive(Debug, Clone)]
pub struct SignInCommand {
    /// Red médica resuelta a partir del subdominio de la solicitud (UUID v7).
    pub network_id: Uuid,
    /// Token JWT emitido por el proveedor de identidad (ej. Google).
    pub id_token: String,
    /// Identificador del proveedor de autenticación (ej. `"google.com"`).
    pub provider_id: String,
}

/// Respuesta exitosa de `SignInUseCase` con la entidad de dominio `User`.
#[derive(Debug, Clone)]
pub struct SignInResponse {
    /// Entidad de dominio del usuario autenticado.
    pub user: User,
}

/// Errores tipados del caso de uso `SignInUseCase`.
#[derive(Debug)]
pub enum SignInError {
    /// El token o proveedor de identidad es inválido, malformado o expiró.
    InvalidToken(ClickCareError),
    /// El usuario aún no está registrado en la red médica solicitada.
    UserNotFound(ClickCareError),
    /// Error técnico o de infraestructura.
    UnknownError(ClickCareError),
}

impl From<ClickCareError> for SignInError {
    fn from(error: ClickCareError) -> Self {
        SignInError::UnknownError(error)
    }
}
