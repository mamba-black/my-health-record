## Context

Motivación en `proposal.md`. Estado actual que condiciona el diseño:

- **No hay sesión en el servidor.** El frontend guarda el `id_token` de Google tras `SignIn`, pero las llamadas gRPC no llevan credencial y el backend no sabe quién llama. `CompleteProfile` recibe el `id_token` en el cuerpo del mensaje.
- **El rol no se persiste.** `SignUpRequest.intent` dispara la creación de la clínica y se pierde. Solo existe `identity.user_account.is_owner`.
- **No hay membresías.** `administration.organization.owner_user_id` y `administration.practitioner (organization_id, user_id)` son la única relación usuario–clínica. La clínica tiene un booleano `active`, no los estados `PENDING_SETUP` / `ACTIVE` de RFC-004 (`PROPUESTA.md`).
- **`PatientApi` es simulada.** `SearchPatient` y `GetPatientById` devuelven datos de prueba fijos, así que protegerlas ahora no rompe datos reales.
- **El aviso de perfil incompleto ya existe.** Al iniciar sesión, `SignInResponse.missing_fields` lista nombre, teléfono y fecha de nacimiento, y el frontend ofrece «Registrar» o «Ignorar». Este cambio sustituye ese cálculo fijo por el catálogo por rol.
- **Migraciones con refinery** (`backend/migrations`, desde V1). Todo cambio de esquema va en `V3__…` en adelante.

## Goals / Non-Goals

**Goals:**
- Una sola definición de «qué datos exige cada acción», compartida por el servidor (que decide) y el frontend (que muestra).
- Autorización calculada en el servidor en cada petición, con revocación inmediata.
- La regla de autorización como función pura, probada con una tabla que reproduce la matriz de este documento.

**Non-Goals:**
- Implementar agenda, citas, atenciones o recetas. Sus acciones se definen ahora en el catálogo y se protegen cuando se construyan.
- Verificar automáticamente el documento con RENIEC, el RUC con SUNAT o la colegiatura con el CMP. Por ahora la confirma una persona de la clínica.
- Roles de recepción, enfermería o caja.
- Permisos configurables por clínica: la matriz es fija y está en el código.

## Catálogo de datos por rol

| Dato | Paciente | Médico | Dueño | Obligatorio para |
|---|---|---|---|---|
| Nombres | ✔ | ✔ | ✔ | Confirmar cita · unirse a clínica · crear clínica · activar perfil médico · activar clínica |
| Apellido paterno | ✔ | ✔ | ✔ | Confirmar cita · activar perfil médico · activar clínica |
| Apellido materno | ✔ | ✔ | ✔ | Nunca |
| Correo | ✔ | ✔ | ✔ | Viene de Google; no se edita |
| Teléfono | ✔ | ✔ | ✔ | Confirmar cita · activar perfil médico · activar clínica |
| Documento (tipo y número: DNI, CE, pasaporte) | ✔ | ✔ | ✔ | Confirmar cita · activar perfil médico · activar clínica |
| Fecha de nacimiento | ✔ | — | — | Primera atención (abrir historia clínica) |
| Sexo | ✔ | — | — | Primera atención (abrir historia clínica) |
| Dirección | ✔ | — | — | Nunca |
| Colegio profesional y número de colegiatura | — | ✔ | — | Activar perfil médico |
| Colegiatura confirmada por la clínica | — | ✔ | — | Registrar atención o receta |
| Especialidad y RNE | — | ✔ | — | Nunca |
| RUC y razón social | — | — | ✔ | Activar clínica |
| Nombre comercial | — | — | ✔ | Nunca |
| Código IPRESS (RENIPRESS) | — | — | ✔ | Activar clínica (**validación legal pendiente**) |

Notas:
- La fecha de nacimiento y el sexo pasan a exigirse en la primera atención, no al iniciar sesión. Los datos de filiación de la historia clínica (NTS 139) los puede completar la clínica en admisión, no solo el paciente.
- El tipo de documento ya existe (`document_type`). Se admite carné de extranjería y pasaporte para no excluir a pacientes extranjeros. El DNI sigue validándose con 8 dígitos.

## Matriz de acciones

| Acción (`ProtectedAction`) | Quién | Membresía | Datos (ver catálogo) | Clínica |
|---|---|---|---|---|
| `ViewOwnProfile` / `EditOwnProfile` | Cualquiera | — | — | — |
| `BrowseClinics` | Cualquiera | — | — | `ACTIVE` |
| `ConfirmAppointment` | Paciente | — | Nombres, ap. paterno, documento, teléfono | `ACTIVE` |
| `ViewOwnRecord` | Paciente | — | Documento verificado, solo para el historial previo | — |
| `JoinClinic` | Médico | `INVITED` | Nombres | — |
| `ActivatePractitioner` | Médico | `INVITED` / `PENDING` | Nombres, ap. paterno, teléfono, documento, colegiatura | `ACTIVE` |
| `SearchPatients` / `ViewPatient` | Médico | `ACTIVE` | — | `ACTIVE` |
| `OpenFirstEncounter` | Médico | `ACTIVE` | Del paciente: fecha de nacimiento y sexo | `ACTIVE` |
| `RecordEncounter` / `IssuePrescription` | Médico | `ACTIVE` | Colegiatura confirmada | `ACTIVE` |
| `CreateClinic` | Cualquiera | — | Nombres | — |
| `ConfigureClinic` / `InviteStaff` | Dueño o administrador | `ACTIVE` | — | `PENDING_SETUP` / `ACTIVE` |
| `ActivateClinic` | Dueño | `ACTIVE` | Nombres, ap. paterno, teléfono, documento, RUC, razón social | `PENDING_SETUP` |

