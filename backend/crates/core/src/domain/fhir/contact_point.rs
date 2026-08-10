use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Sistema de telecomunicación utilizado según FHIR R4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactPointSystem {
    /// Teléfono.
    Phone,
    /// Fax.
    Fax,
    /// Correo electrónico.
    Email,
    /// Localizador / Pager.
    Pager,
    /// Dirección URL.
    Url,
    /// Mensaje SMS.
    Sms,
    /// Otro sistema.
    Other,
}

/// Contexto o propósito de uso del punto de contacto según FHIR R4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactPointUse {
    /// Domicilio personal.
    Home,
    /// Trabajo u oficina.
    Work,
    /// Teléfono celular / móvil.
    Mobile,
    /// Temporal.
    Temp,
    /// Desuso o antiguo.
    Old,
}

/// Tipo de dato HL7 FHIR R4: `ContactPoint`.
///
/// Ref: <https://hl7.org/fhir/R4/datatypes.html#ContactPoint>
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize, Deserialize)]
pub struct ContactPoint {
    /// Canal o sistema empleado para el contacto.
    pub system: ContactPointSystem,
    /// Valor real del punto de contacto (ej. dirección email o número telefónico).
    pub value: String,
    /// Contexto o propósito de uso opcional.
    pub use_type: Option<ContactPointUse>,
}

impl ContactPoint {
    /// Crea un punto de contacto de tipo Email.
    pub fn email(value: impl Into<String>) -> Self {
        Self {
            system: ContactPointSystem::Email,
            value: value.into(),
            use_type: None,
        }
    }

    /// Crea un punto de contacto de tipo Teléfono.
    pub fn phone(value: impl Into<String>, use_type: Option<ContactPointUse>) -> Self {
        Self {
            system: ContactPointSystem::Phone,
            value: value.into(),
            use_type,
        }
    }
}
