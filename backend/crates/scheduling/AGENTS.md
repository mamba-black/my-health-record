# Reserva de Citas Bounded Context (`crates/scheduling`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de agendas de atención médica (`Schedule`), bloques de disponibilidad (`Slot`) y reserva de citas (`Appointment`).
* **Grupo FHIR**: FHIR Workflow.
* **Recursos FHIR Mapeados**: `Schedule`, `Slot`, `Appointment` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `schedule.rs`, `slot.rs`, `appointment.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `patient_id`, `practitioner_id`, `location_id` (UUIDv7).
* **Estado**: contexto acotado **solo diseñado** — aún sin `Cargo.toml` ni código; no es miembro del workspace.

---

## Reglas de Dominio

* **La cita se confirma al 100 %, nunca queda tentativa**: aunque la identidad del paciente esté pendiente de verificación presencial (`LinkAssuranceLevel::Level1`), la reserva se guarda **CONFIRMADA** y el cupo médico queda garantizado. El nivel de certeza de la identidad **no** condiciona la disponibilidad del cupo; se resuelve en el check-in.
* **Datos obligatorios al confirmar**: confirmar una cita exige al paciente nombres, apellido paterno, documento y teléfono (Ley N° 30024, pendiente de validación legal). La acción es `ProtectedAction::ConfirmAppointment` y la regla ya está en `app_core::domain::access`: la API de citas debe llamar a `authorize` y devolver los datos faltantes. Fuente oficial: [`access-control/action-authorization`](../../../openspec/specs/access-control/action-authorization/spec.md) e [`identity/progressive-profiling`](../../../openspec/specs/identity/progressive-profiling/spec.md).
* **`Slot` es la unidad de exclusión mutua**: la disponibilidad se reserva sobre el `Slot`, no sobre el `Appointment`. Dos citas no pueden ocupar el mismo `Slot`; la garantía se implementa en la base de datos, no en la capa de aplicación.

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
