# Tasks

## 1. Esquema y crate

- [x] 1.1 Antes de crear el índice único, revisar que no haya pacientes duplicados por `(organization_id, user_id)` en Neon ni en el contenedor de pruebas, y corregirlos. Verificar con la consulta de duplicados, que debe devolver 0 filas.
- [x] 1.2 Escribir `backend/migrations/V8__agenda.sql`:
  - en `administration`: `organization.timezone` con `'America/Lima'` por defecto, la tabla `patient_link` (`patient.user_id` ya es nulo desde `V6`; la unicidad por cuenta ya la da `uq_patient_org_user`, comprobado en 1.1: 0 duplicados en Neon);
  - el esquema `scheduling` con `btree_gist` y `network_id` en cada tabla: `schedule`, `schedule_block`, `schedule_exception`, `appointment` con su `EXCLUDE (network_id, practitioner_user_id, period)` y `reminder`.

  Verificar: `cargo test -p clickcare --test user_api_test` migra el contenedor `clickcare-test`, y una prueba nueva lee `refinery_schema_history` en la versión 8.
- [x] 1.3 Crear `backend/crates/scheduling`:
  - `Cargo.toml` como miembro del workspace, con `domain`, `application` e `infrastructure`;
  - `lib.rs` con la verificación de arquitectura que usan los demás crates.

  Verificar: `cargo check --workspace` pasa y el test de arquitectura de `scheduling` pasa.

## 2. Regla de acceso

- [x] 2.1 Agregar a `app_core::domain::access` las acciones `ManageSchedule`, `ViewClinicAgenda`, `BookForPatient`, `ManageAppointment`, `CheckIn`, `LinkPatientAccount` y `UnlinkPatientAccount`, con sus membresías, datos y estado de clínica según el diseño (decisión 5). Agregar una fila por cada caso a la matriz `rstest`. Verificar: `cargo test -p app_core` pasa con los casos nuevos.
- [x] 2.2 Sumar las acciones nuevas a `ProtectedAction` en `proto/api.proto` y a `action_to_proto`, y las etiquetas en `entities/profile` del frontend. Verificar: `cargo build -p clickcare` y `bun run check` sin errores.

## 3. Dominio de agenda

- [x] 3.1 Modelar `Schedule` (franjas semanales, duración, validación de franjas que se cruzan) y `ScheduleException`. Verificar con las pruebas unitarias «Médico define su horario» (8 cupos de 30 min de 09:00 a 13:00) y «Franjas que se cruzan».
- [x] 3.2 Implementar el cálculo de cupos libres: horario − excepciones − citas activas − pasado − ventana de 60 días, en la zona de la clínica, con `jiff` y un puerto `Clock`. Verificar con pruebas unitarias para «Cupo de hoy ya pasado», «Fuera de la ventana de reserva», «Vacaciones sin citas», «Se acorta el horario» y un día con cambio de horario de verano en una zona que lo tenga.
- [x] 3.3 Modelar `Appointment`:
  - estados `BOOKED`, `ARRIVED`, `CANCELLED` y `NOSHOW`, con sus transiciones permitidas y su auditoría (quién y cuándo);
  - las reglas «no cancelar ni reprogramar después del inicio» y «no se reprograma a otro médico»;
  - la regla de «no asistió».

  Verificar con pruebas unitarias para «Cita ya empezada», «Cambio de médico», «Antes de la hora» y «Paciente no llega».

## 4. Persistencia y puertos

- [x] 4.1 Implementar `ScheduleRepository` y `AppointmentRepository` sobre el esquema `scheduling`:
  - si la restricción `EXCLUDE` falla (`23P01`), devolver el error de dominio «cupo no disponible»;
  - reprogramar como un `UPDATE` en una transacción.

  Verificar con pruebas de integración contra `clickcare-test`:
  - «Reservas simultáneas» con `tokio::join!`: exactamente una de las dos reservas tiene éxito;
  - «Nuevo cupo ocupado»: la cita conserva su cupo original;
  - el cruce entre clínicas de la misma red a nivel de citas, y que dos redes distintas no chocan aunque compartan base.
