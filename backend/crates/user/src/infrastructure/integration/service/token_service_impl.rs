use crate::application::dto::sign_in::SignInError;
use crate::application::integration::service::token_service::TokenService;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use jsonwebtoken_google2::{Parser, ParserError};
use serde::Deserialize;
use std::sync::OnceLock;

/// Solo se exige lo que el sistema usa: Google no garantiza claims como `locale` o `family_name`.
#[derive(Deserialize)]
struct GoogleIdTokenClaims {
    email: String,
}

pub struct TokenServiceImpl {
    /// Se conserva entre peticiones porque guarda la caché de claves públicas de Google.
    parser: OnceLock<Parser>,
}

impl TokenServiceImpl {
    pub fn new() -> Self {
        TokenServiceImpl {
            parser: OnceLock::new(),
        }
    }
}

#[async_trait]
impl TokenService for TokenServiceImpl {
    async fn validate_and_extract_email(&self, id_token: &str) -> Result<String, SignInError> {
        let id_token = id_token.trim();
        let token_segments: Vec<&str> = id_token.split('.').collect();
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
        let parser = self
            .parser
            .get_or_init(|| Parser::new(google_client_id.as_str()));

        verify_id_token(parser, id_token).await
    }
}

/// Verifica firma, audiencia, emisor y expiración del token, y devuelve su `email`.
async fn verify_id_token(parser: &Parser, id_token: &str) -> Result<String, SignInError> {
    let claims = parser
        .parse::<GoogleIdTokenClaims>(id_token)
        .await
        .map_err(|err| match err {
            // El tipo de este error es privado en la librería: una clave desconocida no se
            // puede distinguir de un fallo de red.
            ParserError::KeyProvider(_) => {
                SignInError::UnknownError(ClickCareError::generic(err.to_string()))
            }
            ParserError::WrongHeader | ParserError::UnknownKid | ParserError::WrongToken(_) => {
                SignInError::InvalidToken(ClickCareError::generic(err.to_string()))
            }
        })?;

    Ok(claims.email)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken_google2::test_helper::{EMAIL, TokenClaims, setup};

    fn segments(id_token: &str) -> Vec<&str> {
        id_token.split('.').collect()
    }

    #[tokio::test]
    async fn accepts_valid_token_without_optional_google_claims() {
        let (id_token, parser, _server) = setup(&TokenClaims::new());

        let email = verify_id_token(&parser, &id_token).await;

        assert!(matches!(email, Ok(email) if email == EMAIL));
    }

    #[tokio::test]
    async fn rejects_token_with_payload_swapped_after_signing() {
        let (id_token, parser, _server) = setup(&TokenClaims::new());
        let mut forged_claims = TokenClaims::new();
        forged_claims.email = "victima@example.com".to_string();
        let (other_token, _other_parser, _other_server) = setup(&forged_claims);
        let original = segments(&id_token);

        let forged = format!(
            "{}.{}.{}",
            original[0],
            segments(&other_token)[1],
            original[2]
        );

        assert!(matches!(
            verify_id_token(&parser, &forged).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_with_garbage_signature() {
        let (id_token, parser, _server) = setup(&TokenClaims::new());
        let original = segments(&id_token);

        let forged = format!("{}.{}.Z2FyYmFnZQ", original[0], original[1]);

        assert!(matches!(
            verify_id_token(&parser, &forged).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_signed_with_another_key() {
        let (_id_token, parser, _server) = setup(&TokenClaims::new());
        let (other_token, _other_parser, _other_server) = setup(&TokenClaims::new());

        assert!(matches!(
            verify_id_token(&parser, &other_token).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_with_symmetric_algorithm() {
        let (id_token, parser, _server) = setup(&TokenClaims::new());
        // Cabecera {"alg":"HS256","kid":"some-kid"}.
        let forged = format!(
            "eyJhbGciOiJIUzI1NiIsImtpZCI6InNvbWUta2lkIn0.{}.Z2FyYmFnZQ",
            segments(&id_token)[1]
        );

        assert!(matches!(
            verify_id_token(&parser, &forged).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_with_unknown_key_id() {
        let (id_token, parser, _server) = setup(&TokenClaims::new());
        let original = segments(&id_token);
        // Cabecera {"alg":"RS256","kid":"otra-clave"}.
        let forged = format!(
            "eyJhbGciOiJSUzI1NiIsImtpZCI6Im90cmEtY2xhdmUifQ.{}.{}",
            original[1], original[2]
        );

        assert!(matches!(
            verify_id_token(&parser, &forged).await,
            Err(SignInError::UnknownError(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_for_another_audience() {
        let mut claims = TokenClaims::new();
        claims.aud = "otro-client-id".to_string();
        let (id_token, parser, _server) = setup(&claims);

        assert!(matches!(
            verify_id_token(&parser, &id_token).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_from_another_issuer() {
        let mut claims = TokenClaims::new();
        claims.iss = "https://evil.example.com".to_string();
        let (id_token, parser, _server) = setup(&claims);

        assert!(matches!(
            verify_id_token(&parser, &id_token).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_expired_token() {
        let (id_token, parser, _server) = setup(&TokenClaims::new_expired());

        assert!(matches!(
            verify_id_token(&parser, &id_token).await,
            Err(SignInError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn reports_key_download_failure_as_unknown_error() {
        let (id_token, _parser, _server) = setup(&TokenClaims::new());
        let parser = Parser::new_with_custom_cert_url("client-id", "http://127.0.0.1:1/");

        assert!(matches!(
            verify_id_token(&parser, &id_token).await,
            Err(SignInError::UnknownError(_))
        ));
    }

    #[tokio::test]
    #[ignore = "requiere acceso a la red"]
    async fn downloads_real_google_keys() {
        let (id_token, _parser, _server) = setup(&TokenClaims::new());
        let original = segments(&id_token);
        // Cabecera {"alg":"RS256","kid":"otra-clave"}.
        let forged = format!(
            "eyJhbGciOiJSUzI1NiIsImtpZCI6Im90cmEtY2xhdmUifQ.{}.{}",
            original[1], original[2]
        );

        let error = verify_id_token(&Parser::new("client-id"), &forged)
            .await
            .expect_err("la clave no existe en Google");

        // "key not found" indica que la descarga funcionó y la clave no estaba.
        assert!(format!("{error:?}").contains("key not found"));
    }
}
