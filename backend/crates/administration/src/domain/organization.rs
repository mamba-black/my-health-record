use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa la Clínica u Organización Sanitaria (FHIR R4 Organization).
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    /// Identificador único de la clínica u organización (UUID v7).
    pub id: Uuid,
    /// Red de clínicas a la que pertenece esta clínica (UUID v7).
    pub network_id: Uuid,
    /// Subdominio exclusivo y plano de la clínica (ej. "san-borja").
    pub subdomain: String,
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
    pub fn new(
        id: Uuid,
        network_id: Uuid,
        subdomain: String,
        name: String,
        tax_id: Option<String>,
        owner_user_id: Uuid,
    ) -> Self {
        Self {
            id,
            network_id,
            subdomain,
            name,
            tax_id,
            owner_user_id,
            active: true,
        }
    }
}
