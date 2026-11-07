# Design

## Context

La motivación está en `proposal.md`, y los requisitos en `specs/`. Lo que condiciona el diseño:

- **Autorización**:
  - Vive en `app_core::domain::access` como la función pura `authorize(action, ctx, target)`. Su orden de evaluación es membresía y rol, luego estado de la clínica, luego datos.
  - `bin/clickcare` arma el `RequestContext` con `AccessContextBuilder`, que usa el usuario y sus membresías y los guarda 5 s en caché.
  - Los datos faltantes viajan como `FAILED_PRECONDITION` con `ActionRequirements` en `grpc-status-details-bin`. El frontend ya los convierte en la ventana «Completa tus datos» con `runWithRequirements`.
- **`ConfirmAppointment` ya existe** en el catálogo: exige nombres, apellido paterno, documento y teléfono, y una clínica activa.
- **El paciente de una clínica**:
  - Es `administration.patient (organization_id, user_id, person jsonb)`.
  - El worker de alta solo crea pacientes para el dueño; el resto «aparece por demanda cuando una clínica lo registra», y eso es esta reserva.
  - `user_id` hoy siempre tiene valor.
- **El documento verificado** existe en `identity.user_account.document_verified`. Si está en `true`, `CompleteProfile` ya lo trata como de solo lectura.
- **La cola es apalis 1.0 (rc) sobre Postgres.**
  - `TaskBuilder::run_at_time` permite tareas diferidas.
  - El worker de `administration` arranca en `bin/clickcare/src/infrastructure/mod.rs` junto con el servidor.
- **Multi-tenant**:
  - La unidad de aislamiento es la **red clínica** (`administration.clinical_network`), que agrupa clínicas (`administration.organization`). Las cuentas ya son por red: `identity.user_account.network_id`. Un mismo médico o paciente en dos redes tiene dos cuentas.
  - El subdominio se resuelve en `SubdomainResolver` con la cabecera `x-subdomain`.
  - En desarrollo no hay subdominios reales: todo corre en `localhost`.
- **No hay zona horaria por clínica**: se asume Lima.
- **Las migraciones son de refinery**: la próxima es `V6`.

## Goals / Non-Goals

**Goals:**
- Que la exclusión de citas se cumpla en Postgres, no en la aplicación, y también con peticiones simultáneas.
- Que ninguna consulta ni restricción cruce redes clínicas: cada red debe poder moverse a su propia base de datos sin cambiar las reglas.
- Que `crates/scheduling` no dependa de `administration` ni de `user`: los habla por puertos que conecta `bin/clickcare`.
- Que la regla de quién puede hacer qué siga en `authorize`. Los casos de uso solo agregan la comprobación de propiedad: si la cita es mía o la agenda es mía.
- Que la ventana «Completa tus datos» funcione sin cambios en el frontend, con la misma `runWithRequirements`.

**Non-Goals:**
- Pagos, tarifas o seguros de la cita.
- Lista de espera y sobrecupos.
- Videoconsulta.
- Agenda de recursos que no son personas, como salas o equipos.
- Rol de recepcionista: lo cumplen el dueño y el administrador.
- SMS y WhatsApp: solo se deja el puerto (`my-health-record-md1`). El servidor de correo real también queda fuera (`my-health-record-vxm`).
- Plantillas de correo con la marca de cada clínica: el texto es fijo.
- Políticas de cancelación con plazo mínimo: se puede cancelar hasta la hora de inicio.

## Decisions

### 1. Cupos calculados, no materializados

Los cupos (`Slot`) no se guardan. Se calculan cuando se piden, así:

`horario semanal − excepciones − citas activas − pasado − más allá de la ventana de reserva`

Un cupo se identifica por médico, clínica e inicio, más una duración. En la API se expone con forma FHIR `Slot`, con un id derivado de esos datos.

- **Por qué**: un horario editable y materializado obliga a regenerar filas en cada cambio y a reconciliarlas con las citas. Calculados, cambiar el horario solo afecta a los cupos libres, como pide «Cambios de horario respetan las citas», sin tocar nada.
- **Descartado**: materializar los cupos para 60 días con un worker diario. Da exclusión con un `UNIQUE (slot_id)`, pero duplica el estado y complica los cambios de horario y excepciones.

### 2. La exclusión vive en la tabla de citas

```sql
CREATE EXTENSION IF NOT EXISTS btree_gist;
-- scheduling.appointment
period tstzrange NOT NULL,
EXCLUDE USING gist (network_id WITH =, practitioner_user_id WITH =, period WITH &&)
  WHERE (status IN ('BOOKED', 'ARRIVED'))
```

