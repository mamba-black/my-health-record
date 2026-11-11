# My Health Record — Guía Global del Repositorio para Agentes

Bienvenido a **My Health Record**. Este repositorio es un monorepo orientado a salud que implementa el estándar **HL7 FHIR R4**, compuesto por un backend en Rust y un frontend web en Svelte / TypeScript.

---

## 1. Estructura General del Monorepo

```
my-health-record/
├── AGENTS.md               # Este archivo — Orquestador global de contexto para agentes
├── CLAUDE.md               # Enlace a AGENTS.md
├── GEMINI.md               # Enlace a AGENTS.md
│
├── backend/                # 🦀 Backend en Rust (Arquitectura Cebolla + DDD + FHIR R4)
│   ├── AGENTS.md           # 📖 Guía técnica detallada del Backend (Arquitectura, Reglas y Comandos)
│   ├── Cargo.toml          # Workspace de Rust (crates por Bounded Contexts)
│   ├── bin/clickcare/      # Servidor ejecutable (gRPC + gRPC-Web con tonic/axum)
│   ├── proto/api.proto     # Contrato Protobuf público (Única fuente de verdad de la API)
│   ├── migrations/         # Migraciones versionadas del esquema PostgreSQL (refinery)
│   ├── ddl/                # Modelo DBML del esquema (referencia)
│   ├── docs/use_cases.md   # Casos de uso del ciclo de vida de identidad
│   └── crates/             # Crates del Cargo Workspace por Contexto Acotado
│
└── frontendts/             # ⚡ Frontend Web (Svelte 5 + SvelteKit + FSD + Tailwind CSS v4)
    ├── AGENTS.md           # 📖 Guía técnica detallada del Frontend (Feature-Sliced Design)
    ├── package.json        # Dependencias y scripts de frontend (Bun/Vite)
    ├── svelte.config.js    # Configuración de SvelteKit y alias de capas FSD
    ├── vite.config.ts      # Configuración de Vite / Rolldown
    ├── docs/use_cases.md   # Casos de uso de frontend y flujos de usuario
    └── src/                # Código fuente FSD (routes, app, pages, widgets, features, entities, shared)
```

---

## 2. Backend (`backend/`)

El backend está desarrollado en **Rust 2024** siguiendo la **Arquitectura Cebolla (Onion Architecture)** y un **Cargo Workspace por Contextos Acotados (*Bounded Context Crates*)**.

> [!IMPORTANT]
> Para conocer las reglas arquitectónicas completas, convenciones de código (UUIDv7, inyección de dependencias `Arc<dyn Trait>`, cero unwrap, bon builder, logging con `tracing`, etc.) y comandos de ejecución, consulta:
> 👉 **[backend/AGENTS.md](backend/AGENTS.md)**

### Mapa de Crates y Contextos Acotados

Cada Bounded Context está encapsulado en su propio crate dentro de `backend/crates/` y cuenta con su documentación técnica dedicada:

| Bounded Context | Ruta del Crate | Estado | Recurso FHIR Mapeado | Documentación Dedicada |
| :--- | :--- | :--- | :--- | :--- |
| **Núcleo Compartido** | `backend/crates/core` | ✅ Implementado | VOs FHIR (`HumanName`, `Identifier`), `UseCase`, `ClickCareError` | [backend/crates/core/AGENTS.md](backend/crates/core/AGENTS.md) |
| **Identity & Security** | `backend/crates/user` | ✅ Implementado | `Person` + `User` | [backend/crates/user/AGENTS.md](backend/crates/user/AGENTS.md) |
| **Gestión Administrativa** | `backend/crates/administration` | ✅ Implementado | `Patient`, `Practitioner`, `Organization` | [backend/crates/administration/AGENTS.md](backend/crates/administration/AGENTS.md) |
| **Expediente de Pacientes** | `backend/crates/patient` | 🚧 Andamiaje | `Patient` | [backend/crates/patient/AGENTS.md](backend/crates/patient/AGENTS.md) |
| **Clínica** | `backend/crates/clinic` | 🚧 Andamiaje | `Organization` | [backend/crates/clinic/AGENTS.md](backend/crates/clinic/AGENTS.md) |
| **Administración de Clínica**| `backend/crates/clinic_admin`| 🚧 Andamiaje | Rol de administrador | [backend/crates/clinic_admin/AGENTS.md](backend/crates/clinic_admin/AGENTS.md) |
| **Reserva de Citas** | `backend/crates/scheduling` | ✅ Implementado | `Schedule`, `Slot`, `Appointment` | [backend/crates/scheduling/AGENTS.md](backend/crates/scheduling/AGENTS.md) |
| **Historia Clínica** | `backend/crates/clinical` | 📄 Solo diseño | `Encounter`, `Condition`, `AllergyIntolerance`, `CarePlan` | [backend/crates/clinical/AGENTS.md](backend/crates/clinical/AGENTS.md) |
| **Laboratorio e Imágenes** | `backend/crates/diagnostics` | 📄 Solo diseño | `ServiceRequest`, `Observation`, `DiagnosticReport` | [backend/crates/diagnostics/AGENTS.md](backend/crates/diagnostics/AGENTS.md) |
| **Farmacia e Insumos** | `backend/crates/pharmacy` | 📄 Solo diseño | `Medication`, `MedicationDispense`, `SupplyRequest` | [backend/crates/pharmacy/AGENTS.md](backend/crates/pharmacy/AGENTS.md) |
| **Aseguradoras y Coberturas** | `backend/crates/coverage` | 📄 Solo diseño | `Coverage`, `Claim`, `CoverageEligibilityRequest` | [backend/crates/coverage/AGENTS.md](backend/crates/coverage/AGENTS.md) |
| **Facturación** | `backend/crates/billing` | 📄 Solo diseño | `Account`, `Invoice`, `ChargeItem` | [backend/crates/billing/AGENTS.md](backend/crates/billing/AGENTS.md) |
| **Archivo Legal y Auditoría** | `backend/crates/legal_archive` | 📄 Solo diseño | `Composition`, `DocumentReference`, `AuditEvent` | [backend/crates/legal_archive/AGENTS.md](backend/crates/legal_archive/AGENTS.md) |
| **Notificaciones y Alertas** | `backend/crates/communication` | 📄 Solo diseño | `Communication`, `Flag` | [backend/crates/communication/AGENTS.md](backend/crates/communication/AGENTS.md) |

