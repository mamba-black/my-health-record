-- Datos de perfil que se registran en `/signup` y `/complete-profile`.
--
-- Son opcionales a propósito: el registro es progresivo y el aviso de perfil
-- incompleto necesita detectar las cuentas que aún no los tienen.
ALTER TABLE identity.user_account
    ADD COLUMN IF NOT EXISTS birth_date DATE,
    ADD COLUMN IF NOT EXISTS address    VARCHAR(255);