Esta restricción cubre tres requisitos: «un cupo, una cita», las reservas simultáneas y el cruce entre clínicas de la misma red, porque usa el usuario médico y no su membresía.

`network_id` va explícito en la restricción y en cada tabla, aunque hoy `practitioner_user_id` ya es único por red. Así, ninguna consulta depende de que todas las redes compartan la base, y separar una red no cambia la regla.

Al reservar, la aplicación vuelve a comprobar que el cupo existe en el horario (que no sea un cupo inventado) y luego hace el `INSERT`. Si viola la restricción (`23P01`), responde `ABORTED` con «el cupo ya no está disponible».

- **Reprogramar** es un `UPDATE` del `period` de la misma fila, dentro de una transacción. Si viola la restricción, la fila no cambia, así que la cita conserva su cupo original.
- **Descartado**: la regla del `AGENTS.md` de scheduling que dice «`Slot` como unidad de exclusión». Se reemplaza por esta restricción, que es más fuerte porque también evita citas que se crucen con duraciones distintas. Se actualiza el `AGENTS.md`.
- **`btree_gist` en Neon**: está disponible. Se crea en la migración.

### 3. Modelo de datos (`V6__agenda.sql`, esquema `scheduling`)

- **`schedule`**: `id`, `network_id`, `organization_id`, `practitioner_user_id`, `slot_minutes`. Un registro por médico y clínica, con `UNIQUE (organization_id, practitioner_user_id)`.
- **`schedule_block`**: `schedule_id`, `weekday` (1–7), `start_time`, `end_time`, que son horas locales.
- **`schedule_exception`**: `schedule_id`, `period tstzrange`, `reason`.
- **`appointment`**:
  - Identidad y relaciones: `id`, `network_id`, `organization_id`, `practitioner_user_id` y `patient_id`. `patient_id` es `administration.patient.id`, una referencia débil sin FK entre esquemas.
  - La cita no guarda la cuenta del paciente. Cuáles registros son de una cuenta lo resuelve `PatientRegistry`, que sigue los vínculos (decisión 10). Por eso vincular o desvincular no reescribe ninguna cita.
  - Horario y estado: `period`, `status` (`BOOKED`, `ARRIVED`, `CANCELLED`, `NOSHOW`; en FHIR `booked`, `arrived`, `cancelled`, `noshow`) y `version`.
  - Auditoría: `booked_by`, `cancelled_by`, `cancelled_at`, `arrived_at`, `arrived_by`, `document_check` (`MATCH`, `MISMATCH` o nulo), `noshow_by`, `noshow_at`, `created_at`.
- **`reminder`**: `appointment_id` (único), `due_at`, `status` (`PENDING`, `SENDING`, `SENT`, `FAILED`, `NO_CHANNEL`, `CANCELLED`), `attempts`, `sent_at` y `last_error`.
- **En `administration`**:
  - `organization.timezone text NOT NULL DEFAULT 'America/Lima'`.
  - `patient.user_id` pasa a admitir nulos, para los pacientes registrados por recepción.
  - Un índice único parcial `(organization_id, user_id) WHERE user_id IS NOT NULL` impide que un paciente con cuenta quede registrado dos veces en la misma clínica.
  - **`patient_link`**: `id`, `network_id`, `organization_id`, `patient_id` (el registro sin cuenta), `user_id`, `linked_by`, `linked_at`, `unlinked_by`, `unlinked_at` y `unlink_reason`. Hay un vínculo activo como máximo por registro.

### 4. Puertos de `crates/scheduling`

`scheduling` define estos puertos en `application/integration`, y `bin/clickcare` los implementa con adaptadores sobre los casos de uso de los otros crates:

| Puerto | Para qué | Adaptador |
|---|---|---|
| `ClinicDirectory` | Clínica por subdominio (estado y zona horaria), médicos activos con nombre y especialidad | `administration` (organización y membresías) + `user` (nombre y especialidad) |
| `PatientRegistry` | Buscar o crear el paciente de la clínica, por usuario o con datos de recepción; resolver los registros de una cuenta en la red, incluidos los vinculados; sugerir y crear vínculos | `administration` (`PatientRepository`) |
| `IdentityVerification` | Marcar el documento de un usuario como verificado | `user` (nuevo `set_document_verified`) |
| `NotificationSender` | Enviar el recordatorio | `SmtpNotificationSender` (lettre) o `LogNotificationSender` |
| `ReminderScheduler` | Programar la tarea diferida | apalis `PostgresStorage` con `run_at_time` |

