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
    /// Clínica en la que ejerce este profesional (UUID v7).
    ///
    /// Discriminador de inquilino: la ficha pertenece a una clínica concreta. Un
    /// mismo médico que atiende en dos clínicas tiene una ficha en cada una, con
    /// su propia colegiatura registrada y su propio estado de actividad.
    pub organization_id: Uuid,
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
    pub fn new(
        id: Uuid,
        organization_id: Uuid,
        user_id: Uuid,
        medical_license_number: String,
        person: Person,
    ) -> Self {
        Self {
            id,
            organization_id,
            user_id,
            active: true,
            medical_license_number,
            specialty: None,
            person,
        }
    }
}
