# Archivo Legal y Auditoría Bounded Context (`crates/legal_archive`)

## Especificación del Dominio

* **Responsabilidad**: Documentación clínica consolidada legal (`Composition`), referencias de adjuntos (`DocumentReference`), registros inmutables de auditoría (`AuditEvent`) y trazabilidad/firmas digitales de autoría (`Provenance`).
* **Grupo FHIR**: FHIR Documents & Security.
* **Recursos FHIR Mapeados**: `Composition`, `DocumentReference`, `AuditEvent`, `Provenance` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `composition.rs`, `document_reference.rs`, `audit_event.rs`, `provenance.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id`, `practitioner_id`, `encounter_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class Composition {
        <<FHIR Compliant: Composition R4>>
        +Uuid id
        +Uuid patient_id
        +Uuid encounter_id
        +String document_type
    }

    class DocumentReference {
        <<FHIR Compliant: DocumentReference R4>>
        +Uuid id
        +Uuid composition_id
        +String attachment_url
    }

    class AuditEvent {
        <<FHIR Compliant: AuditEvent R4>>
        +Uuid id
        +DateTime recorded_at
        +String action
        +Uuid agent_user_id
    }

    class Provenance {
        <<FHIR Compliant: Provenance R4>>
        +Uuid id
        +Uuid target_entity_id
        +String digital_signature
    }

    Composition "1" -- "0..*" DocumentReference : adjunta
    Composition "1" -- "0..*" AuditEvent : audita accesos
    Composition "1" -- "0..1" Provenance : certifica autoría
```
