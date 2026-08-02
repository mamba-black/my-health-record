use app_core::domain::error::ClickCareError;
use bon::{bon, Builder};
use derive_getters::Getters;
use log::{debug, error};
use std::str::FromStr;
use strum_macros::Display;
use uuid::{Uuid, Version};

/// Entidad de dominio que representa la **Cuenta de Usuario** del sistema.
///
/// Separa los metadatos de autenticación/cuenta (`User`) de la identidad física
/// y demográfica de la persona (`Person`), siguiendo **Onion Architecture** y **DDD**.
#[derive(Debug, Clone, Getters, Builder)]
pub struct User {
    /// Identificador único de la cuenta de usuario (UUID v7).
    pub id: Uuid,
    /// Estado activo/inactivo de la cuenta.
    pub active: bool,

    /// Recurso de identidad y demografía de la persona física (FHIR `Person`).
    pub person: Person,

    /// Proveedor de identidad utilizado para la autenticación (ej. Google).
    pub provider_info: IdentityProvider,
    /// Indica si el usuario es propietario o administrador de clínica.
    pub is_owner: bool,
}

/// Proyección mínima de la persona física (FHIR R4 Person) en el contexto de `User`
/// necesaria para la búsqueda de cuentas, autenticación y validación de unicidad de identidad.
///
/// Ref: <https://hl7.org/fhir/R4/person.html>
#[derive(Debug, Clone, Getters, Builder)]
pub struct Person {
    /// Identificador único de la persona (UUID v7).
    pub id: Uuid,
    /// Estructura de nombres de la persona (FHIR: `Person.name` / `HumanName`).
    pub name: HumanName,
    /// Lista de puntos de contacto (FHIR: `Person.telecom` / `ContactPoint`).
    pub telecom: Vec<ContactPoint>,
    /// Documento o identificador oficial opcional (FHIR: `Person.identifier`).
    pub identifier: Option<Identifier>,
    /// Enlaces a los distintos roles y recursos FHIR asociados (FHIR: `Person.link`).
    pub links: Vec<PersonLink>,
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

// ========================================================================
// FHIR Person.link Data Structures
// ========================================================================

/// Enlace entre la persona/usuario y recursos relacionados en FHIR (`Person.link`).
///
/// Ref: <https://hl7.org/fhir/R4/person-definitions.html#Person.link>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonLink {
    /// Recurso de destino vinculado (Patient, Practitioner, RelatedPerson, Organization).
    pub target: PersonLinkTarget,
    /// Nivel de certeza o verificación de la vinculación (FHIR: `Person.link.assurance`).
    pub assurance: Option<LinkAssuranceLevel>,
}

/// Recurso de destino al cual está vinculada la identidad de la persona física.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonLinkTarget {
    /// Vinculado a un expediente de Paciente (`Patient` ID).
    Patient(Uuid),
    /// Vinculado a un perfil de Profesional de Salud (`Practitioner` ID).
    Practitioner(Uuid),
    /// Vinculado a una relación de Tutor o Cuidador Familiar (`RelatedPerson` ID).
    RelatedPerson(Uuid),
    /// Vinculado a una Clínica u Organización Sanitaria (`Organization` ID).
    Organization(Uuid),
}

/// Nivel de certeza del enlace según HL7 FHIR (`Person.link.assurance`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAssuranceLevel {
    /// Nivel 1: Certeza muy baja (declarativa).
    Level1,
    /// Nivel 2: Certeza baja.
    Level2,
    /// Nivel 3: Certeza alta (verificada).
    Level3,
    /// Nivel 4: Certeza absoluta (documentada y verificada oficialmente).
    Level4,
}

impl User {
    /// Crea una nueva instancia validada de `User` y su `Person` asociada.
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
            Ok(id) if id.get_version() == Some(Version::SortRand) => {
                let person = Person {
                    id,
                    name: HumanName::new(name, first_name, Some(last_name)),
                    telecom: vec![ContactPoint::email(email)],
                    identifier,
                    links: vec![],
                };

                Ok(Self {
                    id,
                    active: true,
                    person,
                    provider_info: IdentityProvider::Google,
                    is_owner,
                })
            }
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

    /// Vincula un nuevo recurso FHIR (Patient, Practitioner, Organization) a la identidad de la persona.
    pub fn add_link(&mut self, target: PersonLinkTarget, assurance: Option<LinkAssuranceLevel>) {
        self.person.add_link(target, assurance);
    }

    /// Retorna todos los IDs de recursos `Patient` asociados a esta cuenta de usuario.
    pub fn patient_ids(&self) -> Vec<Uuid> {
        self.person.patient_ids()
    }

    /// Retorna todos los IDs de recursos `Organization` (clínicas) asociados a esta cuenta.
    pub fn organization_ids(&self) -> Vec<Uuid> {
        self.person.organization_ids()
    }
}

impl Person {
    /// Vincula un nuevo recurso FHIR a la persona.
    pub fn add_link(&mut self, target: PersonLinkTarget, assurance: Option<LinkAssuranceLevel>) {
        self.links.push(PersonLink { target, assurance });
    }

    /// Retorna todos los IDs de `Patient` vinculados.
    pub fn patient_ids(&self) -> Vec<Uuid> {
        self.links
            .iter()
            .filter_map(|link| match link.target {
                PersonLinkTarget::Patient(id) => Some(id),
                _ => None,
            })
            .collect()
    }

    /// Retorna todos los IDs de `Organization` vinculados.
    pub fn organization_ids(&self) -> Vec<Uuid> {
        self.links
            .iter()
            .filter_map(|link| match link.target {
                PersonLinkTarget::Organization(id) => Some(id),
                _ => None,
            })
            .collect()
    }
}

// ========================================================================
// 1. FHIR Data Type: HumanName (https://hl7.org/fhir/R4/datatypes.html#HumanName)
// ========================================================================

/// Tipo de dato HL7 FHIR R4: `HumanName`.
///
/// Ref: <https://hl7.org/fhir/R4/datatypes.html#HumanName>
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct HumanName {
    /// Nombres de pila (`given`), ej. `["Juan", "Carlos"]`.
    given: Vec<String>,
    /// Primer apellido (`family`), ej. `"Pérez"`.
    family: String,
    /// Segundo apellido (extensión hispana de `family`), ej. `"Gómez"`.
    second_family: Option<String>,
    /// Nombre completo formateado y representable en texto (`text`).
    text: String,
}

#[bon]
impl HumanName {
    /// Smart Constructor principal: Garantiza que `text` siempre sea pre-calculado e inmutable.
    pub fn new(given: Vec<String>, family: String, second_family: Option<String>) -> Self {
        let text = match &second_family {
            Some(sec) => format!("{} {} {}", given.join(" "), family, sec),
            None => format!("{} {}", given.join(" "), family),
        };
        Self {
            given,
            family,
            second_family,
            text,
        }
    }

    /// Builder fluente provisto por `bon` que reutiliza la lógica de `new()`.
    #[builder]
    pub fn builder(given: Vec<String>, family: String, second_family: Option<String>) -> Self {
        Self::new(given, family, second_family)
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
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder)]
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






