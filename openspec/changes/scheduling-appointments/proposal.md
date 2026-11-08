# Proposal

## Why

La plataforma ya controla quién puede hacer qué y pide los datos justo cuando hacen falta, pero no tiene ninguna acción real de atención. La más frecuente de una clínica es la cita, y sin ella:
- la ventana «Completa tus datos» solo se abre desde «Mis clínicas»;
- la acción `ConfirmAppointment` existe en la regla de acceso, pero nadie la usa;
- el médico activo no tiene nada que hacer después de activarse.

La agenda es el siguiente paso porque da uso real a todo lo construido y cierra escenarios que quedaron sin prueba de punta a punta (`my-health-record-c2p`).

## What Changes

- **Nuevo crate `crates/scheduling`** (hoy solo diseño), como miembro del workspace, con los recursos FHIR `Schedule`, `Slot` y `Appointment`.
- **Horario semanal del médico en cada clínica**: días, franjas y duración de la cita, más excepciones como feriados y vacaciones. Los cupos salen de ese horario.
- **Disponibilidad pública por clínica**: el paciente entra por el enlace o subdominio de la clínica y ve solo a sus médicos activos y sus cupos libres. Una clínica pendiente no muestra nada.
- **Reservar una cita**, de dos maneras:
  - El paciente reserva para sí mismo. Si le faltan datos, se abre la ventana «Completa tus datos» con `ConfirmAppointment`.
  - Recepción reserva en nombre del paciente. Recepción es el dueño, un administrador o un recepcionista. El médico también puede reservar, pero solo en su propia agenda.
- **La cita nace confirmada** (`booked`), siempre, aunque la identidad del paciente aún no esté verificada.
- **Dos citas nunca ocupan el mismo cupo**, y un médico nunca tiene dos citas que se crucen entre las clínicas de su misma red. Esto lo garantiza la base de datos.
- **Aislamiento por red clínica**: ninguna regla de agenda consulta ni compara datos de otra red, para que cada red pueda operar aislada, incluso en otra base de datos.
- **Cancelar y reprogramar**:
  - Las hace el paciente con sus propias citas, o la clínica con cualquiera de las suyas.
  - Reprogramar mueve la cita a otro cupo en un solo paso: si el nuevo cupo ya está tomado, la cita conserva el anterior.
- **Check-in en recepción**:
  - Marca la llegada del paciente.
  - Opcionalmente, registra que recepción verificó el DNI físico. Ese DNI pasa a «verificado» y el usuario ya no puede editarlo.
  - También se puede marcar «no asistió».
- **Recordatorios**:
  - Un worker programa un aviso para cada cita y lo envía antes de que empiece.
  - Ante la duda de si un aviso salió, se reenvía: un aviso repetido no hace daño, uno perdido sí.
  - El canal por defecto es el correo, detrás de un puerto intercambiable para sumar SMS o WhatsApp después. Falta el servidor de correo (`my-health-record-vxm`) y el canal de WhatsApp (`my-health-record-md1`): mientras no existan, los avisos solo se registran en el log.
  - Si la cita se cancela o se reprograma, el aviso pendiente se anula o se vuelve a programar.
- **Vincular pacientes sin cuenta**:
  - Recepción puede registrar pacientes sin cuenta.
  - Cuando esa persona crea su cuenta, recepción la vincula en persona, con el documento físico a la vista. Desde ese momento, la cuenta ve sus citas anteriores y su historia clínica.
  - El sistema sugiere el vínculo cuando coincide el documento, pero nunca vincula solo.
- **Nuevas acciones protegidas** en `app_core::domain::access`, con su regla y sus datos exigidos: gestionar la agenda propia, reservar en nombre de otro, cancelar, reprogramar y hacer check-in y vincular pacientes.
- **API**: el servicio gRPC `SchedulingApi` en `proto/api.proto` y la migración `V7` con el esquema `scheduling`.
- **Frontend**:
  - Para el paciente: «Reservar cita» desde el enlace de la clínica y «Mis citas».
  - Para el médico: «Mi agenda» (horario, excepciones y citas).
  - Para recepción: «Agenda de la clínica», con reserva en nombre del paciente y check-in.

## Capabilities

### New Capabilities
- `scheduling/practitioner-availability`: el horario semanal y las excepciones del médico en cada clínica, cómo se generan los cupos y qué disponibilidad ve el público en el enlace de una clínica activa.
- `scheduling/appointment-booking`: reservar, cancelar y reprogramar citas, por el propio paciente o por la clínica, con la cita siempre confirmada y la exclusión de cupos garantizada.
- `scheduling/check-in`: la llegada del paciente a la clínica, la verificación presencial del documento y la inasistencia.
- `scheduling/appointment-reminders`: el aviso programado antes de cada cita, el canal de envío y el efecto de cancelar o reprogramar.
- `identity/patient-account-linking`: el registro de pacientes sin cuenta y su vinculación posterior, confirmada en persona, con la cuenta que crean, para que vean sus citas y su historia anteriores.

### Modified Capabilities
- `access-control/action-authorization`: se amplían los requisitos «Acciones del paciente», «Acciones del médico» y «Acciones del dueño o administrador» con las nuevas acciones de agenda (reservar, cancelar, reprogramar, gestionar la agenda y hacer check-in). Además, la exploración de médicos y cupos se limita a clínicas activas.

## Impact

- **Backend**:
  - Nuevo crate `crates/scheduling`, organizado en dominio, aplicación e infraestructura.
  - En `crates/core`: las acciones de `access.rs`.
  - En `crates/administration`: crear el expediente del paciente en la clínica cuando se le reserva la primera cita, si aún no existe; registrar pacientes sin cuenta y vincularlos después con su cuenta.
  - En `crates/user`: marcar el documento como verificado desde el check-in.
  - En `bin/clickcare`: `SchedulingApi`, el arranque del worker de recordatorios y la DI.
- **Contrato**: `proto/api.proto` suma un servicio y mensajes nuevos, sin cambios que rompan los existentes.
- **Base de datos**:
  - Migración `V7__agenda.sql`, con el esquema `scheduling` y la extensión `btree_gist` para la exclusión de horarios que se cruzan dentro de la red.
  - Tabla de vínculos de pacientes en `administration`.
  - Tablas de recordatorios en la cola de apalis.
- **Dependencias**: un cliente SMTP (por ejemplo `lettre`) detrás del puerto de notificación. En desarrollo y pruebas, el aviso solo se registra en el log.
- **Variables de entorno**: SMTP_* para el correo, y la anticipación del recordatorio, `REMINDER_LEAD_HOURS`, que por defecto es 24.
- **Frontend**: páginas y rutas nuevas, la entidad `appointment`, las features `book-appointment` y `manage-schedule`, y el cliente `schedulingApi`.
- **Pruebas**:
  - Pruebas de integración con el contenedor `clickcare-test`.
  - Pruebas e2e de reserva con datos faltantes, reprogramación y check-in.
  - Cierra los escenarios de citas de `my-health-record-c2p`.
- **Documentación**: `backend/crates/scheduling/AGENTS.md` pasa de diseño a implementado, y se actualiza el mapa de crates del `AGENTS.md` raíz.
