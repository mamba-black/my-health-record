use app_core::domain::error::ClickCareError;
pub use app_core::domain::fhir::{
    ContactPoint, ContactPointSystem, ContactPointUse, HumanName, Identifier, LinkAssuranceLevel,
    Person, PersonLink, PersonLinkTarget,
};
use bon::Builder;
use derive_getters::Getters;
use log::{debug, error};
use std::str::FromStr;
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

/// Proveedor de Identidad OAuth2/OIDC.
#[derive(Debug, Clone)]
pub enum IdentityProvider {
    /// Autenticación provista por Google (e.g. `"google.com"`).
    Google,
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
        given: Vec<String>,
        family: Option<String>,
        second_family: Option<String>,
        identifier: Option<Identifier>,
        is_owner: bool,
        email: String,
        phone: Option<String>,
        birth_date: Option<String>,
    ) -> Result<Self, ClickCareError> {
        debug!("user.id: {id}");

        match Uuid::from_str(id.as_str()) {
            Ok(id) if id.get_version() == Some(Version::SortRand) => {
                let mut telecom = vec![ContactPoint::email(email)];
                if let Some(p) = phone
                    && !p.is_empty()
                {
                    telecom.push(ContactPoint::phone(p, None));
                }
                let person = Person {
                    id,
                    name: HumanName::new(given, family, second_family),
                    telecom,
                    identifier,
                    birth_date,
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

    /// Retorna el correo electrónico principal de la persona.
    pub fn email(&self) -> String {
        self.person
            .telecom
            .iter()
            .find(|c| c.system == ContactPointSystem::Email)
            .map(|c| c.value.clone())
            .unwrap_or_default()
    }

    /// Retorna el teléfono principal de la persona si existe.
    pub fn phone(&self) -> String {
        self.person
            .telecom
            .iter()
            .find(|c| c.system == ContactPointSystem::Phone)
            .map(|c| c.value.clone())
            .unwrap_or_default()
    }

    /// Retorna los nombres de pila como lista (FHIR: HumanName.given).
    pub fn given(&self) -> Vec<String> {
        self.person.name.given().clone()
    }

    /// Retorna los nombres de pila formateados en texto.
    pub fn given_name(&self) -> String {
        self.person.name.given().join(" ")
    }

    /// Retorna el primer apellido si existe.
    pub fn family_name(&self) -> Option<String> {
        self.person.name.family().clone()
    }

    /// Retorna el segundo apellido si existe.
    pub fn second_family_name(&self) -> Option<String> {
        self.person.name.second_family().clone()
    }

    /// Retorna el número de DNI si está registrado.
    pub fn identifier_dni(&self) -> Option<String> {
        self.person.identifier.as_ref().map(|id| match id {
            Identifier::DNI(val) => val.clone(),
        })
    }

    /// Retorna la fecha de nacimiento formateada en texto.
    pub fn birth_date(&self) -> String {
        self.person.birth_date.clone().unwrap_or_default()
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
