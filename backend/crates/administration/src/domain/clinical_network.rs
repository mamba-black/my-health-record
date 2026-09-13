use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa una Red de Clínicas (Clinical Network).
///
/// Define el límite de autenticación, directorio de usuarios y federación (`network_id`).
/// Toda clínica pertenece a una red médica; si es independiente, pertenece a una red implícita con `is_default = true`.
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClinicalNetwork {
    /// Identificador único de la red clínica (UUID v7).
    pub id: Uuid,
    /// Nombre descriptivo o comercial de la red.
    pub name: String,
    /// Indica si es la red por defecto creada para una clínica independiente.
    pub is_default: bool,
    /// Estado activo/inactivo de la red.
    pub active: bool,
}

impl ClinicalNetwork {
    pub fn new(id: Uuid, name: String, is_default: bool) -> Self {
        Self {
            id,
            name,
            is_default,
            active: true,
        }
    }
}
