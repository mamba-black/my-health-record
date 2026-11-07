-- Estado de la clínica (RFC-004) y datos legales para activarla.
--
-- `status` sustituye al booleano `active`, que se conserva hasta que ningún código
-- lo lea y se eliminará en una migración posterior.
ALTER TABLE administration.organization
    ADD COLUMN IF NOT EXISTS status      VARCHAR(20) NOT NULL DEFAULT 'PENDING_SETUP',
    ADD COLUMN IF NOT EXISTS legal_name  VARCHAR(200),
    ADD COLUMN IF NOT EXISTS trade_name  VARCHAR(200),
    ADD COLUMN IF NOT EXISTS ipress_code VARCHAR(20);

-- Clínicas existentes: activas siguen activas (son datos de prueba sin RUC; revisar
-- antes de producción) y las inactivas quedan suspendidas.
UPDATE administration.organization
SET status = CASE WHEN active THEN 'ACTIVE' ELSE 'SUSPENDED' END;
