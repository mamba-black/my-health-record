use app_core::domain::error::ClickCareError;
use chrono::NaiveDate;
use log::{debug, error};
use std::str::FromStr;
use strum_macros::Display;
use uuid::{Uuid, Version};

/// Entidad de dominio que representa un Usuario en el sistema.
///
/// Alineado con las especificaciones de **HL7 FHIR R4** (`Person` / `Patient`).
/// Sigue la arquitectura Cebolla (Onion Architecture) y requiere identificadores
/// primarios en formato **UUID v7**.
#[derive(Debug, Clone)]
pub struct User {
    /// Identificador único del usuario (obligatorio UUID v7).
    pub id: Uuid,
    /// Estado activo/inactivo del usuario en el sistema.
    pub active: bool,

    /// Estructura de nombres del usuario (FHIR: `Person.name` / `HumanName`).
    pub name: HumanName,
    /// Lista de puntos de contacto (FHIR: `Person.telecom` / `ContactPoint`).
    pub telecom: Vec<ContactPoint>,
    /// Documento o identificador oficial opcional (FHIR: `Person.identifier`).
    pub identifier: Option<Identifier>,
    /// URL de la imagen de perfil o avatar (FHIR: `Person.photo`).
    pub photo_url: Option<String>,
    /// Fecha de nacimiento (FHIR: `Person.birthDate`).
    pub birth_date: Option<NaiveDate>,
    /// Dirección domiciliaria o residencial (FHIR: `Person.address`).
    pub address: Option<String>,

    /// Proveedor de identidad utilizado para la autenticación (ej. Google).
    pub provider_info: IdentityProvider,
    /// Indica si el usuario es propietario o administrador de clínica.
    pub is_owner: bool,
}

/// Identificador oficial del usuario (FHIR Identifier).
#[derive(Debug, Clone, Display)]
pub enum Identifier {
    /// Documento Nacional de Identidad (DNI).
    #[strum(to_string = "DNI: {0}")]
    DNI(String),
}

/// Proveedor de Identidad OAuth2/OIDC.
#[derive(Debug, Clone)]
pub enum IdentityProvider {
    /// Autenticación provista por Google (e.g. `"google.com"`).
    Google,
}

impl User {
    /// Crea una nueva instancia validada de `User`.
    ///
    /// # Parámetros
    /// - `id`: String representation del UUID, **debe ser un UUID v7**.
    /// - `name`: Lista de nombres de pila.
    /// - `first_name`: Primer apellido (family_name).
    /// - `last_name`: Segundo apellido (second_family_name).
    /// - `identifier`: Identificador oficial opcional.
    /// - `is_owner`: Si el usuario posee rol de propietario/administrador.
    /// - `email`: Correo electrónico principal.
    ///
    /// # Errores
    /// Retorna `ClickCareError` si el `id` suministrado no es un UUID v7 válido.
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

/// Tipo de dato HL7 FHIR R4: `HumanName`.
///
/// Ref: <https://hl7.org/fhir/R4/datatypes.html#HumanName>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanName {
    /// Nombres de pila (`given`), ej. `["Juan", "Carlos"]`.
    pub given: Vec<String>,
    /// Primer apellido (`family`), ej. `"Pérez"`.
    pub family: String,
    /// Segundo apellido (extensión hispana de `family`), ej. `"Gómez"`.
    pub second_family: Option<String>,
    /// Nombre completo formateado y representable en texto.
    pub text: Option<String>,
}

impl HumanName {
    /// Construye una nueva instancia de `HumanName` calculando el campo `text`.
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

    /// Retorna la representación en texto completo del nombre de la persona.
    pub fn full_name(&self) -> String {
        self.text.clone().unwrap_or_else(|| match &self.second_family {
            Some(sec) => format!("{} {} {}", self.given.join(" "), self.family, sec),
            None => format!("{} {}", self.given.join(" "), self.family),
        })
    }
}

// ========================================================================
// 2. FHIR Data Type: ContactPoint (https://hl7.org/fhir/R4/datatypes.html#ContactPoint)
// ========================================================================

/// Sistema de telecomunicación utilizado según FHIR R4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContactPointSystem {
    /// Teléfono de voz.
    Phone,
    /// Correo electrónico.
    Email,
    /// Fax.
    Fax,
    /// URL o dirección web.
    Url,
}

/// Uso o propósito del punto de contacto según FHIR R4.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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




