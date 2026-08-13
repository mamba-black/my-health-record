# Expediente de Pacientes Bounded Context (`crates/patient`)

## Especificación del Dominio

* **Responsabilidad**: Gestión de expedientes clínicos y datos demográficos del paciente (`Patient`).
* **Grupo FHIR**: FHIR Individuals.
* **Recurso FHIR Mapeado**: `Patient` (HL7 FHIR R4).
* **Módulos de Dominio (`src/domain/`)**: `patient.rs`.
* **Proyección / Apuntador Débil**: Apunta débilmente a `user_id` (UUIDv7).
