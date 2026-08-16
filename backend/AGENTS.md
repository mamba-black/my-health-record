# Guías del Repositorio y Visión General del Proyecto

## Descripción del Proyecto

**My Health Record (Backend)** es un servicio de backend en Rust 2024 para la gestión de salud que sigue la **Arquitectura Cebolla (Onion Architecture)** y un **Cargo Workspace por Contextos Acotados (*Bounded Context Crates*)** estrictamente alineado con el estándar **HL7 FHIR R4**. Proporciona una API gRPC de alto rendimiento (utilizando `tonic` y `axum`) con soporte gRPC-Web.

---

## 1. Principios Arquitectónicos

### 1.1. Diseño Guiado por el Dominio (DDD) y Alineación con HL7 FHIR
- **Protección y Límites del Dominio**: Adherencia estricta a DDD para proteger el dominio y mantener los contextos acotados aislados. Las entidades del dominio se modelan siguiendo **HL7 FHIR R4**.
- **Arquitectura de Crates por Bounded Contexts (C4 Nivel 2)**:
  Estructurado en un **Cargo Workspace por Contextos Acotados** (`members = ["crates/*"]`), donde cada Bounded Context es una Crate de Rust (`crates/<dominio>`) que agrupa sus Entidades y Agregados (`src/domain/`).

