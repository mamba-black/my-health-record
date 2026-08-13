# Farmacia e Insumos Bounded Context (`crates/pharmacy`)

## Especificación del Dominio

* **Responsabilidad**: Vademécum/catálogo SNOMED CT (`Medication`), dispensación en farmacia (`MedicationDispense`), administración de fármacos (`MedicationAdministration`) e insumos médicos (`SupplyRequest` / `SupplyDelivery`).
* **Grupo FHIR**: FHIR Medications & Supply.
* **Recursos FHIR Mapeados**: `Medication`, `MedicationDispense`, `MedicationAdministration`, `SupplyRequest` / `SupplyDelivery` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `medication.rs`, `medication_dispense.rs`, `medication_administration.rs`, `supply.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `medication_request_id`, `patient_id`, `encounter_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class Medication {
        <<FHIR Compliant: Medication R4>>
        +Uuid id
        +String code_snomed
        +String brand_name
    }

    class MedicationDispense {
        <<FHIR Compliant: MedicationDispense R4>>
        +Uuid id
        +Uuid medication_request_id
        +Uuid medication_id
        +String lot_number
        +DateTime expiration_date
        +Decimal quantity
    }

    class MedicationAdministration {
        <<FHIR Compliant: MedicationAdministration R4>>
        +Uuid id
        +Uuid medication_request_id
        +Uuid practitioner_id
        +DateTime administered_at
    }

    class SupplyRequest {
        <<FHIR Compliant: SupplyRequest R4>>
        +Uuid id
        +Uuid item_id
        +Decimal quantity
    }

    MedicationDispense "0..*" -- "1" Medication : entrega producto
    MedicationAdministration "0..*" -- "1" Medication : aplica producto
```
