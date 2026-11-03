//! Dobles de prueba del contexto de usuario.
//!
//! Solo se compilan en pruebas (`cfg(test)`) o con la feature `test-support`, que activan los
//! tests de integración del binario. Nada de este módulo entra en el binario de producción.

use crate::application::dto::sign_in::SignInError;
use crate::application::integration::service::token_service::TokenService;
use crate::domain::user::User;
use crate::domain::user::user_repository::UserRepository;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::{ContactPointSystem, Identifier};
use async_trait::async_trait;
use tokio::sync::Mutex;

/// Implementación en memoria de `UserRepository`.
pub struct MockUserRepositoryImpl {
    /// Lista de usuarios almacenados en memoria durante el test.
    pub saved_users: Mutex<Vec<User>>,
}

#[async_trait]
impl UserRepository for MockUserRepositoryImpl {
    async fn exist_user_by_document(
        &self,
        network_id: &uuid::Uuid,
        identifier: Identifier,
    ) -> Result<bool, ClickCareError> {
        let users = self.saved_users.lock().await;

        let exists = users.iter().any(|user| {
            user.network_id == *network_id && user.person.identifier.as_ref() == Some(&identifier)
        });

        Ok(exists)
    }

    async fn exist_user_by_email(
        &self,
        network_id: &uuid::Uuid,
        email: &str,
    ) -> Result<bool, ClickCareError> {
        let users = self.saved_users.lock().await;

        let exists = users.iter().any(|user| {
            user.network_id == *network_id
                && user.person.telecom.iter().any(|telecom_entry| {
                    telecom_entry.system == ContactPointSystem::Email
                        && telecom_entry.value == email
                })
        });

        Ok(exists)
    }

    async fn find_user_by_email(
        &self,
        network_id: &uuid::Uuid,
        email: &str,
    ) -> Result<Option<User>, ClickCareError> {
        let users = self.saved_users.lock().await;

        let matched_user = users
            .iter()
            .find(|user| {
                user.network_id == *network_id
                    && user.person.telecom.iter().any(|telecom_entry| {
                        telecom_entry.system == ContactPointSystem::Email
                            && telecom_entry.value == email
                    })
            })
            .cloned();

        Ok(matched_user)
    }

    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError> {
        let users = self.saved_users.lock().await;
        users
            .iter()
            .find(|user| user.id.to_string() == user_id)
            .cloned()
            .ok_or_else(|| {
                ClickCareError::generic(format!(
                    "User with ID {user_id} not found in mock repository"
                ))
            })
    }

    async fn save_user(&self, user: &User) -> Result<(), ClickCareError> {
        let mut users = self.saved_users.lock().await;
        users.push(user.clone());
        Ok(())
    }
}

/// Implementación de [`TokenService`] para pruebas.
///
/// Decodifica el payload JWT sin validar la firma contra Google, pero verifica la estructura
/// de 3 segmentos y la expiración (`exp`).
#[derive(Default)]
pub struct MockTokenServiceImpl;

#[async_trait]
impl TokenService for MockTokenServiceImpl {
    async fn validate_and_extract_email(&self, id_token: &str) -> Result<String, SignInError> {
        let segments: Vec<&str> = id_token.trim().split('.').collect();
        if segments.len() != 3 || segments.iter().any(|s| s.is_empty()) {
            return Err(SignInError::InvalidToken(ClickCareError::generic(
                "El id_token no tiene una estructura JWT válida de 3 segmentos".to_string(),
            )));
        }

        #[derive(serde::Deserialize)]
        struct Claims {
            email: String,
            exp: Option<i64>,
        }

        let token_data = jsonwebtoken::dangerous::insecure_decode::<Claims>(id_token.as_bytes())
            .map_err(|err| {
                SignInError::InvalidToken(ClickCareError::generic(format!("{err:?}")))
            })?;

        if let Some(exp) = token_data.claims.exp {
            let current_epoch = jiff::Timestamp::now().as_second();
            if exp <= current_epoch {
                return Err(SignInError::InvalidToken(ClickCareError::generic(
                    "El id_token ha expirado".to_string(),
                )));
            }
        }

        Ok(token_data.claims.email)
    }
}
