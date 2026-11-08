-- Pacientes sin cuenta (identity/patient-account-linking, «Paciente sin cuenta»).
--
-- Recepción puede registrar a un paciente que aún no tiene cuenta de usuario: su
-- expediente queda sin `user_id` hasta que se vincule con la cuenta que cree después.
--
-- `uq_patient_org_user` se conserva: en PostgreSQL los NULL no chocan entre sí, de modo
-- que una clínica puede tener varios pacientes sin cuenta y sigue sin poder tener dos
-- expedientes de la misma cuenta.
ALTER TABLE administration.patient
    ALTER COLUMN user_id DROP NOT NULL;

-- Membresías: nuevo rol RECEPTIONIST y nuevo estado REQUESTED (la persona pidió entrar
-- a la clínica y espera aprobación). Las columnas son VARCHAR sin CHECK, así que basta
-- con documentarlo.
COMMENT ON COLUMN administration.membership.role IS 'OWNER | ADMIN | PRACTITIONER | RECEPTIONIST';
COMMENT ON COLUMN administration.membership.status IS 'INVITED | REQUESTED | PENDING | ACTIVE | SUSPENDED';
-- Lista de personal de una clínica.
CREATE INDEX IF NOT EXISTS idx_membership_org_id ON administration.membership (organization_id);
