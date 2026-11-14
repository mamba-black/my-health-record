---
schema_version: 9
id: my-health-record-124.2
title: "Backend crates/user: Implementar find_user_by_email y SignInUseCase"
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-10-18T23:41:55Z
started_at: 2026-10-19T04:16:32Z
completed_at: 2026-10-19T04:21:39Z
deps: []
tags:
  - task
  - parent:my-health-record-124
acceptance:
  - "cargo check -p user y cargo test -p user pasan sin errores ni warnings."
---

Implementar la búsqueda de usuario por (network_id, email) en el repositorio y el caso de uso SignInUseCase con validación de JWT y registro en el contenedor DI.

## Design

Archivos: backend/crates/user/src/domain/repository/user_repository.rs (añadir find_user_by_email), backend/crates/user/src/application/sign_in_usecase.rs (SignInCommand, SignInResult, SignInError, SignInUseCaseImpl decodificando JWT base64url, email y exp), backend/crates/user/src/application/mod.rs, backend/crates/user/src/infrastructure/repository/user_repository_impl.rs y backend/crates/user/src/infrastructure/di.rs (MockUserRepositoryImpl y DI::sign_in_use_case).

## Close Reason

Implementados find_user_by_email, SignInUseCase y DI en crates/user; 11/11 tests unitarios en verde
