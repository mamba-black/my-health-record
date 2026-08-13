# Facturación Bounded Context (`crates/billing`)

## Especificación del Dominio

* **Responsabilidad**: Cuentas de facturación del paciente (`Account`), ítems cobrables acumulados por atención médica (`ChargeItem`) e inserción de comprobantes/facturas (`Invoice`).
* **Grupo FHIR**: FHIR Financial.
* **Recursos FHIR Mapeados**: `Account`, `Invoice`, `ChargeItem` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `account.rs`, `invoice.rs`, `charge_item.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id`, `encounter_id`, `coverage_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class Account {
        <<FHIR Compliant: Account R4>>
        +Uuid id
        +Uuid patient_id
        +AccountStatus status
    }

    class ChargeItem {
        <<FHIR Compliant: ChargeItem R4>>
        +Uuid id
        +Uuid account_id
        +Uuid encounter_id
        +Decimal amount
    }

    class Invoice {
        <<FHIR Compliant: Invoice R4>>
        +Uuid id
        +Uuid account_id
        +InvoiceStatus status
        +Decimal total_amount
    }

    Account "1" -- "0..*" ChargeItem : acumula
    Invoice "1" *-- "1..*" ChargeItem : agrupa
```
