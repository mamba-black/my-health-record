# Repository Guidelines & Project Overview

## Project Description

**My Health Record (Backend)** is a Rust 2024 backend service for healthcare management following **Onion Architecture**. It provides a high-performance gRPC API (using `tonic` and `axum`) with gRPC-Web support, handling domain bounded contexts such as users, patients, clinics, and clinic administration.

### Architectural Principles & Memory Directives
- **Domain-Driven Design (DDD)**: Strict adherence to DDD principles to protect the domain and keep bounded contexts isolated, ensuring business logic and invariants are guarded from infrastructure or external leaks. Domain boundaries and entities are modeled following **HL7 FHIR** specifications as the primary guide whenever possible. Clinical terminology and medical coding align with recognized health standards (**SNOMED CT**, **CIE-10/CIE-11** for diagnoses, **LOINC** for laboratory observations, and **DICOM** for medical imaging metadata).
- **Architecture**: Strict **Onion Architecture** (Domain isolated from infrastructure details).
- **Language & Stack**: **Rust 2024** for backend code, **Nushell** (`.nu`) for scripting and automation tasks.
- **API First**: `proto/api.proto` is the single source of truth for public API contracts.
- **Identity**: UUID v7 (`Uuid::now_v7()`) is mandatory for all user and entity primary keys.
- **User Account & Identity Composition (`User` -> `Person`)**: `User` represents the authentication / system account boundary (`id`, `active`, `person`, `provider_info`, `is_owner`). Physical human identity and demographics strictly live inside `User.person`, following **HL7 FHIR R4 Person** (`name`, `telecom`, `identifier`, `photo_url`, `birth_date`, `address`, `links`). Never flatten `Person` fields directly inside `User`.
- **Role Links via `Person.link`**: A `Person` connects to its healthcare roles via `PersonLink` targets (`Patient`, `Practitioner`, `RelatedPerson`, `Organization`). This allows a single user account to manage multiple patient profiles (e.g., parents managing children) or administer a clinic without requiring a dummy patient record.
- **FHIR Terminology & Conventions**: `HumanName` uses `given`, `family`, `second_family` (hispanic extension), and `text`. `ContactPoint` uses `system` (`Phone`, `Email`, etc.) and `use_type`. Note: FHIR `Account` refers exclusively to financial billing/coverage accounts; authentication accounts are mapped to `User` / `Person`.
- **Domain Value Object Encapsulation & Getter Conventions (Rust C-GETTER)**: Domain Value Objects and Entities keep internal fields encapsulated to enforce domain invariants. Smart constructors (`new`) pre-compute and guarantee valid fields (e.g. `text: String`). Read-only getters follow Rust API Guidelines (`C-GETTER` convention) and are auto-generated via `derive_getters::Getters` (or `bon::Builder` for fluent construction) to eliminate boilerplate while maintaining strict encapsulation. For objects with computed internal fields (e.g. `HumanName.text`), do not derive `Builder` on the struct directly; apply `#[bon::bon]` on the `impl` block with `#[builder] pub fn builder(...)` delegating to `new()`, preventing external callers from bypassing calculation rules.
- **BFF Request vs. FHIR Domain Mapping**: External API DTOs (`proto/api.proto`) keep flat, convenient fields matching frontend and OAuth provider payloads (e.g., `provider_avatar_url`, `id_token`, `provider_id`). Application Use Cases must explicitly map these flat request fields into rich FHIR domain Value Objects (`Person`, `HumanName`, `ContactPoint`, `photo_url`) upon entering the domain layer. Never leak raw DTO structures into domain entities.
- **Shared FHIR Data Types (`crates/core`)**: Core FHIR Data Types (`HumanName`, `ContactPoint`, `Identifier`, `Address`, `Attachment`) are defined in `crates/core` (`app_core::domain::fhir`) so all bounded contexts (`user`, `patient`, `clinic`, `clinic_admin`) share unified, immutable Value Objects.
- **Peruvian Healthcare & Terminology Compliance**: National identifiers in `Identifier` map to official Peruvian registries (e.g., `DNI` uses system `http://reniec.gob.pe/dni` or FHIR code `NNPER`). Diagnostic coding aligns with **CIE-10** (official MINSA) and **SNOMED CT**, laboratory observations with **LOINC**, and imaging with **DICOM**.
- **User Onboarding Strategy (Progressive Profiling)**: Single unified user registration flow with progressive data collection. Initial user creation requires minimal data (DNI optional). Document registration is required progressively only when performing specific key operations (e.g., confirming an appointment for patients or activating a clinic/emitting records for clinic admins).