```mermaid
flowchart TB
    classDef container fill:#2d3748,stroke:#4a5568,stroke-width:2px,color:#fff;
    classDef domainMod fill:#1a202c,stroke:#718096,stroke-width:1px,color:#cbd5e0;
    classDef db fill:#0d1117,stroke:#4a5568,stroke-width:1.5px,color:#cbd5e0;

    subgraph Boundaries["ClickCare - Ecosistema Ampliado FHIR (Cargo Workspace)"]
        
        subgraph Domain_User["crates/user (Identity & Security)"]
            direction TB
            UserC["<b>Auth & Account</b><br/><i>Gestión de credenciales e identidades.</i><br/>──────<br/><b>[Foundation / Security]</b><br/>• User / Auth Account<br/>• Person"]:::container
            UM_Auth["src/domain/auth_account.rs"]:::domainMod
            UM_Person["src/domain/person.rs"]:::domainMod
            UserDB[("User DB")]:::db
            UserC --- UM_Auth
            UserC --- UM_Person
            UserC --- UserDB
        end

        subgraph Domain_Administration["crates/administration (Gestión Administrativa)"]
            direction TB
            AdminC["<b>Gestión Administrativa</b><br/><i>Actores de salud, locaciones e infraestructura.</i><br/>──────<br/><b>[FHIR Individuals & Entities]</b><br/>• Patient<br/>• Practitioner<br/>• Location<br/>• HealthcareService"]:::container
            AM_Patient["src/domain/patient.rs"]:::domainMod
            AM_Practitioner["src/domain/practitioner.rs"]:::domainMod
            AM_Location["src/domain/location.rs"]:::domainMod
            AM_Service["src/domain/healthcare_service.rs"]:::domainMod
            AdminDB[("Administration DB")]:::db
            AdminC --- AM_Patient
            AdminC --- AM_Practitioner
            AdminC --- AM_Location
            AdminC --- AM_Service
            AdminC --- AdminDB
        end

        subgraph Domain_Scheduling["crates/scheduling (Reserva de Citas)"]
            direction TB
            SchedC["<b>Reserva de Citas</b><br/><i>Agendas, disponibilidad y flujo de citas.</i><br/>──────<br/><b>[FHIR Workflow]</b><br/>• Schedule<br/>• Slot<br/>• Appointment"]:::container
            SM_Schedule["src/domain/schedule.rs"]:::domainMod
            SM_Slot["src/domain/slot.rs"]:::domainMod
            SM_Appointment["src/domain/appointment.rs"]:::domainMod
            SchedDB[("Scheduling DB")]:::db
            SchedC --- SM_Schedule
            SchedC --- SM_Slot
            SchedC --- SM_Appointment
            SchedC --- SchedDB
        end

        subgraph Domain_Clinical["crates/clinical (Historia Clínica)"]
            direction TB
            ClinicalC["<b>Historia Clínica</b><br/><i>Encuentros, diagnósticos, alergias y recetas.</i><br/>──────<br/><b>[FHIR Management / Summary / Care / Forms]</b><br/>• Encounter<br/>• Condition<br/>• AllergyIntolerance<br/>• CarePlan<br/>• MedicationRequest<br/>• Questionnaire"]:::container
            CM_Encounter["src/domain/encounter.rs"]:::domainMod
            CM_Condition["src/domain/condition.rs"]:::domainMod
            CM_Allergy["src/domain/allergy_intolerance.rs"]:::domainMod
            CM_CarePlan["src/domain/care_plan.rs"]:::domainMod
            CM_MedReq["src/domain/medication_request.rs"]:::domainMod
            CM_Quest["src/domain/questionnaire.rs"]:::domainMod
            ClinicalDB[("Clinical DB")]:::db
            ClinicalC --- CM_Encounter
            ClinicalC --- CM_Condition
            ClinicalC --- CM_Allergy
            ClinicalC --- CM_CarePlan
            ClinicalC --- CM_MedReq
            ClinicalC --- CM_Quest
            ClinicalC --- ClinicalDB
        end

        subgraph Domain_Diagnostics["crates/diagnostics (Laboratorio e Imágenes)"]
            direction TB
            DiagC["<b>Laboratorio e Imágenes</b><br/><i>Órdenes clínicas, hallazgos y estudios radiológicos.</i><br/>──────<br/><b>[FHIR Diagnostics]</b><br/>• ServiceRequest<br/>• Observation<br/>• DiagnosticReport<br/>• ImagingStudy<br/>• Specimen"]:::container
            DM_ServiceReq["src/domain/service_request.rs"]:::domainMod
            DM_Observation["src/domain/observation.rs"]:::domainMod
            DM_Report["src/domain/diagnostic_report.rs"]:::domainMod
            DM_Imaging["src/domain/imaging_study.rs"]:::domainMod
            DM_Specimen["src/domain/specimen.rs"]:::domainMod
            DiagDB[("Diagnostics DB")]:::db
            DiagC --- DM_ServiceReq
            DiagC --- DM_Observation
            DiagC --- DM_Report
            DiagC --- DM_Imaging
            DiagC --- DM_Specimen
            DiagC --- DiagDB
        end

        subgraph Domain_Pharmacy["crates/pharmacy (Farmacia e Insumos)"]
            direction TB
            PharmC["<b>Farmacia e Insumos</b><br/><i>Vademécum, dispensa, aplicación y stock.</i><br/>──────<br/><b>[FHIR Medications & Supply]</b><br/>• Medication<br/>• MedicationDispense<br/>• MedicationAdministration<br/>• SupplyRequest / SupplyDelivery"]:::container
            PM_Medication["src/domain/medication.rs"]:::domainMod
            PM_Dispense["src/domain/medication_dispense.rs"]:::domainMod
            PM_Admin["src/domain/medication_administration.rs"]:::domainMod
            PM_Supply["src/domain/supply.rs"]:::domainMod
            PharmDB[("Pharmacy DB")]:::db
            PharmC --- PM_Medication
            PharmC --- PM_Dispense
            PharmC --- PM_Admin
            PharmC --- PM_Supply
            PharmC --- PharmDB
        end

        subgraph Domain_Coverage["crates/coverage (Aseguradoras y Coberturas)"]
            direction TB
            CovC["<b>Aseguradoras y Coberturas</b><br/><i>Pólizas, elegibilidad y reclamos.</i><br/>──────<br/><b>[FHIR Financial / Claims]</b><br/>• Coverage<br/>• Claim / ClaimResponse<br/>• CoverageEligibilityRequest"]:::container
            CovM_Policy["src/domain/policy.rs"]:::domainMod
            CovM_Claim["src/domain/claim.rs"]:::domainMod
            CovM_Eligibility["src/domain/eligibility.rs"]:::domainMod
            CovDB[("Coverage DB")]:::db
            CovC --- CovM_Policy
            CovC --- CovM_Claim
            CovC --- CovM_Eligibility
            CovC --- CovDB
        end

        subgraph Domain_Billing["crates/billing (Facturación)"]
            direction TB
            BillingC["<b>Facturación</b><br/><i>Cargos, comprobantes y balances de cuenta.</i><br/>──────<br/><b>[FHIR Financial]</b><br/>• Account<br/>• Invoice<br/>• ChargeItem"]:::container
            BM_Account["src/domain/account.rs"]:::domainMod
            BM_Invoice["src/domain/invoice.rs"]:::domainMod
            BM_ChargeItem["src/domain/charge_item.rs"]:::domainMod
            BillingDB[("Billing DB")]:::db
            BillingC --- BM_Account
            BillingC --- BM_Invoice
            BillingC --- BM_ChargeItem
            BillingC --- BillingDB
        end

        subgraph Domain_Legal["crates/legal_archive (Archivo Legal y Auditoría)"]
            direction TB
            LegalC["<b>Archivo Legal y Auditoría</b><br/><i>Documentación clínica consolidada y auditoría.</i><br/>──────<br/><b>[FHIR Documents & Security]</b><br/>• Composition<br/>• DocumentReference<br/>• AuditEvent<br/>• Provenance"]:::container
            LM_Composition["src/domain/composition.rs"]:::domainMod
            LM_DocRef["src/domain/document_reference.rs"]:::domainMod
            LM_AuditEvent["src/domain/audit_event.rs"]:::domainMod
            LM_Provenance["src/domain/provenance.rs"]:::domainMod
            LegalDB[("Legal Archive DB")]:::db
            LegalC --- LM_Composition
            LegalC --- LM_DocRef
            LegalC --- LM_AuditEvent
            LegalC --- LM_Provenance
            LegalC --- LegalDB
        end

        subgraph Domain_Communication["crates/communication (Notificaciones y Alertas)"]
            direction TB
            CommC["<b>Notificaciones y Alertas</b><br/><i>Mensajería, avisos y alertas clínicas.</i><br/>──────<br/><b>[FHIR Workflow / Support]</b><br/>• Communication<br/>• Flag"]:::container
            ComM_Message["src/domain/message.rs"]:::domainMod
            ComM_Flag["src/domain/flag.rs"]:::domainMod
            CommDB[("Communication DB")]:::db
            CommC --- ComM_Message
            CommC --- ComM_Flag
            CommC --- CommDB
        end

        %% Relaciones débiles (Apuntadores de Identificador UUIDv7)
        AdminC -.->|user_id| UserC
        SchedC -.->|patient_id / practitioner_id / location_id| AdminC
        ClinicalC -.->|encounter_id / appointment_id| SchedC
        ClinicalC -.->|patient_id / practitioner_id| AdminC
        DiagC -.->|patient_id| AdminC
        DiagC -.->|encounter_id| ClinicalC
        PharmC -.->|patient_id / practitioner_id| AdminC
        PharmC -.->|encounter_id / medication_request_id| ClinicalC
        CovC -.->|patient_id| AdminC
        BillingC -.->|patient_id| AdminC
        BillingC -.->|encounter_id| ClinicalC
        BillingC -.->|coverage_id / claim_id| CovC
        LegalC -.->|patient_id / practitioner_id| AdminC
        LegalC -.->|encounter_id| ClinicalC
        CommC -.->|patient_id| AdminC
    end
```

