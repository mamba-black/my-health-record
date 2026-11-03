use crate::application::dto::sign_in::SignInError;
use async_trait::async_trait;

#[async_trait]
pub trait TokenService: Send + Sync {
    /// Valida la estructura del JWT (`header.payload.signature`), su expiración (`exp`)
    /// y extrae el correo electrónico (`email`).
    async fn validate_and_extract_email(&self, id_token: &str) -> Result<String, SignInError>;
}
