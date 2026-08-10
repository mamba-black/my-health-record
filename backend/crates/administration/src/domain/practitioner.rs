use app_core::domain::fhir::Person;
use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa la Ficha de Profesional de Salud / Médico (FHIR R4 Practitioner).
///
/// Registra las credenciales profesionales (CMP/COP) y almacena el recurso FHIR `Person` con la demografía.
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Practitioner {
    /// Identificador único del perfil profesional del médico (UUID v7).
    pub id: Uuid,
    /// Apuntador débil al usuario global (UUID v7).
    pub user_id: Uuid,
    /// Estado activo del perfil médico.
    pub active: bool,

    /// Número de colegiatura médica oficial (ej. CMP o COP).
    pub medical_license_number: String,
    /// Especialidad médica principal (opcional).
    pub specialty: Option<String>,

    /// Recurso FHIR R4 de la persona física (Demografía completa e identidad).
    pub person: Person,
}

impl Practitioner {
    pub fn new(id: Uuid, user_id: Uuid, medical_license_number: String, person: Person) -> Self {
        Self {
            id,
            user_id,
            active: true,
            medical_license_number,
            specialty: None,
            person,
        }
    }
}