### 1.2. Decisiones de Diseño y Convenciones
- **Estrategia de Identificadores (UUIDv7)**: Claves primarias e IDs de usuario utilizan UUIDv7 (`Uuid::now_v7()`) para ordenamiento temporal optimizado en índices B-Tree de PostgreSQL.
- **Particionamiento por Hash en PostgreSQL**: Aplicado sobre la clave primaria (UUIDv7) para tablas maestras de crecimiento continuo (`Patient`, `Encounter`).
  - *Lógica de Enrutamiento*: PostgreSQL aplica una función hash criptográfica con efecto avalancha y resuelve `hash(UUIDv7) mod N_particiones = residuo`. Esto garantiza una distribución probabilística y estadísticamente uniforme entre particiones, sin sesgar la carga hacia un solo nodo pese a que los UUIDv7 son temporalmente contiguos.
  - *Transparencia en Consultas*: El motor enruta automáticamente las consultas `WHERE id = $1` calculando el hash internamente. **Nunca** hay que pasar el hash ni la partición en la cláusula SQL.
- **Criterios de Depreciación vs. Conservación Histórica**: La estrategia de particionamiento se elige según el ciclo de vida del dato, no por tamaño.
  - *Conservación Permanente (Hash sobre la clave)*: Tablas maestras e historiales normativos obligatorios (`Patient`, `Encounter`, `Composition` y el resto del archivo legal).
  - *Depreciación / Archivo por Fecha (Range Partitioning)*: Entidades transaccionales que caducan o pierden valor operativo pasado su ciclo de retención legal o financiero (`Invoice`, `AuditEvent`, `Communication`).
