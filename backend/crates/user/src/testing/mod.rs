//! Dobles de prueba del contexto de usuario.
//!
//! Solo se compilan en pruebas (`cfg(test)`) o con la feature `test-support`, que activan los
//! tests de integración del binario. Nada de este módulo entra en el binario de producción.

use crate::application::dto::sign_in::SignInError;
use crate::application::integration::service::token_service::TokenService;
use crate::domain::user::User;
use crate::domain::user::user_repository::{ProfileUpdate, UserRepository};
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::{ContactPoint, ContactPointSystem, HumanName, Identifier};
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

    async fn find_users_by_email(&self, email: &str) -> Result<Vec<User>, ClickCareError> {
        let users = self.saved_users.lock().await;
        Ok(users
            .iter()
            .filter(|user| user.active && user.email() == email)
            .cloned()
            .collect())
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

    async fn find_user_by_id(&self, user_id: &uuid::Uuid) -> Result<Option<User>, ClickCareError> {
        let users = self.saved_users.lock().await;
        Ok(users.iter().find(|user| user.id == *user_id).cloned())
    }

    async fn set_license_verified(&self, user_id: &uuid::Uuid) -> Result<(), ClickCareError> {
        let mut users = self.saved_users.lock().await;
        if let Some(user) = users.iter_mut().find(|user| user.id == *user_id) {
            user.license_verified = true;
        }
        Ok(())
    }

    async fn update_profile(
        &self,
        user_id: &uuid::Uuid,
        profile: &ProfileUpdate,
    ) -> Result<(), ClickCareError> {
        let mut users = self.saved_users.lock().await;
        let user = users
            .iter_mut()
            .find(|user| user.id == *user_id)
            .ok_or_else(|| ClickCareError::generic(format!("User {user_id} not found")))?;

        // Igual que el repositorio real: un dato en `None` conserva el valor actual.
        let current_name = user.person.name.clone();
        user.person.name = HumanName::new(
            match &profile.given_name {
                Some(given_name) => vec![given_name.clone()],
                None => current_name.given().clone(),
            },
            profile
                .family_name
                .clone()
                .or_else(|| current_name.family().clone()),
            profile
                .second_family_name
                .clone()
                .or_else(|| current_name.second_family().clone()),
        );
        if let Some(phone) = &profile.phone {
            user.person
                .telecom
                .retain(|telecom_entry| telecom_entry.system != ContactPointSystem::Phone);
            user.person
                .telecom
                .push(ContactPoint::phone(phone.clone(), None));
        }
        let keep = |new: &Option<String>, current: &mut Option<String>| {
            if new.is_some() {
                *current = new.clone();
            }
        };
        keep(&profile.birth_date, &mut user.person.birth_date);
        keep(&profile.address, &mut user.address);
        keep(
            &profile.professional_body,
            &mut user.professional.professional_body,
        );
        keep(
            &profile.license_number,
            &mut user.professional.license_number,
        );
        keep(&profile.specialty, &mut user.professional.specialty);
        keep(&profile.rne, &mut user.professional.rne);
        if profile.identifier.is_some() {
            user.person.identifier = profile.identifier.clone();
        }
        if profile.sex.is_some() {
            user.sex = profile.sex;
        }
        if let Some(role) = profile.preferred_role {
            user.preferred_role = role;
        }
        Ok(())
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
