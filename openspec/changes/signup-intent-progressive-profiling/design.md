## Context

Ver `proposal.md`, RFC-004 (`PROPUESTA.md`), `backend/AGENTS.md` y `backend/crates/administration/AGENTS.md`.
En el backend de My Health Record:
1. El aislamiento entre clínicas se realiza a nivel de fila (`organization_id`) y las tablas transaccionales de alto volumen usan particionamiento declarativo Hash (`PARTITION BY HASH (organization_id)` con MODULUS 8) para podado eficiente de particiones.
2. Todas las llaves primarias e identificadores del sistema son obligatoriamente UUIDv7 (`Uuid::now_v7()`).
3. Un usuario puede participar y ser propietario de múltiples organizaciones clínicas (relación N:M). No debe haber restricciones `UNIQUE` artificiales que limiten a una sola clínica por dueño.
4. Para evitar estados nulos (`tenant_id = null`) en el ciclo de vida del usuario fundador, el `organization_id` se genera de antemano mediante `Uuid::now_v7()` durante el registro.

## Goals / Non-Goals

**Goals:**
- Extender `proto/api.proto` con el enum `SignUpIntent` (tags: `INTENT_UNSPECIFIED = 0`, `INTENT_CLINIC_OWNER = 1`, `INTENT_PRACTITIONER = 2`, `INTENT_PATIENT = 3`).
- Actualizar `SignUpCommand` en `crates/user` para transportar `intent: SignUpIntent`.
- Generar el `organization_id` (UUIDv7) de antemano cuando `intent == INTENT_CLINIC_OWNER` e incluirlo en el evento `FounderRegistered(user_id, person_id, organization_id)`.
- No requerir datos fiscales ni colegiaturas en el alta inicial (fricción cero).
- Permitir multi-clínica eliminando restricciones `UNIQUE` sobre `owner_user_id`.

**Non-Goals:**
- Validar colegiaturas médicas o RUCs en `crates/user` (eso compete a los contextos especializados de administración/clínica en fases posteriores).
- DDL en tiempo de ejecución (las particiones son estáticas en PostgreSQL).

## Decisions

1. **Generación anticipada de `organization_id` (UUIDv7)**:
   - *Decisión*: El caso de uso `SignUp` (o Gateway) genera `organization_id: Uuid = Uuid::now_v7()` antes de emitir `FounderRegistered`.
   - *Razón*: Elimina tokens JWT con `tenant_id = null`, permitiendo que el cliente ya nazca con el contexto de su clínica de forma determinista y sin consultas adicionales de polling.
2. **Definición del evento `FounderRegistered` en `crates/core`**:
   - *Decisión*: Ubicar la struct `FounderRegistered { user_id: Uuid, person_id: Uuid, organization_id: Uuid }` en `crates/core/src/domain/events.rs`.
   - *Razón*: Permite respetar la Arquitectura Cebolla, sirviendo de contrato transversal entre `crates/user` y el consumidor de clínicas.
3. **Multi-Clínica sin restricción 1:1**:
   - *Decisión*: Modelar la pertenencia mediante membresías / roles y permitir que un mismo usuario funde múltiples clínicas si lo desea.

## Risks / Trade-offs

- [Eventos duplicados por at-least-once delivery en Apalis] → Mitigado: el consumidor de `FounderRegistered` es idempotente basándose en la presencia de `organization_id`.
- [Desfase de clientes antiguos] → Mitigado con fallback de `create_clinic == true` mapeado a `INTENT_CLINIC_OWNER`.
