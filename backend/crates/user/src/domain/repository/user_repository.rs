use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;

/// Puerto de repositorio de dominio para la persistencia y recuperación de `User`.
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Verifica si existe un usuario registrado con un documento determinado (ej. DNI).
    async fn exist_user_by_document(&self, document_type: &str, document_value: &str) -> Result<bool, ClickCareError>;

    /// Busca un usuario por su identificador único (UUID v7).
    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError>;

    /// Persiste o actualiza la entidad `User` en la base de datos.
    async fn save_user(&self, user: &User) -> Result<(), ClickCareError>;
}

