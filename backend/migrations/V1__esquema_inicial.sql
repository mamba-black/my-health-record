-- Esquema inicial de ClickCare (línea base).
--
-- Es idempotente (`IF NOT EXISTS`) para que pueda registrarse sobre una base que ya
-- tenía el esquema creado a mano con el antiguo `ddl/table.sql`, sin borrar datos.
-- No modifiques esta migración: refinery valida su checksum. Los cambios van en una
-- migración nueva (`V{n}__descripcion.sql`).
--
-- Excepción (2026-11-11): se comentaron las tablas legadas `public.clinic`,
-- `public.patient` y `public.patient_search` (y `pg_trgm`, que solo usaba su índice)
-- al recrear la base desde cero. Ningún código las usa: `administration.organization`
-- y `administration.patient` las reemplazaron. Una base creada antes de ese cambio
-- tiene otro checksum de V1 y debe recrearse (ver `backend/AGENTS.md`).

CREATE SCHEMA IF NOT EXISTS identity;

CREATE SCHEMA IF NOT EXISTS administration;

-- Toasty no admite nombres de tabla calificados por esquema (`#[table = "..."]` se
-- serializa como un único identificador entrecomillado), así que los esquemas de los
-- contextos acotados deben ser alcanzables por `search_path`.
--
-- El nombre de la base se resuelve con `current_database()` en lugar de escribirse a
-- mano: en el contenedor de pruebas es `postgres`, pero en un entorno gestionado (Neon)
-- es otro, y un nombre fijo dejaría el `search_path` sin aplicar en silencio.
--
-- `ALTER DATABASE` solo afecta a las sesiones nuevas, no a la que ejecuta este script.
DO $$
BEGIN
    EXECUTE format(
        'ALTER DATABASE %I SET search_path TO identity, administration, public',
        current_database()
    );
END
$$;

CREATE TABLE IF NOT EXISTS identity.user_account (
    id              uuid PRIMARY KEY,
    network_id      uuid NOT NULL,
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    is_owner        BOOLEAN NOT NULL DEFAULT FALSE,
    provider_info   VARCHAR(50) NOT NULL DEFAULT 'Google',

    -- Person / HumanName
    given_name      VARCHAR(100) NOT NULL,
    family_name     VARCHAR(50),
    second_family_name VARCHAR(50),

    -- Person / Identifier (e.g. DNI)
    document_type   VARCHAR(20),
    document_value  VARCHAR(50),

    -- Person / ContactPoint & Audit
    email           VARCHAR(100) NOT NULL,
    phone           VARCHAR(20),
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_user_network_email UNIQUE (network_id, email)
);

CREATE INDEX IF NOT EXISTS idx_user_account_network_id ON identity.user_account (network_id);

-- ========================================================================
-- Bounded Context: administration (crates/administration)
--
-- El worker de Apalis consume `UserCreatedEvent` y materializa aquí las réplicas
-- demográficas autónomas de cada clínica. El recurso FHIR `Person` se guarda como
-- JSON en una sola columna para conservar el recurso completo sin aplanarlo.
--
-- MULTI-CLÍNICA & REDES:
-- La Red de Clínicas (`clinical_network`) define el límite de autenticación e
-- identidad (`network_id`), mientras que `organization` representa a la clínica
-- o sede física con su propio `subdomain` único.
--
-- El aislamiento entre clínicas es a nivel de fila, con `organization_id` en cada
-- tabla local de clínica. No se usa `PARTITION BY LIST`: crear una organización
-- tomaría un `AccessExclusiveLock` sobre la tabla padre.
--
-- La unicidad es **compuesta** `(organization_id, user_id)`, no global sobre
-- `user_id`: la misma persona puede ser paciente de varias clínicas, y el mismo
-- médico puede atender en más de una. Además respalda la idempotencia del handler,
-- porque la entrega de la cola es at-least-once y el mismo evento puede repetirse.
-- ========================================================================

