# Expediente de Pacientes Bounded Context (`crates/patient`)

## Estado del Crate

⚠️ **Andamiaje sin implementar.** El crate es miembro del workspace y compila, pero
`src/lib.rs` contiene únicamente la plantilla `add()` generada por `cargo new --lib`.
No existe `src/domain/`, ni `src/application/`, ni `src/infrastructure/`.

**Solapamiento pendiente de resolver**: el recurso `Patient` **sí está implementado**, pero en
[crates/administration](../administration/AGENTS.md) (`src/domain/patient.rs`). Antes de
escribir código aquí hay que decidir si este contexto acotado absorbe el expediente del
paciente o si se elimina en favor de `administration`. Mientras eso no se decida, la
especificación de abajo es intención de diseño, no descripción del código.

---

## Especificación del Dominio (planificada)

* **Responsabilidad**: Gestión de expedientes clínicos y datos demográficos del paciente (`Patient`).
* **Grupo FHIR**: FHIR Individuals.
* **Recurso FHIR Mapeado**: `Patient` (HL7 FHIR R4).
* **Módulos de Dominio previstos (`src/domain/`)**: `patient.rs`.
* **Proyección / Apuntador Débil**: Apuntaría débilmente a `user_id` (UUIDv7).
