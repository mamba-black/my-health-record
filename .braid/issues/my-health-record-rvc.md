---
schema_version: 9
id: my-health-record-rvc
title: SignUp no valida el id_token
priority: P1
status: open
owner: miuler@gmail.com
created_at: 2026-11-05T23:48:22Z
deps: []
tags:
  - bug
acceptance: []
---

CreateUserUseCase no usa TokenService: SignUp registra el email recibido sin comprobar el id_token, por lo que se puede crear una cuenta con un correo ajeno. Validar el token y usar el email de sus claims.

## Notes

SignUpForm.svelte:57 envía 'local-dev-token' cuando no hay id_token de Google; quitar ese respaldo al validar el token en el backend.
