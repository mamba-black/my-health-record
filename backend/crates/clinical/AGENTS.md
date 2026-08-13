# Historia Clínica Bounded Context (`crates/clinical`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de encuentros clínicos (`Encounter`), diagnósticos CIE-10 (`Condition`), alergias (`AllergyIntolerance`), planes de cuidado (`CarePlan`), prescripciones de medicamentos (`MedicationRequest`) y formularios dinámicos (`Questionnaire`).
* **Grupos FHIR**: FHIR Management / FHIR Clinical Summary / FHIR Care Provision / FHIR Diagnostics & Forms.
* **Recursos FHIR Mapeados**: `Encounter`, `Condition`, `AllergyIntolerance`, `CarePlan`, `MedicationRequest`, `Questionnaire` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `encounter.rs`, `condition.rs`, `allergy_intolerance.rs`, `care_plan.rs`, `medication_request.rs`, `questionnaire.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id`, `practitioner_id`, `appointment_id` (UUIDv7). Nota: De acuerdo a HL7 FHIR R4, `appointment_id` es opcional (`Option<Uuid>`) para permitir atenciones directas/urgencias.

---

## Diagrama de Clases

```mermaid
classDiagram
    class Encounter {
        <<FHIR Compliant: Encounter R4>>
        +Uuid id
        +EncounterStatus status
        +Uuid appointment_id
        +Uuid patient_id
        +Uuid practitioner_id
        +DateTime period_start
    }

    class Condition {
        <<FHIR Compliant: Condition R4>>
        +Uuid id
        +Uuid encounter_id
        +String cie10_code
        +String clinical_status
        +Option~String~ note
    }

    class AllergyIntolerance {
        <<FHIR Compliant: AllergyIntolerance R4>>
        +Uuid id
        +Uuid patient_id
        +String substance_code
    }

    class CarePlan {
        <<FHIR Compliant: CarePlan R4>>
        +Uuid id
        +Uuid encounter_id
        +String title
    }

    class MedicationRequest {
        <<FHIR Compliant: MedicationRequest R4>>
        +Uuid id
        +Uuid encounter_id
        +Uuid condition_id
        +String dosage_instruction
        +String timing_schedule
    }

    class Questionnaire {
        <<FHIR Compliant: Questionnaire R4>>
        +Uuid id
        +String title
        +Vec~Question~ items
    }

    Encounter "1" -- "0..*" Condition : diagnostica
    Encounter "1" -- "0..*" AllergyIntolerance : identifica
    Encounter "1" -- "0..*" CarePlan : establece
    Encounter "1" -- "0..*" MedicationRequest : prescribe
    MedicationRequest "1" -- "1" Condition : fundamentada en
```
