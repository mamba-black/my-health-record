use crate::domain::fhir::{ContactPoint, HumanName, Identifier};
use bon::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Nivel de certeza del enlace según HL7 FHIR (`Person.link.assurance`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Recurso de destino al cual está vinculada la identidad de la persona física.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Enlace entre la persona/usuario y recursos relacionados en FHIR (`Person.link`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonLink {
    pub target: PersonLinkTarget,
    pub assurance: Option<LinkAssuranceLevel>,
}

/// Recurso Persona (Identidad Humana) según la especificación HL7 FHIR R4.
/// Ref: <https://hl7.org/fhir/R4/person.html>
#[derive(Debug, Clone, Getters, Builder, Serialize, Deserialize, PartialEq, Eq)]
pub struct Person {
    /// Identificador único de la persona (UUID v7).
    pub id: Uuid,
    /// Estructura de nombres de la persona (FHIR: `Person.name` / `HumanName`).
    pub name: HumanName,
    /// Lista de puntos de contacto (FHIR: `Person.telecom` / `ContactPoint`).
    pub telecom: Vec<ContactPoint>,
    /// Documento o identificador oficial opcional (FHIR: `Person.identifier`).
    pub identifier: Option<Identifier>,
    /// Fecha de nacimiento oficial (FHIR: `Person.birthDate`).
    pub birth_date: Option<String>,
    /// Enlaces a los distintos roles y recursos FHIR asociados (FHIR: `Person.link`).
    pub links: Vec<PersonLink>,
}

impl Person {
    pub fn new(
        id: Uuid,
        name: HumanName,
        telecom: Vec<ContactPoint>,
        identifier: Option<Identifier>,
        birth_date: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            telecom,
            identifier,
            birth_date,
            links: vec![],
        }
    }

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
