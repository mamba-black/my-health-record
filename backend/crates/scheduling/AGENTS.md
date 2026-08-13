# Reserva de Citas Bounded Context (`crates/scheduling`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de agendas de atención médica (`Schedule`), bloques de disponibilidad (`Slot`) y reserva de citas (`Appointment`).
* **Grupo FHIR**: FHIR Workflow.
* **Recursos FHIR Mapeados**: `Schedule`, `Slot`, `Appointment` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `schedule.rs`, `slot.rs`, `appointment.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id`, `practitioner_id`, `location_id` (UUIDv7).

---

## Diagrama de Clases

```mermaid
classDiagram
    class Schedule {
        <<FHIR Compliant: Schedule R4>>
        +Uuid id
        +Uuid practitioner_id
        +Uuid location_id
    }

    class Slot {
        <<FHIR Compliant: Slot R4>>
        +Uuid id
        +Uuid schedule_id
        +SlotStatus status
        +DateTime start
        +DateTime end
    }

    class Appointment {
        <<FHIR Compliant: Appointment R4>>
        +Uuid id
        +AppointmentStatus status
        +Uuid slot_id
        +Uuid patient_id
        +Uuid practitioner_id
    }

    Schedule "1" -- "0..*" Slot : define
    Appointment "0..*" -- "1" Slot : reserva
```
