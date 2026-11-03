use crate::application::dto::sign_in::{SignInCommand, SignInError, SignInResponse};
use crate::application::integration::service::token_service::TokenService;
use crate::domain::user::user_repository::UserRepository;
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

/// Caso de uso para autenticar un usuario existente en una red médica (`network_id`)
/// mediante un token de identidad OAuth/OIDC (`id_token`).
pub type SignInUseCase =
    dyn UseCase<Command = SignInCommand, Response = SignInResponse, Error = SignInError>;

pub(crate) struct SignInUseCaseImpl {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) token_service: Arc<dyn TokenService>,
}

#[async_trait]
impl UseCase for SignInUseCaseImpl {
    type Command = SignInCommand;
    type Response = SignInResponse;
    type Error = SignInError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {
        if command.id_token.trim().is_empty() {
            return Err(SignInError::InvalidToken(ClickCareError::generic(
                "El id_token es obligatorio para iniciar sesión".to_string(),
            )));
        }

        if command.provider_id.trim().is_empty() {
            return Err(SignInError::InvalidToken(ClickCareError::generic(
                "El provider_id es obligatorio para iniciar sesión".to_string(),
            )));
        }

        let email = self
            .token_service
            .validate_and_extract_email(&command.id_token)
            .await?;
        debug!(
            "Iniciando sesión para email={} en network_id={}",
            email, command.network_id
        );

        let existing_user = self
            .user_repository
            .find_user_by_email(&command.network_id, &email)
            .await
            .map_err(SignInError::UnknownError)?;

        let Some(user) = existing_user else {
            warn!(
                "Usuario con email={} no encontrado en network_id={}",
                email, command.network_id
            );
            return Err(SignInError::UserNotFound(ClickCareError::generic(format!(
                "No existe cuenta registrada para el correo {email} en la red clínica {}",
                command.network_id
            ))));
        };

        Ok(SignInResponse { user })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::User;
    use crate::testing::{MockTokenServiceImpl, MockUserRepositoryImpl};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    fn encode_base64url(input_bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut encoded = String::new();
        let mut index = 0;

        while index < input_bytes.len() {
            let byte0 = input_bytes[index];
            let byte1 = if index + 1 < input_bytes.len() {
                input_bytes[index + 1]
            } else {
                0
            };
            let byte2 = if index + 2 < input_bytes.len() {
                input_bytes[index + 2]
            } else {
                0
            };

            encoded.push(ALPHABET[(byte0 >> 2) as usize] as char);
            encoded.push(ALPHABET[(((byte0 & 0x03) << 4) | (byte1 >> 4)) as usize] as char);
            if index + 1 < input_bytes.len() {
                encoded.push(ALPHABET[(((byte1 & 0x0F) << 2) | (byte2 >> 6)) as usize] as char);
            }
            if index + 2 < input_bytes.len() {
                encoded.push(ALPHABET[(byte2 & 0x3F) as usize] as char);
            }

            index += 3;
        }

        encoded
    }

    fn build_mock_jwt(email: &str, exp: Option<i64>) -> String {
        let header = encode_base64url(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload_json = match exp {
            Some(expiration) => {
                format!(r#"{{"sub":"google-123","email":"{email}","exp":{expiration}}}"#)
            }
            None => format!(r#"{{"sub":"google-123","email":"{email}"}}"#),
        };
        let payload = encode_base64url(payload_json.as_bytes());
        format!("{header}.{payload}.mock-signature")
    }

    #[tokio::test]
    async fn sign_in_succeeds_for_existing_user_in_same_network() {
        let network_id = Uuid::now_v7();
        let user_id = Uuid::now_v7().to_string();
        let email = "dra.garcia@example.com".to_string();

        let existing_user = User::new(
            user_id.clone(),
            network_id,
            vec!["Elena".to_string()],
            Some("García".to_string()),
            None,
            None,
            true,
            email.clone(),
            Some("999888777".to_string()),
            Some("1985-05-10".to_string()),
        )
        .expect("User creation must succeed");

        let user_repository = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(vec![existing_user]),
        });

        let token_service = Arc::new(MockTokenServiceImpl);
        let use_case = SignInUseCaseImpl {
            user_repository,
            token_service,
        };

        let future_exp = jiff::Timestamp::now().as_second() + 3600;
        let command = SignInCommand {
            network_id,
            id_token: build_mock_jwt(&email, Some(future_exp)),
            provider_id: "google.com".to_string(),
        };

        let response = use_case
            .execute(command)
            .await
            .expect("sign_in must succeed");
        assert_eq!(response.user.id.to_string(), user_id);
        assert!(response.user.is_owner);
    }

    #[tokio::test]
    async fn sign_in_returns_user_not_found_when_email_is_in_different_network() {
        let network_a = Uuid::now_v7();
        let network_b = Uuid::now_v7();
        let email = "paciente@example.com".to_string();

        let user_in_network_a = User::new(
            Uuid::now_v7().to_string(),
            network_a,
            vec!["Carlos".to_string()],
            Some("Ruiz".to_string()),
            None,
            None,
            false,
            email.clone(),
            None,
            None,
        )
        .expect("User creation must succeed");

        let user_repository = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(vec![user_in_network_a]),
        });
        let token_service = Arc::new(MockTokenServiceImpl);
        let use_case = SignInUseCaseImpl {
            user_repository,
            token_service,
        };

        let command = SignInCommand {
            network_id: network_b,
            id_token: build_mock_jwt(&email, None),
            provider_id: "google.com".to_string(),
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(SignInError::UserNotFound(_))));
    }

    #[tokio::test]
    async fn sign_in_rejects_invalid_or_expired_id_token() {
        let user_repository = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let token_service = Arc::new(MockTokenServiceImpl);
        let use_case = SignInUseCaseImpl {
            user_repository,
            token_service,
        };

        let network_id = Uuid::now_v7();

        let malformed_result = use_case
            .execute(SignInCommand {
                network_id,
                id_token: "not-a-valid-jwt".to_string(),
                provider_id: "google.com".to_string(),
            })
            .await;
        assert!(matches!(
            malformed_result,
            Err(SignInError::InvalidToken(_))
        ));

        let expired_epoch = jiff::Timestamp::now().as_second() - 60;
        let expired_result = use_case
            .execute(SignInCommand {
                network_id,
                id_token: build_mock_jwt("user@example.com", Some(expired_epoch)),
                provider_id: "google.com".to_string(),
            })
            .await;
        assert!(matches!(expired_result, Err(SignInError::InvalidToken(_))));
    }
}
