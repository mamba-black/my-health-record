---
schema_version: 9
id: my-health-record-124.1
title: "Contrato Protobuf: Enriquecer SignInResponse en api.proto"
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-10-18T23:41:55Z
started_at: 2026-10-19T04:15:46Z
completed_at: 2026-10-19T04:16:31Z
deps: []
tags:
  - task
  - parent:my-health-record-124
acceptance:
  - "El contrato backend/proto/api.proto compila limpiamente al ejecutar cargo check --workspace."
---

Actualizar SignInResponse en backend/proto/api.proto para retornar los datos básicos de identidad del usuario autenticado (user_id, email, given_name, optional family_name, is_owner).

## Design

Archivo: backend/proto/api.proto. Mantener SignInRequest (id_token, provider_id) y definir en SignInResponse: 1: user_id (string), 2: email (string), 3: given_name (string), 4: optional family_name (string), 5: is_owner (bool).

## Close Reason

SignInResponse enriquecido en proto/api.proto y verificado con cargo check --workspace
