# Gestión Administrativa Bounded Context (`crates/administration`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de expedientes de pacientes, profesionales de salud (con colegiatura CMP/COP), locaciones/consultorios y oferta de servicios sanitarios.
* **Grupos FHIR**: FHIR Individuals / FHIR Entities.
* **Recursos FHIR Mapeados**: `Patient`, `Practitioner`, `Location`, `HealthcareService` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `patient.rs`, `practitioner.rs`, `location.rs`, `healthcare_service.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `user_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class Patient {
        <<FHIR Compliant: Patient R4>>
        +Uuid id
        +bool active
        +Option~Gender~ gender
        +Option~NaiveDate~ birth_date
        +Uuid user_id
    }

    class Practitioner {
        <<FHIR Compliant: Practitioner R4>>
        +Uuid id
        +bool active
        +String medical_license_number
        +Uuid user_id
    }

    class Location {
        <<FHIR Compliant: Location R4>>
        +Uuid id
        +String name
        +LocationStatus status
    }

    class HealthcareService {
        <<FHIR Compliant: HealthcareService R4>>
        +Uuid id
        +Uuid location_id
        +String specialty_code
    }

    Location "1" -- "0..*" HealthcareService : ofrece
```
