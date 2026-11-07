use app_core::domain::access::{MembershipRole, MembershipStatus, MembershipView};
use uuid::Uuid;

/// Relación de un usuario con una clínica y su rol en ella.
///
/// Es lo único que concede permisos dentro de una clínica
/// (`openspec/specs/identity/role-membership`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: MembershipRole,
    pub status: MembershipStatus,
    /// Un dueño o administrador confirmó la colegiatura (solo médicos).
    pub license_confirmed: bool,
    /// El médico ya pidió activar su perfil con todos sus datos.
    pub activation_requested: bool,
}

impl Membership {
    /// Membresía nueva con el estado indicado.
    pub fn new(
        organization_id: Uuid,
        user_id: Uuid,
        role: MembershipRole,
        status: MembershipStatus,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            organization_id,
            user_id,
            role,
            status,
            license_confirmed: false,
            activation_requested: false,
        }
    }

    /// Vista para la regla de autorización.
    pub fn view(&self) -> MembershipView {
        MembershipView {
            id: self.id,
            organization_id: self.organization_id,
            role: self.role,
            status: self.status,
            license_confirmed: self.license_confirmed,
        }
    }
}
