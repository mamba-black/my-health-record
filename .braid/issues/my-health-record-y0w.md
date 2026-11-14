---
schema_version: 9
id: my-health-record-y0w
title: Las pruebas de integración dejan contenedores de Postgres huérfanos
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-07T22:41:46Z
completed_at: 2026-11-07T22:43:39Z
deps: []
tags:
  - bug
acceptance: []
---

user_api_test crea clickcare-test-<timestamp> y lo suelta con mem::forget; el borrado programado está comentado desde d6fb942 (y borraba a los 10 s aunque hubiera pruebas corriendo). Cada vez que la caché no responde se crea otro contenedor. Arreglo: nombre fijo clickcare-test, borrar el anterior si no responde y aplicar migraciones también al reutilizarlo.

## Close Reason

Implementado y probado
