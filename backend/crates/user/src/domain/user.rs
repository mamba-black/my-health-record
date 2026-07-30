use app_core::domain::error::ClickCareError;
use log::{debug, error};
use std::str::FromStr;
use strum_macros::Display;
use uuid::{Uuid, Version};
use chrono::NaiveDate;


#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub active: bool,

    // Nombres según HL7 FHIR
    pub name: HumanName,                // FHIR: Person.name (given, family)
    pub telecom: Vec<ContactPoint>,     // FHIR: Person.telecom (email, phone)
    pub identifier: Option<Identifier>,  // FHIR: Person.identifier
    pub photo_url: Option<String>,      // 👈 FHIR: Person.photo (mapeado desde provider_avatar_url)
    pub birth_date: Option<NaiveDate>,  // FHIR: Person.birthDate
    pub address: Option<String>,        // FHIR: Person.address

    // Metadatos de Autenticación / Negocio
    pub provider_info: IdentityProvider, // Encapsula provider_id y provider_name
    pub is_owner: bool,
}

#[derive(Debug, Clone, Display)]
pub enum Identifier {
    #[strum(to_string = "DNI: {0}")]
    DNI(String),
}

#[derive(Debug, Clone)]
pub enum IdentityProvider {
    Google,   // Ej: "google.com"
}

impl User {
    pub fn new(
        id: String,
        name: Vec<String>,
        first_name: String,
        last_name: String,
        identifier: Option<Identifier>,
        is_owner: bool,
        email: String,
    ) -> Result<Self, ClickCareError> {
        debug!("user.id: {id}");

        match Uuid::from_str(id.as_str()) {
            Ok(id) if id.get_version() == Some(Version::SortRand) => Ok(Self {
                id,
                active: true,
                name: HumanName::new(name, first_name, Some(last_name)),
                telecom: vec![],
                identifier,
                photo_url: None,
                birth_date: None,
                address: None,
                provider_info: IdentityProvider::Google,
                is_owner,
            }),
            Ok(id) => Err(ClickCareError::generic(format!(
                "El id no es un UUID V7, id: {}",
                id
            ))),
            Err(e) => {
                error!("Error desconocido al parsear el id, error: {e}");
                Err(ClickCareError::generic(format!(
                    "Error desconocido al parsear el id, error: {}",
                    e
                )))
            }
        }
    }
}


// ========================================================================
// 1. FHIR Data Type: HumanName (https://hl7.org/fhir/R4/datatypes.html#HumanName)
// ========================================================================
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanName {
    /// Nombres de pila ("given") -> ej. ["Juan", "Carlos"]
    pub given: Vec<String>,
    /// Primer Apellido ("family") -> ej. "Pérez"
    pub family: String,
    /// Segundo Apellido (extensión hispana / FHIR family) -> ej. "Gómez"
    pub second_family: Option<String>,
    /// Nombre completo formateado (opcional) -> ej. "Juan Carlos Pérez Gómez"
    pub text: Option<String>,
}

impl HumanName {
    pub fn new(given: Vec<String>, family: String, second_family: Option<String>) -> Self {
        let text = match &second_family {
            Some(sec) => format!("{} {} {}", given.join(" "), family, sec),
            None => format!("{} {}", given.join(" "), family),
        };
        Self {
            given,
            family,
            second_family,
            text: Some(text),
        }
    }

    pub fn full_name(&self) -> String {
        self.text.clone().unwrap_or_else(|| {
            match &self.second_family {
                Some(sec) => format!("{} {} {}", self.given.join(" "), self.family, sec),
                None => format!("{} {}", self.given.join(" "), self.family),
            }
        })
    }
}


// ========================================================================
// 2. FHIR Data Type: ContactPoint (https://hl7.org/fhir/R4/datatypes.html#ContactPoint)
// ========================================================================
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContactPointSystem {
    Phone,
    Email,
    Fax,
    Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContactPointUse {
    Home,
    Work,
    Mobile,
    Temp,
    Old,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactPoint {
    pub system: ContactPointSystem,
    pub value: String,
    pub use_type: Option<ContactPointUse>,
}

impl ContactPoint {
    pub fn email(value: impl Into<String>) -> Self {
        Self {
            system: ContactPointSystem::Email,
            value: value.into(),
            use_type: None,
        }
    }

    pub fn phone(value: impl Into<String>, use_type: Option<ContactPointUse>) -> Self {
        Self {
            system: ContactPointSystem::Phone,
            value: value.into(),
            use_type,
        }
    }
}