- **Encapsulamiento del Dominio y Convención C-GETTER**: Getters de solo lectura autogenerados vía `derive_getters::Getters` (o `bon::Builder` para construcción fluente).
- **Tipos de Datos FHIR Compartidos (`crates/core`)**: `HumanName`, `ContactPoint`, `Identifier`, `Address`, `Attachment` se definen en `app_core::domain::fhir`.
- **Arquitectura Orientada a Eventos con Apalis (`apalis-postgres`)**: `crates/user` emite `UserCreatedEvent` para sincronización asíncrona at-least-once entre Bounded Contexts.

---

### 1.3. Capas de la Arquitectura Cebolla

| Capa | Ruta | Responsabilidad | Dependencias permitidas |
|---|---|---|---|
| **Core** | `crates/core/` | Traits base (`UseCase`), error transversal (`ClickCareError`), eventos de dominio y Value Objects FHIR compartidos | Ninguna |
| **Domain** | `crates/*/src/domain/` | Entidades, agregados y traits de repositorio | `crates/core` |
| **Application** | `crates/*/src/application/` | Casos de uso implementando `app_core::application::UseCase` | `domain`, `crates/core` |
| **Infrastructure** | `crates/*/src/infrastructure/` | Repositorios, contenedor DI (`di.rs`), adaptadores externos (Apalis, gRPC clients) | `application`, `domain`, `crates/core` |
| **Entrypoint** | `bin/clickcare/` | Controladores gRPC, orquestación de workers y arranque del proceso | `crates/*` |

**Regla estructural**: las dependencias apuntan **estrictamente hacia adentro**. Una capa jamás importa de una capa exterior.

```
bin/clickcare (Punto de entrada gRPC + orquestación de workers)
  └── crates/*/infrastructure (Repositorios, adaptadores, cableado DI)
        └── crates/*/application (Casos de uso)
              └── crates/*/domain (Entidades y agregados FHIR)
                    └── crates/core (Contratos app_core)
```

Corolarios que se derivan de esta regla y se verifican en compilación:
- La capa de aplicación **no** conoce tipos de infraestructura. Un handler de evento es una función pura (`async fn(Evento) -> Result<(), ClickCareError>`); es la infraestructura quien lo registra en Apalis.
- Cada crate instancia **solo sus propias estructuras**, dentro de su `src/infrastructure/di.rs`. `bin/clickcare` orquesta ciclos de vida, no construye tipos ajenos ni declara sus dependencias (p. ej. `apalis` no figura en `bin/clickcare/Cargo.toml`).

#### Flujo de una Solicitud a través de las Capas

