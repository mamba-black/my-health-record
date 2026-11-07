# Tasks

## 1. Sesión autenticada en cada llamada

- [x] 1.1 Emitir un token de sesión propio (JWT con `user_id` y `network_id`, vida corta y renovación) en `SignIn` y `SignUp`, y devolverlo en la respuesta del proto; verificar con pruebas de `user_api_test` que el token se emite y que sus claims coinciden con la cuenta
- [x] 1.2 Agregar a `bin/clickcare` un interceptor que valide `authorization: Bearer` en las RPC protegidas y deje pasar `SignUp`, `SignIn` y la reflexión; verificar con pruebas que sin token o con token vencido responde `UNAUTHENTICATED` sin datos
- [x] 1.3 Enviar el token en cada llamada gRPC desde `shared/api/grpc` y llevar a `/login` ante `UNAUTHENTICATED`; verificar con una e2e que una sesión vencida termina en el login
- [x] 1.4 Permitir `authorization` y exponer `grpc-status-details-bin` en el CORS, y actualizar la memoria de Beads sobre CORS gRPC-Web; verificar con una e2e que el navegador recibe los detalles de un error
- [x] 1.5 Documentar la sesión en `backend/AGENTS.md` y `frontendts/AGENTS.md`; verificar que ambos describen el encabezado y la renovación

## 2. Rol preferido y datos de perfil ampliados

- [x] 2.1 Crear `V3__rol_y_datos_de_perfil.sql` (`preferred_role`, `sex`, `professional_body`, `license_number`, `specialty`, `rne`, con relleno de `preferred_role` desde `is_owner`); verificar que `user_api_test` migra una base nueva y que un segundo arranque contra Neon informa «Esquema al día»
- [x] 2.2 Persistir `intent` como rol preferido en el registro (delta de `identity/signup-intent`); verificar con el escenario «La intención se conserva como rol preferido»
- [x] 2.3 Ampliar `ProfileField`, `CompleteProfileRequest` y `SignInResponse` con los datos nuevos, y validar tipo y número de documento, sexo, colegio y colegiatura; verificar con pruebas unitarias del caso de uso los escenarios de «Formato y validación de los datos»
- [x] 2.4 Impedir que el usuario edite el documento o la colegiatura ya verificados; verificar con los escenarios «DNI verificado» y «DNI no verificado»

## 3. Membresías y estado de la clínica

- [x] 3.1 Crear `V4__membresias.sql` con `administration.membership` y rellenarla desde `organization.owner_user_id` y `practitioner`; verificar con una prueba que cada dueño y cada médico existente tienen su membresía
- [x] 3.2 Crear la entidad `Membership`, su puerto y su repositorio en `crates/administration`, y crear la membresía de dueño activa en `CreateClinicUseCase`; verificar con el escenario «Creación de clínica»
- [x] 3.3 Crear `V5__estado_de_clinica.sql` (`status` rellenado desde `active`) y usarlo en `Organization`; verificar que las clínicas nuevas nacen en `PENDING_SETUP` y las existentes quedan `ACTIVE`
- [x] 3.4 Implementar invitación, activación y suspensión de membresías de médico (incluida la confirmación de colegiatura y la protección del último dueño); verificar con los escenarios de `identity/role-membership`
- [x] 3.5 Actualizar `crates/administration/AGENTS.md` y `crates/clinic_admin/AGENTS.md` (se resuelve el solapamiento con `is_owner`); verificar que ningún documento describe `is_owner` como fuente de permisos

## 4. Catálogo de datos y regla de autorización

- [x] 4.1 Definir en `crates/user` el catálogo de datos (dato → roles → acciones que lo exigen) tal como está en `design.md`; verificar con una prueba que recorre cada fila de la tabla del catálogo
- [x] 4.2 Implementar en `app_core` `authorize(action, contexto, clínica)` con el orden de evaluación del diseño; verificar con una prueba `#[rstest]` que tiene un `#[case]` por cada fila de la matriz de acciones y por cada orden de error
- [x] 4.3 Armar el contexto de la petición en `bin/clickcare` (perfil y membresías, con caché corta e invalidación al suspender); verificar con el escenario «Médico suspendido»
- [x] 4.4 Traducir `AccessDenied` a estados gRPC, con `ActionRequirements` en los detalles cuando faltan datos; verificar con los escenarios «Error distinguible» y «Sin rol no se listan datos»
- [x] 4.5 Agregar la RPC `GetProfileRequirements` y recalcular `SignInResponse.missing_fields` según el rol preferido; verificar con pruebas que paciente, médico y dueño reciben los campos de su rol

