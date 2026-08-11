# GEMINI.md

Foundational mandates and operational workflows for "My Health Record" (Backend).

## 1. Core Mandates
1. **Safety & Integrity:** No secrets/PII in code/logs. Use environment variables via `dotenvy`. Never use `unwrap()` in production paths.
2. **Rust Standards:**
    - **Edition:** Rust 2024.
    - **Style:** `rustfmt` (default) + `clippy` (`-D warnings`).
    - **Naming & Conventions:** See `AGENTS.md` (Section 2: Encapsulamiento del Dominio y Convenciones de Código) for comprehensive naming rules and domain patterns.
3. **Observability:** Use `tracing` macros (`info!`, `warn!`, `error!`), never `println!`. Logger initialized via `init_logger()`.
4. **Dependency Management:** Declare new dependencies in the **workspace `Cargo.toml`** and reference them with `.workspace = true` in crate-level files.

## 2. Architectural Alignment (Onion)
Strictly follow the dependency direction: **infrastructure → application → domain → core**.

### Project Structure
- `bin/clickcare/`: gRPC server entry point (tonic + axum).
- `crates/core/`: Shared traits, `ClickCareError`, base contracts.
- `crates/*/`: Domain bounded contexts (`user`, `patient`, `clinic`, etc.).

### Layer Responsibilities
| Layer | Path | Responsibility |
|---|---|---|
| **Domain** | `src/domain/` | Entities, aggregates, repository traits. |
| **Application** | `src/application/` | Use cases implementing `app_core::application::UseCase`. |
| **Infrastructure** | `src/infrastructure/` | Repository implementations, DI container, external adapters. |

## 3. Workflow & Guidelines
1. **Research:** Map dependencies and contracts before any change.
2. **Strategy:** Propose a plan summary and get approval before action.
3. **Execution:** Atomic, focused commits using [Conventional Commits](https://www.conventionalcommits.org/).
    - Types: `feat:`, `fix:`, `chore:`, `test:`, `refactor:`, `docs:`.
    - **Antigravity Mark:** Every commit produced by Antigravity MUST include the `[antigravity]` tag in the commit message.
4. **Validation:** MUST run locally before completion:
    - `cargo fmt --all`
    - `cargo clippy --workspace -- -D warnings`
    - `cargo test --workspace`
5. **Pull Requests:** Target `develop` branch. Ensure local validation passes.

## 4. Development Commands
```bash
cargo check --workspace           # Check compilation
cargo build -p clickcare          # Build gRPC server
cargo test --workspace            # Run all tests
cargo fmt --all                   # Format code
cargo clippy --workspace -- -D warnings  # Linting
```
*Note: Proto files are compiled automatically by `bin/clickcare/build.rs` during build.*

## 5. Testing Standards
- **Frameworks:** `rstest` (parameterized), `fake` (data generation), `testcontainers` (integration).
- **Mocks:** Defined in `infrastructure/di.rs` (e.g., `MockUserRepositoryImpl`) and injected via `DIOverrides`.
- **Location:**
    - **Unit tests:** `mod test` block at the bottom of the source file.
    - **Integration tests:** `tests/` folder in the relevant crate or bin.
- **Naming:** Descriptive `snake_case` (e.g., `sign_up_fails_with_invalid_user_id`).
- **Async:** Use `#[tokio::test]` + `#[rstest]`. Use `#[future(awt)]` for async fixtures.

## 6. Technical Conventions
- **API Single Source of Truth:** `proto/api.proto`. Any API change starts here.
- **Error Handling:** Use `thiserror` for domain errors; propagate with `?`.
- **Async:** Use `async-trait` for trait methods; `tokio` multi-threaded runtime.
- **Dependency Injection (DI):**
    - Inject as `Arc<dyn Trait + Send + Sync>`.
    - Never instantiate concrete types outside the `infrastructure/di.rs` module.
    - Each domain crate exposes `di::new(DBType)` and `DIOverrides`.
