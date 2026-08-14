-- ============================================================================
-- Esquemas por Bounded Context
-- ============================================================================
-- Cada contexto acotado es dueño de su propio esquema, de modo que permisos,
-- ownership y backups se puedan gestionar por dominio.
--
-- El dominio `user` usa el esquema `identity`: `user` es palabra reservada en
-- PostgreSQL y `CREATE SCHEMA user` es un error de sintaxis. `identity` coincide
-- además con el nombre del bounded context ("Identity & Security", ver AGENTS.md
-- §2), y evita tener que entrecomillar el identificador en cada referencia.
--
-- Las tablas se referencian SIN calificar desde el código Rust: toasty 0.8 no
-- soporta nombres calificados por esquema (serializa `#[table = "x"]` como el
-- identificador `"x"`, sin separar por el punto), y su parser de URL descarta el
-- parámetro `options`, así que tampoco se puede fijar el search_path por
-- conexión. La resolución se hace con un search_path a nivel de base de datos.

CREATE SCHEMA IF NOT EXISTS identity;
CREATE SCHEMA IF NOT EXISTS administration;

-- `ALTER DATABASE` solo afecta a sesiones nuevas, por eso se fija además el
-- search_path de la sesión actual: el resto de este script depende de él.
-- Se usa `current_database()` para no acoplar el script a un nombre de BD.
DO $$
BEGIN
    EXECUTE format(
        'ALTER DATABASE %I SET search_path = identity, administration, public',
        current_database()
    );
END
$$;

SET search_path = identity, administration, public;

-- pg_trgm se instala en `public` para que sus operadores queden visibles desde
-- cualquier esquema del search_path.
CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA public;

-- ============================================================================
-- Limpieza (orden inverso al de dependencias)
-- ============================================================================
DROP TABLE IF EXISTS administration.patient_search;
DROP TABLE IF EXISTS administration.patient;
DROP TABLE IF EXISTS administration.clinic;
DROP TABLE IF EXISTS identity.user_account;

-- Restos de la etapa previa a la separación por esquemas, cuando todas las
-- tablas vivían en `public`.
DROP TABLE IF EXISTS public.patient_search;
DROP TABLE IF EXISTS public.patient;
DROP TABLE IF EXISTS public.clinic;
DROP TABLE IF EXISTS public.user_account;

-- ============================================================================
-- Bounded Context: Identity & Security  (crates/user  ->  esquema `identity`)
-- ============================================================================

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


-- ============================================================================
-- Bounded Context: Gestión Administrativa
-- (crates/administration  ->  esquema `administration`)
-- ============================================================================

CREATE TABLE administration.clinic
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


CREATE TABLE administration.patient
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
        FOREIGN KEY (clinic_id) REFERENCES administration.clinic (clinic_id) ON DELETE CASCADE
)
    PARTITION BY LIST (clinic_id);

CREATE TABLE administration.patient_search
(
    clinic_id  uuid         NOT NULL,
    patient_id uuid         NOT NULL,
    full_name  VARCHAR(200) NOT NULL,

    PRIMARY KEY (clinic_id, patient_id),

    CONSTRAINT fk_patient_search_clinic_id
        FOREIGN KEY (clinic_id) REFERENCES administration.clinic (clinic_id) ON DELETE CASCADE,
    CONSTRAINT fk_patient_search_patient_id
        FOREIGN KEY (clinic_id, patient_id) REFERENCES administration.patient (clinic_id, patient_id) ON DELETE CASCADE
)
    PARTITION BY LIST (clinic_id);

CREATE INDEX idx_patient_search_full_name
    ON administration.patient_search
        USING GIN (full_name gin_trgm_ops);





-- ========================================================================
-- USAR LA ZONA HORARIA 'America/Lima' PARA EXTRAER LA FECHA Y HORA DEL UUIDv7
SELECT uuid_extract_timestamp(patient_id) AT TIME ZONE 'America/Lima', *
FROM administration.patient;

-- OTRA FORMA DE ESTABLECER LA ZONA HORARIA
SET TIMEZONE = 'America/Lima';
SELECT uuid_extract_timestamp(patient_id), *
FROM administration.patient;
-- ========================================================================


-- INSERT INTO administration.patient(first_name, last_name, date_of_birth)
-- VALUES ('John',
--         'Doe',
--         '1980-05-15');