```mermaid
sequenceDiagram
    autonumber
    actor Client as Cliente / Frontend
    participant gRPC as Adaptador gRPC (bin/clickcare)
    participant DI as Contenedor DI (infrastructure/di.rs)
    participant UseCase as Caso de Uso (Capa de aplicación)
    participant Domain as Entidad / Dominio (Capa de dominio)
    participant Repo as Impl Repositorio (Capa de infraestructura)
    participant DB as Base de Datos PostgreSQL

    Client->>gRPC: Solicitud gRPC (ej. CreateUserRequest)
    gRPC->>DI: Resolver dependencias del Caso de Uso
    DI-->>gRPC: Instancia del UseCase (Arc<dyn Trait>)
    gRPC->>UseCase: execute(Command)
    UseCase->>Domain: User::new(UUID v7, parámetros...)
    Domain-->>UseCase: Ok(Entidad User)
    UseCase->>Repo: repository.save_user(&user)
    Repo->>DB: INSERT INTO user_account ... (SQL)
    DB-->>Repo: Éxito SQL / Filas afectadas
    Repo-->>UseCase: Ok(())
    UseCase-->>gRPC: Ok(CreateUserResponse)
    gRPC-->>Client: Respuesta gRPC (Protobuf)
```

---

### 1.4. Estructura del Cargo Workspace

El `Cargo.toml` raíz declara explícitamente sus miembros. Los contextos acotados aún no
implementados existen como **documentación de diseño** (solo `AGENTS.md`, sin `Cargo.toml`
ni código) y **no** son miembros del workspace hasta que se implementen.

```
backend/
├── Cargo.toml                      # Workspace: members declarados uno a uno
├── proto/api.proto                 # Contrato público de la API (única fuente de verdad)
├── ddl/table.sql                   # Esquema SQL (PK por defecto uuidv7())
├── docs/use_cases.md               # Casos de uso del ciclo de vida de identidad
├── bin/
│   └── clickcare/                  # Servidor ejecutable (gRPC + gRPC-Web, orquestador)
│       ├── build.rs                # Compilación de proto vía tonic-prost-build
│       ├── src/infrastructure/grpc/
│       └── tests/                  # Pruebas de integración (testcontainers)
└── crates/
    ├── core/                       # app_core — UseCase, ClickCareError, eventos, VOs FHIR
    ├── user/                       # ✅ Implementado — Identity & Security
    ├── administration/             # ✅ Implementado — Gestión Administrativa
    ├── patient/                    # 🚧 Andamiaje — Expediente de Pacientes
    ├── clinic/                     # 🚧 Andamiaje — Clínica
    ├── clinic_admin/               # 🚧 Andamiaje — Administración de Clínica
    │
    ├── scheduling/                 # 📄 Solo diseño — Reserva de Citas
    ├── clinical/                   # 📄 Solo diseño — Historia Clínica
    ├── diagnostics/                # 📄 Solo diseño — Laboratorio e Imágenes
    ├── pharmacy/                   # 📄 Solo diseño — Farmacia e Insumos
    ├── coverage/                   # 📄 Solo diseño — Aseguradoras y Coberturas
    ├── billing/                    # 📄 Solo diseño — Facturación
    ├── legal_archive/              # 📄 Solo diseño — Archivo Legal y Auditoría
    └── communication/              # 📄 Solo diseño — Notificaciones y Alertas
```

Cada crate implementado sigue la misma estructura interna: `src/domain/`, `src/application/`,
`src/infrastructure/`. Agrupar todas las entidades de un contexto acotado en un solo crate
—en lugar de un sub-crate por entidad— evita la sobreingeniería y permite `JOIN` y
transacciones nativas de base de datos dentro del mismo contexto.

---

## 2. Mapa de Documentación y Contextos Acotados

Para consultar la especificación detallada del modelo de dominio y las **reglas de dominio**
de cada contexto acotado, consulta sus guías dedicadas. Las reglas transversales viven en
este archivo (§1.3 y §3); las reglas funcionales de cada dominio viven en su crate.

Leyenda de estado: ✅ implementado · 🚧 andamiaje (miembro del workspace, sin dominio) · 📄 solo diseño (sin `Cargo.toml` ni código).

