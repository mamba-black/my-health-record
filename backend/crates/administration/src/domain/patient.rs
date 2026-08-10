use app_core::domain::fhir::Person;
use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa el Expediente Clínico del Paciente (FHIR R4 Patient).
///
/// En una arquitectura de clínicas autónomas, almacena la referencia global (`user_id`)
/// y el recurso FHIR `Person` con la demografía para atenciones autónomas.
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Patient {
    /// Identificador único del expediente del paciente en esta clínica (UUID v7).
    pub id: Uuid,
    /// Apuntador débil al usuario global (UUID v7).
    pub user_id: Uuid,
    /// Estado activo/inactivo del expediente en la clínica.
    pub active: bool,

    /// Recurso FHIR R4 de la persona física (Demografía completa e identidad).
    pub person: Person,
}

impl Patient {
    pub fn new(id: Uuid, user_id: Uuid, person: Person) -> Self {
        Self {
            id,
            user_id,
            active: true,
            person,
        }
    }
}