## 5. Protección de las APIs existentes

- [x] 5.1 Proteger `SearchPatient` y `GetPatientById` con `SearchPatients` y `ViewPatient`, limitadas a la clínica del médico; verificar con los escenarios «Paciente busca pacientes», «Médico de clínica pendiente», «Paciente de otra clínica» y «Misma persona en dos clínicas»
- [x] 5.2 Proteger `CreateClinic` con `CreateClinic` y dejar la clínica en `PENDING_SETUP`; verificar con los escenarios «Crear clínica sin RUC» y «Dueño configura antes de activar»

## 6. Frontend: perfil, ventana y rutas por rol

- [x] 6.1 Rehacer `/complete-profile` a partir de `GetProfileRequirements`: campos agrupados por rol y motivo junto a cada dato; verificar con las e2e «Médico que también es paciente» y «Datos opcionales siempre»
- [x] 6.2 Crear `RequirementsDialog` («Completa tus datos») con solo los datos faltantes, «Completar» y «Rechazar»; verificar con las e2e «Contenido de la ventana», «Datos ya registrados no se piden» y «Validación dentro de la ventana»
- [x] 6.3 Agregar «Editar todos mis datos» y «Ver solo lo necesario» a la ventana, con un borrador compartido entre ambas vistas; verificar con las e2e «Pasar al formulario completo», «Lo escrito no se pierde al cambiar de vista», «Editar datos opcionales y continuar», «Faltan los obligatorios en el formulario completo» y «Volver a la vista reducida»
- [x] 6.4 Conectar el envoltorio de errores gRPC con la ventana y reintentar la acción al completar; verificar con la e2e «Completar y continuar»
- [x] 6.5 Implementar la advertencia de pérdida al rechazar (también con Escape o clic fuera) y descartar el borrador y el estado de la acción al salir; verificar con las e2e «Advertencia al rechazar», «Volver a completar», «Salir sin continuar» y «Cierre de la ventana por otros medios»
- [x] 6.6 Ajustar el aviso al iniciar sesión al rol preferido y adaptar `e2e/incomplete-profile.spec.ts`; verificar con las e2e «Aviso ignorable» y «Sin datos faltantes no hay aviso»
- [x] 6.7 Mostrar menús y rutas según los roles y agregar la página de acceso denegado; verificar con las e2e «Paciente entra por URL a la búsqueda» y «Médico activo»
- [x] 6.8 Documentar en `frontendts/AGENTS.md` la ventana y el patrón de reintento; verificar que el documento explica cómo una acción nueva se suma al patrón

## 7. Verificación integral y documentación oficial

- [x] 7.1 Ejecutar `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `bun run check` y la suite e2e completa; verificar que todo pasa, salvo los fallos previos documentados en Beads
- [ ] 7.2 Revisar que cada escenario de las tres especificaciones nuevas y del delta tiene al menos una prueba que lo cubre; verificar con una tabla escenario → prueba en el PR
- [x] 7.3 Enlazar las especificaciones desde los `AGENTS.md` de `user`, `administration` y `scheduling`, y quitar las reglas duplicadas; verificar que la tabla de registro progresivo de `crates/user/AGENTS.md` apunta a `identity/progressive-profiling`

## Workflow follow-up

- Revisión legal o médica de las normas citadas (Ley 30024, Ley 29733, NTS 139-MINSA, colegiatura y SUSALUD/IPRESS) antes de considerar oficiales las especificaciones.
- Revisar antes de producción las clínicas marcadas `ACTIVE` sin RUC por la migración.
- Archivar el cambio cuando se cumplan los requisitos de revisión del proyecto, y verificar que las especificaciones quedan en `openspec/specs/`.
