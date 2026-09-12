# ESPECIFICACIÓN TÉCNICA DE ARQUITECTURA

| Metadato | Detalle |
| :--- | :--- |
| **Documento** | RFC-004: Aislamiento Multi-Inquilino, Persistencia Particionada y Aprovisionamiento Reactivo de Organizaciones |
| **Estado** | Aprobado / Especificación Normativa |
| **Alcance** | `crates/user`, `crates/clinic`, `crates/scheduling`, `crates/clinical`, Capa de Infraestructura |
| **Motor de Persistencia** | PostgreSQL 16+ (Particionamiento Declarativo por Hash) |
| **Estándares** | HL7 FHIR R4 (`Person`, `Organization`), Ley N° 30024 (RNHCE - MINSA), Arquitectura Cebolla |

---

### 1. Objeto y Alcance del Diseño

La presente especificación formaliza el ciclo de vida, aprovisionamiento e infraestructura multi-inquilino (*multi-tenant*) del sistema. Se adopta un modelo reactivo guiado por eventos del dominio con **particionamiento declarativo por Hash** en PostgreSQL.

Este diseño elimina la ejecución de sentencias DDL en tiempo de ejecución (`CREATE SCHEMA` o `CREATE TABLE` dinámicos), suprime tiempos de espera artificiales en la experiencia de usuario y garantiza que la creación técnica de una clínica se complete en milisegundos mediante un registro esquelético mínimo, difiriendo la captura de información fiscal y regulatoria al primer inicio de sesión.

---

### 2. Invariantes Arquitectónicas y Reglas de Persistencia

#### 2.1. Creación Reactiva Mínima por Evento

* **Disparo**: Al completarse el registro de un usuario con intención de fundador en `crates/user`, se emite el evento de dominio `FounderRegistered`.
* **Reacción en `crates/clinic`**: Un manejador de eventos (*Event Consumer*) inicializa de forma asíncrona in-process la organización clínica, generando un identificador UUID v7 e insertando un registro en la tabla particionada `organizations` con `status = 'PENDING_SETUP'`.
* **Latencia Operacional**: La persistencia del esqueleto toma $\approx 2\text{ ms}$, prescindiendo de colas de espera pesadas o pantallas de bloqueo en el cliente.
* **Cero Datos Fiscales Preliminares**: El evento no requiere ni procesa RUC, razón social, dirección fiscal ni licencias operativas.

#### 2.2. Ubicuidad de la Clave de Partición (`organization_id`)

* Toda tabla que almacene datos transaccionales, clínicos o administrativos (`appointments`, `encounters`, `prescriptions`, `invoices`) debe declarar de manera mandatoria la columna `organization_id UUID NOT NULL` indexada y prefijada.
* Queda terminantemente prohibido resolver la pertenencia a una clínica mediante recorridos transitivos entre tablas.

#### 2.3. Claves Primarias Compuestas para Poda de Particiones (*Partition Pruning*)

* Las tablas particionadas por hash sobre `organization_id` definen su clave primaria como:
  $$\text{PRIMARY KEY } (organization\_id, id)$$
* Esto asegura unicidad estricta y garantiza que el planificador de PostgreSQL descarte las cubetas no coincidentes de forma determinista.

---

### 3. Modelo DDL Canónico (Particionamiento por Hash)

Las cubetas físicas se configuran en el despliegue inicial de infraestructura; ninguna migración DDL ocurre durante la interacción del usuario:

