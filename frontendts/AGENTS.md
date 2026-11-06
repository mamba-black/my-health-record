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
│                    # patient-profile, encounter, not-found
├── widgets/         # navbar, patient-detail, encounter-list, encounter-detail
├── features/
│   ├── auth/        # api/auth.api.ts (signInWithGoogle, signUpAccount), ui/
│   └── patient-search/  # api/, ui/
├── entities/
│   ├── session/     # model/ (store de sesión)
│   ├── patient/     # api/, model/, ui/
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

## 5. Comandos (desde `frontendts/`)

```bash
bun install      # Instalar dependencias
bun run dev      # Servidor de desarrollo Vite
bun run check    # Tipos de SvelteKit y svelte-check
bun run build    # Build estático de producción
bun run test:e2e # Pruebas e2e con Playwright (e2e/); levanta Vite si no está corriendo
```

Las pruebas e2e de búsqueda requieren el backend en `:50051`. La primera vez instala el navegador con `bun x playwright install chromium`.
