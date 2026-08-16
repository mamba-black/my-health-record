DROP SCHEMA IF EXISTS identity CASCADE;
CREATE SCHEMA identity;

DROP SCHEMA IF EXISTS administration CASCADE;
CREATE SCHEMA administration;

-- Toasty no admite nombres de tabla calificados por esquema (`#[table = "..."]` se
-- serializa como un único identificador entrecomillado), así que los esquemas de los
-- contextos acotados deben ser alcanzables por `search_path`.
--
-- El nombre de la base se resuelve con `current_database()` en lugar de escribirse a
-- mano: en el contenedor de pruebas es `postgres`, pero en un entorno gestionado (Neon)
-- es otro, y un nombre fijo dejaría el `search_path` sin aplicar en silencio.
--
-- Nota: `administration.patient` precede a la tabla legada `public.patient`, que ya no
-- usa ningún código Rust.
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

DROP TABLE IF EXISTS patient_search;
DROP TABLE IF EXISTS patient;
DROP TABLE IF EXISTS clinic;
DROP TABLE IF EXISTS identity.user_account;

CREATE TABLE identity.user_account (
    id              uuid PRIMARY KEY,
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
    email           VARCHAR(100),
    phone           VARCHAR(20),
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


-- ========================================================================
-- Bounded Context: administration (crates/administration)
--
-- El worker de Apalis consume `UserCreatedEvent` y materializa aquí las réplicas
-- demográficas autónomas de cada clínica. El recurso FHIR `Person` se guarda como
-- JSON en una sola columna para conservar el recurso completo sin aplanarlo.
--
-- MULTI-CLÍNICA: los esquemas representan fronteras de dominio FHIR, no inquilinos.
-- El aislamiento entre clínicas es a nivel de fila, con `organization_id` en cada
-- tabla local de clínica. No se usa `PARTITION BY LIST`: crear una organización
-- tomaría un `AccessExclusiveLock` sobre la tabla padre.
--
-- La unicidad es **compuesta** `(organization_id, user_id)`, no global sobre
-- `user_id`: la misma persona puede ser paciente de varias clínicas, y el mismo
-- médico puede atender en más de una. Además respalda la idempotencia del handler,
-- porque la entrega de la cola es at-least-once y el mismo evento puede repetirse.
-- ========================================================================

CREATE TABLE administration.organization
(
    id            uuid PRIMARY KEY,
    name          VARCHAR(200) NOT NULL,
    tax_id        VARCHAR(20),
    owner_user_id uuid         NOT NULL UNIQUE,
    active        BOOLEAN      NOT NULL DEFAULT TRUE,

    created_at    TIMESTAMP             DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP             DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE administration.practitioner
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
CREATE INDEX idx_practitioner_org_id ON administration.practitioner (organization_id, id);

CREATE TABLE administration.patient
(
    id              uuid PRIMARY KEY,
    organization_id uuid    NOT NULL,
    user_id         uuid    NOT NULL,
    active          BOOLEAN NOT NULL DEFAULT TRUE,

    -- Recurso FHIR R4 Person serializado
    person          TEXT    NOT NULL,

    created_at      TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_patient_org_user UNIQUE (organization_id, user_id),
    CONSTRAINT fk_patient_organization
        FOREIGN KEY (organization_id) REFERENCES administration.organization (id) ON DELETE CASCADE
);

CREATE INDEX idx_patient_org_id ON administration.patient (organization_id, id);


CREATE TABLE clinic
(
    clinic_id    uuid PRIMARY KEY DEFAULT uuidv7(),
    name         VARCHAR(100) NOT NULL,
    address      TEXT,
    phone_number VARCHAR(15),
    email        VARCHAR(100),

    -- Audit fields
    created_at   TIMESTAMP        DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP        DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE patient
(
    patient_id       uuid        NOT NULL DEFAULT uuidv7(),
    clinic_id        uuid        NOT NULL,

    first_name       VARCHAR(50) NOT NULL,
    last_name        VARCHAR(50) NOT NULL,
    second_last_name VARCHAR(50),
    date_of_birth    DATE        NOT NULL,
    gender           VARCHAR(10),
    phone_number     VARCHAR(15),
    email            VARCHAR(100),
    address          TEXT,
    -- Audit fields
    created_at       TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (clinic_id, patient_id),

    CONSTRAINT fk_patient_clinic
        FOREIGN KEY (clinic_id) REFERENCES clinic (clinic_id) ON DELETE CASCADE
)
    PARTITION BY LIST (clinic_id);

CREATE TABLE "patient_search"
(
    clinic_id  uuid         NOT NULL,
    patient_id uuid         NOT NULL,
    full_name  VARCHAR(200) NOT NULL,

    PRIMARY KEY (clinic_id, patient_id),

    CONSTRAINT fk_patient_search_clinic_id
        FOREIGN KEY (clinic_id) REFERENCES clinic (clinic_id) ON DELETE CASCADE,
    CONSTRAINT fk_patient_search_patient_id
        FOREIGN KEY (clinic_id, patient_id) REFERENCES patient (clinic_id, patient_id) ON DELETE CASCADE
)
    PARTITION BY LIST (clinic_id);


CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_patient_search_full_name
    ON patient_search
        USING GIN (full_name gin_trgm_ops);





-- ========================================================================
-- USAR LA ZONA HORARIA 'America/Lima' PARA EXTRAER LA FECHA Y HORA DEL UUIDv7
SELECT uuid_extract_timestamp(patient_id) AT TIME ZONE 'America/Lima', *
FROM patient;

-- OTRA FORMA DE ESTABLECER LA ZONA HORARIA
SET TIMEZONE = 'America/Lima';
SELECT uuid_extract_timestamp(patient_id), *
FROM patient;
-- ========================================================================


-- INSERT INTO patient(first_name, last_name, date_of_birth)
-- VALUES ('John',
--         'Doe',
--         '1980-05-15');