### Comandos Rápidos del Backend (desde `backend/`)

```bash
cargo check --workspace
cargo test --workspace
cargo build -p clickcare
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 3. Frontend (`frontendts/`)

El frontend está estructurado en **Svelte 5** utilizando **SvelteKit** (con adaptador estático `@sveltejs/adapter-static`), TypeScript, **Tailwind CSS v4** y arquitectura **Feature-Sliced Design (FSD v2.1)** alineada con los recursos **HL7 FHIR R4**.

> [!IMPORTANT]
> Para conocer las reglas arquitectónicas de Feature-Sliced Design (jerarquía de capas `app` → `pages` → `widgets` → `features` → `entities` → `shared`, aislamiento de slices, API pública `index.ts` y convenciones de Svelte 5), consulta:
> 👉 **[frontendts/AGENTS.md](frontendts/AGENTS.md)**

* **Ruta**: `frontendts/`
* **Tecnologías**: Svelte 5, SvelteKit, Feature-Sliced Design (FSD), TypeScript, Vite / Rolldown, Tailwind CSS v4, Lucide Icons, TipTap, grpc-web.
* **Documentación y Casos de Uso**: [frontendts/docs/use_cases.md](frontendts/docs/use_cases.md)

### Comandos de Frontend (desde `frontendts/`)

```bash
bun install             # Instalar dependencias
bun run dev             # Servidor de desarrollo Vite
bun run build           # Compilación para producción
bun run check           # Verificación de tipos con svelte-check
```

---

## 4. Convenciones Globales del Repositorio

1. **Scripts y Automatización**:
   * Se prefieren scripts escritos en **Nushell (`.nu`)** para automatización de tareas y DevOps.
2. **Commits y Pull Requests**:
   * Formato Conventional Commits (`feat:`, `fix:`, `refactor:`, `chore:`, etc.).
3. **Seguimiento de Tareas**:
   * Usar **`bd` (Beads)** para gestión de tareas e issues. No usar TODOs informales en markdown.

<!-- braid:agents:start v8 -->
## braid workflow

this repo uses braid (`brd`) for issue tracking. issues live in `.braid/issues/` as markdown files.

basic flow:

1. `brd start` — claim the next ready issue
2. do the work, commit as usual
3. `brd done <id>` — mark the issue complete
4. ship your work:
   * in a worktree: `brd agent merge` (rebase + ff-merge to main)
   * on main: just `git push` (you're already there)

useful commands:
* `brd ls` — list all issues
* `brd ready` — show issues with no unresolved dependencies
* `brd show <id>` — view issue details (shows deps and dependents)
* `brd show <id> --context` — view issue with full content of related issues
* `brd config` — show current workflow configuration
* `brd path <id>` — print the absolute file path of an issue (for hand-editing)

**tip:** before starting work, use `brd show <id> --context` to see the issue plus all its dependencies and dependents in one view.

## working on main vs in a worktree

**quick check — am i in a worktree?**

```bash
cat .braid/agent.toml 2>/dev/null && echo "yes, worktree" || echo "no, main"
```

**if you're in a worktree (feature branch):**
* `brd start` handles syncing automatically
* use `brd agent merge` to ship (rebase + ff-merge to main)
* if you see schema mismatch errors, rebase onto latest main

**if you're on main:**
* `brd start` syncs and claims
* after `brd done`, just `git push` your code commits
* no `brd agent merge` needed — you're already on main

## design and meta issues

**design issues** (`type: design`) require human collaboration:
* don't close autonomously — discuss with human first
* research options, write up trade-offs in the issue body
* produce output before closing (implementation issues or a plan)
* only mark done after human approves

**meta issues** (`type: meta`) are tracking issues:
* group related work under a parent issue
* show progress as "done/total" in `brd ls`
* typically not picked up directly — work on the child issues instead

## syncing issues (issues with code)

this repo stores issues **with code** — issues live in `.braid/issues/` and sync via git.

**how it works:**
* `brd start` auto-syncs: fetches, rebases, claims, commits, and pushes
* `brd done` marks complete and auto-pushes (if auto_push enabled)
* issue changes flow through your normal git workflow

**in a worktree (feature branch):**

```bash
brd done <id>        # marks done, auto-pushes issue state
brd agent merge      # ship code to main (rebase + ff-merge)
```

**on main:**

```bash
brd done <id>        # marks done, auto-pushes issue state
git push             # push your code commits
```

**changing settings:**
* `brd config` — show current config
* `brd config issues-branch <name>` — enable issues-branch for multi-agent setups
<!-- braid:agents:end -->
