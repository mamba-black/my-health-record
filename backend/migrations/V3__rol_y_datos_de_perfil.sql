-- Rol preferido y datos de perfil ampliados (cambio `role-based-access-by-profile-data`).
--
-- El rol preferido solo orienta la interfaz; los permisos salen de las membresías
-- (`administration.membership`, V4). Los datos verificados no los edita el usuario.
ALTER TABLE identity.user_account
    ADD COLUMN IF NOT EXISTS preferred_role    VARCHAR(20) NOT NULL DEFAULT 'PATIENT',
    ADD COLUMN IF NOT EXISTS sex               VARCHAR(10),
    ADD COLUMN IF NOT EXISTS professional_body VARCHAR(20),
    ADD COLUMN IF NOT EXISTS license_number    VARCHAR(30),
    ADD COLUMN IF NOT EXISTS specialty         VARCHAR(100),
    ADD COLUMN IF NOT EXISTS rne               VARCHAR(30),
    ADD COLUMN IF NOT EXISTS document_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS license_verified  BOOLEAN NOT NULL DEFAULT FALSE;

-- Cuentas previas: quien es dueño prefiere el rol de dueño; el resto, paciente.
UPDATE identity.user_account SET preferred_role = 'CLINIC_OWNER' WHERE is_owner;
