---
schema_version: 9
id: my-health-record-ed6
title: Pruebas e2e de login con Google simulado y firma real
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-06T04:54:38Z
completed_at: 2026-11-06T04:58:15Z
deps: []
tags:
  - task
acceptance: []
---

Simular Google Identity Services en Playwright y firmar tokens con una clave de prueba para que SignIn verifique la firma de verdad contra un JWKS local. Usa la base del .env por decisión del usuario.

## Close Reason

Suite e2e con Playwright: login con Google simulado y firma real, búsqueda y sesión. 17 pruebas estables.
