# Repository Guidelines & Project Overview

## Project Description

**My Health Record (Backend)** is a Rust 2024 backend service for healthcare management following **Onion Architecture**. It provides a high-performance gRPC API (using `tonic` and `axum`) with gRPC-Web support, handling domain bounded contexts such as users, patients, clinics, and clinic administration.

### Architectural Principles & Memory Directives

#### 1. Domain-Driven Design (DDD) & HL7 FHIR Alignment
- **Domain Boundaries & Protection**: Strict adherence to DDD principles to protect the domain and keep bounded contexts isolated, ensuring business logic and invariants are guarded from infrastructure or external leaks. Domain boundaries and entities are modeled following **HL7 FHIR** specifications as the primary guide whenever possible.
- **Bounded Contexts & FHIR Resource Mapping**:
  | Crate / Bounded Context | Primary FHIR Resource | Responsibility & Domain Scope |
  |---|---|---|
  | **`crates/user`** | **`Person`** + `User` | **System Account & Physical Identity**: `User` manages system auth (`id`, `active`, `provider_info`, `is_owner`). Physical human identity lives strictly in **FHIR R4 `Person`** (`name`, `telecom`, `identifier`, `birth_date`, `photo_url`, `address`, `links`). |
  | **`crates/patient`** | **`Patient`** | **Clinical Record**: Patient health record, emergency contacts, primary practitioner (`generalPractitioner`), and care histories. |
  | **`crates/clinic`** | **`Organization`** / **`Location`** | **Clinic & Physical Facilities**: `Organization` represents legal entity (RUC, legal name, billing). `Location` represents physical branches, consult rooms, or care areas. |
  | **`crates/clinic_admin`** | **`Practitioner`** / **`PractitionerRole`** | **Health Practitioners & Admin**: `Practitioner` stores medical credentials (CMP/COP, specialty). `PractitionerRole` maps doctor roles, schedules, and clinic associations. |
  | **`crates/core`** | FHIR Data Types | Shared Value Objects (`HumanName`, `ContactPoint`, `Identifier`, `Address`). |
- **User Account & Identity Composition (`User` -> `Person`)**: `User` represents the authentication / system account boundary (`id`, `active`, `person`, `provider_info`, `is_owner`). Physical human identity and demographics strictly live inside `User.person`, following **HL7 FHIR R4 Person** (`name`, `telecom`, `identifier`, `photo_url`, `birth_date`, `address`, `links`). Never flatten `Person` fields directly inside `User`.
- **Role Links via `Person.link`**: A `Person` connects to its healthcare roles via `PersonLink` targets (`Patient`, `Practitioner`, `RelatedPerson`, `Organization`). This allows a single user account to manage multiple patient profiles (e.g., parents managing children) or administer a clinic without requiring a dummy patient record.
- **FHIR Terminology & Conventions**: `HumanName` uses `given`, `family`, `second_family` (hispanic extension), and `text`. `ContactPoint` uses `system` (`Phone`, `Email`, etc.) and `use_type`. Note: FHIR `Account` refers exclusively to financial billing/coverage accounts; authentication accounts are mapped to `User` / `Person`.

#### 2. Domain Encapsulation & Code Conventions
- **Domain Value Object Encapsulation & Getter Conventions (Rust C-GETTER)**: Domain Value Objects and Entities keep internal fields encapsulated to enforce domain invariants. Smart constructors (`new`) pre-compute and guarantee valid fields (e.g. `text: String`). Read-only getters follow Rust API Guidelines (`C-GETTER` convention) and are auto-generated via `derive_getters::Getters` (or `bon::Builder` for fluent construction) to eliminate boilerplate while maintaining strict encapsulation.
- **Safe Builder Pattern (`bon`)**: For objects with computed internal fields (e.g. `HumanName.text`), do not derive `Builder` on the struct directly; apply `#[bon::bon]` on the `impl` block with `#[builder] pub fn builder(...)` delegating to `new()`, preventing external callers from bypassing calculation rules.

