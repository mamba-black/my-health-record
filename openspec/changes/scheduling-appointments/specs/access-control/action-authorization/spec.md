## MODIFIED Requirements

### Requirement: Acciones del paciente
Un paciente SHALL poder confirmar sus propias citas en clínicas activas cuando tenga los datos exigidos, y SHALL NOT poder buscar pacientes ni ver expedientes ajenos. Confirmar una cita incluye reservarla para sí mismo. Sobre sus propias citas, el paciente SHALL poder verlas, cancelarlas y reprogramarlas, y SHALL NOT poder ver, cancelar ni reprogramar citas de otras personas.

#### Scenario: Paciente busca pacientes
- **WHEN** un paciente sin membresía de médico llama a la búsqueda de pacientes
- **THEN** el servidor responde «permiso denegado»

#### Scenario: Paciente con datos completos confirma cita
- **WHEN** un paciente con nombres, apellido paterno, documento y teléfono confirma una cita en una clínica activa
- **THEN** el servidor autoriza la confirmación

#### Scenario: Paciente reserva en una clínica pendiente
- **WHEN** un paciente con datos completos intenta reservar en una clínica pendiente de activación
- **THEN** el servidor responde que la clínica no está activa

#### Scenario: Paciente reprograma la cita de otro
- **WHEN** un paciente intenta reprogramar una cita que no es suya
- **THEN** el servidor responde que la cita no existe

### Requirement: Acciones del médico
Un médico SHALL poder unirse a una clínica que lo invitó y activar su perfil cuando tenga los datos exigidos. Solo con membresía activa en una clínica activa SHALL poder:
- buscar y ver pacientes de esa clínica;
- ver su propia agenda, y reservar, cancelar y reprogramar citas en ella;
- registrar atenciones y recetas, y definir su horario y excepciones, si su colegiatura está confirmada.

SHALL NOT poder actuar sobre la agenda de otro médico ni registrar llegadas o inasistencias.

#### Scenario: Médico de clínica pendiente
- **WHEN** un médico con membresía activa busca pacientes en una clínica pendiente de activación
- **THEN** el servidor responde que la clínica no está activa

#### Scenario: Médico con colegiatura sin confirmar
- **WHEN** un médico activo cuya colegiatura aún no confirma la clínica intenta registrar una receta
- **THEN** el servidor responde «precondición no cumplida» indicando que falta la confirmación de la colegiatura

#### Scenario: Médico define horario sin colegiatura confirmada
- **WHEN** un médico activo cuya colegiatura aún no confirma la clínica intenta guardar su horario
- **THEN** el servidor responde «precondición no cumplida» indicando que falta la confirmación de la colegiatura

#### Scenario: Médico cancela una cita de otro médico
- **WHEN** un médico activo intenta cancelar una cita de la agenda de otro médico de la misma clínica
- **THEN** el servidor responde «permiso denegado»

### Requirement: Acciones del dueño o administrador
Un dueño o administrador con membresía activa SHALL poder configurar su clínica e invitar personal mientras la clínica esté pendiente o activa. Solo el dueño SHALL poder activar la clínica, y solo cuando tenga los datos exigidos.

Con la clínica activa, dueño y administrador actúan como recepción y SHALL poder:
- ver la agenda de todos los médicos de la clínica;
- reservar en nombre de un paciente;
- cancelar y reprogramar cualquier cita futura de la clínica;
- registrar llegadas, verificaciones de documento e inasistencias;
- vincular a un paciente registrado sin cuenta con su cuenta.

SHALL NOT poder definir el horario de un médico en su lugar.

#### Scenario: Dueño configura antes de activar
- **WHEN** el dueño de una clínica pendiente invita a un médico
- **THEN** la invitación se crea, aunque la clínica aún no pueda atender

#### Scenario: Administrador intenta activar
- **WHEN** un administrador que no es dueño intenta activar la clínica
- **THEN** el servidor responde «permiso denegado»

#### Scenario: Administrador reserva por un paciente
- **WHEN** un administrador de una clínica activa reserva un cupo libre en nombre de un paciente con datos completos
- **THEN** el servidor autoriza la reserva

#### Scenario: Administrador de otra clínica
- **WHEN** un administrador de la clínica A intenta registrar la llegada de una cita de la clínica B
- **THEN** el servidor responde que la cita no existe

#### Scenario: Recepción vincula un paciente a su cuenta
- **WHEN** un administrador confirma, con el documento físico a la vista, que un paciente registrado sin cuenta es el dueño de una cuenta con el mismo documento
- **THEN** el servidor autoriza la vinculación
