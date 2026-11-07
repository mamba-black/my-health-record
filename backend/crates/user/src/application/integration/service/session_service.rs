use app_core::domain::error::ClickCareError;
use uuid::Uuid;

/// Tokens de una sesión emitida por el backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTokens {
    /// Se envía en cada llamada protegida como `authorization: Bearer <token>`.
    pub access_token: String,
    /// Vencimiento del token de acceso, en segundos Unix.
    pub access_expires_at: i64,
    /// Se canjea por un token de acceso nuevo sin volver a pasar por Google.
    pub refresh_token: String,
}

/// Identidad que contiene un token de sesión válido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionClaims {
    pub user_id: Uuid,
    pub network_id: Uuid,
}

/// Motivo por el que un token de sesión no es aceptado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// El token venció.
    Expired,
    /// Firma, formato o tipo de token inválidos.
    Invalid(String),
}

/// Emite y verifica los tokens de sesión propios del backend.
///
/// El token no lleva roles ni membresías: se leen en cada petición para que una
/// suspensión surta efecto de inmediato.
pub trait SessionService: Send + Sync {
    /// Abre una sesión para la cuenta.
    fn issue(&self, user_id: Uuid, network_id: Uuid) -> Result<SessionTokens, ClickCareError>;

    /// Verifica un token de acceso.
    fn verify_access(&self, token: &str) -> Result<SessionClaims, SessionError>;

    /// Verifica un token de renovación.
    fn verify_refresh(&self, token: &str) -> Result<SessionClaims, SessionError>;
}
