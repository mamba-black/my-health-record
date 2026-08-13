# Identity & Security Bounded Context (`crates/user`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de credenciales, autenticación de sistema e identidad física del usuario.
* **Grupo FHIR**: Foundation / Security.
* **Recurso FHIR Mapeado**: `Person` (HL7 FHIR R4) + `User` (Cuenta de Sistema).
* **Módulos de Dominio (`src/domain/`)**: `user.rs`, `person.rs`, `identity_provider.rs`.

---

## Arquitectura y Composición de Identidad FHIR

```mermaid
graph TD
    subgraph auth_boundary["Límite de Autenticación de Sistema"]
        User["User (Cuenta de Sistema)<br/>id: UUID v7, active, provider_info"]
    end

    subgraph physical_identity["Identidad Física (FHIR R4 Person)"]
        Person["Person<br/>name: HumanName<br/>telecom: ContactPoint[]<br/>identifier: Identifier (DNI/CE)<br/>links: PersonLink[]"]
    end

    subgraph healthcare_roles["Roles Sanitarios (Recursos FHIR)"]
        Patient["Patient (crates/administration/src/domain/patient.rs)<br/>Expediente Clínico"]
        Practitioner["Practitioner (crates/administration/src/domain/practitioner.rs)<br/>Colegiatura Médica (CMP/COP)"]
        Organization["Organization (crates/administration/src/domain)<br/>Clínica / Entidad Legal"]
        RelatedPerson["RelatedPerson<br/>Tutor / Cuidador"]
    end

    User -->|Composición 1:1| Person
    Person -->|Person.link| Patient
    Person -->|Person.link| Practitioner
    Person -->|Person.link| Organization
    Person -->|Person.link| RelatedPerson
```

---

## Diagrama de Clases

```mermaid
classDiagram
    class User {
        +Uuid id
        +bool active
        +Person person
        +IdentityProvider provider_info
        +bool is_owner
        +new(...) Result~User, ClickCareError~
        +add_link(target, assurance)
        +patient_ids() Vec~Uuid~
        +organization_ids() Vec~Uuid~
    }

    class Person {
        +Uuid id
        +HumanName name
        +Vec~ContactPoint~ telecom
        +Option~Identifier~ identifier
        +Vec~PersonLink~ links
        +add_link(target, assurance)
        +patient_ids() Vec~Uuid~
        +organization_ids() Vec~Uuid~
    }

    class HumanName {
        -Vec~String~ given
        -String family
        -Option~String~ second_family
        -String text
        +new(given, family, second_family) HumanName
        +builder() HumanNameBuilder
        +given() Vec~String~
        +family() String
        +second_family() Option~String~
        +text() String
    }

    class ContactPoint {
        +ContactPointSystem system
        +String value
        +Option~ContactPointUse~ use_type
        +email(value) ContactPoint
        +phone(value, use_type) ContactPoint
    }

    class ContactPointSystem {
        <<enumeration>>
        Phone
        Email
        Fax
        Url
    }

    class ContactPointUse {
        <<enumeration>>
        Home
        Work
        Mobile
        Temp
        Old
    }

    class Identifier {
        +IdentifierType doc_type
        +String value
        +Option~String~ system
        +dni(value) Identifier
    }

    class IdentityProvider {
        <<enumeration>>
        Google
    }

    class PersonLink {
        +PersonLinkTarget target
        +Option~LinkAssuranceLevel~ assurance
    }

    class PersonLinkTarget {
        <<enumeration>>
        Patient(Uuid)
        Practitioner(Uuid)
        RelatedPerson(Uuid)
        Organization(Uuid)
    }

    class LinkAssuranceLevel {
        <<enumeration>>
        Level1
        Level2
        Level3
        Level4
    }

    User "1" *-- "1" Person : contiene (User -> Person)
    User "1" *-- "1" IdentityProvider : autenticado por
    Person "1" *-- "1" HumanName : nombra a
    Person "1" *-- "0..*" ContactPoint : contactado vía
    Person "1" *-- "0..1" Identifier : identificado por
    Person "1" *-- "0..*" PersonLink : enlaza a
    ContactPoint "1" *-- "1" ContactPointSystem : sistema
    ContactPoint "0..1" *-- "1" ContactPointUse : uso
    PersonLink "1" *-- "1" PersonLinkTarget : destino
    PersonLink "0..1" *-- "1" LinkAssuranceLevel : certeza
```