| Bounded Context | Crate | Estado | Recurso FHIR Mapeado | Documentación dedicada |
| :--- | :--- | :--- | :--- | :--- |
| **Casos de Uso e Identidad** | N/A | — | Flujos de Identidad y Registro | [docs/use_cases.md](docs/use_cases.md) |
| **Núcleo Compartido** | `crates/core` | ✅ | Value Objects FHIR, `UseCase`, `ClickCareError`, eventos | [crates/core/AGENTS.md](crates/core/AGENTS.md) |
| **Identity & Security** | `crates/user` | ✅ | `Person` + `User` | [crates/user/AGENTS.md](crates/user/AGENTS.md) |
| **Gestión Administrativa** | `crates/administration` | ✅ | `Patient`, `Practitioner`, `Organization` | [crates/administration/AGENTS.md](crates/administration/AGENTS.md) |
| **Expediente de Pacientes** | `crates/patient` | 🚧 | `Patient` (solapa con `administration`) | [crates/patient/AGENTS.md](crates/patient/AGENTS.md) |
| **Clínica** | `crates/clinic` | 🚧 | `Organization` (solapa con `administration`) | [crates/clinic/AGENTS.md](crates/clinic/AGENTS.md) |
| **Administración de Clínica** | `crates/clinic_admin` | 🚧 | Rol de administrador (solapa con `user.is_owner`) | [crates/clinic_admin/AGENTS.md](crates/clinic_admin/AGENTS.md) |
| **Reserva de Citas** | `crates/scheduling` | 📄 | `Schedule`, `Slot`, `Appointment` | [crates/scheduling/AGENTS.md](crates/scheduling/AGENTS.md) |
| **Historia Clínica** | `crates/clinical` | 📄 | `Encounter`, `Condition`, `AllergyIntolerance`, `CarePlan`, `MedicationRequest`, `Questionnaire` | [crates/clinical/AGENTS.md](crates/clinical/AGENTS.md) |
| **Laboratorio e Imágenes** | `crates/diagnostics` | 📄 | `ServiceRequest`, `Observation`, `DiagnosticReport`, `ImagingStudy`, `Specimen` | [crates/diagnostics/AGENTS.md](crates/diagnostics/AGENTS.md) |
| **Farmacia e Insumos** | `crates/pharmacy` | 📄 | `Medication`, `MedicationDispense`, `MedicationAdministration`, `SupplyRequest` / `SupplyDelivery` | [crates/pharmacy/AGENTS.md](crates/pharmacy/AGENTS.md) |
| **Aseguradoras y Coberturas** | `crates/coverage` | 📄 | `Coverage`, `Claim`, `ClaimResponse`, `CoverageEligibilityRequest` | [crates/coverage/AGENTS.md](crates/coverage/AGENTS.md) |
| **Facturación** | `crates/billing` | 📄 | `Account`, `Invoice`, `ChargeItem` | [crates/billing/AGENTS.md](crates/billing/AGENTS.md) |
| **Archivo Legal y Auditoría** | `crates/legal_archive` | 📄 | `Composition`, `DocumentReference`, `AuditEvent`, `Provenance` | [crates/legal_archive/AGENTS.md](crates/legal_archive/AGENTS.md) |
| **Notificaciones y Alertas** | `crates/communication` | 📄 | `Communication`, `CommunicationRequest`, `Flag` | [crates/communication/AGENTS.md](crates/communication/AGENTS.md) |

---

## 3. Reglas Principales y Mandatos Técnicos

1. **Inyección de Dependencias Estricta (DI)**
   - Inyectar dependencias como `Arc<dyn Trait + Send + Sync>`.
   - Nunca instanciar tipos concretos fuera de `src/infrastructure/di.rs` **ni fuera del crate dueño del tipo**: cada contexto acotado construye únicamente sus propias estructuras. Es el compilador —y no la disciplina de quien cablea— el que debe garantizar el límite.
   - El cableado de producción se resuelve **siempre** dentro de `di::new(DBType)`, decidiendo a partir del `DBType`. Está prohibido usar `DIOverrides` para seleccionar una implementación real.
   - `DIOverrides` (campos `Option<Arc<dyn Trait>>`) existe **exclusivamente** para sustituir dependencias por mocks en pruebas, vía `di::new_with_overrides`. Si una dependencia se resuelve por override en producción, la regla está rota.
   - Los tipos genéricos de una librería de infraestructura no cruzan la frontera del crate: se encapsulan detrás de un método del `DI` (p. ej. `DI::run_worker()`), nunca se devuelven al llamador.
   - **Aislamiento transaccional**: la cola de eventos y los repositorios de entidades abren pools independientes. Encolar un evento jamás comparte conexión ni transacción con la persistencia del agregado.

