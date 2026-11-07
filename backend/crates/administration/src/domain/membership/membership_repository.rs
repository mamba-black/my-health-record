use crate::domain::membership::Membership;
use app_core::domain::access::MembershipRole;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de las membresías.
#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Todas las membresías de un usuario, en cualquier clínica y estado.
    async fn find_by_user(&self, user_id: &Uuid) -> Result<Vec<Membership>, ClickCareError>;

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Membership>, ClickCareError>;

    /// Membresía de un usuario en una clínica con un rol, si existe.
    async fn find(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        role: MembershipRole,
    ) -> Result<Option<Membership>, ClickCareError>;

    /// Cantidad de dueños activos de una clínica.
    async fn count_active_owners(&self, organization_id: &Uuid) -> Result<usize, ClickCareError>;

    /// Crea la membresía.
    async fn insert(&self, membership: &Membership) -> Result<(), ClickCareError>;

    /// Guarda el estado, la confirmación de colegiatura y la solicitud de activación.
    async fn update(
        &self,
        membership: &Membership,
        confirmed_by: Option<Uuid>,
    ) -> Result<(), ClickCareError>;
}