```sql
-- 1. Tabla de Organizaciones (FHIR Organization)
CREATE TABLE organizations (
    id              UUID NOT NULL DEFAULT gen_random_uuid(),
    status          VARCHAR(32) NOT NULL DEFAULT 'PENDING_SETUP', -- PENDING_SETUP, ACTIVE, SUSPENDED, MAINTENANCE
    tax_id          VARCHAR(11),                                  -- RUC (11 dígitos - SUNAT)
    legal_name      VARCHAR(255),
    trade_name      VARCHAR(255),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),

    CONSTRAINT pk_organizations PRIMARY KEY (id),
    CONSTRAINT uq_organizations_tax_id UNIQUE (tax_id)
);

-- 2. Tabla Transaccional Particionada (Ejemplo: Citas Médicas)
CREATE TABLE scheduling_appointments (
    organization_id UUID NOT NULL,
    id              UUID NOT NULL DEFAULT gen_random_uuid(),
    patient_id      UUID NOT NULL,
    practitioner_id UUID NOT NULL,
    start_time      TIMESTAMPTZ NOT NULL,
    end_time        TIMESTAMPTZ NOT NULL,
    status          VARCHAR(32) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),

    CONSTRAINT pk_scheduling_appointments PRIMARY KEY (organization_id, id)
) PARTITION BY HASH (organization_id);

-- 3. Pre-aprovisionamiento de Cubetas Fijas (Módulo 8)
CREATE TABLE scheduling_appointments_p0 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 0);
CREATE TABLE scheduling_appointments_p1 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 1);
CREATE TABLE scheduling_appointments_p2 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 2);
CREATE TABLE scheduling_appointments_p3 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 3);
CREATE TABLE scheduling_appointments_p4 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 4);
CREATE TABLE scheduling_appointments_p5 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 5);
CREATE TABLE scheduling_appointments_p6 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 6);
CREATE TABLE scheduling_appointments_p7 PARTITION OF scheduling_appointments FOR VALUES WITH (MODULUS 8, REMAINDER 7);

-- 4. Índice de Cobertura para Poda
CREATE INDEX idx_appointments_lookup
    ON scheduling_appointments (organization_id, practitioner_id, start_time);
```

---

### 4. Ciclo de Vida: Registro Mínimo por Evento y Onboarding Progresivo

El aprovisionamiento se articula en dos fases disjuntas: el nacimiento reactivo de la entidad y la posterior formalización regulatoria ante MINSA/SUNAT al iniciar sesión.