- [x] 4.2 Definir los puertos `ClinicDirectory`, `PatientRegistry`, `IdentityVerification`, `NotificationSender`, `ReminderScheduler` y `Clock`, con dobles en memoria en `scheduling/src/testing.rs`. Verificar: compilan y los usan las pruebas unitarias de los grupos 5 a 7.
- [x] 4.3 Implementar los adaptadores en `bin/clickcare`:
  - `ClinicDirectory` sobre `administration` (clínica por subdominio, membresías activas) y `user` (nombre, especialidad);
  - `PatientRegistry` sobre `PatientRepository`, que busca o crea el paciente de la clínica y resuelve los registros de una cuenta en la red, incluidos los vinculados;
  - `IdentityVerification` sobre un `set_document_verified` nuevo en `user`.

  Verificar con pruebas de integración para «Primera cita en la clínica» y «Segunda cita» (un solo registro de paciente).

## 5. Casos de uso de agenda y citas

- [x] 5.1 `SetWeeklySchedule`, `GetSchedule`, `AddScheduleException` y `RemoveScheduleException`:
  - autorizan con `ManageSchedule`;
  - comprueban que el horario no se cruce con otras clínicas de la misma red, con un advisory lock por red y médico, sin consultar otras redes;
  - rechazan excepciones que tienen citas.

  Verificar con pruebas unitarias para «Médico sin colegiatura confirmada», «Cruce con otra clínica de la red» (solo indica día y franja), «Clínicas de redes distintas», «Excepción con citas reservadas» y «Médico define horario sin colegiatura confirmada».
- [x] 5.2 Disponibilidad pública: `GetClinic`, `ListPractitioners` y `GetAvailability`, solo para clínicas activas y médicos activos con horario. Verificar con pruebas para «Paciente abre el enlace de la clínica» (sin datos de más), «Clínica pendiente» y «Médico invitado sin activar».
- [x] 5.3 `BookAppointment`, para que el paciente reserve para sí mismo:
  - autoriza con `ConfirmAppointment`;
  - valida que el cupo exista en el horario;
  - registra al paciente en la clínica;
  - inserta la cita en `BOOKED` y su recordatorio en la misma transacción.

  Verificar con pruebas para «Paciente con datos completos», «Paciente sin documento» (`FAILED_PRECONDITION` con `ActionRequirements`), «Identidad sin verificar», «Cupo ocupado» y «Paciente reserva en una clínica pendiente».
- [x] 5.4 `BookForPatient`:
  - autoriza con `BookForPatient`;
  - el médico solo reserva en su propia agenda;
  - si el paciente es nuevo, exige sus datos y responde `INVALID_ARGUMENT` con el campo que falta.

  Verificar con pruebas para «Recepción reserva para un paciente sin cuenta», «Recepción sin los datos del paciente», «Médico reserva en la agenda de otro médico» y «Administrador reserva por un paciente».
- [x] 5.5 `ListMyAppointments` y `ListClinicAppointments`, en las que el médico ve solo lo suyo y nadie ve otra clínica. Verificar con pruebas para «Paciente revisa sus citas» (dos clínicas de la misma red) y «Médico revisa su agenda».
- [x] 5.6 `CancelAppointment` y `RescheduleAppointment`:
  - el paciente actúa sobre lo suyo, y lo ajeno responde `NOT_FOUND`;
  - la clínica usa `ManageAppointment`, y el médico solo actúa sobre su agenda;
  - en ambos casos se actualiza el recordatorio.

  Verificar con pruebas para «Paciente cancela», «Cita ajena», «Paciente reprograma la cita de otro», «Médico cancela una cita de otro médico» y «Reprogramación exitosa».

## 6. Check-in y vínculo de pacientes

