---
schema_version: 9
id: my-health-record-bwy
title: El test basic_tests de clickcare no compila
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-06T04:58:16Z
completed_at: 2026-11-07T22:44:06Z
deps: []
tags:
  - bug
acceptance: []
---

bin/clickcare/tests/basic_tests.rs importa ulid, que no es dependencia del crate, por lo que cargo test y clippy con --all-targets fallan en clickcare.

## Close Reason

Duplicado de my-health-record-bo3
