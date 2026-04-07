# Repository Guidelines

## Project Structure & Module Organization

This is a Cargo workspace following **Onion Architecture**. Business rules are isolated from infrastructure details.

```
backend/
├── Cargo.toml              # Workspace root — shared dependency versions live here
├── bin/
│   └── clickcare/          # gRPC server entry point (tonic + axum)
│       ├── build.rs        # Proto compilation via tonic-prost-build
│       ├── src/
│       │   ├── main.rs
│       │   ├── lib.rs
│       │   └── infrastructure/
│       │       └── grpc/   # gRPC service implementations (e.g. UserApiImpl)
│       └── tests/          # Integration tests
├── crates/
│   ├── core/               # app_core — shared traits, ClickCareError, base contracts
│   ├── user/               # User bounded context
│   ├── patient/            # Patient bounded context
│   ├── clinic/             # Clinic bounded context
│   └── clinic_admin/       # Clinic admin bounded context
├── ddl/                    # SQL schema definitions
├── docs/                   # Architecture diagrams and FHIR references
└── proto/                  # Protobuf definitions (api.proto)
```

Each domain crate (`user`, `patient`, etc.) is organized into three layers:

| Layer | Path | Responsibility |
|---|---|---|
| **Domain** | `crates/*/src/domain/` | Entities, aggregates, repository traits |
| **Application** | `crates/*/src/application/` | Use cases implementing `app_core::application::UseCase` |
| **Infrastructure** | `crates/*/src/infrastructure/` | Repository implementations, DI container |

Dependencies only point **inward**: infrastructure → application → domain → core.

## Build, Test, and Development Commands

```bash
# Check compilation across the entire workspace
cargo check --workspace

# Build the gRPC server
cargo build -p clickcare

# Run all tests (unit + integration)
cargo test --workspace

# Run tests for a specific crate
cargo test -p clickcare
cargo test -p user

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Build for production
cargo build --release -p clickcare
```

Proto files are compiled automatically by `bin/clickcare/build.rs` during `cargo build`.

## Coding Style & Naming Conventions

- **Formatter**: `rustfmt` with default settings. Run `cargo fmt --all` before committing.
- **Linter**: `clippy` — treat warnings as errors in CI (`-D warnings`).
- **Edition**: Rust 2024 (set in `[workspace.package]`).
- **Naming**: follow standard Rust conventions — `snake_case` for functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- **Error handling**: use `thiserror` for domain errors; propagate with `?`. Avoid `unwrap()` in production paths.
- **Async**: use `async-trait` for async trait methods; runtime is `tokio` (multi-thread).
- **Dependency injection**: inject dependencies as `Arc<dyn Trait + Send + Sync>` — never instantiate concrete types outside the `infrastructure/di.rs` module.
- **Shared dependencies**: always declare new dependencies in the **workspace `Cargo.toml`** and reference them with `.workspace = true` in crate-level `Cargo.toml` files.

## Testing Guidelines

- **Frameworks**: `rstest` for parameterized tests, `fake` for generating test data, `testcontainers` / `testcontainers-modules` for integration tests requiring a real database.
- **Mocks**: define mock implementations inside `infrastructure/di.rs` (e.g. `MockUserRepositoryImpl`) and inject them via `DIOverrides`.
- **Unit tests**: place in a `#[cfg(test)] mod test { ... }` block at the bottom of the source file.
- **Integration tests**: place in `tests/` under the relevant crate (e.g. `bin/clickcare/tests/`).
- **Test naming**: use descriptive snake_case names that state the scenario — `sign_up_fails_with_invalid_user_id`.
- **Parameterized cases**: use `#[rstest]` with `#[case::<label>]` annotations to label each scenario clearly.
- **Async tests**: annotate with both `#[rstest]` and `#[tokio::test]`; use `#[future(awt)]` for async fixtures.
- **Shared state**: use `std::sync::Once` for one-time initialization and `tokio::sync::OnceCell` for async singletons in test modules.

## Commit & Pull Request Guidelines

- **Commit style**: use [Conventional Commits](https://www.conventionalcommits.org/) — `feat:`, `fix:`, `chore:`, `test:`, `refactor:`, `docs:`.
  ```
  feat: add UUID v7 validation to CreateUserUseCase
  fix: return AlreadyExists status on duplicate user sign-up
  chore: update dependencies in Cargo.toml to latest versions
  ```
- **Scope**: keep each commit focused on a single logical change.
- **PRs**: target the `develop` branch. Include a short description of what changed and why. Link related issues where applicable.
- **Before opening a PR**: run `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` locally to ensure everything passes.

## Architecture Notes

- **Proto contract**: the public API surface is defined in `proto/api.proto`. Any change to the API must start there.
- **DI container**: each domain crate exposes a `di::new(DBType)` function and a `DIOverrides` struct to allow test-time injection without touching production wiring.
- **Environment**: configuration is loaded via `dotenvy` from a `.env` file. Do not commit secrets; use `.env.example` as a template if one is added.
- **Observability**: use the `tracing` macros (`info!`, `warn!`, `error!`) — not `println!` — in production code. Logger is initialized once at startup via `init_logger()`.
