## Purpose

Define qué hace recepción cuando el paciente llega a la clínica, o no llega, y cómo la verificación presencial del documento eleva la certeza de su identidad.

## ADDED Requirements

### Requirement: Registrar la llegada
El dueño, un administrador o un recepcionista con membresía activa SHALL poder marcar como «llegó» una cita confirmada de su clínica el mismo día de la cita. Se SHALL registrar quién marcó la llegada y a qué hora. El médico de la cita SHALL ver que el paciente llegó.

#### Scenario: Paciente llega
- **WHEN** un administrador marca la llegada del paciente de la cita de las 10:00 de hoy
- **THEN** la cita pasa a «llegó», con la hora y el administrador que la marcó, y el médico lo ve en su agenda

#### Scenario: Llegada en otro día
- **WHEN** un administrador intenta marcar la llegada de una cita de la próxima semana
- **THEN** el servidor rechaza la operación indicando que solo se registra el día de la cita

#### Scenario: Médico intenta registrar la llegada
- **WHEN** un médico sin membresía de dueño ni de administrador intenta marcar una llegada
- **THEN** el servidor responde «permiso denegado»

### Requirement: Verificación presencial del documento
Al registrar la llegada, recepción SHALL poder indicar que verificó el documento físico del paciente contra el que tiene registrado. Si el paciente tiene cuenta, su documento SHALL quedar verificado: el usuario ya no podrá editarlo y la interfaz lo mostrará como verificado. La verificación SHALL registrar quién la hizo, cuándo y en qué clínica.

#### Scenario: Recepción verifica el DNI
- **WHEN** un administrador marca la llegada indicando que verificó el DNI físico del paciente
- **THEN** el documento del paciente queda verificado y, al abrir su perfil, el paciente lo ve como verificado y no editable

#### Scenario: El documento no coincide
- **WHEN** el administrador indica que el documento físico no coincide con el registrado
- **THEN** la llegada se registra, el documento queda sin verificar y se le pide al administrador corregir los datos del paciente

#### Scenario: Paciente sin cuenta
- **WHEN** un administrador verifica el documento de un paciente que fue registrado por la clínica y no tiene cuenta
- **THEN** la verificación queda registrada en la cita y en el paciente de esa clínica

### Requirement: Inasistencia
Una vez pasada la hora de inicio de una cita confirmada sin llegada registrada, recepción SHALL poder marcarla como «no asistió». Se SHALL registrar quién la marcó y cuándo. Una cita marcada así SHALL NOT poder cancelarse ni reprogramarse, y no libera ningún cupo.

#### Scenario: Paciente no llega
- **WHEN** a las 10:40 un administrador marca como «no asistió» la cita de las 10:00
- **THEN** la cita queda como «no asistió», con quién la marcó, y no puede reprogramarse

#### Scenario: Antes de la hora
- **WHEN** un administrador intenta marcar como «no asistió» una cita que todavía no empieza
- **THEN** el servidor rechaza la operación