```mermaid
sequenceDiagram
    autonumber
    actor Founder as Usuario Fundador
    participant UI as Frontend / Portal Web
    participant Gateway as Servidor gRPC (bin/clickcare)
    participant UserCtx as Bounded Context: User (crates/user)
    participant ClinicCtx as Bounded Context: Clinic (crates/clinic)
    participant DB as PostgreSQL (Particionado)

    %% Fase 1: Registro Mínimo y Disparo Reactivo
    rect rgb(240, 245, 255)
    note right of Founder: Fase 1: Registro Mínimo y Creación Asíncrona del Esqueleto
    Founder->>UI: Registro inicial (Email, Password, Nombre)
    UI->>Gateway: gRPC: SignUpFounder(SignUpFounderRequest)
    Gateway->>UserCtx: Execute(CreateFounderUserCommand)
    UserCtx->>DB: INSERT INTO users, persons (Assurance: Level1)
    DB-->>UserCtx: Registros persistidos (UUID v7)

    %% Emisión de Evento de Dominio
    UserCtx-)ClinicCtx: Publicar Evento: FounderRegistered(user_id, person_id)
    UserCtx-->>Gateway: Ok(FounderRegisteredResponse)
    Gateway-->>UI: JWT Base emitido (Claims: user_id, tenant_id = null)

    %% Reacción en Segundo Plano (Sin bloqueo al usuario)
    ClinicCtx->>ClinicCtx: Generar UUID v7 (organization_id)
    ClinicCtx->>DB: INSERT INTO organizations (id, status='PENDING_SETUP')
    ClinicCtx->>DB: INSERT INTO organization_memberships (user_id, organization_id, role='OWNER')
    DB-->>ClinicCtx: Registro confirmado (~2ms)
    end

    %% Fase 2: Primer Acceso e Identificación de Estado
    rect rgb(245, 255, 245)
    note right of Founder: Fase 2: Autenticación Inicial y Detección de Estado
    Founder->>UI: Inicia sesión / Accede al sistema
    UI->>Gateway: gRPC: GetMyWorkspaceStatus()
    Gateway->>ClinicCtx: GetOrganizationByUserId(user_id)
    ClinicCtx->>DB: SELECT id, status FROM organizations WHERE ...
    DB-->>ClinicCtx: Organization(id, status='PENDING_SETUP')
    ClinicCtx-->>Gateway: WorkspaceStatusResponse(status='PENDING_SETUP', org_id)
    Gateway-->>UI: Redirección forzosa al Formulario de Onboarding Legal
    end

    %% Fase 3: Formalización Legal y Regulatoria
    rect rgb(255, 250, 240)
    note right of Founder: Fase 3: Onboarding Progresivo (RUC, SUNAT y Ley 30024)
    Founder->>UI: Ingresa RUC, Razón Social y DNI del Representante
    UI->>Gateway: gRPC: CompleteOrganizationSetup(CompleteSetupRequest)

    %% Verificación de Identidad del Representante Legal (Ley 30024)
    Gateway->>UserCtx: ElevateAssurance(user_id, DNI, Level3)
    UserCtx->>DB: UPDATE persons SET identifier = DNI WHERE id = person_id
    DB-->>UserCtx: Identidad verificada

    %% Activación de la Organización Clínica
    Gateway->>ClinicCtx: Execute(ActivateOrganizationCommand)
    ClinicCtx->>ClinicCtx: Validar estructura RUC (Módulo 11)
    ClinicCtx->>DB: UPDATE organizations SET tax_id=RUC, legal_name=..., status='ACTIVE'
    DB-->>ClinicCtx: Estado actualizado
    ClinicCtx-->>Gateway: Ok(OrganizationActivated)
    end

    %% Fase 4: Contexto de Operación Pleno
    rect rgb(240, 255, 255)
    note right of UI: Fase 4: Sincronización de Contexto Operativo
    Gateway-->>UI: Nuevo JWT (Claims: user_id, organization_id, role='OWNER', status='ACTIVE')
    UI-->>Founder: Redirección automática a /app/dashboard-clinica
    end
```

---

### 5. Máquina de Estados de la Organización

```mermaid
stateDiagram-v2
    [*] --> PENDING_SETUP : Evento FounderRegistered (~2ms)

    state PENDING_SETUP {
        [*] --> RequiereDatosLegales
        RequiereDatosLegales --> ValidandoRucYDni : CompleteOrganizationSetup()
    }

    PENDING_SETUP --> ACTIVE : Validación Exitosa (RUC SUNAT + DNI MINSA)
    PENDING_SETUP --> ABANDONED : Expiración TTL (Janitor Worker tras 14 días)

    state ACTIVE {
        [*] --> Operativo
        Operativo --> MAINTENANCE : Mantenimiento / Migración VIP
        MAINTENANCE --> Operativo : Migración Finalizada
    }

    ACTIVE --> SUSPENDED : Incumplimiento Tributario / Pago
    SUSPENDED --> ACTIVE : Reactivación
    ABANDONED --> [*] : DROP / Purga lógica
```

* **`PENDING_SETUP`**: Estado inicial esquelético. La entidad existe a nivel relacional, pero tiene prohibido agendar citas (`crates/scheduling`), admitir pacientes (`crates/patient`) o emitir recetas (`crates/clinical`).
* **`ACTIVE`**: Organización validada con RUC formal y DNI del representante legal verificado conforme a la Ley N° 30024. Acceso operativo irrestricto.
* **`MAINTENANCE`**: Bloqueo selectivo de peticiones mutables (`POST`, `PUT`, `DELETE` retornan `gRPC: UNAVAILABLE` / HTTP 503) durante ventanas de migración o exportación hacia infraestructura dedicada.

---

### 6. Protocolo de Extracción Hacia Base de Datos Dedicada

Cuando un cliente requiere cómputo aislado o soberanía de datos estricta, la presencia ubicua de `organization_id` permite realizar la extracción sin transformaciones intermedias ni reconstrucción de árboles de relaciones:

