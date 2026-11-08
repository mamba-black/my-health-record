-- Organization raíz del dueño (FHIR R4) y vínculos de la persona (Person.link).
--
-- La red deja de ser un atributo suelto: es la Organization raíz del dueño, su empresa
-- o grupo. Se guarda en `administration.clinical_network` (Organization sin partOf) y
-- cada clínica es una Organization cuyo `network_id` es su `partOf`: apunta a la raíz.
--
-- Cada dueño tiene una única raíz. Las cuentas no se mueven entre redes: la raíz
-- existe desde que el dueño se registra (o, a más tardar, desde su primera clínica).
ALTER TABLE administration.clinical_network
    ADD COLUMN IF NOT EXISTS owner_user_id uuid;

-- Relleno: la raíz de cada dueño es la red de su primera clínica. Las demás redes que
-- haya creado antes quedan sin dueño (legado) y sus clínicas no se mueven.
UPDATE administration.clinical_network n
SET owner_user_id = first_clinic.owner_user_id
FROM (SELECT DISTINCT ON (o.owner_user_id) o.owner_user_id, o.network_id
      FROM administration.organization o
      ORDER BY o.owner_user_id, o.created_at, o.id) first_clinic
WHERE n.id = first_clinic.network_id
  AND n.owner_user_id IS NULL
  AND NOT EXISTS (SELECT 1
                  FROM administration.clinical_network other
                  WHERE other.owner_user_id = first_clinic.owner_user_id);

CREATE UNIQUE INDEX IF NOT EXISTS uq_clinical_network_owner
    ON administration.clinical_network (owner_user_id)
    WHERE owner_user_id IS NOT NULL;

COMMENT ON TABLE administration.clinical_network IS
    'Organization raíz del dueño (FHIR R4 Organization sin partOf): su empresa o grupo y límite de aislamiento';
COMMENT ON COLUMN administration.organization.network_id IS
    'Organization.partOf: la Organization raíz del dueño de la clínica';

-- Person.link: la persona de una cuenta vinculada con otros recursos FHIR.
CREATE TABLE IF NOT EXISTS identity.person_link
(
    user_id     uuid        NOT NULL,
    target_type VARCHAR(20) NOT NULL, -- ORGANIZATION | PATIENT | PRACTITIONER | RELATED_PERSON
    target_id   uuid        NOT NULL,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (user_id, target_type, target_id)
);

-- Relleno: cada dueño queda vinculado a su raíz y a sus clínicas.
INSERT INTO identity.person_link (user_id, target_type, target_id)
SELECT owner_user_id, 'ORGANIZATION', id
FROM administration.clinical_network
WHERE owner_user_id IS NOT NULL
UNION
SELECT owner_user_id, 'ORGANIZATION', id
FROM administration.organization
ON CONFLICT DO NOTHING;