2. **Requerimiento Obligatorio de UUID v7**
   - Todas las Llaves Primarias e IDs de usuario **deben** ser UUID v7 (`Uuid::now_v7()`).
   - Los constructores de dominio validan el cumplimiento de UUID v7 y devuelven `ClickCareError` en formatos inválidos.

3. **Manejo de Errores y Observabilidad**
   - Utilizar `ClickCareError` (`crates/core`) para errores transversales.
   - Utilizar `thiserror` para errores específicos del dominio y propagar con `?`.
   - **Cero `unwrap()` en rutas de producción**.
   - Utilizar macros de `tracing` (`info!`, `warn!`, `error!`), **nunca** usar `println!`.

4. **Scripts y Herramientas**
   - Para automatización, despliegue o scripts auxiliares, **preferir scripts de Nushell (`.nu`)**.

5. **Protobuf API como Única Fuente de Verdad**
   - `proto/api.proto` define todos los endpoints externos. Cualquier cambio en la API debe comenzar actualizando las definiciones `.proto`.

6. **Marca de Commits Generados por Antigravity**
   - Cada vez que se realice un commit con cambios producidos por la IA (**Antigravity**), el mensaje del commit **debe incluir obligatoriamente la marca `[antigravity]`** (por ejemplo: `feat(user): [antigravity] implement user domain model` o en el cuerpo/footer del commit).

7. **Nomenclatura Clara y Descriptiva**
   - Seguir los estándares idiomáticos de Rust: `PascalCase` para tipos y traits, `snake_case` para funciones, variables y módulos, `SCREAMING_SNAKE_CASE` para constantes.
   - Los nombres de variables, funciones y campos **deben ser descriptivos y completos**. Está **estrictamente prohibido** el uso de variables de una sola letra (`e`, `f`, `u`, `r`) o de abreviaturas crípticas formadas por iniciales.
   - Aplica también a los bindings efímeros: usar `|error|`, `|user|`, `|row|` en cierres y `match`, nunca `|e|` ni `|u|`. Toda variable debe comunicar explícitamente su propósito.

8. **Patrón Builder Seguro (`bon`)**
   - Para entidades con campos internos calculados a partir de otros (p. ej. `HumanName.text`), **no derivar `Builder` directamente sobre la struct**: eso permite a un llamador externo construir el objeto saltándose las reglas de cálculo y romper el invariante.
   - En su lugar, aplicar `#[bon::bon]` sobre el bloque `impl` y exponer `#[builder] pub fn builder(...)` que **delegue en el Smart Constructor `new()`**, el único autorizado a calcular los campos derivados.
   - Los getters de solo lectura se autogeneran con `derive_getters::Getters` siguiendo la convención `C-GETTER` de las API Guidelines de Rust; los campos permanecen privados.

9. **Frontera entre DTOs de la API y Entidades de Dominio**
   - Los DTOs de `proto/api.proto` son planos y modelados para el frontend y los proveedores OAuth (`provider_avatar_url`, `id_token`, `provider_id`). Es correcto que lo sean.
   - Los Casos de Uso **deben** mapear explícitamente esos campos planos a Value Objects ricos del dominio FHIR (`Person`, `HumanName`, `ContactPoint`) al entrar a la capa de dominio.
   - **Nunca filtrar estructuras planas de DTOs dentro de entidades de dominio**, ni al revés: una entidad de dominio no se serializa tal cual hacia la API.

---

## 4. Comandos de Compilación, Pruebas y Desarrollo

```bash
# Verificar compilación en todo el workspace
cargo check --workspace

# Compilar el servidor gRPC
cargo build -p clickcare

# Ejecutar todas las pruebas (unitarias + integración)
cargo test --workspace

# Ejecutar pruebas para una crate específica (ej. user)
cargo test -p user

# Formatear código
cargo fmt --all

# Linter (Modo estricto CI)
cargo clippy --workspace -- -D warnings
```

