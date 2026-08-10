use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa la Ficha de Profesional de Salud / Médico (FHIR R4 Practitioner).
///
/// Registra las credenciales profesionales (CMP/COP) y replica la demografía local para firma de actos médicos.
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Practitioner {
    /// Identificador único del perfil profesional del médico (UUID v7).
    pub id: Uuid,
    /// Apuntador débil al usuario/persona global (UUID v7).
    pub user_id: Uuid,
    /// Apuntador débil a la persona física (UUID v7).
    pub person_id: Uuid,
    /// Estado activo del perfil médico.
    pub active: bool,

    /// Número de colegiatura médica oficial (ej. CMP o COP).
    pub medical_license_number: String,
    /// Especialidad médica principal (opcional).
    pub specialty: Option<String>,

    // Réplica demográfica local
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub identifier_dni: Option<String>,
    pub phone: String,
}

impl Practitioner {
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        person_id: Uuid,
        medical_license_number: String,
        given_name: String,
        family_name: Option<String>,
        second_family_name: Option<String>,
        identifier_dni: Option<String>,
        phone: String,
    ) -> Self {
        Self {
            id,
            user_id,
            person_id,
            active: true,
            medical_license_number,
            specialty: None,
            given_name,
            family_name,
            second_family_name,
            identifier_dni,
            phone,
        }
    }
}
