## Purpose

Define cómo un paciente que la clínica registró sin cuenta queda vinculado a la cuenta que crea después, para que vea sus citas anteriores y su historia clínica sin que nadie pueda apropiarse del historial de otra persona solo con conocer su documento.

## ADDED Requirements

### Requirement: Paciente sin cuenta
Recepción SHALL poder registrar a un paciente en su clínica sin cuenta de usuario, con nombres, apellido paterno, documento y teléfono. Sus citas y su historia clínica SHALL guardarse a nombre de ese registro de paciente, y quedan listas para vincularse a una cuenta después.

#### Scenario: Paciente que llamó por teléfono
- **WHEN** recepción registra a un paciente sin cuenta y le reserva una cita
- **THEN** la cita queda a nombre de ese registro y el paciente no necesita cuenta para ser atendido

### Requirement: Sugerencia de vínculo, nunca automático
Cuando una cuenta y un registro sin cuenta de la misma red clínica tengan el mismo tipo y número de documento, el sistema SHALL mostrar a recepción una sugerencia de vínculo al atender a ese paciente: al reservar en su nombre, al registrar su llegada o en su ficha. El sistema SHALL NOT vincularlos automáticamente solo porque coincide el documento.

#### Scenario: Paciente crea su cuenta y vuelve a la clínica
- **WHEN** un paciente registrado sin cuenta crea su cuenta con el mismo DNI y recepción registra su siguiente llegada
- **THEN** recepción ve la sugerencia de vincular la cuenta con el registro anterior, y no se vincula nada hasta que la confirme

#### Scenario: Mismo documento en otra red
- **WHEN** existe un registro sin cuenta con el mismo DNI en una clínica de otra red
- **THEN** no se sugiere ningún vínculo, porque las redes no se comparan entre sí

### Requirement: Vinculación confirmada en persona
Solo el dueño, un administrador o un recepcionista con membresía activa SHALL poder vincular un registro de paciente con una cuenta, y solo tras declarar que comprobó el documento físico de la persona. La vinculación SHALL registrar quién la hizo, cuándo y en qué clínica, y SHALL dejar verificado el documento de la cuenta. El paciente SHALL NOT poder vincularse a sí mismo un registro existente.

#### Scenario: Recepción vincula con el DNI a la vista
- **WHEN** un administrador confirma la vinculación con el DNI físico del paciente a la vista
- **THEN** el registro queda vinculado a la cuenta, el documento de la cuenta queda verificado y la acción queda registrada con el administrador, la fecha y la clínica

#### Scenario: Paciente intenta reclamar un historial
- **WHEN** un usuario con sesión intenta vincular a su cuenta un registro de paciente indicando solo un número de DNI
- **THEN** el servidor responde «permiso denegado» y no revela si existe un registro con ese DNI

#### Scenario: Documentos distintos
- **WHEN** un administrador intenta vincular una cuenta con un registro cuyo documento no coincide
- **THEN** el servidor rechaza la vinculación indicando que el documento no coincide

### Requirement: Efecto de la vinculación
Tras vincular, la cuenta SHALL ver en «Mis citas» las citas anteriores y futuras del registro, en todas las clínicas de la red donde se vincule. La historia clínica de ese registro SHALL quedar asociada a la cuenta, y los recordatorios siguientes SHALL usar los canales de la cuenta. Si la cuenta ya tenía su propio registro en esa clínica, ambos SHALL unirse en uno, con todas las citas.

#### Scenario: Citas anteriores visibles
- **WHEN** se vincula una cuenta con un registro que tenía dos citas pasadas y una futura
- **THEN** el paciente ve las tres citas en «Mis citas», y el recordatorio de la futura llega al correo de su cuenta

#### Scenario: La cuenta ya tenía registro en la clínica
- **WHEN** se vincula un registro sin cuenta a una cuenta que ya tenía su propio registro en la misma clínica
- **THEN** la clínica queda con un solo registro de ese paciente con las citas de ambos, y la búsqueda de pacientes lo muestra una sola vez

### Requirement: Deshacer una vinculación equivocada
El dueño de la clínica SHALL poder deshacer una vinculación hecha en su clínica, indicando el motivo. Deshacerla SHALL devolver el registro a su estado sin cuenta, con sus citas y su historia, y SHALL quedar registrada junto con la vinculación original.

#### Scenario: Vinculación por error
- **WHEN** el dueño deshace una vinculación indicando que fue a la persona equivocada
- **THEN** el registro vuelve a estar sin cuenta, la cuenta deja de ver esas citas y quedan registradas la vinculación y su anulación
