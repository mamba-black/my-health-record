## Why

Hoy cualquier sesión puede abrir la búsqueda de pacientes, sus perfiles y sus atenciones, sin importar si es paciente, médico o dueño de clínica, ni qué datos registró. El backend ni siquiera identifica a quien llama: las llamadas gRPC no llevan credencial, y el rol que la persona eligió al registrarse (`intent`) no se guarda. En una clínica eso es inaceptable, porque los datos de salud son datos sensibles y los actos médicos exigen identidad y colegiatura verificadas.

Al mismo tiempo, la documentación pide registro progresivo: nada de burocracia al entrar. Este cambio fija una regla única para ambas cosas: **cada rol ve todos los datos que llegará a necesitar, pero solo se le exigen cuando intenta una acción que los requiere.**

## What Changes

- **Catálogo de datos por rol**: se define qué datos necesita cada rol (paciente, médico, dueño/administrador de clínica) y para qué acción concreta se vuelve obligatorio cada uno.
- **Formulario de perfil por rol**: `/complete-profile` muestra todos los datos del rol, con el motivo de cada uno ("lo pediremos al confirmar tu primera cita"). Ninguno es obligatorio al guardar, salvo los que exige la acción que el usuario está intentando.
- **Exigencia en el momento (just-in-time)**: cada acción protegida verifica sus datos requeridos. Si faltan, el backend responde con la lista de datos que faltan y el frontend abre una ventana «Completa tus datos» con un formulario solo de esos datos y las opciones «Completar» y «Rechazar». Completar guarda los datos y retoma la acción; rechazar advierte que sin esos datos no podrá continuar y que perderá lo ingresado, y pide confirmación.
- **BREAKING** — **Aviso al iniciar sesión**: deja de exigir nombre, teléfono y fecha de nacimiento a todos. Se vuelve un recordatorio ignorable, calculado según el rol, de los datos que faltan para las acciones habituales de ese rol.
- **Rol persistido y membresías por clínica**: se guarda el `intent` del registro como rol preferido (solo orienta la interfaz) y se crean membresías `(clínica, usuario, rol, estado)`, que son las que autorizan.
- **Autenticación de cada llamada**: el backend identifica al usuario en cada petición gRPC protegida. Es requisito previo de cualquier control de acceso.
- **Control de acceso en el backend**: `SearchPatient`, `GetPatientById` y las atenciones solo responden a un médico activo en una clínica activa, y solo con pacientes de esa clínica. Un paciente solo accede a lo suyo y a lo de sus dependientes.
- **Trazabilidad legal**: cada regla cita la norma que la respalda (Ley 30024, Ley 29733, NTS 139-MINSA). Las citas quedan marcadas como **pendientes de validación legal**.

## Capabilities

### New Capabilities
- `identity/progressive-profiling`: qué datos necesita cada rol, cómo se muestran en el perfil y en qué acción se vuelve obligatorio cada uno.
- `identity/role-membership`: rol preferido de la cuenta y membresías de cada usuario en cada clínica, con su estado.
- `access-control/action-authorization`: qué acciones puede ejecutar cada rol según sus datos, su membresía y el estado de la clínica, y cómo se rechaza una acción a la que le faltan datos.

### Modified Capabilities
- `identity/signup-intent`: el `intent` del registro deja de ser solo una señal del alta y se persiste como rol preferido de la cuenta.

## Impact

- **Proto (`proto/api.proto`)**: el campo `missing_fields` de `SignInResponse` pasa a calcularse por rol; nuevos valores en `ProfileField` (sexo, documento, colegiatura, RUC…); detalle de error estándar para "faltan datos para esta acción"; credencial en la metadata de las llamadas protegidas.
- **`crates/user`**: rol preferido, catálogo de datos por rol y cálculo de faltantes; validación de nuevos campos (sexo, tipo de documento, colegiatura).
- **`crates/administration`**: membresías por clínica; estado de la clínica (`PENDING_SETUP` / `ACTIVE`) en lugar del booleano `active`; colegiatura en `Practitioner`.
- **`bin/clickcare`**: interceptor que autentica cada llamada y guardas de autorización en `PatientApi` y en las futuras APIs de citas y atenciones.
- **Migraciones (`backend/migrations`)**: nuevas columnas y tablas vía refinery (`V3__…` en adelante).
- **Frontend**: formulario de perfil por rol, manejo del rechazo "faltan datos" con reintento, rutas y menús según el rol, y envío de la credencial en cada llamada.
- **Pruebas**: un escenario e2e o de backend por cada regla de la matriz.
- **Fuera de alcance**: agenda y citas reales (`crates/scheduling`), recetas, verificación automática contra RENIEC, SUNAT o el CMP, y el rol de recepcionista. Aquí solo se definen sus reglas de acceso, para que nazcan protegidos.
