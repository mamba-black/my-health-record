# Aseguradoras y Coberturas Bounded Context (`crates/coverage`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de pólizas de seguro médico (`Coverage`), verificación de elegibilidad (`CoverageEligibilityRequest`) y reclamaciones financieras (`Claim` / `ClaimResponse`).
* **Grupo FHIR**: FHIR Financial / Claims.
* **Recursos FHIR Mapeados**: `Coverage`, `Claim`, `ClaimResponse`, `CoverageEligibilityRequest` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `policy.rs`, `claim.rs`, `eligibility.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class Coverage {
        <<FHIR Compliant: Coverage R4>>
        +Uuid id
        +Uuid patient_id
        +String subscriber_id
        +String insurer_code
    }

    class CoverageEligibilityRequest {
        <<FHIR Compliant: CoverageEligibilityRequest R4>>
        +Uuid id
        +Uuid coverage_id
        +DateTime requested_at
    }

    class Claim {
        <<FHIR Compliant: Claim R4>>
        +Uuid id
        +Uuid coverage_id
        +Uuid patient_id
        +Decimal total_amount
    }

    Coverage "1" -- "0..*" CoverageEligibilityRequest : verifica
    Coverage "1" -- "0..*" Claim : ampara
```