- [x] 6.1 `CheckIn`:
  - autoriza con `CheckIn` y solo vale el día local de la cita;
  - registra quién y cuándo;
  - con `MATCH`, llama a `IdentityVerification` e invalida el contexto en `AccessContextBuilder`;
  - con `MISMATCH`, solo registra la llegada.

  Verificar con pruebas de integración para «Paciente llega», «Llegada en otro día», «Médico intenta registrar la llegada», «Recepción verifica el DNI» (luego `CompleteProfile` responde de solo lectura para el documento), «El documento no coincide», «Paciente sin cuenta» y «Administrador de otra clínica».
- [x] 6.2 `MarkNoShow`, solo después del inicio. Verificar con la prueba de que una cita «no asistió» no se puede cancelar ni reprogramar.
- [x] 6.3 `SuggestPatientLinks`: busca registros sin cuenta con el mismo documento, solo en la red y solo para recepción. Verificar con pruebas para «Paciente crea su cuenta y vuelve a la clínica», «Mismo documento en otra red» y «Paciente intenta reclamar un historial» (`PERMISSION_DENIED`, sin revelar si existe).
- [x] 6.4 `LinkPatientAccount`:
  - exige `document_seen` y que el documento coincida;
  - crea el `patient_link` auditado;
  - verifica el documento de la cuenta e invalida su contexto de acceso.

  Verificar con pruebas para «Paciente que llamó por teléfono», «Recepción vincula con el DNI a la vista», «Documentos distintos», «Citas anteriores visibles» (el recordatorio pendiente pasa al correo de la cuenta) y «La cuenta ya tenía registro en la clínica» (un solo paciente en la búsqueda).
- [x] 6.5 `UnlinkPatientAccount`, solo para el dueño y con motivo. Cierra el vínculo sin mover citas. Verificar con la prueba «Vinculación por error»: la cuenta deja de ver las citas y quedan registradas la vinculación y su anulación.

## 7. Recordatorios

- [x] 7.1 Implementar el worker `scheduling-reminders`:
  - el reclamo con `UPDATE … WHERE status='PENDING' AND due_at=$2`;
  - el barrido que devuelve a `PENDING` las filas en `SENDING` vencidas (`REMINDER_SENDING_TIMEOUT`), para que se reenvíen antes que perderse;
  - el envío por `NotificationSender`;
  - los estados `SENT`, `NO_CHANNEL` y `FAILED`, y los reintentos con backoff hasta `REMINDER_MAX_ATTEMPTS`;
  - el barrido al arrancar, que también vuelve a encolar los `PENDING` vencidos.

  Verificar con pruebas, usando un `NotificationSender` espía y un `Clock` fijo, para «Cita reservada con días de anticipación», «Cita para dentro de 3 horas», «Paciente sin correo», «Cita cancelada», «Cita reprogramada», «Falla temporal del correo», «Aviso entregado dos veces», «Caída a mitad del envío» (una fila en `SENDING` vencida se reenvía) y «Reintentos agotados».
- [x] 7.2 Implementar el `ReminderScheduler` con apalis (`run_at_time`) y arrancar el worker junto al de `administration` en `bin/clickcare/src/infrastructure/mod.rs`. Verificar con una prueba de integración: reservar una cita a 1 minuto con `REMINDER_LEAD_HOURS=0` y comprobar que el espía recibe el aviso.
- [x] 7.3 Implementar `SmtpNotificationSender` (lettre) y `LogNotificationSender`, elegidos según `SMTP_HOST`:
  - el texto del correo no lleva datos clínicos, y el log no lleva datos personales;
  - documentar las variables en `backend/AGENTS.md`.

  Verificar con una prueba unitaria del texto del correo, que lleva fecha, hora local, médico, clínica y dirección, y no lleva motivo.

## 8. API gRPC

- [x] 8.1 Definir en `proto/api.proto` los servicios `PublicSchedulingApi` (sin sesión) y `SchedulingApi` (con el interceptor), con sus mensajes. Registrar ambos en el servidor y en la DI. Verificar: `cargo build -p clickcare` y una prueba de que `SchedulingApi` sin credencial responde `UNAUTHENTICATED`, mientras que `PublicSchedulingApi` responde sin credencial.
- [x] 8.2 Mapear los errores:
  - «cupo no disponible» a `ABORTED`;
  - lo ajeno a `NOT_FOUND`;
  - los datos faltantes propios a `FAILED_PRECONDITION` con `ActionRequirements`;
  - los datos de un paciente nuevo a `INVALID_ARGUMENT`.

  Verificar con las pruebas de integración de los grupos 5 y 6 a través de gRPC.

