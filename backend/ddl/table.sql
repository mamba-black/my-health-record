DROP TABLE IF EXISTS patient_search;
DROP TABLE IF EXISTS patient;
DROP TABLE IF EXISTS clinic;
DROP TABLE IF EXISTS user_account;

CREATE TABLE user_account (
    id uuid PRIMARY KEY ,
    first_name       VARCHAR(50) NOT NULL,
    last_name        VARCHAR(50) NOT NULL,
    second_last_name VARCHAR(50)
);


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