#### 3. Data Mapping & Interoperability Compliance
- **BFF Request vs. FHIR Domain Mapping**: External API DTOs (`proto/api.proto`) keep flat, convenient fields matching frontend and OAuth provider payloads (e.g., `provider_avatar_url`, `id_token`, `provider_id`). Application Use Cases must explicitly map these flat request fields into rich FHIR domain Value Objects (`Person`, `HumanName`, `ContactPoint`, `photo_url`) upon entering the domain layer. Never leak raw DTO structures into domain entities.
- **Shared FHIR Data Types (`crates/core`)**: Core FHIR Data Types (`HumanName`, `ContactPoint`, `Identifier`, `Address`, `Attachment`) are defined in `crates/core` (`app_core::domain::fhir`) so all bounded contexts (`user`, `patient`, `clinic`, `clinic_admin`) share unified, immutable Value Objects.
- **Peruvian Healthcare & Terminology Compliance**: National identifiers in `Identifier` map to official Peruvian registries (e.g., `DNI` uses system `http://reniec.gob.pe/dni` or FHIR code `NNPER`). Diagnostic coding aligns with **CIE-10** (official MINSA) and **SNOMED CT**, laboratory observations with **LOINC**, and imaging with **DICOM**.

#### 4. Business & Onboarding Strategy
- **User Onboarding Strategy (Progressive Profiling)**: Single unified user registration flow with progressive data collection. Initial user creation requires minimal data (DNI optional). National identity document registration (`Identifier`) is enforced progressively depending on role-based operational triggers:

  | Rol del Usuario | ¿DNI al registrarse? | Disparador Mandatorio de DNI | Razón de Negocio / Legal (Perú) |
  |---|---|---|---|
  | **Paciente (`Patient`)** | ❌ Opcional | Al **confirmar su primera cita médica** o emitir una receta / atención. | Ley N° 30024 RNHCE / MINSA (asociación de Historia Clínica a persona real). |
  | **Administrador de Clínica (`Clinic Admin`)** | ❌ Opcional | Al **activar/crear la Clínica (`Organization`)** o configurar facturación/RUC. | Verificación de identidad legal del representante de la clínica. |
  | **Profesional de Salud (`Practitioner`)** | ❌ Opcional | Al **activar perfil médico**, habilitar agenda o **firmar atenciones/recetas**. | Verificación de identidad + Colegiatura (CMP/COP) para emitir actos médicos. |

- **Uniqueness & Identity Invariants**:
  - `User.email`: Strictly unique per system account (Primary authentication credential).
  - `User.person.identifier` (DNI): Unique per primary `User` account to ensure a single Electronic Health Record (EHR / Ley N° 30024) per physical citizen.
  - `ContactPoint` (Phone): Non-strict / Shared uniqueness (allows family members or parents managing dependents to share home/contact numbers).

- **Account Recovery, Re-binding & Presencial Verification**:
  - **Lost Email / Phone Recovery**: When a user registers a new account with a DNI that is already bound to an existing identity whose credentials were lost:
    1. A new `User` account is created with `PersonLink` in a `Pending Verification` state (`LinkAssuranceLevel::Level1`).
    2. **Appointment Booking is 100% CONFIRMED** (never tentative; medical slot is fully guaranteed for the patient).
    3. **Presencial Check-in & Approval**: On the appointment date, during physical receptionist check-in, the receptionist verifies the physical DNI card, completing the check-in and elevating `LinkAssuranceLevel` to verified (`Level3`/`Level4`), unlocking past medical history access in the app seamlessly.

- **API Onboarding Response Statuses (`proto/api.proto`)**:
  - `SignUpStatus::SUCCESS`: Account created cleanly. Response message: `"Usuario registrado exitosamente."`
  - `SignUpStatus::LINK_PENDING_PRESENCIAL_VERIFICATION`: Account created with explicit confirmation (`confirm_pending_presencial_link = true`); prior DNI history detected. `PersonLink` set to `Level1` (Pending). Response message: `"Cuenta creada exitosamente. Se detectó una Historia Clínica asociada a tu DNI. La vinculación final se completará durante tu verificación presencial en tu próxima cita médica."`
  - `DNI_ALREADY_VERIFIED_CONFLICT` (`gRPC Status: ALREADY_EXISTS`): Returned on initial registration attempt when DNI already exists and `confirm_pending_presencial_link` is `false`/`None`. Response message: `"El DNI ingresado ya está asociado a una cuenta. ¿Deseas iniciar sesión o solicitar la vinculación presencial en tu próxima cita médica?"`