Orden de evaluación, que fija qué error se devuelve: no autenticado → sin membresía o rol → clínica en estado no válido → datos faltantes. Así nunca se revelan los datos faltantes a alguien que de todos modos no tiene permiso.

## Decisions

### 1. Token de sesión propio emitido en `SignIn`
`SignIn` sigue validando el `id_token` de Google y, además, devuelve un token de sesión firmado por el backend (JWT con `user_id` y `network_id`, de vida corta y con renovación). El frontend lo envía en `authorization: Bearer` y un interceptor de `bin/clickcare` lo valida en cada llamada protegida.
- *Alternativa descartada*: reenviar el `id_token` de Google en cada llamada. Vence en una hora, renovarlo exige volver a pedírselo a Google, y ata toda la API al proveedor de identidad.
- **El token no lleva roles.** Las membresías se leen en cada petición, con una caché de pocos segundos, para que una suspensión surta efecto de inmediato (requisito «Revocación de membresías»).

### 2. La autorización es una función pura en `app_core`
`authorize(action, &RequestContext, Option<&ClinicState>) -> Result<(), AccessDenied>`, donde `AccessDenied` es `Unauthenticated | Forbidden | ClinicNotActive | MissingData { action, fields }`. `bin/clickcare` arma el `RequestContext` (perfil de identidad + membresías de `administration`) y cada caso de uso llama a `authorize` antes de actuar.
- *Por qué en `core`*: la decisión cruza `user` y `administration`, y ningún contexto acotado debe depender del otro. `core` ya es el núcleo compartido.
- *Alternativa descartada*: solo un interceptor gRPC. No conoce el recurso pedido (paciente de qué clínica), así que no puede aplicar el aislamiento por clínica.
- Las pruebas recorren la matriz fila por fila con `#[rstest]` y `#[case::…]`.

### 3. El catálogo vive en `app_core` y se publica al frontend
El catálogo (dato → roles → acciones que lo exigen) se define una sola vez en `app_core::domain::access::CATALOG`, junto a `authorize`, porque lo usan tanto la regla (que cruza identidad y administración) como `crates/user` (`User::missing_for`). *Cambio respecto del borrador*: se pensó en `crates/user`, pero `authorize` vive en `app_core` y no puede depender de `user`. Una RPC nueva, `GetProfileRequirements`, devuelve para el usuario actual los campos que aplican a sus roles, el motivo de cada uno y cuáles faltan. El frontend arma el formulario de perfil y la ventana a partir de esa respuesta, sin duplicar reglas.
- `ProfileField` crece con: `FAMILY_NAME`, `DOCUMENT`, `SEX`, `ADDRESS`, `PROFESSIONAL_LICENSE`, `SPECIALTY`, `CLINIC_TAX_ID`, `CLINIC_LEGAL_NAME` e `IPRESS_CODE`.
- `SignInResponse.missing_fields` se mantiene, pero se calcula con las acciones habituales del rol preferido: paciente → `ConfirmAppointment`; médico → `ActivatePractitioner`; dueño con clínica pendiente → `ActivateClinic`.

### 4. Error «faltan datos»: `FAILED_PRECONDITION` con detalle tipado
`MissingData` se traduce en `Status::failed_precondition` con un mensaje protobuf `ActionRequirements { action, repeated ProfileField missing }` en los detalles del estado (`grpc-status-details-bin`). `Forbidden` se traduce en `PERMISSION_DENIED`, `Unauthenticated` en `UNAUTHENTICATED` y los recursos de otra clínica en `NOT_FOUND`.
- El CORS debe exponer `grpc-status-details-bin`; queda anotado en la memoria de Beads sobre CORS gRPC-Web.
- *Alternativa descartada*: codificar los campos en el texto de `grpc-message`. Es frágil y no tiene tipos.

### 5. Ventana «Completa tus datos» genérica en el frontend
- **Detección**: un envoltorio en `shared/api/grpc` detecta `FAILED_PRECONDITION` con `ActionRequirements` y publica en un store la acción, los campos y la función que reintenta la llamada.
- **Ventana**: `features/profile/ui/RequirementsDialog.svelte` (`role="dialog"`, foco atrapado) usa los mismos componentes de campo que `/complete-profile`. «Completar» llama a `CompleteProfile` y luego a la función de reintento.
- **Formulario completo**: «Editar todos mis datos» cambia el contenido de la misma ventana al formulario completo del perfil, construido con `GetProfileRequirements` como `/complete-profile`. No se navega a otra ruta, para conservar la acción pendiente y su función de reintento. El borrador es uno solo y se comparte entre ambas vistas, así que cambiar de vista no pierde lo escrito. «Completar» envía todo lo editado y solo reintenta la acción si quedan completos los datos que ella exige.
- **Rechazo**: «Rechazar», Escape y el clic fuera abren la advertencia de pérdida, con «Volver a completar» o «Salir sin continuar». Salir descarta el borrador de la ventana y el estado de la acción, por ejemplo la selección de médico y horario.
- El borrador de la ventana vive solo en memoria y no se guarda en `localStorage`, para que «perderás los datos» sea literal y no queden datos sensibles en el navegador.

