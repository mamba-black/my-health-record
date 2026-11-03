//! Modelos de transporte (`serde`) de los objetos de dominio compartidos.
//!
//! Viven en `infrastructure` para que el dominio no dependa de `serde`. Conservan
//! la misma forma JSON que tenían los tipos del dominio, así no se rompen los datos
//! ya guardados ni los mensajes que siguen en cola.

use crate::domain::event::UserCreatedEvent;
use crate::domain::events::FounderRegistered;
use crate::domain::fhir::{
    ContactPoint, ContactPointSystem, ContactPointUse, HumanName, Identifier, LinkAssuranceLevel,
    Person, PersonLink, PersonLinkTarget,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkAssuranceLevelDto {
    Level1,
    Level2,
    Level3,
    Level4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonLinkTargetDto {
    Patient(Uuid),
    Practitioner(Uuid),
    RelatedPerson(Uuid),
    Organization(Uuid),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonLinkDto {
    pub target: PersonLinkTargetDto,
    pub assurance: Option<LinkAssuranceLevelDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactPointSystemDto {
    Phone,
    Fax,
    Email,
    Pager,
    Url,
    Sms,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactPointUseDto {
    Home,
    Work,
    Mobile,
    Temp,
    Old,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactPointDto {
    pub system: ContactPointSystemDto,
    pub value: String,
    pub use_type: Option<ContactPointUseDto>,
    pub rank: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierDto {
    DNI(String),
}

/// Forma JSON de `HumanName`. Incluye `text`, igual que el tipo de dominio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanNameDto {
    given: Vec<String>,
    family: Option<String>,
    second_family: Option<String>,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonDto {
    pub id: Uuid,
    pub name: HumanNameDto,
    pub telecom: Vec<ContactPointDto>,
    pub identifier: Option<IdentifierDto>,
    pub birth_date: Option<String>,
    pub links: Vec<PersonLinkDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCreatedEventDto {
    pub user_id: Uuid,
    pub person: PersonDto,
    pub create_clinic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FounderRegisteredDto {
    pub user_id: Uuid,
    pub person_id: Uuid,
    pub organization_id: Uuid,
}

// ─── Dominio → transporte ────────────────────────────────────────────────────

impl From<LinkAssuranceLevel> for LinkAssuranceLevelDto {
    fn from(level: LinkAssuranceLevel) -> Self {
        match level {
            LinkAssuranceLevel::Level1 => Self::Level1,
            LinkAssuranceLevel::Level2 => Self::Level2,
            LinkAssuranceLevel::Level3 => Self::Level3,
            LinkAssuranceLevel::Level4 => Self::Level4,
        }
    }
}

impl From<PersonLinkTarget> for PersonLinkTargetDto {
    fn from(target: PersonLinkTarget) -> Self {
        match target {
            PersonLinkTarget::Patient(id) => Self::Patient(id),
            PersonLinkTarget::Practitioner(id) => Self::Practitioner(id),
            PersonLinkTarget::RelatedPerson(id) => Self::RelatedPerson(id),
            PersonLinkTarget::Organization(id) => Self::Organization(id),
        }
    }
}

impl From<PersonLink> for PersonLinkDto {
    fn from(link: PersonLink) -> Self {
        Self {
            target: link.target.into(),
            assurance: link.assurance.map(Into::into),
        }
    }
}

impl From<ContactPointSystem> for ContactPointSystemDto {
    fn from(system: ContactPointSystem) -> Self {
        match system {
            ContactPointSystem::Phone => Self::Phone,
            ContactPointSystem::Fax => Self::Fax,
            ContactPointSystem::Email => Self::Email,
            ContactPointSystem::Pager => Self::Pager,
            ContactPointSystem::Url => Self::Url,
            ContactPointSystem::Sms => Self::Sms,
            ContactPointSystem::Other => Self::Other,
        }
    }
}

impl From<ContactPointUse> for ContactPointUseDto {
    fn from(use_type: ContactPointUse) -> Self {
        match use_type {
            ContactPointUse::Home => Self::Home,
            ContactPointUse::Work => Self::Work,
            ContactPointUse::Mobile => Self::Mobile,
            ContactPointUse::Temp => Self::Temp,
            ContactPointUse::Old => Self::Old,
        }
    }
}

impl From<ContactPoint> for ContactPointDto {
    fn from(point: ContactPoint) -> Self {
        Self {
            system: point.system.into(),
            value: point.value,
            use_type: point.use_type.map(Into::into),
            rank: point.rank,
        }
    }
}

impl From<Identifier> for IdentifierDto {
    fn from(identifier: Identifier) -> Self {
        match identifier {
            Identifier::DNI(dni) => Self::DNI(dni),
        }
    }
}

impl From<HumanName> for HumanNameDto {
    fn from(name: HumanName) -> Self {
        Self {
            given: name.given().clone(),
            family: name.family().clone(),
            second_family: name.second_family().clone(),
            text: name.text().clone(),
        }
    }
}

impl From<Person> for PersonDto {
    fn from(person: Person) -> Self {
        Self {
            id: person.id,
            name: person.name.into(),
            telecom: person.telecom.into_iter().map(Into::into).collect(),
            identifier: person.identifier.map(Into::into),
            birth_date: person.birth_date,
            links: person.links.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<UserCreatedEvent> for UserCreatedEventDto {
    fn from(event: UserCreatedEvent) -> Self {
        Self {
            user_id: event.user_id,
            person: event.person.into(),
            create_clinic: event.create_clinic,
        }
    }
}

impl From<FounderRegistered> for FounderRegisteredDto {
    fn from(event: FounderRegistered) -> Self {
        Self {
            user_id: event.user_id,
            person_id: event.person_id,
            organization_id: event.organization_id,
        }
    }
}

// ─── Transporte → dominio ────────────────────────────────────────────────────

impl From<LinkAssuranceLevelDto> for LinkAssuranceLevel {
    fn from(level: LinkAssuranceLevelDto) -> Self {
        match level {
            LinkAssuranceLevelDto::Level1 => Self::Level1,
            LinkAssuranceLevelDto::Level2 => Self::Level2,
            LinkAssuranceLevelDto::Level3 => Self::Level3,
            LinkAssuranceLevelDto::Level4 => Self::Level4,
        }
    }
}

impl From<PersonLinkTargetDto> for PersonLinkTarget {
    fn from(target: PersonLinkTargetDto) -> Self {
        match target {
            PersonLinkTargetDto::Patient(id) => Self::Patient(id),
            PersonLinkTargetDto::Practitioner(id) => Self::Practitioner(id),
            PersonLinkTargetDto::RelatedPerson(id) => Self::RelatedPerson(id),
            PersonLinkTargetDto::Organization(id) => Self::Organization(id),
        }
    }
}

impl From<PersonLinkDto> for PersonLink {
    fn from(link: PersonLinkDto) -> Self {
        Self {
            target: link.target.into(),
            assurance: link.assurance.map(Into::into),
        }
    }
}

impl From<ContactPointSystemDto> for ContactPointSystem {
    fn from(system: ContactPointSystemDto) -> Self {
        match system {
            ContactPointSystemDto::Phone => Self::Phone,
            ContactPointSystemDto::Fax => Self::Fax,
            ContactPointSystemDto::Email => Self::Email,
            ContactPointSystemDto::Pager => Self::Pager,
            ContactPointSystemDto::Url => Self::Url,
            ContactPointSystemDto::Sms => Self::Sms,
            ContactPointSystemDto::Other => Self::Other,
        }
    }
}

impl From<ContactPointUseDto> for ContactPointUse {
    fn from(use_type: ContactPointUseDto) -> Self {
        match use_type {
            ContactPointUseDto::Home => Self::Home,
            ContactPointUseDto::Work => Self::Work,
            ContactPointUseDto::Mobile => Self::Mobile,
            ContactPointUseDto::Temp => Self::Temp,
            ContactPointUseDto::Old => Self::Old,
        }
    }
}

impl From<ContactPointDto> for ContactPoint {
    fn from(point: ContactPointDto) -> Self {
        Self {
            system: point.system.into(),
            value: point.value,
            use_type: point.use_type.map(Into::into),
            rank: point.rank,
        }
    }
}

impl From<IdentifierDto> for Identifier {
    fn from(identifier: IdentifierDto) -> Self {
        match identifier {
            IdentifierDto::DNI(dni) => Self::DNI(dni),
        }
    }
}

impl From<HumanNameDto> for HumanName {
    fn from(name: HumanNameDto) -> Self {
        // `HumanName::new` recalcula `text` a partir de las partes, igual que al crear el dato.
        HumanName::new(name.given, name.family, name.second_family)
    }
}

impl From<PersonDto> for Person {
    fn from(person: PersonDto) -> Self {
        Self {
            id: person.id,
            name: person.name.into(),
            telecom: person.telecom.into_iter().map(Into::into).collect(),
            identifier: person.identifier.map(Into::into),
            birth_date: person.birth_date,
            links: person.links.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<UserCreatedEventDto> for UserCreatedEvent {
    fn from(event: UserCreatedEventDto) -> Self {
        Self {
            user_id: event.user_id,
            person: event.person.into(),
            create_clinic: event.create_clinic,
        }
    }
}

impl From<FounderRegisteredDto> for FounderRegistered {
    fn from(event: FounderRegisteredDto) -> Self {
        Self {
            user_id: event.user_id,
            person_id: event.person_id,
            organization_id: event.organization_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::fhir::ContactPoint;

    #[test]
    fn person_dto_round_trips_through_json_without_losing_data() {
        let person = Person {
            id: Uuid::nil(),
            name: HumanName::new(
                vec!["Ana".into()],
                Some("Pérez".into()),
                Some("Gómez".into()),
            ),
            telecom: vec![ContactPoint::primary_email("ana@example.com")],
            identifier: Some(Identifier::DNI("12345678".into())),
            birth_date: Some("1990-01-01".into()),
            links: vec![PersonLink {
                target: PersonLinkTarget::Patient(Uuid::nil()),
                assurance: Some(LinkAssuranceLevel::Level3),
            }],
        };

        let json = serde_json::to_string(&PersonDto::from(person.clone())).expect("serializar");
        let back: PersonDto = serde_json::from_str(&json).expect("deserializar");

        assert_eq!(Person::from(back), person);
    }
}
