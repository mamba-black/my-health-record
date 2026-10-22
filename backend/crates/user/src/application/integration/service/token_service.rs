use async_trait::async_trait;
use crate::application::use_case::sign_in_usecase::model::SignInError;

#[async_trait]
pub(crate) trait TokenService: Send + Sync {
    /// Valida la estructura del JWT (`header.payload.signature`), su expiración (`exp`)
    /// y extrae el correo electrónico (`email`).
    async fn validate_and_extract_email(&self, id_token: &str)  -> Result<String, SignInError>;
}
