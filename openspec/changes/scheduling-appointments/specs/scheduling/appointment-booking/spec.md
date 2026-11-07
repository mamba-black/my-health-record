## Purpose

Define cómo se reserva, cancela y reprograma una cita, la haga el paciente o la clínica en su nombre, y garantiza que cada cita nazca confirmada y que un cupo nunca tenga dos citas.

## ADDED Requirements

### Requirement: Paciente reserva su cita
Un usuario autenticado SHALL poder reservar para sí mismo un cupo libre de un médico en una clínica activa, siempre que tenga nombres, apellido paterno, documento y teléfono. Si le faltan, el servidor SHALL rechazar la reserva indicando los datos que faltan, y el frontend SHALL abrir la ventana «Completa tus datos» y reintentar la reserva al completarlos.

#### Scenario: Paciente con datos completos
- **WHEN** un paciente con nombres, apellido paterno, documento y teléfono reserva un cupo libre en una clínica activa
- **THEN** la cita queda reservada y confirmada a su nombre en ese cupo

#### Scenario: Paciente sin documento
- **WHEN** un paciente sin documento intenta reservar un cupo libre
- **THEN** el servidor responde «precondición no cumplida» con la lista de datos faltantes, y el frontend abre la ventana «Completa tus datos» con el documento

#### Scenario: Completa sus datos y continúa
- **WHEN** el paciente completa el documento en la ventana y pulsa «Completar»
- **THEN** sus datos se guardan y la reserva del mismo cupo se reintenta sin que tenga que volver a elegirlo

### Requirement: La cita nace confirmada
Toda cita reservada SHALL quedar confirmada desde el primer momento, sin estado tentativo ni aprobación posterior de la clínica, aunque la identidad del paciente todavía no esté verificada en persona.

#### Scenario: Identidad sin verificar
- **WHEN** un paciente cuyo documento aún no fue verificado en persona reserva un cupo
- **THEN** la cita queda confirmada y el cupo deja de ofrecerse

### Requirement: Un cupo, una cita
Un cupo SHALL tener como máximo una cita activa, y un médico SHALL NOT tener dos citas activas que se crucen en el tiempo dentro de su red clínica, aunque sean de clínicas distintas de esa red. La garantía SHALL cumplirse también con reservas simultáneas: solo una tiene éxito y la otra recibe que el cupo ya no está disponible. Entre redes distintas no se compara nada.

#### Scenario: Reservas simultáneas
- **WHEN** dos pacientes reservan el mismo cupo al mismo tiempo
- **THEN** exactamente una reserva tiene éxito y la otra recibe «el cupo ya no está disponible»

#### Scenario: Cupo ocupado
- **WHEN** un paciente intenta reservar un cupo que ya tiene una cita activa
- **THEN** el servidor responde que el cupo ya no está disponible y no crea la cita

### Requirement: La clínica reserva en nombre del paciente
El dueño o un administrador con membresía activa (recepción) SHALL poder reservar cualquier cupo libre de su clínica en nombre de un paciente. Un médico activo SHALL poder hacerlo solo en su propia agenda. El paciente SHALL elegirse entre los de la clínica o registrarse en ese momento con nombres, apellido paterno, documento y teléfono, aunque no tenga cuenta.

#### Scenario: Recepción reserva para un paciente sin cuenta
- **WHEN** un administrador registra a un paciente que llamó por teléfono con nombres, apellido, DNI y teléfono, y le reserva un cupo
- **THEN** el paciente queda registrado en la clínica y la cita queda confirmada a su nombre

#### Scenario: Recepción sin los datos del paciente
- **WHEN** un administrador intenta reservar para un paciente nuevo sin indicar su teléfono
- **THEN** el servidor rechaza la reserva indicando que falta el teléfono del paciente

#### Scenario: Médico reserva en la agenda de otro médico
- **WHEN** un médico activo intenta reservar un cupo de la agenda de otro médico
- **THEN** el servidor responde «permiso denegado»

### Requirement: Expediente del paciente en la clínica
La primera cita de un paciente en una clínica SHALL dejarlo registrado como paciente de esa clínica, de modo que el médico pueda verlo en la búsqueda de pacientes. Las citas siguientes SHALL reutilizar ese mismo registro.

#### Scenario: Primera cita en la clínica
- **WHEN** un paciente con cuenta reserva por primera vez en una clínica
- **THEN** el médico de esa clínica lo encuentra en la búsqueda de pacientes

#### Scenario: Segunda cita
- **WHEN** el mismo paciente reserva otra cita en esa clínica
- **THEN** la clínica sigue teniendo un solo registro de ese paciente

### Requirement: Ver mis citas
Un paciente SHALL poder ver sus citas en todas las clínicas de su red clínica: fecha y hora, clínica, médico y estado. La clínica SHALL ver las citas de su agenda: dueño y administrador, las de todos los médicos; cada médico, solo las suyas. Nadie SHALL ver citas de otra clínica, y ninguna consulta SHALL cruzar a otra red.

#### Scenario: Paciente revisa sus citas
- **WHEN** un paciente con citas en dos clínicas de la misma red abre «Mis citas»
- **THEN** ve las citas de ambas clínicas con su fecha, médico y estado

#### Scenario: Médico revisa su agenda
- **WHEN** un médico abre su agenda del día
- **THEN** ve solo sus propias citas en esa clínica

### Requirement: Cancelar una cita
El paciente SHALL poder cancelar sus propias citas antes de que empiecen, y recepción SHALL poder cancelar cualquier cita futura de su clínica. El médico SHALL poder cancelar solo las de su agenda. Al cancelar, el cupo SHALL volver a ofrecerse, y la cita SHALL conservarse como cancelada, con quién la canceló y cuándo.

#### Scenario: Paciente cancela
- **WHEN** un paciente cancela su cita de mañana
- **THEN** la cita queda cancelada, se registra que la canceló el paciente y el cupo vuelve a estar libre

#### Scenario: Cita ya empezada
- **WHEN** un paciente intenta cancelar una cita cuya hora de inicio ya pasó
- **THEN** el servidor rechaza la cancelación

#### Scenario: Cita ajena
- **WHEN** un paciente intenta cancelar la cita de otro paciente
- **THEN** el servidor responde que la cita no existe

### Requirement: Reprogramar una cita
Quien puede cancelar una cita SHALL poder reprogramarla a otro cupo libre del mismo médico en la misma clínica. El cambio SHALL hacerse en un solo paso: si el nuevo cupo ya no está libre, la cita SHALL conservar el cupo original sin cambios.

#### Scenario: Reprogramación exitosa
- **WHEN** un paciente mueve su cita del lunes 10:00 al martes 11:00, que está libre
- **THEN** la cita queda en el martes 11:00, confirmada, y el lunes 10:00 vuelve a estar libre

#### Scenario: Nuevo cupo ocupado
- **WHEN** un paciente intenta mover su cita a un cupo que otro acaba de reservar
- **THEN** el servidor responde que el cupo ya no está disponible y la cita sigue en su cupo original

#### Scenario: Cambio de médico
- **WHEN** un paciente intenta reprogramar su cita a un cupo de otro médico
- **THEN** el servidor rechaza la reprogramación y le indica que cancele y reserve de nuevo
