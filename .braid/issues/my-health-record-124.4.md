---
schema_version: 9
id: my-health-record-124.4
title: "Frontend FSD (Shared & Entities): Decodificador JWT, cliente gRPC-Web UserApi y Session Store"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-10-18T23:41:55Z
started_at: 2026-10-19T04:26:31Z
completed_at: 2026-10-19T04:29:46Z
deps:
  - my-health-record-124.1
tags:
  - task
  - parent:my-health-record-124
acceptance:
  - "bun run check valida los tipos sin errores en las capas shared y entities."
---

Implementar utilidades base en el frontend FSD para decodificar claims del JWT de Google, invocar UserApi/SignIn y UserApi/SignUp por gRPC-Web, agregar ruta SIGNUP y almacenar pendingRegistration en el store de sesión.

## Design

Archivos: frontendts/src/shared/lib/jwt/{jwt.ts,index.ts} (decodeGoogleJwt, GoogleJwtClaims), frontendts/src/shared/api/user/{user-api.ts,index.ts} (cliente gRPC-Web para SignIn y SignUp), frontendts/src/shared/config/routes/{paths.ts,index.ts} (SIGNUP = resolve('/signup')), frontendts/src/entities/session/model/session.store.ts y index.ts (pendingRegistration y setPendingRegistration).

## Close Reason

Implementadas capas shared/lib/jwt, shared/api/user, shared/config/routes y entities/session; svelte-check 0 errores