CREATE TABLE IF NOT EXISTS administration.clinical_network
(
    id          uuid PRIMARY KEY,
    name        VARCHAR(200) NOT NULL,
    is_default  BOOLEAN      NOT NULL DEFAULT TRUE,
    active      BOOLEAN      NOT NULL DEFAULT TRUE,

    created_at  TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP             DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS administration.organization
(
    id            uuid PRIMARY KEY,
    network_id    uuid         NOT NULL,
    subdomain     VARCHAR(100) NOT NULL UNIQUE,
    name          VARCHAR(200) NOT NULL,
    tax_id        VARCHAR(20),
    owner_user_id uuid         NOT NULL,
    active        BOOLEAN      NOT NULL DEFAULT TRUE,

    created_at    TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_organization_network
        FOREIGN KEY (network_id) REFERENCES administration.clinical_network (id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_organization_subdomain ON administration.organization (subdomain);
CREATE INDEX IF NOT EXISTS idx_organization_network_id ON administration.organization (network_id);

CREATE TABLE IF NOT EXISTS administration.practitioner
(
    id                     uuid PRIMARY KEY,
    organization_id        uuid        NOT NULL,
    user_id                uuid        NOT NULL,
    active                 BOOLEAN     NOT NULL DEFAULT TRUE,
    medical_license_number VARCHAR(50) NOT NULL,
    specialty              VARCHAR(100),

    -- Recurso FHIR R4 Person serializado
    person                 TEXT        NOT NULL,

    created_at             TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,
    updated_at             TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_practitioner_org_user UNIQUE (organization_id, user_id),
    CONSTRAINT fk_practitioner_organization
        FOREIGN KEY (organization_id) REFERENCES administration.organization (id) ON DELETE CASCADE
);

-- Índice compuesto B-Tree: toda consulta de este contexto entra acotada por clínica.
CREATE INDEX IF NOT EXISTS idx_practitioner_org_id ON administration.practitioner (organization_id, id);

CREATE TABLE IF NOT EXISTS administration.patient
(
    id                     uuid PRIMARY KEY,
    organization_id        uuid    NOT NULL,
    user_id                uuid    NOT NULL,
    active                 BOOLEAN NOT NULL DEFAULT TRUE,

    -- Recurso FHIR R4 Person serializado
    person                 TEXT    NOT NULL,

    -- IDs de expedientes previos absorbidos de otras clínicas (HL7 FHIR Patient.link seeAlso / replaces)
    referenced_patient_ids uuid[]  NOT NULL DEFAULT '{}',

    created_at             TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at             TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_patient_org_user UNIQUE (organization_id, user_id),
    CONSTRAINT fk_patient_organization
        FOREIGN KEY (organization_id) REFERENCES administration.organization (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_patient_org_id ON administration.patient (organization_id, id);
CREATE INDEX IF NOT EXISTS idx_patient_referenced_ids ON administration.patient USING GIN (referenced_patient_ids);

-- ========================================================================
-- LEGADO (desactivado): primer modelo de clínicas y pacientes, previo a los
-- contextos acotados. Lo reemplazan `administration.organization` y
-- `administration.patient`. Se conserva comentado solo como referencia histórica.
-- ========================================================================
-- CREATE TABLE IF NOT EXISTS public.clinic
-- (
--     clinic_id    uuid PRIMARY KEY DEFAULT uuidv7(),
--     name         VARCHAR(100) NOT NULL,
--     address      TEXT,
--     phone_number VARCHAR(15),
--     email        VARCHAR(100),
--
--     -- Audit fields
--     created_at   TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
--     updated_at   TIMESTAMP        DEFAULT CURRENT_TIMESTAMP
-- );
--
-- CREATE TABLE IF NOT EXISTS public.patient
-- (
--     patient_id       uuid        NOT NULL DEFAULT uuidv7(),
--     clinic_id        uuid        NOT NULL,
--
--     first_name       VARCHAR(50) NOT NULL,
--     last_name        VARCHAR(50) NOT NULL,
--     second_last_name VARCHAR(50),
--     date_of_birth    DATE        NOT NULL,
--     gender           VARCHAR(10),
--     phone_number     VARCHAR(15),
--     email            VARCHAR(100),
--     address          TEXT,
--     -- Audit fields
--     created_at       TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,
--     updated_at       TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,
--
--     PRIMARY KEY (clinic_id, patient_id),
--
--     CONSTRAINT fk_patient_clinic
--         FOREIGN KEY (clinic_id) REFERENCES public.clinic (clinic_id) ON DELETE CASCADE
-- )
--     PARTITION BY LIST (clinic_id);
--
-- CREATE TABLE IF NOT EXISTS public.patient_search
-- (
--     clinic_id  uuid         NOT NULL,
--     patient_id uuid         NOT NULL,
--     full_name  VARCHAR(200) NOT NULL,
--
--     PRIMARY KEY (clinic_id, patient_id),
--
--     CONSTRAINT fk_patient_search_clinic_id
--         FOREIGN KEY (clinic_id) REFERENCES public.clinic (clinic_id) ON DELETE CASCADE,
--     CONSTRAINT fk_patient_search_patient_id
--         FOREIGN KEY (clinic_id, patient_id) REFERENCES public.patient (clinic_id, patient_id) ON DELETE CASCADE
-- )
--     PARTITION BY LIST (clinic_id);
--
-- CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA public;
-- CREATE INDEX IF NOT EXISTS idx_patient_search_full_name
--     ON public.patient_search
--         USING GIN (full_name gin_trgm_ops);
