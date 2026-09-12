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
│   ├── ddl/                # Esquema de base de datos PostgreSQL (UUIDv7)
│   ├── docs/use_cases.md   # Casos de uso del ciclo de vida de identidad
│   └── crates/             # Crates del Cargo Workspace por Contexto Acotado
│
└── frontendts/             # ⚡ Frontend Web (Svelte 5 + SvelteKit + TypeScript + Tailwind CSS v4)
    ├── package.json        # Dependencias y scripts de frontend (Bun/Vite)
    ├── svelte.config.js    # Configuración de SvelteKit (@sveltejs/adapter-static)
    ├── vite.config.ts      # Configuración de Vite / Rolldown
    ├── docs/use_cases.md   # Casos de uso de frontend y flujos de usuario
    └── src/                # Código fuente de la aplicación Svelte 5
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
| **Reserva de Citas** | `backend/crates/scheduling` | 📄 Solo diseño | `Schedule`, `Slot`, `Appointment` | [backend/crates/scheduling/AGENTS.md](backend/crates/scheduling/AGENTS.md) |
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

El frontend está estructurado en **Svelte 5** utilizando **SvelteKit** (con adaptador estático `@sveltejs/adapter-static`), TypeScript, **Tailwind CSS v4** y comunicación gRPC-Web hacia el backend.

* **Ruta**: `frontendts/`
* **Tecnologías**: Svelte 5, SvelteKit, TypeScript, Vite / Rolldown, Tailwind CSS v4, Lucide Icons, TipTap, grpc-web.
* **Documentación y Casos de Uso**: [frontendts/docs/use_cases.md](frontendts/docs/use_cases.md)

### Comandos de Frontend (desde `frontendts/`)
```bash
bun install             # Instalar dependencias
bun run dev             # Servidor de desarrollo Vite
bun run build           # Compilación para producción
bun run check           # Verificación de tipos con svelte-check y tsc
```

---

## 4. Convenciones Globales del Repositorio

1. **Scripts y Automatización**:
   * Se prefieren scripts escritos en **Nushell (`.nu`)** para automatización de tareas y DevOps.
2. **Commits y Pull Requests**:
   * Formato Conventional Commits (`feat:`, `fix:`, `refactor:`, `chore:`, etc.).
   * **Obligatorio**: Todo commit generado o asistido por IA debe incluir la marca **`[antigravity]`** (ejemplo: `feat(user): [antigravity] add user domain model`).
3. **Seguimiento de Tareas**:
   * Usar **`bd` (Beads)** para gestión de tareas e issues. No usar TODOs informales en markdown.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:970c3bf2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

* Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
* Run `bd prime` for detailed command reference and session close protocol
* Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See <https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md> for details and anti-patterns.
<!-- END BEADS INTEGRATION -->