### 6. Rol preferido en identidad, membresías en administración
- **Rol preferido**: `identity.user_account.preferred_role` (`PATIENT | PRACTITIONER | CLINIC_OWNER`). Se rellena desde `intent`; los registros previos toman `CLINIC_OWNER` si `is_owner` es verdadero y `PATIENT` en los demás casos.
- **Membresías**: nueva tabla `administration.membership (id, organization_id, user_id, role, status, license_confirmed_at, license_confirmed_by, timestamps)`, con `UNIQUE (organization_id, user_id, role)`. Se rellena con los dueños (`organization.owner_user_id`) y los médicos (`practitioner`) actuales.
- **Colegiatura**: se guarda en identidad (`professional_body`, `license_number`, `specialty`, `rne`), porque es de la persona y vale en todas las clínicas. La confirmación es por clínica y va en la membresía.
- `is_owner` queda obsoleto y se elimina en un cambio posterior, cuando nada lo lea.

### 7. Estado de la clínica
`administration.organization.status` (`PENDING_SETUP | ACTIVE | SUSPENDED`) sustituye a `active`. Migración de datos: `active = true` → `ACTIVE` y `false` → `SUSPENDED`. Las clínicas de prueba existentes no tienen RUC, pero seguirán activas para no romper el entorno de desarrollo; las nuevas nacen en `PENDING_SETUP`.

### 8. Página «Mis clínicas» como punto de entrada de las acciones
Hoy no existen la agenda ni las citas, así que la única acción de la interfaz que puede exigir datos es la de las membresías. Se agregó `/my-clinics`: lista las membresías y ofrece «Unirme», «Activar mi perfil médico» y «Activar clínica». Las pruebas e2e de la ventana usan «Activar mi perfil médico»; los escenarios con «confirmar cita» quedan cubiertos por la regla (`authorize`) y se probarán de punta a punta cuando exista la API de citas.

### 9. El inicio de sesión con contraseña no abre sesión
El formulario con contraseña creaba una sesión falsa (`tus`). Con sesiones del backend, esa sesión sería rechazada en la primera llamada; ahora el formulario muestra que el acceso con contraseña aún no está disponible.

## Risks / Trade-offs

- **[El frontend actual deja de funcionar al exigir credencial]** → El interceptor y el envío del token se despliegan juntos. Las e2e cubren el flujo antes de activar el rechazo.
- **[grpc-web y los detalles del estado]** → Se valida con una prueba e2e temprana que el frontend lee `grpc-status-details-bin`. Si no se puede, el plan B es un campo de respuesta en las RPC protegidas.
- **[Una caché de membresías demasiado larga retrasa las revocaciones]** → TTL de pocos segundos, invalidado al suspender en el mismo proceso.
- **[Clínicas existentes marcadas `ACTIVE` sin RUC]** → Solo hay datos de prueba (acordado). Queda anotado en las tareas para revisar antes de producción.
- **[Interpretación legal]** → Cada regla con base normativa queda marcada como pendiente de validación legal. Si la asesoría cambia una exigencia, se cambia el catálogo, sin rediseñar.
- **[Perder la selección al rechazar frustra al usuario]** → Es intencional (pedido explícito) y la advertencia lo dice antes de confirmar.

## Migration Plan

1. `V3__rol_y_datos_de_perfil.sql`: `preferred_role`, `sex` y los campos profesionales en `identity.user_account`, con relleno de `preferred_role`.
2. `V4__membresias.sql`: crea `administration.membership` y la rellena desde `organization` y `practitioner`.
3. `V5__estado_de_clinica.sql`: agrega `status`, lo rellena desde `active` y elimina `active` cuando el código ya no lo use (puede ir en una migración posterior).
4. Se despliega el backend (token de sesión, interceptor y guardas) junto con el frontend (envío del token, ventana y rutas por rol).

Las migraciones solo avanzan. Para revertir: volver a desplegar la versión anterior. Las columnas y tablas nuevas no estorban al código previo, salvo la eliminación de `active`, que por eso va separada y al final.

## Open Questions

- ¿Se exige el código IPRESS para activar la clínica, o solo para emitir ciertos documentos? Depende de la validación legal; solo cambia una fila del catálogo.
- ¿Qué colegios profesionales se aceptan además del CMP (médicos), el COP (odontólogos), el CEP (enfermeros) y el CPsP (psicólogos)? Por ejemplo, obstetras o nutricionistas. Es una lista de valores.
- ¿Cuánto dura el token de sesión y cómo se renueva? Se puede ajustar sin cambiar las especificaciones.
