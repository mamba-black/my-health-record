use bon::Builder;
use derive_getters::Getters;
use uuid::Uuid;

/// Organization raíz del dueño (FHIR R4 `Organization` sin `partOf`): su empresa o
/// grupo, y la unidad de aislamiento («red clínica»).
///
/// Cada clínica es una `Organization` cuyo `partOf` apunta a la raíz de su dueño
/// (`Organization::network_id`). Todas las clínicas de un mismo dueño cuelgan de la
/// misma raíz, y ninguna regla cruza de una raíz a otra.
#[derive(Debug, Clone, Getters, Builder, PartialEq, Eq)]
pub struct ClinicalNetwork {
    /// Identificador único de la raíz (UUID v7). Es el `network_id` de sus clínicas.
    pub id: Uuid,
    /// Nombre de la empresa o grupo.
    pub name: String,
    /// Marca heredada de las redes creadas antes de la Organization raíz.
    pub is_default: bool,
    /// Estado activo/inactivo.
    pub active: bool,
    /// Dueño de la raíz. Cada dueño tiene una sola; `None` solo en redes heredadas.
    pub owner_user_id: Option<Uuid>,
}

impl ClinicalNetwork {
    /// Crea una red activa sin dueño (forma heredada).
    pub fn new(id: Uuid, name: String, is_default: bool) -> Self {
        Self {
            id,
            name,
            is_default,
            active: true,
            owner_user_id: None,
        }
    }

    /// Crea la Organization raíz de un dueño.
    pub fn root(id: Uuid, name: String, owner_user_id: Uuid) -> Self {
        Self {
            id,
            name,
            is_default: false,
            active: true,
            owner_user_id: Some(owner_user_id),
        }
    }
}
