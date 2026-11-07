-- Membresías: la relación de un usuario con una clínica y su rol en ella.
--
-- Son lo único que concede permisos dentro de una clínica. Un usuario puede tener
-- varias, en distintas clínicas o con distintos roles en la misma.
CREATE TABLE IF NOT EXISTS administration.membership
(
    id                     uuid PRIMARY KEY,
    organization_id        uuid        NOT NULL,
    user_id                uuid        NOT NULL,
    role                   VARCHAR(20) NOT NULL, -- OWNER | ADMIN | PRACTITIONER
    status                 VARCHAR(20) NOT NULL, -- INVITED | PENDING | ACTIVE | SUSPENDED
    license_confirmed_at   TIMESTAMP,
    license_confirmed_by   uuid,
    activation_requested   BOOLEAN     NOT NULL DEFAULT FALSE,

    created_at             TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,
    updated_at             TIMESTAMP            DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_membership_org_user_role UNIQUE (organization_id, user_id, role),
    CONSTRAINT fk_membership_organization
        FOREIGN KEY (organization_id) REFERENCES administration.organization (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_membership_user_id ON administration.membership (user_id);

-- Relleno: cada dueño actual tiene su membresía de dueño activa.
INSERT INTO administration.membership (id, organization_id, user_id, role, status)
SELECT uuidv7(), o.id, o.owner_user_id, 'OWNER', 'ACTIVE'
FROM administration.organization o
ON CONFLICT (organization_id, user_id, role) DO NOTHING;

-- Relleno: cada ficha de médico, salvo la que se creó automáticamente para el dueño,
-- queda como membresía de médico pendiente de activación (aún sin colegiatura confirmada).
INSERT INTO administration.membership (id, organization_id, user_id, role, status)
SELECT uuidv7(), p.organization_id, p.user_id, 'PRACTITIONER', 'PENDING'
FROM administration.practitioner p
JOIN administration.organization o ON o.id = p.organization_id
WHERE p.user_id <> o.owner_user_id
ON CONFLICT (organization_id, user_id, role) DO NOTHING;
