# Guía del Frontend (`frontendts/`) — Feature-Sliced Design (FSD)

Aplicación web de **My Health Record** construida con **Svelte 5**, **SvelteKit** (`@sveltejs/adapter-static`), **TypeScript** y **Tailwind CSS v4**. La organización del código sigue **Feature-Sliced Design (FSD v2.1)**. Estas son las únicas reglas de arquitectura del frontend; las reglas del backend viven en `backend/AGENTS.md`.

## 1. Conceptos de FSD

1. **Layers (capas):** directorios de primer nivel en `src/`.
2. **Slices (rebanadas):** subdirectorios de `pages/`, `widgets/`, `features/` y `entities/`, que separan el código por dominio de negocio (`patient`, `encounter`, `session`, `auth`…). `app/` y `shared/` no tienen slices.
3. **Segments (segmentos):** subdirectorios técnicos dentro de cada slice o de `app/` y `shared/`:
   * `ui/`: componentes Svelte.
   * `model/`: tipos, stores y lógica de dominio.
   * `api/`: llamadas al backend, DTOs y mappers.
   * `lib/`: utilidades.
   * `config/`: constantes y rutas.

## 2. Reglas

### 2.1. Importación entre capas
Un módulo solo importa de capas **estrictamente inferiores**:

| Capa | Alias | Puede importar de |
| :--- | :--- | :--- |
| `routes/` | (SvelteKit) | `$app_layer`, `$pages` |
| `app/` | `$app_layer` | `$pages`, `$widgets`, `$features`, `$entities`, `$shared` |
| `pages/` | `$pages` | `$widgets`, `$features`, `$entities`, `$shared` |
| `widgets/` | `$widgets` | `$features`, `$entities`, `$shared` |
| `features/` | `$features` | `$entities`, `$shared` |
| `entities/` | `$entities` | `$shared` |
| `shared/` | `$shared` | Solo paquetes externos |

Los paquetes externos (Svelte, SvelteKit, Tailwind, gRPC-Web…) pueden usarse en cualquier capa. Esto incluye el runtime de SvelteKit (`$app/navigation`, `$app/state`, `$app/paths`, `$app/environment`), que no se abstrae detrás de `shared/`.

### 2.2. Aislamiento entre slices
* Dos slices de la misma capa **no se importan entre sí**.
* La composición entre entidades o features se hace en una capa superior (`widgets/` o `pages/`).

### 2.3. API pública (`index.ts`)
* Cada slice y cada segmento de `shared/` y `app/` expone su API en un `index.ts` en su raíz.
* Las capas externas importan solo desde ese `index.ts`:
  * ✅ `import { Patient } from "$entities/patient";`
  * ✅ `import { Input } from "$shared/ui";`
  * ✅ `import { log } from "$shared/lib";`
  * ❌ `import Patient from "$entities/patient/model/Patient";`
  * ❌ `import Input from "$shared/ui/input/Input.svelte";`

## 3. Estructura actual

```text
src/
├── routes/          # Adaptador delgado de SvelteKit: solo monta páginas y el layout
├── app/             # Layout global, guardia de autenticación y estilos (app/ui, app/styles)
├── pages/           # Una página por ruta: home, login, signup, patient-search,
│                    # patient-profile, encounter, not-found, clinic-public,
│                    # my-appointments, my-schedule, clinic-agenda, register-patient
├── widgets/         # navbar, patient-detail, encounter-list, encounter-detail
├── features/
│   ├── auth/        # api/auth.api.ts (signInWithGoogle, signUpAccount), ui/
│   ├── patient-search/  # api/, ui/
│   ├── book-appointment/  # SlotPicker y reserva pendiente
│   ├── manage-schedule/   # ScheduleEditor
│   ├── reception/         # BookForPatientForm
│   ├── clinic-staff/      # solicitudes de ingreso y personal
│   └── register-patient/  # paciente sin cuenta
├── entities/
│   ├── session/     # model/ (store de sesión)
│   ├── patient/     # api/, model/, ui/
│   ├── appointment/ # lib/ (estados y formato de fechas)
│   └── encounter/   # model/
└── shared/
    ├── api/         # index público: grpc/ (clientes gRPC-Web), user/ (userApi)
    ├── assets/      # index público: svelteLogo
    ├── config/      # index público: routes/ y auth/
    ├── lib/         # index público: log (logger) y jwt (decodeGoogleJwt, generateUuidV7)
    └── ui/          # Kit de componentes base (button, checkbox, input, modal, title)
```

## 4. Convenciones

