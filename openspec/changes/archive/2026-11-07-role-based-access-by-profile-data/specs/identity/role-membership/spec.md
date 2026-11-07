## Purpose

Registra qué papel cumple cada usuario: un rol preferido de la cuenta, que solo orienta la interfaz, y membresías por clínica, que son lo único que concede permisos dentro de una clínica.

## ADDED Requirements

### Requirement: Rol preferido de la cuenta
El sistema SHALL guardar el rol preferido de cada cuenta (paciente, médico o dueño de clínica) a partir de la intención declarada al registrarse, y SHALL permitir que el usuario lo cambie. El rol preferido SHALL usarse solo para decidir qué pantallas y datos se muestran; SHALL NOT conceder acceso a datos de ninguna clínica.

#### Scenario: Rol preferido no autoriza
- **WHEN** un usuario con rol preferido «médico» y sin ninguna membresía busca pacientes
- **THEN** el sistema rechaza la búsqueda por falta de permisos

#### Scenario: Cuenta sin intención declarada
- **WHEN** una cuenta se registró sin intención (`INTENT_UNSPECIFIED` y sin `create_clinic`)
- **THEN** su rol preferido es «paciente»

### Requirement: Membresía por clínica
El sistema SHALL representar la relación de un usuario con una clínica como una membresía con clínica, usuario, rol (dueño, administrador o médico) y estado (invitada, pendiente de activación, activa o suspendida). Un usuario SHALL poder tener varias membresías, en distintas clínicas o con distintos roles en la misma clínica.

#### Scenario: Dueño que también atiende
- **WHEN** el dueño de una clínica se registra también como médico de esa clínica
- **THEN** el usuario tiene dos membresías en esa clínica, una de dueño y otra de médico, cada una con su propio estado

#### Scenario: Médico en dos clínicas
- **WHEN** un médico acepta la invitación de una segunda clínica
- **THEN** obtiene una membresía nueva para esa clínica, sin alterar la de la primera

### Requirement: Membresía de dueño al crear una clínica
Al crearse una clínica, el sistema SHALL crear la membresía de dueño de quien la creó en estado activo, aunque la clínica siga pendiente de activación.

#### Scenario: Creación de clínica
- **WHEN** un usuario crea una clínica
- **THEN** existe una membresía de dueño para ese usuario y esa clínica en estado activo

### Requirement: Activación de la membresía de médico
Una membresía de médico SHALL nacer en estado invitada o pendiente de activación, y SHALL pasar a activa solo cuando el médico tenga registrados nombres, apellido paterno, teléfono, documento y colegiatura, y un dueño o administrador de esa clínica haya confirmado la colegiatura.

#### Scenario: Activación sin colegiatura
- **WHEN** un médico sin colegiatura registrada intenta activar su perfil en una clínica
- **THEN** el sistema rechaza la activación indicando que faltan el colegio profesional y el número de colegiatura

#### Scenario: Activación completa
- **WHEN** un médico con todos los datos exigidos activa su perfil y el administrador confirma la colegiatura
- **THEN** su membresía pasa a activa

### Requirement: Revocación de membresías
Un dueño o administrador SHALL poder suspender la membresía de otro miembro de su clínica; desde ese momento, el miembro suspendido SHALL perder los permisos que le daba esa membresía. El último dueño activo de una clínica SHALL NOT poder ser suspendido.

#### Scenario: Médico suspendido
- **WHEN** un administrador suspende la membresía de un médico
- **THEN** la siguiente búsqueda de pacientes de ese médico en esa clínica se rechaza por falta de permisos

#### Scenario: Último dueño
- **WHEN** se intenta suspender al único dueño activo de una clínica
- **THEN** el sistema rechaza la operación
