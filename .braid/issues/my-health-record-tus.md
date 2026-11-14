---
schema_version: 9
id: my-health-record-tus
title: Login acepta cualquier contraseña
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:18Z
completed_at: 2026-11-07T22:44:06Z
deps: []
tags:
  - bug
acceptance: []
---

LoginForm.svelte (loginHandler) crea la sesión con setAuthToken("1234567890", ...) sin validar credenciales; verificado con contraseña incorrecta. SignUpForm.svelte también usa el respaldo "local-dev-token". La prueba e2e 'el login rechaza credenciales inválidas' documenta el fallo con test.fail.

## Close Reason

Resuelto en 18a7ef6: el login con contraseña ya no crea sesión y avisa que no está disponible. El respaldo local-dev-token de SignUpForm queda anotado en rvc