- **Por qué**: así se respeta la arquitectura cebolla y la regla de no acoplar contextos acotados. Además, en las pruebas unitarias se reemplazan por dobles en memoria, como se hizo en `administration/src/testing.rs`.

### 5. Acciones nuevas en `app_core::domain::access`

| Acción | Membresía | Datos | Clínica |
|---|---|---|---|
| `ConfirmAppointment` (ya existe) | — (paciente para sí) | Nombres, apellido paterno, documento, teléfono | `ACTIVE` |
| `ManageSchedule` | Médico `ACTIVE` | Colegiatura confirmada | `ACTIVE` |
| `ViewClinicAgenda` | Médico `ACTIVE`, dueño o administrador `ACTIVE` | — | `ACTIVE` |
| `BookForPatient` | Médico `ACTIVE`, dueño o administrador `ACTIVE` | Del paciente: los de `ConfirmAppointment` | `ACTIVE` |
| `ManageAppointment` (cancelar o reprogramar como clínica) | Médico `ACTIVE`, dueño o administrador `ACTIVE` | — | `ACTIVE` |
| `CheckIn` (llegada, verificación, inasistencia) | Dueño o administrador `ACTIVE` | — | `ACTIVE` |
| `LinkPatientAccount` (sugerir y vincular) | Dueño o administrador `ACTIVE` | — | `ACTIVE` |
| `UnlinkPatientAccount` | Dueño `ACTIVE` | — | `ACTIVE` |

- **Propiedad**: `authorize` no conoce las citas. Las comprobaciones de propiedad las hace el caso de uso después de autorizar:
  - el médico solo actúa sobre su propia agenda; si no, «permiso denegado»;
  - el paciente solo actúa sobre sus propias citas, y lo ajeno responde `NOT_FOUND`, como «Expediente ajeno».
- **Paciente que cancela o reprograma lo suyo**: no es una acción nueva. Basta con estar autenticado y que el `patient_id` de la cita sea uno de sus registros en la red, según `PatientRegistry`.
- **Datos que faltan del paciente en `BookForPatient`**: se responden con `INVALID_ARGUMENT` indicando el campo. No se usa `FAILED_PRECONDITION`, porque la ventana es para los datos propios y aquí los escribe recepción en su formulario.

### 6. Dos servicios gRPC: público y protegido

- **`PublicSchedulingApi`**, sin interceptor de sesión:
  - `GetClinic(subdomain)`: nombre, estado y zona horaria.
  - `ListPractitioners(subdomain)`.
  - `GetAvailability(subdomain, practitioner_user_id, from, to)`.
- **`SchedulingApi`**, con el interceptor de `Authenticator`:
  - Agenda del médico: `SetWeeklySchedule`, `GetSchedule`, `AddScheduleException` y `RemoveScheduleException`.
  - Reservas: `BookAppointment` (para sí) y `BookForPatient`.
  - Consultas: `ListMyAppointments` y `ListClinicAppointments` (día o rango; un médico solo ve lo suyo).
  - Cambios: `CancelAppointment` y `RescheduleAppointment`.
  - Recepción: `CheckIn(appointment_id, document_check)` y `MarkNoShow`.
  - Vínculos: `SuggestPatientLinks(patient_id)`, `LinkPatientAccount(patient_id, user_id, document_seen)` y `UnlinkPatientAccount(link_id, reason)`.

La clínica va como campo `subdomain` en los mensajes públicos, no solo en la cabecera. Así el enlace funciona en desarrollo con la ruta `/c/<subdominio>`, sin DNS.

- **Descartado**: un solo servicio con una lista de métodos públicos exentos en el interceptor. Es más fácil equivocarse y dejar expuesto un método protegido.

### 7. Recordatorios: tabla propia + tarea diferida de apalis

1. **Al reservar**, en la misma transacción de la cita, se inserta la fila `reminder` en `PENDING` con `due_at = max(inicio − REMINDER_LEAD_HOURS, ahora)`. Después del commit se programa la tarea `ReminderDue { reminder_id, due_at }` con `run_at_time(due_at)`.
2. **El worker reclama el envío** con:
   ```sql
   UPDATE reminder SET status='SENDING', attempts=attempts+1
   WHERE id=$1 AND status='PENDING' AND due_at=$2
   ```
   Si no se actualiza ninguna fila, el aviso está obsoleto o duplicado y el worker termina sin hacer nada. Esto cubre «Aviso entregado dos veces» y reprogramar, porque el `due_at` viejo ya no coincide.
