use app_core::domain::access::{ProfileField, ProtectedAction, Role, missing_person_fields};
use app_core::domain::error::ClickCareError;
use app_core::domain::event::UserCreatedEvent;
pub use app_core::domain::fhir::{
    ContactPoint, ContactPointSystem, ContactPointUse, HumanName, Identifier, LinkAssuranceLevel,
    Person, PersonLink, PersonLinkTarget,
};
use bon::Builder;
use derive_getters::Getters;
use std::collections::HashSet;
use std::str::FromStr;
use tracing::{debug, error};
use uuid::{Uuid, Version};

/// Sexo registrado (FHIR `AdministrativeGender`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Female,
    Male,
    Other,
}

impl Sex {
    /// Código persistido en `identity.user_account.sex`.
    pub fn code(self) -> &'static str {
        match self {
            Sex::Female => "FEMALE",
            Sex::Male => "MALE",
            Sex::Other => "OTHER",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "FEMALE" => Some(Sex::Female),
            "MALE" => Some(Sex::Male),
            "OTHER" => Some(Sex::Other),
            _ => None,
        }
    }
}

/// Datos profesionales del médico. Son de la persona y valen en todas las clínicas.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfessionalProfile {
    /// Colegio profesional (CMP, COP, CEP, CPsP…).
    pub professional_body: Option<String>,
    /// Número de colegiatura.
    pub license_number: Option<String>,
    pub specialty: Option<String>,
    /// Registro Nacional de Especialista.
    pub rne: Option<String>,
}

/// Entidad de dominio que representa la **Cuenta de Usuario** del sistema.
///
/// Separa los metadatos de autenticación/cuenta (`User`) de la identidad física
/// y demográfica de la persona (`Person`), siguiendo **Onion Architecture** y **DDD**.
#[derive(Debug, Clone, Getters, Builder)]
pub struct User {
    /// Identificador único de la cuenta de usuario (UUID v7).
    pub id: Uuid,
    /// Red de clínicas a la que pertenece esta cuenta de usuario (UUID v7).
    pub network_id: Uuid,
    /// Estado activo/inactivo de la cuenta.
    pub active: bool,

    /// Recurso de identidad y demografía de la persona física (FHIR `Person`).
    pub person: Person,

    /// Proveedor de identidad utilizado para la autenticación (ej. Google).
    pub provider_info: IdentityProvider,
    /// Indica si el usuario es propietario o administrador de clínica.
    pub is_owner: bool,
    /// Dirección domiciliaria.
    pub address: Option<String>,
    /// Rol preferido: orienta la interfaz, no concede permisos.
    #[builder(default)]
    pub preferred_role: Role,
    pub sex: Option<Sex>,
    #[builder(default)]
    pub professional: ProfessionalProfile,
    /// El documento fue verificado presencialmente: el usuario ya no lo edita.
    #[builder(default)]
    pub document_verified: bool,
    /// Una clínica confirmó la colegiatura: el usuario ya no la edita.
    #[builder(default)]
    pub license_verified: bool,
}

/// Proveedor de Identidad OAuth2/OIDC.
#[derive(Debug, Clone)]
pub enum IdentityProvider {
    /// Autenticación provista por Google (e.g. `"google.com"`).
    Google,
}

/// Intención inicial declarada por el usuario durante el registro preliminar (perfilado progresivo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignUpIntent {
    /// Intención no especificada.
    #[default]
    Unspecified,
    /// Dueño o fundador de clínica/organización.
    ClinicOwner,
    /// Profesional de la salud (médico, enfermero, etc.).
    Practitioner,
    /// Paciente o receptor de atención de salud.
    Patient,
    /// Recepción de una clínica existente. No crea clínicas: pide entrar a una.
    Receptionist,
}