---

### 4.1. Account & Identity Lifecycle Use Cases

#### Use Case 1: Progressive Onboarding & Confirmed Booking
User signs up with minimal info (Google OIDC/Email). When booking an appointment, DNI & Phone become mandatory. The appointment is **100% CONFIRMED** in the clinic schedule.

```mermaid
sequenceDiagram
    autonumber
    actor Patient as Patient / App
    participant App as Mobile/Web App
    participant UserDomain as crates/user
    participant ClinicDomain as crates/clinic
    participant DB as PostgreSQL DB

    Patient->>App: 1. Sign Up (Email / OIDC)
    App->>UserDomain: Create User (Minimal Person)
    UserDomain->>DB: Save User (active=true, DNI=None)
    
    Patient->>App: 2. Book Appointment
    App->>Patient: Prompt DNI & Phone (Mandatory per Ley 30024)
    Patient->>App: Submits DNI & Phone
    App->>UserDomain: Update Person (identifier=DNI, telecom=Phone)
    App->>ClinicDomain: Book Appointment (Doctor, Slot)
    ClinicDomain-->>App: Appointment Status: CONFIRMED
    App-->>Patient: Display Booking Confirmation
```

#### Use Case 2: Lost Credentials Recovery & Presencial Re-binding
User lost access to email/phone. Registers a new account with their DNI. Account is created with `LinkAssuranceLevel::Level1` (Pending). Appointment is **100% CONFIRMED**. On appointment day, Receptionist verifies physical DNI at check-in, promoting assurance to `Level3`/`Level4` (Verified).

```mermaid
sequenceDiagram
    autonumber
    actor Patient as Patient
    actor Receptionist as Clinic Receptionist
    participant App as App / System
    participant UserDomain as crates/user
    participant AdminDomain as crates/clinic_admin
    participant DB as PostgreSQL DB

    Patient->>App: Sign Up with New Email + Existing DNI
    App->>UserDomain: Detect Existing DNI History
    UserDomain->>UserDomain: Create User + PersonLink (Assurance: Level1 Pending)
    App->>App: Book Appointment (Status: CONFIRMED)
    
    Note over Patient, Receptionist: Appointment Day (Presencial Check-in)
    Patient->>Receptionist: Arrives at Clinic & Present Physical DNI
    Receptionist->>AdminDomain: Perform Check-in (DNI 10000001)
    AdminDomain->>AdminDomain: Detect Pending Link Request (carlos_nuevo@gmail.com)
    Receptionist->>AdminDomain: Verify Physical DNI & Click "Approve Link"
    AdminDomain->>UserDomain: Elevate PersonLink Assurance (Level3/Level4 Verified)
    UserDomain->>DB: Update Link Assurance & Deactivate Old User
    AdminDomain-->>Patient: Patient Checked-in & App Fully Linked
```

#### Use Case 3: DNI Mistake Correction & Family Profile Conversion
User accidentally registered a family member's DNI (e.g., child or parent) on their main account. User converts the family member's DNI to a managed `Patient` profile (`PersonLinkTarget::Patient`) and sets their own DNI on the main account.

```mermaid
sequenceDiagram
    autonumber
    actor User as Account Owner
    participant App as Mobile App
    participant UserDomain as crates/user
    participant PatientDomain as crates/patient
    participant DB as PostgreSQL DB

    User->>App: Select "Correct DNI / Move DNI to Dependent"
    App->>UserDomain: Initiate Profile Conversion (DNI 77777777)
    UserDomain->>PatientDomain: Create Dependent Patient Profile (DNI 77777777)
    PatientDomain->>DB: Save Patient (Managed by Owner)
    UserDomain->>UserDomain: Add PersonLinkTarget::Patient(dependent_id)
    
    User->>App: Enter Owner's Real DNI (10000001)
    App->>UserDomain: Update User.person.identifier = DNI 10000001
    UserDomain->>DB: Save Owner Identity
    App-->>User: Profile Reorganized (Owner + Dependent Patient)
```

