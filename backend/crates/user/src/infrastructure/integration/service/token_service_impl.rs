use async_trait::async_trait;
use google_jwt_verify::Client;
use app_core::domain::error::ClickCareError;
use crate::application::integration::service::token_service::TokenService;
use crate::application::use_case::sign_in_usecase::model::SignInError;

pub struct TokenServiceImpl {}

impl TokenServiceImpl {
    pub fn new() -> Self {
        TokenServiceImpl {}
    }
}

#[async_trait]
impl TokenService for TokenServiceImpl {
    async fn validate_and_extract_email(&self, id_token: &str) -> Result<String, SignInError> {
        let token_segments: Vec<&str> = id_token.trim().split('.').collect();
        if token_segments.len() != 3
            || token_segments[0].is_empty()
            || token_segments[1].is_empty()
            || token_segments[2].is_empty()
        {
            return Err(SignInError::InvalidToken(ClickCareError::generic(
                "El id_token no tiene una estructura JWT válida de 3 segmentos".to_string(),
            )));
        }

        let google_client_id = std::env::var("PUBLIC_GOOGLE_CLIENT_ID")
            .map_err(|err| SignInError::UnknownError(ClickCareError::generic(err.to_string())))?;
        let client = Client::new(google_client_id.as_str());
        let token = client.verify_id_token(id_token)
            .map_err(|err| SignInError::InvalidToken(ClickCareError::generic(format!("{:?}", err))))?;
        let claims = token.get_claims();
        let email = token.get_payload().get_email();

        let current_epoch_seconds: u64 = jiff::Timestamp::now().as_second().unsigned_abs();
        if claims.get_expires_at() <= current_epoch_seconds {
            return Err(SignInError::InvalidToken(ClickCareError::generic(
                "El id_token ha expirado".to_string(),
            )));
        }

        Ok(email)
    }
}
