use crate::application::integration::service::session_service::{
    SessionError, SessionService, SessionTokens,
};
use crate::domain::user::user_repository::UserRepository;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;

/// Errores de la renovación de sesión.
#[derive(Debug)]
pub enum RefreshSessionError {
    /// El token de renovación es inválido, venció o la cuenta ya no está activa.
    Unauthenticated(String),
    UnknownError(ClickCareError),
}

/// Canjea un token de renovación por una sesión nueva sin volver a pasar por Google.
pub struct RefreshSessionUseCase {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) session_service: Arc<dyn SessionService>,
}

impl RefreshSessionUseCase {
    pub async fn execute(&self, refresh_token: &str) -> Result<SessionTokens, RefreshSessionError> {
        let claims = self
            .session_service
            .verify_refresh(refresh_token)
            .map_err(|error| {
                RefreshSessionError::Unauthenticated(match error {
                    SessionError::Expired => "La sesión venció".to_string(),
                    SessionError::Invalid(reason) => reason,
                })
            })?;

        let user = self
            .user_repository
            .find_user_by_id(&claims.user_id)
            .await
            .map_err(RefreshSessionError::UnknownError)?;
        match user {
            Some(user) if user.active && user.network_id == claims.network_id => self
                .session_service
                .issue(user.id, user.network_id)
                .map_err(RefreshSessionError::UnknownError),
            _ => Err(RefreshSessionError::Unauthenticated(
                "La cuenta de la sesión ya no está activa".to_string(),
            )),
        }
    }
}