#### Use Case 4: Pre-check Query by DNI & Dynamic Options
App queries API before or during registration. Backend checks DNI existence and status to guide UX choices.

```mermaid
sequenceDiagram
    autonumber
    actor Client as App / Client
    participant API as UserApi (gRPC)
    participant UserDomain as crates/user
    participant DB as PostgreSQL DB

    Client->>API: Query DNI Status (DNI 10000001)
    API->>UserDomain: Check DNI Existence
    UserDomain->>DB: SELECT FROM users/persons WHERE identifier = DNI 10000001
    
    alt DNI Not Found
        DB-->>UserDomain: Not Found
        UserDomain-->>API: Available
        API-->>Client: Status: OK (DNI Available for normal registration)
    else DNI Exists on Active Account (Level3/Level4 Verified)
        DB-->>UserDomain: Active User (carlos@gmail.com)
        UserDomain-->>API: Conflict (Verified Account)
        API-->>Client: Status: ALREADY_EXISTS (Prompt User to Sign In / Recover Account)
    else DNI Exists on Clinical History / Unverified Account
        DB-->>UserDomain: Patient History Found
        UserDomain-->>API: History Found (Link Available)
        API-->>Client: Status: LINK_AVAILABLE (Prompt User to Request Presencial Link)
    end
```

