use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa la Clínica u Organización Sanitaria (FHIR R4 Organization).
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    /// Identificador único de la clínica u organización (UUID v7).
    pub id: Uuid,
    /// Razon social o nombre comercial de la clínica.
    pub name: String,
    /// Registro RUC o identificador fiscal (opcional).
    pub tax_id: Option<String>,
    /// Identificador del usuario propietario / administrador principal (UUID v7).
    pub owner_user_id: Uuid,
    /// Estado activo/inactivo de la clínica.
    pub active: bool,
}

impl Organization {
    pub fn new(id: Uuid, name: String, owner_user_id: Uuid) -> Self {
        Self {
            id,
            name,
            tax_id: None,
            owner_user_id,
            active: true,
        }
    }
}
