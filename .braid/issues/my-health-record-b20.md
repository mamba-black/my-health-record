---
schema_version: 9
id: my-health-record-b20
title: Versionar bun.lock y Cargo.lock
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:20Z
deps: []
tags:
  - chore
acceptance: []
---

El patrón *.lock en .gitignore (línea 192) excluye bun.lock y Cargo.lock del repo, por lo que las instalaciones no son reproducibles.