#### Use Case 5: Dependent Profile Independence (Child Turns 18)
A managed dependent child (`Patient` linked to parent's account) registers their own autonomous `User` account with their email and DNI.

```mermaid
sequenceDiagram
    autonumber
    actor Child as Dependent (Now Adult)
    participant App as Mobile App
    participant UserDomain as crates/user
    participant AdminDomain as crates/clinic_admin
    participant DB as PostgreSQL DB

    Child->>App: Register Autonomous Account (email + DNI 77777777)
    App->>UserDomain: SignUpRequest (confirm_pending_presencial_link=true)
    UserDomain->>UserDomain: Create User + PersonLink (Assurance: Level1 Pending)
    UserDomain->>DB: Save User Account
    App-->>Child: Display Pending Verification Notice

    Note over Child, AdminDomain: Presencial Appointment at Clinic
    Child->>AdminDomain: Present Physical DNI at Check-in
    AdminDomain->>UserDomain: Approve Link & Transfer Record (Assurance: Level3/Level4)
    UserDomain->>DB: Update Link & Unlink Managed Dependent Status
    AdminDomain-->>Child: App Unlocked & History Fully Independent
```

#### Use Case 6: Profile Data Updates & Assurance Level Controls
User attempts to update identity fields (Name, DNI, Telecom). Updates are allowed freely for unverified accounts (`Level1`) and restricted/audit-logged for verified accounts (`Level3`/`Level4`).

```mermaid
sequenceDiagram
    autonumber
    actor User as User / Patient
    participant App as Mobile App
    participant UserDomain as crates/user
    participant DB as PostgreSQL DB

    User->>App: Edit Profile Request (Name / DNI)
    App->>UserDomain: UpdateProfile(UserCommand)
    
    alt Assurance is Level1 (Unverified / Pending)
        UserDomain->>UserDomain: Update Person Identifier / Name
        UserDomain->>DB: Save Updated Person
        UserDomain-->>App: Success (Profile Updated)
    else Assurance is Level3/Level4 (Verified in Clinic)
        UserDomain-->>App: Error / Restricted (DNI edit requires in-clinic approval)
        App-->>User: Display Notice ("Contact Reception to update verified DNI")
    end
```

---

#### 5. General Tech Stack Directives
- **Architecture**: Strict **Onion Architecture** (Domain isolated from infrastructure details).
- **Language & Stack**: **Rust 2024** for backend code, **Nushell** (`.nu`) for scripting and automation tasks.
- **API First**: `proto/api.proto` is the single source of truth for public API contracts.
- **Identity**: UUID v7 (`Uuid::now_v7()`) is mandatory for all user and entity primary keys.

---

### Architecture & FHIR Identity Composition

The diagram below illustrates how system authentication (`User`) composes physical identity (`Person`) and links to healthcare role resources (`Patient`, `Practitioner`, `Organization`):

```mermaid
graph TD
    subgraph auth_boundary["System Auth Boundary"]
        User["User (System Account)<br/>id: UUID v7, active, provider_info"]
    end

    subgraph physical_identity["Physical Identity (FHIR R4 Person)"]
        Person["Person<br/>name: HumanName<br/>telecom: ContactPoint[]<br/>identifier: Identifier (DNI/CE)<br/>links: PersonLink[]"]
    end

    subgraph healthcare_roles["Healthcare Roles (FHIR Resources)"]
        Patient["Patient (crates/patient)<br/>Clinical Record"]
        Practitioner["Practitioner (crates/clinic_admin)<br/>Medical License (CMP/COP)"]
        Organization["Organization (crates/clinic)<br/>Clinic / Legal Entity"]
        RelatedPerson["RelatedPerson<br/>Tutor / Guardian"]
    end

    User -->|1:1 Composition| Person
    Person -->|Person.link| Patient
    Person -->|Person.link| Practitioner
    Person -->|Person.link| Organization
    Person -->|Person.link| RelatedPerson
```

---

### Domain Model Class Diagram

The following class diagram visualizes the domain entities, value objects, and relationships following HL7 FHIR and DDD:

```mermaid
classDiagram
    class User {
        +Uuid id
        +bool active
        +Person person
        +IdentityProvider provider_info
        +bool is_owner
        +new(...) Result~User, ClickCareError~
        +add_link(target, assurance)
        +patient_ids() Vec~Uuid~
        +organization_ids() Vec~Uuid~
    }

    class Person {
        +Uuid id
        +HumanName name
        +Vec~ContactPoint~ telecom
        +Option~Identifier~ identifier
        +Option~String~ photo_url
        +Option~NaiveDate~ birth_date
        +Option~String~ address
        +Vec~PersonLink~ links
        +add_link(target, assurance)
        +patient_ids() Vec~Uuid~
        +organization_ids() Vec~Uuid~
    }

    class HumanName {
        -Vec~String~ given
        -String family
        -Option~String~ second_family
        -String text
        +new(given, family, second_family) HumanName
        +builder() HumanNameBuilder
        +given() Vec~String~
        +family() String
        +second_family() Option~String~
        +text() String
    }

    class ContactPoint {
        +ContactPointSystem system
        +String value
        +Option~ContactPointUse~ use_type
        +email(value) ContactPoint
        +phone(value, use_type) ContactPoint
    }

    class ContactPointSystem {
        <<enumeration>>
        Phone
        Email
        Fax
        Url
    }

    class ContactPointUse {
        <<enumeration>>
        Home
        Work
        Mobile
        Temp
        Old
    }

    class Identifier {
        +IdentifierType doc_type
        +String value
        +Option~String~ system
        +dni(value) Identifier
    }

    class IdentityProvider {
        <<enumeration>>
        Google
    }

    class PersonLink {
        +PersonLinkTarget target
        +Option~LinkAssuranceLevel~ assurance
    }

    class PersonLinkTarget {
        <<enumeration>>
        Patient(Uuid)
        Practitioner(Uuid)
        RelatedPerson(Uuid)
        Organization(Uuid)
    }

    class LinkAssuranceLevel {
        <<enumeration>>
        Level1
        Level2
        Level3
        Level4
    }

    User "1" *-- "1" Person : contains (User -> Person)
    User "1" *-- "1" IdentityProvider : authenticated by
    Person "1" *-- "1" HumanName : named by
    Person "1" *-- "0..*" ContactPoint : reached via
    Person "1" *-- "0..1" Identifier : identified by
    Person "1" *-- "0..*" PersonLink : links to
    ContactPoint "1" *-- "1" ContactPointSystem : system
    ContactPoint "0..1" *-- "1" ContactPointUse : use
    PersonLink "1" *-- "1" PersonLinkTarget : targets
    PersonLink "0..1" *-- "1" LinkAssuranceLevel : assurance
```



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

