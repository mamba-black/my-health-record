use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Evento de dominio emitido cuando una nueva cuenta de usuario y persona física es registrada (`crates/user`).
///
/// Este evento es consumido por otros Bounded Contexts (ej. `crates/administration`) de forma asíncrona y reactiva
/// para inicializar la réplica demográfica local de `Patient` o `Practitioner` / `Organization`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserCreatedEvent {
    /// Identificador único de la cuenta de usuario (UUID v7).
    pub user_id: Uuid,
    /// Identificador único de la persona física (FHIR R4 Person - UUID v7).
    pub person_id: Uuid,
    /// Correo electrónico principal del usuario.
    pub email: String,
    /// Nombres de pila de la persona (FHIR: HumanName.given).
    pub given_name: String,
    /// Primer apellido (FHIR: HumanName.family).
    pub family_name: Option<String>,
    /// Segundo apellido (Extensión hispana).
    pub second_family_name: Option<String>,
    /// Documento de Identidad (DNI) si fue proporcionado.
    pub identifier_dni: Option<String>,
    /// Teléfono de contacto de la persona.
    pub phone: String,
    /// Fecha de nacimiento en formato YYYY-MM-DD.
    pub birth_date: String,
    /// Flag que indica si se debe inicializar una clínica u organización para el usuario.
    pub create_clinic: bool,
}