```mermaid
sequenceDiagram
    autonumber
    actor Ops as Ingeniero de Infraestructura / SRE
    participant Gateway as Servidor gRPC (Router de Conexión)
    participant SharedDB as PostgreSQL (Instancia Compartida)
    participant DedicatedDB as PostgreSQL (Instancia Dedicada)

    Note over Ops, Gateway: 1. Aislamiento Transaccional
    Ops->>SharedDB: UPDATE organizations SET status = 'MAINTENANCE' WHERE id = 'uuid-org'
    Gateway->>Gateway: Rechazar escrituras para 'uuid-org' (Fallback solo lectura o 503)

    Note over Ops, DedicatedDB: 2. Extracción Streaming (Nushell / Binary Pipeline)
    Ops->>DedicatedDB: Aplicar DDL consolidado (Tablas idénticas)
    Ops->>SharedDB: COPY (SELECT * FROM organizations WHERE id = 'uuid-org') TO STDOUT BINARY
    SharedDB-->>DedicatedDB: COPY organizations FROM STDIN BINARY
    Ops->>SharedDB: COPY (SELECT * FROM scheduling_appointments WHERE organization_id = 'uuid-org') TO STDOUT BINARY
    SharedDB-->>DedicatedDB: COPY scheduling_appointments FROM STDIN BINARY
    Note over DedicatedDB: Iterar copia sobre el resto de tablas del dominio

    Note over Ops, SharedDB: 3. Purga en Instancia Compartida
    Ops->>SharedDB: DELETE FROM scheduling_appointments WHERE organization_id = 'uuid-org'
    Ops->>SharedDB: DELETE FROM organizations WHERE id = 'uuid-org'

    Note over Ops, Gateway: 4. Re-enrutamiento y Activación
    Ops->>Gateway: Actualizar Tenant Registry (uuid-org -> DedicatedDB Pool)
    Ops->>DedicatedDB: UPDATE organizations SET status = 'ACTIVE' WHERE id = 'uuid-org'
    Gateway-->>Ops: Recarga en caliente confirmada (Cero tiempo de inactividad global)
```

#### Script Operativo de Extracción en Nushell (`export_tenant.nu`)

```bash
#!/usr/bin/env nu

def export-tenant [
    org_id: string,
    shared_url: string,
    dedicated_url: string
] {
    let target_tables = [
        "organizations",
        "organization_memberships",
        "scheduling_appointments",
        "clinical_encounters",
        "clinical_prescriptions",
        "billing_invoices"
    ]

    print $"Iniciando extracción para Organización: ($org_id)"

    for table in $target_tables {
        let condition = if $table == "organizations" {
            $"id = '($org_id)'"
        } else {
            $"organization_id = '($org_id)'"
        }

        print $"Transfiriendo tabla ($table)..."

        psql $shared_url -c $"COPY (SELECT * FROM ($table) WHERE ($condition)) TO STDOUT BINARY;"
        | psql $dedicated_url -c $"COPY ($table) FROM STDIN BINARY;"
    }

    print "Transferencia binaria completada exitosamente."
}
```

---

### 7. Criterios de Aceptación Normativa

1. **Latencia del Alta**: El endpoint de registro del fundador debe retornar una respuesta exitosa en un tiempo $\le 100\text{ ms}$, completando la inserción de `organizations` en segundo plano en $\le 5\text{ ms}$.
2. **Barrera de Acceso (`Guard`)**: Toda llamada a los servicios de citas (`scheduling`), pacientes (`patient`) o historias clínicas (`clinical`) debe validar a nivel de middleware que el `organization_id` posea el estado `ACTIVE`.
3. **Verificación Formal**: Es condición ineludible para la activación que el RUC sea validado mediante el algoritmo oficial de Módulo 11 (SUNAT) y que el DNI del titular quede registrado en `crates/user` con nivel de aseguramiento verificado.