* **Svelte 5:** runes (`$props`, `$state`, `$derived`, `$effect`, `$bindable`), `{@render children?.()}` en lugar de `<slot />`, y eventos nativos (`onclick`, `onsubmit`) en lugar de `on:click`.
* **TypeScript:** primitivos (`string`, `number`, `boolean`), nunca los tipos envoltorio (`String`, `Number`, `Boolean`).
* **Verificación:** `bun run check` debe terminar con 0 errores y 0 advertencias antes de dar un cambio por bueno.

## 5. Sesión, roles y datos exigidos

Fuente oficial: [`identity/progressive-profiling`](../openspec/specs/identity/progressive-profiling/spec.md) y
[`access-control/action-authorization`](../openspec/specs/access-control/action-authorization/spec.md).

* **Sesión** (`shared/api/session`): los tokens del backend viven en `localStorage` (`auth_token`, `auth_refresh`, `auth_expires`). El interceptor de `shared/api/grpc/client.ts` agrega `authorization: Bearer` a toda llamada protegida. Envuelve cada llamada protegida en `withSession(...)`: renueva el token si está por vencer, reintenta una vez ante `UNAUTHENTICATED` y, si no puede renovar, cierra la sesión y lleva al login.
* **Rol y membresías** (`entities/session`): `profileContext` guarda la respuesta de `GetProfileRequirements`; `canSearchPatients` y `hasClinicMemberships` deciden menús y rutas. `AppLayout` redirige a `/access-denied` las rutas de médico. Es solo presentación: el servidor autoriza igual.
* **Acción que exige datos**: ejecútala con `runWithRequirements(() => api.accion(...))` de `$features/profile`. Si el servidor responde `FAILED_PRECONDITION` con `ActionRequirements`, se abre la ventana global «Completa tus datos» (`RequirementsDialog`, montada en `AppLayout`) con solo los datos faltantes, el acceso a «Editar todos mis datos», y «Completar»/«Rechazar». Al completar se guardan los datos y se reintenta la acción; devuelve `undefined` si el usuario sale sin continuar, y la página debe descartar el estado de la acción.
* Etiquetas de datos y frases por acción: `entities/profile` (`FIELD_LABELS`, `ACTION_PURPOSE`).

## 5.1. Agenda y citas

Fuente oficial: `openspec/specs/scheduling/*` (o `openspec/changes/scheduling-appointments` mientras no se archive).

| Ruta | Página | Quién | Qué hace |
|---|---|---|---|
| `/c/[subdomain]` | `pages/clinic-public` | Cualquiera, sin sesión | La clínica, sus médicos activos con horario y sus cupos libres. Reservar exige sesión: sin ella guarda el cupo (`features/book-appointment`) y vuelve tras iniciarla. La reserva usa `runWithRequirements` con `ACTION_CONFIRM_APPOINTMENT`. Recuerda el enlace (`shared/lib/clinic-link`) para que una cuenta nueva quede en la organización de esa clínica. |
| `/appointments` | `pages/my-appointments` | Toda cuenta | «Mis citas»: cancelar y reprogramar (mismo médico, `SlotPicker`). |
| `/schedule` | `pages/my-schedule` | Médico activo (`practitionerClinics`) | «Mi agenda»: horario semanal, ausencias (`features/manage-schedule`) y sus citas. |
| `/clinic-agenda` | `pages/clinic-agenda` | Recepción o médico activo (`agendaClinics`) | Agenda del día; recepción reserva por un paciente (`features/reception`), registra la llegada con la verificación del documento, marca inasistencias y vincula pacientes sin cuenta con «Vi el documento físico». El médico solo ve y reserva en su agenda. |

* Las fechas viajan en RFC 3339 (UTC) y se muestran en la zona de la clínica con `entities/appointment` (`formatDateTime`, `formatTime`).
* La API pública (`publicSchedulingApi`) no lleva token: sus métodos están en `PUBLIC_METHODS` de `shared/api/grpc/client.ts`.

## 6. Comandos (desde `frontendts/`)

```bash
bun install      # Instalar dependencias
bun run dev      # Servidor de desarrollo Vite
bun run check    # Tipos de SvelteKit y svelte-check
bun run build    # Build estático de producción
bun run test:e2e # Pruebas e2e con Playwright (e2e/); levanta sus propios servidores
```

Las pruebas e2e levantan el backend en `:50052` (con la feature `test-support`), Vite en `:5174` y un Google simulado en `:50053`, sin interferir con los servidores de desarrollo. Usan la base de `backend/.env`. La primera vez instala el navegador con `bun x playwright install chromium`.
