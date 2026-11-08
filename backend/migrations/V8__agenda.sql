-- Agenda y citas (openspec/changes/scheduling-appointments).
--
-- Toda tabla lleva `network_id` (la Organization raíz): ninguna regla compara datos de
-- dos organizaciones, para que cada una pueda vivir en su propia base.

-- ── administration ──────────────────────────────────────────────────────────
ALTER TABLE administration.organization
    ADD COLUMN IF NOT EXISTS timezone TEXT NOT NULL DEFAULT 'America/Lima',
    ADD COLUMN IF NOT EXISTS address  VARCHAR(300);

-- Verificación presencial del documento de un paciente sin cuenta (check-in).
ALTER TABLE administration.patient
    ADD COLUMN IF NOT EXISTS document_verified_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS document_verified_by uuid;

-- Vínculo de un paciente registrado sin cuenta con la cuenta que creó después. No
-- fusiona registros: deshacerlo solo cierra el vínculo.
CREATE TABLE IF NOT EXISTS administration.patient_link
(
    id              uuid PRIMARY KEY,
    network_id      uuid      NOT NULL,
    organization_id uuid      NOT NULL,
    patient_id      uuid      NOT NULL, -- registro sin cuenta
    user_id         uuid      NOT NULL, -- cuenta vinculada
    linked_by       uuid      NOT NULL,
    linked_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    unlinked_by     uuid,
    unlinked_at     TIMESTAMPTZ,
    unlink_reason   TEXT,

    CONSTRAINT fk_patient_link_patient
        FOREIGN KEY (patient_id) REFERENCES administration.patient (id) ON DELETE CASCADE
);
-- Un solo vínculo activo por registro.
CREATE UNIQUE INDEX IF NOT EXISTS uq_patient_link_active
    ON administration.patient_link (patient_id) WHERE unlinked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_patient_link_user
    ON administration.patient_link (network_id, user_id) WHERE unlinked_at IS NULL;

-- ── scheduling ──────────────────────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE SCHEMA IF NOT EXISTS scheduling;

-- Horario de un médico en una clínica. Los cupos no se guardan: salen de aquí.
CREATE TABLE IF NOT EXISTS scheduling.schedule
(
    id                   uuid PRIMARY KEY,
    network_id           uuid     NOT NULL,
    organization_id      uuid     NOT NULL,
    practitioner_user_id uuid     NOT NULL,
    slot_minutes         SMALLINT NOT NULL CHECK (slot_minutes BETWEEN 5 AND 480),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_schedule_clinic_practitioner UNIQUE (organization_id, practitioner_user_id)
);
CREATE INDEX IF NOT EXISTS idx_schedule_practitioner
    ON scheduling.schedule (network_id, practitioner_user_id);

-- Franjas semanales en hora local de la clínica (1 = lunes … 7 = domingo).
CREATE TABLE IF NOT EXISTS scheduling.schedule_block
(
    schedule_id uuid     NOT NULL REFERENCES scheduling.schedule (id) ON DELETE CASCADE,
    weekday     SMALLINT NOT NULL CHECK (weekday BETWEEN 1 AND 7),
    start_time  TIME     NOT NULL,
    end_time    TIME     NOT NULL,
    CHECK (start_time < end_time),
    PRIMARY KEY (schedule_id, weekday, start_time)
);

-- Feriados, vacaciones y otras ausencias.
CREATE TABLE IF NOT EXISTS scheduling.schedule_exception
(
    id          uuid PRIMARY KEY,
    schedule_id uuid      NOT NULL REFERENCES scheduling.schedule (id) ON DELETE CASCADE,
    period      TSTZRANGE NOT NULL CHECK (NOT isempty(period)),
    reason      TEXT
);
CREATE INDEX IF NOT EXISTS idx_schedule_exception_period
    ON scheduling.schedule_exception USING gist (schedule_id, period);

-- Citas. La restricción EXCLUDE es la exclusión: un médico nunca tiene dos citas
-- activas que se crucen dentro de su organización, aunque sean de clínicas distintas.
CREATE TABLE IF NOT EXISTS scheduling.appointment
(
    id                   uuid PRIMARY KEY,
    network_id           uuid        NOT NULL,
    organization_id      uuid        NOT NULL,
    practitioner_user_id uuid        NOT NULL,
    patient_id           uuid        NOT NULL, -- administration.patient.id (referencia débil)
    period               TSTZRANGE   NOT NULL CHECK (NOT isempty(period)),
    status               VARCHAR(12) NOT NULL, -- BOOKED | ARRIVED | CANCELLED | NOSHOW
    version              INTEGER     NOT NULL DEFAULT 1,
    booked_by            uuid        NOT NULL,
    cancelled_by         uuid,
    cancelled_at         TIMESTAMPTZ,
    arrived_by           uuid,
    arrived_at           TIMESTAMPTZ,
    document_check       VARCHAR(10), -- MATCH | MISMATCH
    noshow_by            uuid,
    noshow_at            TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ex_appointment_practitioner_overlap
        EXCLUDE USING gist (network_id WITH =, practitioner_user_id WITH =, period WITH &&)
        WHERE (status IN ('BOOKED', 'ARRIVED'))
);
CREATE INDEX IF NOT EXISTS idx_appointment_clinic_period
    ON scheduling.appointment USING gist (organization_id, period);
CREATE INDEX IF NOT EXISTS idx_appointment_patient
    ON scheduling.appointment (network_id, patient_id);

-- Recordatorio de cada cita: la fuente de verdad del envío. La cola solo despierta.
CREATE TABLE IF NOT EXISTS scheduling.reminder
(
    id             uuid PRIMARY KEY,
    network_id     uuid        NOT NULL,
    appointment_id uuid        NOT NULL UNIQUE REFERENCES scheduling.appointment (id) ON DELETE CASCADE,
    due_at         TIMESTAMPTZ NOT NULL,
    status         VARCHAR(12) NOT NULL, -- PENDING | SENDING | SENT | FAILED | NO_CHANNEL | CANCELLED
    attempts       SMALLINT    NOT NULL DEFAULT 0,
    claimed_at     TIMESTAMPTZ,
    sent_at        TIMESTAMPTZ,
    last_error     TEXT
);
CREATE INDEX IF NOT EXISTS idx_reminder_due
    ON scheduling.reminder (status, due_at);
