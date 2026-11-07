# access-control/action-authorization Specification

## Purpose

Decide qué acciones puede ejecutar cada usuario según su membresía, los datos que registró y el estado de la clínica, y garantiza que esa decisión se tome en el servidor para cada petición, porque los datos de salud son datos sensibles.

> **Validación legal pendiente**: Ley N° 29733 (datos sensibles de salud), Ley N° 30024 (RNHCE) y NTS N° 139-MINSA (historia clínica). Revisar con asesoría legal o médica.

## Requirements

### Requirement: Identificación de quien llama
Toda petición a una operación protegida SHALL incluir una credencial que el servidor valida en cada petición. Sin credencial válida, el servidor SHALL rechazar la petición como no autenticada, sin revelar si el recurso existe. Solo el registro, el inicio de sesión y las páginas públicas quedan sin proteger.

#### Scenario: Petición sin credencial
- **WHEN** se llama a la búsqueda de pacientes sin credencial
- **THEN** el servidor responde «no autenticado» y no devuelve ningún dato

#### Scenario: Credencial vencida
- **WHEN** se llama a una operación protegida con una credencial vencida
- **THEN** el servidor responde «no autenticado» y el frontend lleva al usuario a iniciar sesión

### Requirement: Autorización por membresía, datos y estado de la clínica
El servidor SHALL autorizar cada acción protegida solo si se cumplen a la vez tres condiciones: el rol y el estado de la membresía, los datos que exige la acción (ver `identity/progressive-profiling`) y el estado de la clínica. La matriz completa está en `design.md`.

#### Scenario: Las tres condiciones
- **WHEN** un médico con membresía activa y colegiatura confirmada registra una atención en una clínica activa
- **THEN** el servidor autoriza la acción

### Requirement: Acciones de cualquier usuario autenticado
Cualquier usuario autenticado SHALL poder ver y editar su propio perfil, explorar clínicas activas y sus médicos, y crear una clínica si tiene sus nombres registrados.

#### Scenario: Usuario recién registrado
- **WHEN** un usuario que solo tiene su cuenta de Google abre su perfil
- **THEN** puede verlo y editarlo

#### Scenario: Clínicas pendientes no se exploran
- **WHEN** un usuario explora clínicas
- **THEN** solo ve las clínicas activas

### Requirement: Acciones del paciente
Un paciente SHALL poder confirmar sus propias citas en clínicas activas cuando tenga los datos exigidos, y SHALL NOT poder buscar pacientes ni ver expedientes ajenos.

#### Scenario: Paciente busca pacientes
- **WHEN** un paciente sin membresía de médico llama a la búsqueda de pacientes
- **THEN** el servidor responde «permiso denegado»

#### Scenario: Paciente con datos completos confirma cita
- **WHEN** un paciente con nombres, apellido paterno, documento y teléfono confirma una cita en una clínica activa
- **THEN** el servidor autoriza la confirmación

### Requirement: Acciones del médico
Un médico SHALL poder unirse a una clínica que lo invitó y activar su perfil cuando tenga los datos exigidos. Solo con membresía activa en una clínica activa SHALL poder buscar y ver pacientes de esa clínica, y registrar atenciones y recetas si su colegiatura está confirmada.

#### Scenario: Médico de clínica pendiente
- **WHEN** un médico con membresía activa busca pacientes en una clínica pendiente de activación
- **THEN** el servidor responde que la clínica no está activa

#### Scenario: Médico con colegiatura sin confirmar
- **WHEN** un médico activo cuya colegiatura aún no confirma la clínica intenta registrar una receta
- **THEN** el servidor responde «precondición no cumplida» indicando que falta la confirmación de la colegiatura

### Requirement: Acciones del dueño o administrador
Un dueño o administrador con membresía activa SHALL poder configurar su clínica e invitar personal mientras la clínica esté pendiente o activa. Solo el dueño SHALL poder activar la clínica, y solo cuando tenga los datos exigidos.

#### Scenario: Dueño configura antes de activar
- **WHEN** el dueño de una clínica pendiente invita a un médico
- **THEN** la invitación se crea, aunque la clínica aún no pueda atender

#### Scenario: Administrador intenta activar
- **WHEN** un administrador que no es dueño intenta activar la clínica
- **THEN** el servidor responde «permiso denegado»

### Requirement: Rechazo por datos faltantes
Cuando un usuario tiene el rol y la membresía correctos pero le faltan datos que la acción exige, el servidor SHALL rechazar la acción con un error de «precondición no cumplida» que incluya la acción y la lista de datos faltantes, distinto del error «permiso denegado».

#### Scenario: Error distinguible
- **WHEN** un médico invitado sin colegiatura intenta activar su perfil
- **THEN** el servidor responde «precondición no cumplida» con la acción «activar perfil médico» y los datos faltantes «colegio profesional» y «número de colegiatura»

#### Scenario: Sin rol no se listan datos
- **WHEN** un paciente sin membresía intenta activar un perfil médico
- **THEN** el servidor responde «permiso denegado» y no lista datos faltantes

### Requirement: Aislamiento por clínica
Toda consulta de pacientes, atenciones o recetas hecha por un miembro de una clínica SHALL limitarse a los registros de esa clínica. Un registro de otra clínica SHALL tratarse como inexistente, aunque pertenezca a la misma persona.

#### Scenario: Paciente de otra clínica
- **WHEN** un médico de la clínica A pide por su identificador a un paciente que solo está registrado en la clínica B
- **THEN** el servidor responde «no encontrado»

#### Scenario: Misma persona en dos clínicas
- **WHEN** un médico de la clínica A busca a una persona atendida en las clínicas A y B
- **THEN** el resultado incluye solo el expediente de la clínica A

### Requirement: Acceso del paciente a lo propio
Un paciente SHALL acceder solo a sus propios expedientes y a los de los dependientes que administra. El historial generado antes de crear su cuenta SHALL quedar oculto hasta que su documento esté verificado.

#### Scenario: Expediente ajeno
- **WHEN** un paciente pide un expediente que no es suyo ni de un dependiente suyo
- **THEN** el servidor responde «no encontrado»

#### Scenario: Historial previo sin verificar
- **WHEN** un paciente con vinculación pendiente de verificación consulta su historia clínica
- **THEN** ve las atenciones registradas desde que creó su cuenta y un aviso de que el historial anterior se habilita al verificar su documento en la clínica

### Requirement: Interfaz coherente con los permisos
El frontend SHALL mostrar en menús y rutas solo las acciones que el usuario puede iniciar con sus roles, y SHALL redirigir a una página de acceso denegado si se entra por URL a una ruta no permitida. Esta restricción es solo de presentación: el servidor SHALL aplicar la autorización de todos modos.

#### Scenario: Paciente entra por URL a la búsqueda
- **WHEN** un paciente sin membresía de médico abre `/search` escribiendo la URL
- **THEN** ve la página de acceso denegado y no se muestra ningún dato de pacientes

#### Scenario: Médico activo
- **WHEN** un médico con membresía activa en una clínica activa inicia sesión
- **THEN** el menú muestra la búsqueda de pacientes de su clínica
