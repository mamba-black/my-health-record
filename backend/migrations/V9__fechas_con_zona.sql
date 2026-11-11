-- Fechas con zona horaria (TIMESTAMPTZ) en todas las columnas de auditoría.
--
-- Los modelos de toasty declaran `created_at`/`updated_at` como `jiff::Timestamp`, que
-- corresponde a `TIMESTAMPTZ`. Con `TIMESTAMP` sin zona, toasty no puede leer las filas
-- con el modelo y el código tenía que mapear columnas a mano. Un instante sin zona
-- además es ambiguo; el esquema `scheduling` (V8) ya nació con `TIMESTAMPTZ`.
--
-- Los valores existentes se escribieron con `CURRENT_TIMESTAMP` en sesiones UTC (Neon y
-- los contenedores de prueba), así que se interpretan como UTC.

ALTER TABLE identity.user_account
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE identity.person_link
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

ALTER TABLE administration.clinical_network
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE administration.organization
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE administration.practitioner
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE administration.patient
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE administration.membership
    ALTER COLUMN license_confirmed_at TYPE TIMESTAMPTZ USING license_confirmed_at AT TIME ZONE 'UTC',
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';