## 9. Frontend

- [x] 9.1 Generar los clientes `schedulingApi` y `publicSchedulingApi` en `shared/api`, con la entidad `appointment` (modelo, estados y etiquetas). Verificar: `bun run check` sin errores.
- [x] 9.2 Página pública `/c/[subdomain]`:
  - la clínica, sus médicos y los cupos de los próximos días;
  - al reservar, si no hay sesión, pide iniciarla y vuelve al mismo cupo;
  - reserva con `runWithRequirements(CONFIRM_APPOINTMENT)`.

  Verificar con un e2e: un paciente sin documento reserva, se abre la ventana «Completa tus datos», completa el documento y la cita queda confirmada sin volver a elegir el cupo. Con esto queda cubierto el escenario de `my-health-record-c2p`.
- [x] 9.3 «Mis citas» (`/appointments`), con cancelar y reprogramar. Verificar con e2e para cancelar (el cupo vuelve a verse libre en la página pública) y reprogramar.
- [x] 9.4 «Mi agenda» (`/schedule`), con el editor del horario semanal y de las excepciones, y sus citas. Verificar con un e2e: un médico activo con colegiatura confirmada guarda su horario y sus cupos aparecen en `/c/[subdomain]`.
- [x] 9.5 «Agenda de la clínica» (`/clinic-agenda`), con:
  - la agenda del día;
  - la reserva en nombre de un paciente (buscar o registrar sin cuenta);
  - el check-in con verificación del documento;
  - las sugerencias de vínculo con la confirmación «vi el documento físico».

  Verificar con dos e2e:
  - recepción registra la llegada con `MATCH`, y el perfil del paciente muestra el documento como verificado y no editable;
  - recepción vincula a un paciente registrado sin cuenta, y en «Mis citas» de esa cuenta aparecen sus citas anteriores.
- [x] 9.6 Mostrar en el `Navbar` las entradas «Mis citas», «Mi agenda» y «Agenda de la clínica» según el contexto de perfil, y documentar las páginas en `frontendts/AGENTS.md`. Verificar con el e2e `role-access.spec.ts` ampliado para que cada rol vea solo sus entradas.

## 10. Documentación y cierre

- [x] 10.1 Actualizar la documentación:
  - `backend/crates/scheduling/AGENTS.md`: implementado, la exclusión en `appointment` en lugar del `Slot`, y los puertos;
  - el mapa de crates del `AGENTS.md` raíz;
  - `backend/AGENTS.md`: las variables `SMTP_*`, `REMINDER_LEAD_HOURS`, `REMINDER_MAX_ATTEMPTS` y `REMINDER_SENDING_TIMEOUT`, y la regla de no cruzar redes clínicas en ninguna consulta;

  Verificar que los enlaces a `openspec/specs/scheduling/*` funcionan después de archivar.
- [x] 10.2 Integración completa:
  - `cargo test --workspace`;
  - `cargo fmt --all -- --check`;
  - `bun run check`;
  - la suite e2e completa, y con `--repeat-each=2`.

  Escribir `verification.md` con la tabla escenario → prueba de los seis specs, y actualizar `my-health-record-c2p` con los escenarios de citas que quedan cubiertos.
- [x] 10.3 Abrir en beads la validación legal del contenido del recordatorio y de la vinculación de pacientes (Ley 29733), y anotar en `my-health-record-vxm` (correo) y `my-health-record-md1` (WhatsApp) lo que este cambio deja listo. Verificar con `bd show` de cada issue.

## Workflow follow-up

- Aplicar `V8` en Neon al desplegar (se aplica sola al arrancar) y comprobar el historial de refinery.
- Archivar el change cuando esté revisado, y verificar que `openspec/specs/scheduling/` quedó con sus cuatro capacidades.
