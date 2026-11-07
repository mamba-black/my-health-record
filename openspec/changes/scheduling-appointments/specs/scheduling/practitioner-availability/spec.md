## Purpose

Define cómo publica un médico su disponibilidad en cada clínica: un horario semanal con duración de cita, sus excepciones, los cupos que salen de ellos y lo que ve el público en el enlace de una clínica activa.

## ADDED Requirements

### Requirement: Horario semanal por clínica
Un médico con membresía activa y colegiatura confirmada en una clínica activa SHALL poder definir su horario semanal en esa clínica: franjas por día de la semana y duración de la cita. Cada clínica tiene su propio horario, independiente del de las demás. Las horas SHALL interpretarse en la zona horaria de la clínica (por defecto America/Lima).

#### Scenario: Médico define su horario
- **WHEN** un médico activo con colegiatura confirmada guarda lunes y miércoles de 09:00 a 13:00 con citas de 30 minutos
- **THEN** su agenda en esa clínica ofrece 8 cupos cada lunes y cada miércoles, desde las 09:00 hasta las 12:30

#### Scenario: Médico sin colegiatura confirmada
- **WHEN** un médico activo cuya colegiatura aún no confirma la clínica intenta guardar su horario
- **THEN** el servidor responde «precondición no cumplida» indicando que falta la confirmación de la colegiatura

#### Scenario: Franjas que se cruzan
- **WHEN** un médico guarda dos franjas del mismo día que se superponen
- **THEN** el servidor rechaza el horario como inválido y no cambia el anterior

### Requirement: Horario sin cruces dentro de la red
Un médico SHALL NOT tener cupos que se crucen entre dos clínicas de la misma red clínica. Al guardar un horario que se cruza con otra clínica de su red, el servidor SHALL rechazarlo indicando el día y la franja, sin revelar nada más de la otra clínica. Las reglas de agenda SHALL NOT consultar ni comparar datos de otras redes: cada red debe poder operar aislada, incluso en otra base de datos.

#### Scenario: Cruce con otra clínica de la red
- **WHEN** un médico que atiende los lunes de 09:00 a 13:00 en la clínica A guarda lunes de 12:00 a 15:00 en la clínica B de la misma red
- **THEN** el servidor rechaza el horario en la clínica B indicando el conflicto del lunes entre 12:00 y 13:00

#### Scenario: Clínicas de redes distintas
- **WHEN** el mismo médico tiene cuenta en otra red y guarda allí lunes de 12:00 a 15:00
- **THEN** el horario se guarda, porque las redes no se comparan entre sí

### Requirement: Excepciones del horario
El médico SHALL poder registrar excepciones en su agenda de una clínica (feriados, vacaciones u otras ausencias) como rangos de fecha y hora en los que no se ofrecen cupos. Una excepción SHALL NOT cancelar por sí sola las citas ya reservadas: el servidor SHALL rechazarla si se cruza con citas activas, indicando cuántas son, para que primero se reprogramen o cancelen.

#### Scenario: Vacaciones sin citas
- **WHEN** un médico registra vacaciones del 1 al 15 de febrero y no tiene citas en esos días
- **THEN** su agenda no ofrece cupos en ese rango

#### Scenario: Excepción con citas reservadas
- **WHEN** un médico registra un feriado en un día en el que tiene 3 citas reservadas
- **THEN** el servidor rechaza la excepción indicando que hay 3 citas en ese rango

### Requirement: Cambios de horario respetan las citas
Cambiar el horario semanal SHALL afectar solo a los cupos libres. Las citas ya reservadas SHALL conservarse aunque queden fuera del nuevo horario.

#### Scenario: Se acorta el horario
- **WHEN** un médico con una cita reservada el lunes a las 12:30 cambia su horario de los lunes para que termine a las 12:00
- **THEN** la cita de las 12:30 sigue reservada y el lunes ya no ofrece cupos libres después de las 12:00

### Requirement: Disponibilidad pública de una clínica activa
Cualquier visitante, con sesión o sin ella, SHALL poder ver en el enlace o subdominio de una clínica activa sus médicos con membresía activa y horario definido, y los cupos libres de cada uno en un rango de fechas futuro. Por cada médico SHALL mostrarse solo nombre, especialidad y cupos. Una clínica pendiente o suspendida SHALL NOT mostrar médicos ni cupos.

#### Scenario: Paciente abre el enlace de la clínica
- **WHEN** un visitante abre el enlace de una clínica activa con dos médicos activos con horario
- **THEN** ve a los dos médicos con su nombre, especialidad y cupos libres de los próximos días, y ningún otro dato

#### Scenario: Clínica pendiente
- **WHEN** un visitante abre el enlace de una clínica pendiente de activación
- **THEN** no ve médicos ni cupos, y se le indica que la clínica aún no atiende

#### Scenario: Médico invitado sin activar
- **WHEN** una clínica activa tiene un médico invitado que aún no activó su perfil
- **THEN** ese médico no aparece en la disponibilidad pública

### Requirement: Cupos solo en el futuro
La disponibilidad SHALL ofrecer solo cupos que empiezan después del momento actual y dentro de una ventana máxima de reserva (por defecto 60 días). Un cupo ocupado, pasado o cubierto por una excepción SHALL NOT ofrecerse.

#### Scenario: Cupo de hoy ya pasado
- **WHEN** un paciente consulta la disponibilidad de hoy a las 11:10
- **THEN** no ve los cupos que empiezan a las 11:00 o antes

#### Scenario: Fuera de la ventana de reserva
- **WHEN** un paciente consulta la disponibilidad a 90 días
- **THEN** no ve cupos más allá de los 60 días