---

## 5. Guía de Testing

- **Frameworks**: `rstest` para pruebas parametrizadas, `rstest-bdd` (+ `rstest-bdd-macros`) para escenarios BDD en Gherkin, `fake` para generar datos de prueba, `testcontainers` / `testcontainers-modules` para integración contra una base de datos real.
- **Mocks**: definir las implementaciones mock dentro de `infrastructure/di.rs` (p. ej. `MockUserRepositoryImpl`) e inyectarlas vía `DIOverrides` en `di::new_with_overrides`. Ver §3.1: los overrides son **solo** para pruebas.
- **Pruebas unitarias**: en un bloque `#[cfg(test)] mod test { ... }` al final del archivo fuente.
- **Pruebas de integración**: en `tests/` de la crate correspondiente (p. ej. `bin/clickcare/tests/`).
- **Escenarios BDD**: los `.feature` en Gherkin viven en `bin/clickcare/tests/features/`, los bindings en `tests/scenarios/` y las implementaciones de pasos en `tests/steps/`.
- **Nombres de prueba**: `snake_case` descriptivo que enuncie el escenario completo — `sign_up_fails_with_invalid_user_id`, no `test_signup`.
- **Casos parametrizados**: `#[rstest]` con anotaciones `#[case::<etiqueta>]` para que cada escenario quede identificado en la salida.
- **Pruebas asíncronas**: anotar con `#[rstest]` **y** `#[tokio::test]`; usar `#[future(awt)]` para fixtures asíncronos.
- **Estado compartido**: `tokio::sync::OnceCell` para singletons asíncronos en módulos de prueba.
- **Contenedor de Postgres**: se levanta **una sola vez por suite** vía `testcontainers` sobre Podman, con el esquema de `ddl/table.sql` copiado a `/docker-entrypoint-initdb.d/`. La exclusión mutua entre procesos de prueba se resuelve con `flock`, y el desmontaje se registra con `#[dtor]`.
- **Antes de dar por terminado un cambio**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace` deben pasar.

---

## 6. Commits y Pull Requests

- **Estilo de commit**: [Conventional Commits](https://www.conventionalcommits.org/) — `feat:`, `fix:`, `chore:`, `test:`, `refactor:`, `docs:`, con el contexto acotado como *scope*.
  ```
  feat(user): [antigravity] add UUID v7 validation to CreateUserUseCase
  fix(user): return AlreadyExists status on duplicate user sign-up
  docs(agents): restore domain rules lost in the modular refactor
  ```
- **Marca obligatoria**: todo commit con cambios producidos por la IA lleva `[antigravity]` (ver §3.6).
- **Alcance**: un commit por cambio lógico. No mezclar refactor y funcionalidad.
- **PRs**: apuntan a la rama `develop`. Incluir qué cambió y por qué, y enlazar el issue `bd` correspondiente.
- **Seguimiento de tareas**: se usa `bd` (beads) para **todo** el seguimiento. No crear listas TODO en markdown.

---

## 7. Entorno e Infraestructura Local

- **Variable requerida**: `PG_URL` (URL de conexión a PostgreSQL), cargada desde `.env` vía `dotenvy`. Si no está definida, el DI cae al valor por defecto `postgres://user:password@localhost:5432`.
- **Secretos**: **no commitear credenciales**. Usar un `.env.example` como plantilla y mantener `.env` fuera del control de versiones.
- **Servicios locales**: `devops/postgres.yaml` y `devops/tempo.yaml` levantan los servicios; `devops/simulate_telemetry.nu` genera telemetría de prueba.
- **Esquema SQL**: `ddl/table.sql` (modelo en `ddl/schema.dbml`). Las columnas de clave primaria usan `uuidv7()` como valor por defecto.
- **Cola de eventos**: el esquema `apalis` lo crean las migraciones embebidas de `apalis-postgres` al construir el DI (`PostgresStorage::setup`). No se versiona en `ddl/`.
- **Servidor**: escucha en `[::1]:50051` con soporte gRPC-Web y reflexión habilitada.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:970c3bf2 -->
## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   bd dolt push
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

