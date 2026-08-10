use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recurso de dominio que representa el Expediente Clínico del Paciente (FHIR R4 Patient).
///
/// En una arquitectura de clínicas autónomas, almacena la referencia global (`user_id` / `person_id`)
/// y replican localmente la demografía esencial para permitir consultas e historias clínicas autónomas.
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Patient {
    /// Identificador único del expediente del paciente en esta clínica (UUID v7).
    pub id: Uuid,
    /// Apuntador débil al usuario/persona global (UUID v7).
    pub user_id: Uuid,
    /// Apuntador débil a la persona física (UUID v7).
    pub person_id: Uuid,
    /// Estado activo/inactivo del expediente en la clínica.
    pub active: bool,

    // Réplica demográfica local para autonomía de lectura y atenciones clínicas
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub identifier_dni: Option<String>,
    pub phone: String,
    pub birth_date: String,
}

impl Patient {
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        person_id: Uuid,
        given_name: String,
        family_name: Option<String>,
        second_family_name: Option<String>,
        identifier_dni: Option<String>,
        phone: String,
        birth_date: String,
    ) -> Self {
        Self {
            id,
            user_id,
            person_id,
            active: true,
            given_name,
            family_name,
            second_family_name,
            identifier_dni,
            phone,
            birth_date,
        }
    }
}
