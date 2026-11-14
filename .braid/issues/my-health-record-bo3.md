---
schema_version: 9
id: my-health-record-bo3
title: "cargo test -p clickcare no compila/pasa: basic_tests usa ulid y new_mock sin TokenService"
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-07T14:16:59Z
deps: []
tags:
  - bug
acceptance: []
---

Previo a refinery, en código commiteado: (1) bin/clickcare/tests/basic_tests.rs importa ulid, que no es dependencia de clickcare -> E0432 y aborta cargo test -p clickcare. (2) UserApiImpl::new_mock usa DBType::Mock sin DIOverrides.token_service, y el DI exige uno -> fallan sign_up_fails_with_invalid_user_id (2 casos) y user_service_server_tests en user_api_impl.rs.

## Notes

Además, cargo clippy --workspace -- -D warnings se detiene en core: import sin uso ContactPointSystem en crates/core/src/domain/fhir/person.rs:1 y dependencias sin uso en crates/core/Cargo.toml (thiserror, tracing, strum). Las pruebas unitarias rotas de user_api_impl.rs (new_mock) se eliminaron en el cambio role-based-access-by-profile-data: ese constructor ya no existe.

Ignorados mientras siguen pendientes (cargo test --workspace pasa):
- bin/clickcare/tests/basic_tests.rs: excluido de la compilación con #![cfg(false)] porque usa ulid. Para resolverlo: agregar ulid como dev-dependency o borrar el archivo.
- user_api_test administration_worker::worker_persists_organization_and_practitioner_for_a_clinic_owner: #[ignore], intermitente. Falla a veces en la primera corrida tras compilar porque el worker no llega a Done en 15 s. Investigar el arranque del worker (apalis) o subir el plazo.
