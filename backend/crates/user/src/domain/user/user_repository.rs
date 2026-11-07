use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use async_trait::async_trait;
use uuid::Uuid;

/// Datos de perfil a guardar. Un dato opcional en `None` conserva el valor actual.
#[derive(Debug, Clone)]
pub struct ProfileUpdate {
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub identifier: Option<Identifier>,
    pub phone: String,
    /// Fecha en formato `YYYY-MM-DD`.
    pub birth_date: String,
    pub address: Option<String>,
}

/// Puerto de repositorio de dominio para la persistencia y recuperación de `User`.
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Verifica si existe un usuario registrado con un documento determinado dentro de una red médica.
    async fn exist_user_by_document(
        &self,
        network_id: &Uuid,
        identifier: Identifier,
    ) -> Result<bool, ClickCareError>;

    /// Verifica si existe un usuario registrado con un email determinado dentro de una red médica.
    async fn exist_user_by_email(
        &self,
        network_id: &Uuid,
        email: &str,
    ) -> Result<bool, ClickCareError>;

    /// Busca un usuario por su correo electrónico dentro de una red médica (`network_id`).
    async fn find_user_by_email(
        &self,
        network_id: &Uuid,
        email: &str,
    ) -> Result<Option<User>, ClickCareError>;

    /// Busca un usuario por su identificador único (UUID v7).
    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError>;

    /// Actualiza los datos de perfil de una cuenta existente.
    async fn update_profile(
        &self,
        user_id: &Uuid,
        profile: &ProfileUpdate,
    ) -> Result<(), ClickCareError>;

    /// Persiste o actualiza la entidad `User` en la base de datos.
    async fn save_user(&self, user: &User) -> Result<(), ClickCareError>;
}