3. **El resultado del envío**:
   - si se envía, la fila pasa a `SENT`;
   - si el paciente no tiene correo, pasa a `NO_CHANNEL`;
   - si falla, vuelve a `PENDING` y se reprograma con backoff, hasta `REMINDER_MAX_ATTEMPTS` (3); al agotarlos, queda en `FAILED`.
4. **Cancelar** pone el recordatorio en `CANCELLED`. **Reprogramar** actualiza `due_at` y programa una tarea nueva, y la anterior se descarta sola en el paso 2.
5. **Si la tarea no se pudo encolar** después del commit, la cita no se pierde: un barrido al arrancar el worker vuelve a encolar los `PENDING` vencidos.

- **Por qué la tabla propia además de la cola**: la cola no permite anular ni consultar una tarea por cita. La tabla es la fuente de verdad y la cola es solo el despertador.
- **Antes repetido que perdido**: si el proceso muere después de que el SMTP aceptó el correo y antes de marcarlo `SENT`, la fila queda en `SENDING`. El barrido devuelve a `PENDING` las filas en `SENDING` con más de `REMINDER_SENDING_TIMEOUT` (5 min) y las vuelve a encolar, así que el paciente puede recibir el aviso dos veces. Se acepta, porque un aviso perdido puede hacer que falte a la cita. El reclamo del paso 2 sigue evitando los duplicados en el caso normal: entregas repetidas de la cola o dos workers a la vez.
- **El correo**:
  - Lo envía `lettre` por SMTP con `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` y `SMTP_FROM`.
  - Sin `SMTP_HOST`, se usa `LogNotificationSender`, que solo escribe en el log a nivel `info`. Esto aplica en desarrollo y en las pruebas, y en ninguno de los dos casos se escriben datos personales.

### 8. Zona horaria y tiempo

- **Las horas del horario son locales**: las franjas se guardan como `time` y se expanden con `jiff` en la zona de la clínica, así el horario de verano y los cambios de zona se resuelven bien.
- **Todo instante se guarda en UTC**: `timestamptz` y `tstzrange` con límites `[)`.
- **El reloj se inyecta**: es un puerto `Clock` con un doble fijo en las pruebas, para probar «Cupo de hoy ya pasado» y las ventanas de check-in sin depender de la hora real.

### 9. Check-in y verificación del documento

- **`CheckIn`** pasa la cita a `ARRIVED` solo si el día local de la clínica coincide con el de la cita.
- **Con `document_check = MATCH`**:
  - si el registro del paciente tiene cuenta (propia o vinculada), se llama a `IdentityVerification` (que pone `document_verified = true`) y se invalida su contexto en `AccessContextBuilder`;
  - si no tiene cuenta, se registra en la cita y en el registro del paciente, y se muestran las sugerencias de vínculo (decisión 10).
- **Con `MISMATCH`**: se registra la llegada, el documento no se verifica y el frontend abre los datos del paciente para que recepción los corrija.

### 10. Vincular pacientes sin cuenta

- **Vincular no fusiona filas.** Se agrega un vínculo en `administration.patient_link`, y el registro sin cuenta conserva su `id`, sus citas y, más adelante, su historia clínica. `PatientRegistry` resuelve los registros de una cuenta en la red: los que tienen su `user_id` más los vinculados activos.
- **Unión con un registro propio**: si la cuenta ya tenía registro en la clínica, la búsqueda de pacientes y la agenda muestran un solo paciente, el registro de la cuenta, con las citas de ambos. Ninguna fila se borra, así que **deshacer** solo cierra el vínculo (`unlinked_at`), y cada cita vuelve a su registro original sin migrar datos.
- **Sugerencias**: se buscan registros sin cuenta en la misma red con el mismo `document_type` y `document_value` que la cuenta, y nunca en otra red. Solo las ve recepción. El paciente no tiene ninguna API para reclamar registros, y la respuesta no revela si existe uno con ese documento.
- **Al vincular**: se exige `document_seen = true` y que el documento de la cuenta coincida con el del registro. Además se marca verificado el documento de la cuenta, con `IdentityVerification`, y se invalida su contexto de acceso.
- **Historia clínica**: hoy no existe. Cuando `crates/clinical` cuelgue la historia de `patient_id`, la regla de este vínculo bastará para que la cuenta la vea, sin migraciones.
- **Descartado**: fusionar registros copiando citas y borrando el duplicado. Es irreversible, y deshacer un error exigiría reconstruir qué venía de cada registro.

### 11. Frontend (FSD)