impl SignUpIntent {
    /// Rol preferido que se guarda a partir de la intención del registro. Sin intención
    /// declarada, `create_clinic` (cliente heredado) equivale a dueño; si no, paciente.
    pub fn preferred_role(self, create_clinic: bool) -> Role {
        match self {
            SignUpIntent::ClinicOwner => Role::ClinicOwner,
            SignUpIntent::Practitioner => Role::Practitioner,
            SignUpIntent::Patient => Role::Patient,
            SignUpIntent::Receptionist => Role::Receptionist,
            SignUpIntent::Unspecified if create_clinic => Role::ClinicOwner,
            SignUpIntent::Unspecified => Role::Patient,
        }
    }
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
        network_id: Uuid,
        given: Vec<String>,
        family: Option<String>,
        second_family: Option<String>,
        identifier: Option<Identifier>,
        is_owner: bool,
        email: String,
        phone: Option<String>,
        birth_date: Option<String>,
    ) -> Result<Self, ClickCareError> {
        debug!("user.id: {id} network_id: {network_id}");

        match Uuid::from_str(id.as_str()) {
            Ok(id) if id.get_version() == Some(Version::SortRand) => {
                let mut telecom = vec![ContactPoint::primary_email(email)];
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
                    network_id,
                    active: true,
                    person,
                    provider_info: IdentityProvider::Google,
                    is_owner,
                    address: None,
                    preferred_role: Role::Patient,
                    sex: None,
                    professional: ProfessionalProfile::default(),
                    document_verified: false,
                    license_verified: false,
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

    pub fn email(&self) -> String {
        self.person
            .telecom()
            .iter()
            .find(|telecom_entry| telecom_entry.system == ContactPointSystem::Email)
            .map(|telecom_entry| telecom_entry.value.clone())
            .unwrap_or_default()
    }

    /// Retorna el teléfono de contacto, si la cuenta tiene uno.
    pub fn phone(&self) -> Option<String> {
        self.person
            .telecom()
            .iter()
            .find(|telecom_entry| telecom_entry.system == ContactPointSystem::Phone)
            .map(|telecom_entry| telecom_entry.value.clone())
    }

    /// Datos personales y profesionales que la cuenta ya tiene registrados.
    pub fn present_fields(&self) -> HashSet<ProfileField> {
        let filled =
            |value: &Option<String>| value.as_deref().is_some_and(|v| !v.trim().is_empty());
        let mut present = HashSet::new();
        let mut mark = |condition: bool, field: ProfileField| {
            if condition {
                present.insert(field);
            }
        };
        mark(
            !self.person.name().given().join("").trim().is_empty(),
            ProfileField::GivenName,
        );
        mark(
            filled(self.person.name().family()),
            ProfileField::FamilyName,
        );
        mark(
            filled(self.person.name().second_family()),
            ProfileField::SecondFamilyName,
        );
        mark(!self.email().trim().is_empty(), ProfileField::Email);
        mark(filled(&self.phone()), ProfileField::Phone);
        mark(self.person.identifier().is_some(), ProfileField::Document);
        mark(
            !self.birth_date().trim().is_empty(),
            ProfileField::BirthDate,
        );
        mark(self.sex.is_some(), ProfileField::Sex);
        mark(filled(&self.address), ProfileField::Address);
        mark(
            filled(&self.professional.professional_body)
                && filled(&self.professional.license_number),
            ProfileField::ProfessionalLicense,
        );
        mark(
            filled(&self.professional.specialty),
            ProfileField::Specialty,
        );
        present
    }

    /// Datos personales que le faltan para una acción.
    pub fn missing_for(&self, action: ProtectedAction) -> Vec<ProfileField> {
        missing_person_fields(action, &self.present_fields())
    }

    /// Retorna la fecha de nacimiento formateada en texto.
    pub fn birth_date(&self) -> String {
        self.person.birth_date.clone().unwrap_or_default()
    }
}

impl From<User> for UserCreatedEvent {
    fn from(user: User) -> Self {
        UserCreatedEvent {
            user_id: user.id,
            person: user.person.clone(),
            create_clinic: user.is_owner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Escenarios «Cuenta sin intención declarada» y persistencia de la intención.
    #[test]
    fn la_intencion_define_el_rol_preferido() {
        assert_eq!(
            SignUpIntent::Unspecified.preferred_role(false),
            Role::Patient
        );
        assert_eq!(
            SignUpIntent::Unspecified.preferred_role(true),
            Role::ClinicOwner
        );
        assert_eq!(
            SignUpIntent::Practitioner.preferred_role(false),
            Role::Practitioner
        );
        assert_eq!(
            SignUpIntent::ClinicOwner.preferred_role(false),
            Role::ClinicOwner
        );
        assert_eq!(SignUpIntent::Patient.preferred_role(true), Role::Patient);
        assert_eq!(
            SignUpIntent::Receptionist.preferred_role(true),
            Role::Receptionist
        );
    }
}
