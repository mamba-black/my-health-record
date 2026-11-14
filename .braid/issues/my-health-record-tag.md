---
schema_version: 9
id: my-health-record-tag
title: Agregar pruebas e2e al frontend con Playwright
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T19:05:53Z
started_at: 2026-11-05T19:06:21Z
completed_at: 2026-11-06T04:58:15Z
deps: []
tags:
  - task
acceptance: []
---

frontendts no tiene pruebas e2e ni runner. Agregar Playwright con specs iniciales de los flujos principales (login, rutas protegidas, búsqueda de pacientes) que corran contra el dev server.

## Close Reason

Suite e2e con Playwright: login con Google simulado y firma real, búsqueda y sesión. 17 pruebas estables.