---

## Project Structure & Module Organization

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

### Onion Architecture Layers

| Layer | Path | Responsibility | Dependencies |
|---|---|---|---|
| **Core** | `crates/core/` | Base traits (`UseCase`), cross-cutting error (`ClickCareError`) | None |
| **Domain** | `crates/*/src/domain/` | Entities, aggregates, domain events, repository traits | `crates/core` |
| **Application** | `crates/*/src/application/` | Business use cases implementing `app_core::application::UseCase` | `domain`, `crates/core` |
| **Infrastructure** | `crates/*/src/infrastructure/` | DB repositories, DI container (`di.rs`), external integrations | `application`, `domain`, `crates/core` |
| **gRPC Server** | `bin/clickcare/` | gRPC controllers & service entry point | `crates/*` |

Dependencies strictly point **inward**:
```
bin/clickcare (gRPC entry point)
  └── crates/*/infrastructure (DB repos, DI wiring)
        └── crates/*/application (use cases)
              └── crates/*/domain (entities & repository traits)
                    └── crates/core (app_core contracts)
```

---

## Request Flow Sequence Diagram

The following sequence diagram illustrates how a request flows through the Onion Architecture layers during execution (e.g., User Sign-Up or Patient Record creation):

```mermaid
sequenceDiagram
    autonumber
    actor Client as Client / Frontend
    participant gRPC as gRPC Adapter (bin/clickcare)
    participant DI as DI Container (infrastructure/di.rs)
    participant UseCase as Use Case (application Layer)
    participant Domain as Entity / Domain (domain Layer)
    participant Repo as Repository Impl (infrastructure Layer)
    participant DB as PostgreSQL Database

    Client->>gRPC: gRPC Request (e.g. CreateUserRequest)
    gRPC->>DI: Resolve Use Case dependencies
    DI-->>gRPC: UseCase instance (Arc<dyn Trait>)
    gRPC->>UseCase: execute(Command)
    UseCase->>Domain: User::new(UUID v7, parameters...)
    Domain-->>UseCase: Ok(User Entity)
    UseCase->>Repo: repository.save(&user)
    Repo->>DB: INSERT INTO users ... (SQL)
    DB-->>Repo: SQL Success / Affected rows
    Repo-->>UseCase: Ok(())
    UseCase-->>gRPC: Ok(CreateUserResponse)
    gRPC-->>Client: gRPC Response (Protobuf)
```

---

## Core Rules & Technical Mandates

1. **Strict Dependency Injection (DI)**
   - Inject dependencies as `Arc<dyn Trait + Send + Sync>`.
   - Never instantiate concrete types outside `src/infrastructure/di.rs`.
   - Domain crates expose `di::new(DBType)` and `DIOverrides` for test mock injection.

2. **UUID v7 Requirement**
   - All Primary Keys and User IDs **must** be UUID v7 (`Uuid::now_v7()`).
   - Domain constructors validate UUID v7 compliance and return `ClickCareError` on invalid formats.

3. **Error Handling & Observability**
   - Use `ClickCareError` (`crates/core`) for cross-cutting errors.
   - Use `thiserror` for domain-specific errors and propagate with `?`.
   - **Zero `unwrap()` in production paths**.
   - Use `tracing` macros (`info!`, `warn!`, `error!`), **never** use `println!`.

4. **Scripts & Tooling**
   - For automation, deployment, or helper scripts, **prefer Nushell scripts (`.nu`)**.

5. **Protobuf API Single Source of Truth**
   - `proto/api.proto` defines all external endpoints. Any API change must begin by updating `.proto` definitions.

---

## Build, Test, and Development Commands

```bash
# Check compilation across the entire workspace
cargo check --workspace

# Build the gRPC server
cargo build -p clickcare

# Run all tests (unit + integration)
cargo test --workspace

# Run tests for a specific crate
cargo test -p user

# Format code
cargo fmt --all

# Linting (CI strict mode)
cargo clippy --workspace -- -D warnings
```

