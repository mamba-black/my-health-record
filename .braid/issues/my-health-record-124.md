---
schema_version: 9
id: my-health-record-124
title: Validación de Usuario con Google (SignIn) y Registro Progresivo (SignUp)
priority: P1
status: done
type: meta
owner: miuler@gmail.com
created_at: 2026-10-18T23:41:38Z
started_at: 2026-10-19T04:15:46Z
completed_at: 2026-10-19T04:31:31Z
deps: []
tags: []
acceptance:
  - "cargo test -p user, cargo test -p clickcare --test user_api_test sign_in, cargo clippy --workspace --all-targets -- -D warnings, bun run check y bun run build pasan sin errores."
---

Implementar el flujo completo de autenticación con Google entre el Frontend (Svelte 5 + FSD) y el Backend (Rust + Arquitectura Cebolla + gRPC-Web), resolviendo network_id por subdominio y redirigiendo a /signup con perfilado progresivo cuando el usuario no existe (NOT_FOUND).

## Design

Ver implementation_plan.md: 1) Contrato Protobuf SignInResponse, 2) crates/user SignInUseCase + find_user_by_email, 3) bin/clickcare UserApi::sign_in + CORS + tests, 4) Frontend FSD jwt, user-api, session.store, LoginForm, SignUpForm y ruta /signup.

## Close Reason

Todos los nodos (124.1 a 124.5) implementados y verificados con tests de backend y build de frontend
