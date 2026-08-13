# Laboratorio e Imágenes Bounded Context (`crates/diagnostics`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de órdenes de exámenes clínicos (`ServiceRequest`), hallazgos/observaciones LOINC (`Observation`), reportes diagnósticos (`DiagnosticReport`), muestras biológicas (`Specimen`) y estudios radiológicos DICOM (`ImagingStudy`).
* **Grupo FHIR**: FHIR Diagnostics.
* **Recursos FHIR Mapeados**: `ServiceRequest`, `Observation`, `DiagnosticReport`, `ImagingStudy`, `Specimen` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `service_request.rs`, `observation.rs`, `diagnostic_report.rs`, `imaging_study.rs`, `specimen.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id`, `encounter_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class ServiceRequest {
        <<FHIR Compliant: ServiceRequest R4>>
        +Uuid id
        +Uuid encounter_id
        +Uuid patient_id
        +String code
    }

    class DiagnosticReport {
        <<FHIR Compliant: DiagnosticReport R4>>
        +Uuid id
        +Uuid service_request_id
        +ReportStatus status
    }

    class Observation {
        <<FHIR Compliant: Observation R4>>
        +Uuid id
        +Uuid diagnostic_report_id
        +String loinc_code
        +String value_quantity
    }

    class Specimen {
        <<FHIR Compliant: Specimen R4>>
        +Uuid id
        +Uuid service_request_id
        +String type_code
    }

    class ImagingStudy {
        <<FHIR Compliant: ImagingStudy R4>>
        +Uuid id
        +Uuid service_request_id
        +String dicom_uid
    }

    ServiceRequest "1" -- "0..1" DiagnosticReport : genera
    ServiceRequest "1" -- "0..*" Specimen : toma
    ServiceRequest "1" -- "0..*" ImagingStudy : produce
    DiagnosticReport "1" *-- "0..*" Observation : contiene
```
