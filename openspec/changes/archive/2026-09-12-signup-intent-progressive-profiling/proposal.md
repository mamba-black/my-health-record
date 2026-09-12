## Why

El flujo actual de `SignUp` dependía de un flag booleano deprecado (`create_clinic`) y no ofrecía un mecanismo limpio para que el usuario exprese su intención inicial al registrarse (Dueño de clínica, Profesional médico o Paciente). Con la aprobación del RFC-004, se requiere un perfilado progresivo y flexible donde el alta de usuario capture la intención inicial sin fricción burocrática (sin obligar a ingresar RUC, razón social o licencias médicas como CMP en el registro preliminar), permitiendo que roles como Practitioner o Patient se activen de forma incremental según la intención y demanda.

## What Changes

- **Contrato gRPC (`proto/api.proto`)**:
  - Incorporar el enum `SignUpIntent` (`INTENT_UNSPECIFIED = 0`, `INTENT_CLINIC_OWNER = 1`, `INTENT_PRACTITIONER = 2`, `INTENT_PATIENT = 3`).
  - Añadir el campo `SignUpIntent intent = 18;` a `SignUpRequest`.
- **Capa de Dominio y Aplicación (`crates/user`)**:
  - Actualizar `SignUpCommand` y el caso de uso `SignUp` para aceptar y procesar `intent`.
  - Definir la emisión condicional del evento de dominio `FounderRegistered(user_id, person_id)` únicamente cuando `intent == INTENT_CLINIC_OWNER`.
- **Compatibilidad**:
  - Mantener retrocompatibilidad temporal con clientes existentes mapeando `create_clinic = true` a `INTENT_CLINIC_OWNER` cuando `intent` no esté especificado.

## Capabilities

### New Capabilities
- `identity/signup-intent`: Especificación del perfilado progresivo basado en intenciones de usuario (`SignUpIntent`) y emisión condicional de eventos de fundación.

### Modified Capabilities
<!-- Ninguna previa registrada en openspec/specs -->

## Impact

- **APIs y Contratos**: Modificación retrocompatible de `backend/proto/api.proto`.
- **Crates afectados**: `backend/crates/user`, `backend/crates/core` (eventos de dominio), `backend/bin/clickcare` (adaptador gRPC).
- **Beads**: Corresponde a la tarea `my-health-record-32o.1` dentro de la épica `my-health-record-32o`.
