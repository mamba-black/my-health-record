---
schema_version: 9
id: my-health-record-9iv
title: Adoptar refinery para migraciones versionadas del esquema
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-07T14:08:51Z
started_at: 2026-11-07T14:09:04Z
completed_at: 2026-11-07T14:17:00Z
deps: []
tags:
  - task
acceptance: []
---

Hoy el esquema vive en backend/ddl/table.sql (drop-and-create) y los cambios en Neon se aplican a mano. Adoptar refinery: migraciones SQL versionadas en backend/migrations, embebidas en el binario y aplicadas al arrancar clickcare, con su propia tabla de control (no la _sqlx_migrations de apalis). Las pruebas con testcontainers deben usar las mismas migraciones.

## Close Reason

refinery 0.10 con migraciones V1 (línea base idempotente) y V2 (birth_date, address) en backend/migrations; se aplican al arrancar clickcare y en el contenedor de pruebas; Neon registrado sin borrar datos.