- **`entities/appointment`**: modelo, estado y etiquetas.
- **`features/book-appointment`**:
  - Elegir médico y cupo, y reservar.
  - Usa `runWithRequirements(ProtectedAction.CONFIRM_APPOINTMENT, …)`, con lo que la ventana «Completa tus datos» queda conectada sin código nuevo.
- **`features/manage-schedule`**: el editor del horario semanal y de las excepciones.
- **`features/reception`**: la reserva en nombre de un paciente (buscar o registrar), y el check-in con la verificación.
- **Páginas y rutas**:
  - `/c/[subdomain]` (pública): la clínica, sus médicos y los cupos.
  - `/appointments`: «Mis citas».
  - `/schedule`: «Mi agenda».
  - `/clinic-agenda`: la agenda del día de la clínica.
- **El `Navbar`** muestra cada entrada según el contexto de perfil, como ya se hace con «Búsqueda».

### 12. Pruebas

- **`app_core`**: casos nuevos en la matriz `rstest` de `authorize`, uno por cada acción nueva × membresía × estado de la clínica × datos.
- **`scheduling`**:
  - Unitarias con dobles para el cálculo de cupos: franjas, excepciones, pasado, ventana y horario de verano.
  - También para la propiedad de las citas y las transiciones de estado.
- **De integración, en `user_api_test.rs`** o en un `scheduling_api_test.rs` que use el mismo contenedor `clickcare-test`:
  - reservas simultáneas con `tokio::join!`;
  - el cruce entre clínicas de la misma red, y que entre redes distintas no se compare nada;
  - vincular, ver las citas anteriores, unir con un registro propio y deshacer;
  - reprogramar contra un cupo ocupado;
  - una entrega repetida de la cola no envía dos veces, y una fila en `SENDING` vencida sí se reenvía, comprobado con un `NotificationSender` espía.
- **e2e**: reservar con datos faltantes, que abre la ventana y luego reintenta; cancelar; reprogramar; y check-in con verificación, tras el cual el documento aparece como no editable.

## Risks / Trade-offs

- **[`btree_gist` no disponible en algún Postgres administrado]** → En Neon y en `postgres:18` sí lo está. La migración falla de forma explícita si no lo está; en ese caso no se cambia a una exclusión en la aplicación sin revisarlo.
- **[Cupos calculados en cada consulta]** → El costo es una consulta por médico y rango, a lo sumo de 60 días. Si crece, se agrega caché por médico y día, que se invalida al reservar o al cambiar el horario.
- **[Verificar el documento desde otra clínica afecta la cuenta global del usuario]** → Es intencional, porque el documento es de la persona. Queda registrado quién lo verificó y en qué clínica, para auditar.
- **[Un vínculo equivocado expone citas e historia a otra persona]** → Solo lo hace recepción, con el documento físico a la vista, nunca el sistema por coincidencia de DNI. Queda auditado, y el dueño puede deshacerlo sin pérdida de datos.
- **[Recordatorios duplicados tras una caída]** → Se aceptan a propósito (ver la decisión 7). El texto del aviso es idempotente: no dice «nuevo» ni cambia entre envíos.
- **[Correos con datos de salud]** → El recordatorio no lleva motivo ni datos clínicos (ver el spec). Fecha, médico y clínica sí revelan que hay una cita: queda pendiente de validación legal (Ley 29733).
- **[Que apalis sea rc]** → Se usa solo `push` con `run_at`, que ya se usa en el proyecto. El barrido al arrancar cubre las tareas perdidas.

## Migration Plan

1. `V6__agenda.sql`:
   - en `administration`: la zona horaria de la clínica, `patient.user_id` nulo y el índice único parcial;
   - el esquema `scheduling` con `btree_gist`.
   Es un cambio aditivo, y en Neon se aplica solo al arrancar.
2. Desplegar el backend con el worker de recordatorios. Sin `SMTP_HOST`, los avisos solo se registran en el log, así que no se envía correo por accidente.
3. Configurar SMTP cuando haya un dominio de envío.
4. **Rollback**: el código anterior ignora el esquema `scheduling`. No se borra nada; la columna nueva tiene valor por defecto, y el índice no afecta a los pacientes existentes porque todos tienen `user_id`.

## Open Questions

- **Remitente y texto definitivo del correo**: qué remitente usar (dominio propio o de un proveedor) y el texto final. No cambia el diseño.
- **Anticipación máxima y duración por defecto de las citas**: si cada clínica configura la anticipación máxima (hoy 60 días fijos) y la duración por defecto. Se puede agregar a la configuración de la clínica después.
