use crate::domain::user::{Sex, User};
use app_core::domain::access::Role;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use async_trait::async_trait;
use uuid::Uuid;

/// Datos de perfil a guardar. Todo dato en `None` conserva el valor actual.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileUpdate {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub identifier: Option<Identifier>,
    pub phone: Option<String>,
    /// Fecha en formato `YYYY-MM-DD`.
    pub birth_date: Option<String>,
    pub address: Option<String>,
    pub sex: Option<Sex>,
    pub professional_body: Option<String>,
    pub license_number: Option<String>,
    pub specialty: Option<String>,
    pub rne: Option<String>,
    pub preferred_role: Option<Role>,
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
    /// Cuentas activas con ese correo en **todas** las redes. Solo sirve para iniciar
    /// sesión sin subdominio: encontrar la cuenta, nunca mezclar datos entre redes.
    async fn find_users_by_email(&self, email: &str) -> Result<Vec<User>, ClickCareError>;

    async fn find_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, ClickCareError>;

    /// Cuentas activas de la red con ese tipo y número de documento.
    async fn find_users_by_document(
        &self,
        network_id: &Uuid,
        identifier: &Identifier,
    ) -> Result<Vec<User>, ClickCareError>;

    /// Marca el documento como verificado en persona: el usuario ya no lo edita.
    async fn set_document_verified(&self, user_id: &Uuid) -> Result<(), ClickCareError>;

    /// Marca la colegiatura como confirmada por una clínica: desde entonces no es editable.
    async fn set_license_verified(&self, user_id: &Uuid) -> Result<(), ClickCareError>;

    /// Actualiza los datos de perfil de una cuenta existente.
    async fn update_profile(
        &self,
        user_id: &Uuid,
        profile: &ProfileUpdate,
    ) -> Result<(), ClickCareError>;

    /// Persiste o actualiza la entidad `User` en la base de datos.
    async fn save_user(&self, user: &User) -> Result<(), ClickCareError>;
}
