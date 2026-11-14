---
schema_version: 9
id: my-health-record-124.3
title: "Backend bin/clickcare: Endpoint gRPC UserApi::sign_in, CORS gRPC-Web y tests de integración"
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-10-18T23:41:55Z
started_at: 2026-10-19T04:21:39Z
completed_at: 2026-10-19T04:26:30Z
deps:
  - my-health-record-124.1
  - my-health-record-124.2
tags:
  - task
  - parent:my-health-record-124
acceptance:
  - "cargo test -p clickcare --test user_api_test sign_in y cargo clippy --workspace --all-targets -- -D warnings pasan exitosamente."
---

Conectar SignInUseCase en UserApiImpl resolviendo network_id por subdominio, configurar encabezados CORS para gRPC-Web y habilitar los 3 tests de integración en user_api_test.rs.

## Design

Archivos: backend/bin/clickcare/src/infrastructure/grpc/user_api_impl.rs (inyectar sign_in_use_case, resolver network_id con SubdomainResolver, mapear InvalidToken -> Status::unauthenticated, UserNotFound -> Status::not_found, UnknownError -> Status::internal), configurar CORS gRPC-Web (content-type, x-grpc-web, x-user-agent, x-subdomain, grpc-status, grpc-message) y actualizar backend/bin/clickcare/tests/user_api_test.rs (sign_in_succeeds_with_valid_credentials, sign_in_fails_when_user_not_found, sign_in_fails_when_id_token_is_invalid).

## Close Reason

UserApi::sign_in, CORS gRPC-Web y 3 tests de integración en user_api_test.rs implementados y en verde
