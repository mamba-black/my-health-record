## Purpose

Permite a los nuevos usuarios declarar su intención inicial (dueño de clínica, profesional médico o paciente) durante el registro, facilitando el perfilado progresivo sin fricciones ni requerimientos burocráticos tempranos. La generación del identificador de clínica (`organization_id`) se realiza de antemano mediante UUIDv7 para evitar estados nulos en la identidad del inquilino y permitir que un usuario posea o participe en múltiples organizaciones clínicas.

## ADDED Requirements

### Requirement: Captura de intención de registro (SignUpIntent) y generación anticipada de organization_id
El sistema SHALL permitir que un usuario especifique su intención inicial mediante el campo `intent` en `SignUpRequest`. Cuando la intención es `INTENT_CLINIC_OWNER`, el sistema SHALL generar deterministamente un identificador `organization_id` de tipo UUIDv7 de antemano.

#### Scenario: Registro como Dueño de Clínica con organization_id anticipado
- **WHEN** un usuario envía `SignUpRequest` con `intent = INTENT_CLINIC_OWNER`
- **THEN** el sistema registra la cuenta, genera un `organization_id` (UUIDv7), emite el evento `FounderRegistered(user_id, person_id, organization_id)` y retorna el contexto de sesión conteniendo el identificador de la organización creada

#### Scenario: Registro como Profesional Médico
- **WHEN** un usuario envía `SignUpRequest` con `intent = INTENT_PRACTITIONER`
- **THEN** el sistema registra el usuario sin exigir colegiatura CMP ni emitir `FounderRegistered`

#### Scenario: Registro como Paciente
- **WHEN** un usuario envía `SignUpRequest` con `intent = INTENT_PATIENT`
- **THEN** el sistema registra el usuario sin vincularlo a ninguna organización clínica obligatoria

#### Scenario: Multi-Clínica sin restricción de unicidad por propietario
- **WHEN** un usuario que ya posee una clínica registrada decide fundar una nueva organización
- **THEN** el sistema genera un nuevo `organization_id` (UUIDv7) independiente y le otorga la membresía de propietario (`OWNER`) sin rechazar por duplicidad de dueño

#### Scenario: Retrocompatibilidad con create_clinic
- **WHEN** un cliente legacy envía `SignUpRequest` con `intent = INTENT_UNSPECIFIED` y `create_clinic = true`
- **THEN** el sistema interpreta la intención como `INTENT_CLINIC_OWNER`, genera el `organization_id` y emite `FounderRegistered`
