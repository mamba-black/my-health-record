---
schema_version: 9
id: my-health-record-cvk.1
title: "Base: esquema V6, crate scheduling y acciones protegidas"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:07Z
started_at: 2026-11-08T03:54:39Z
completed_at: 2026-11-08T04:28:36Z
deps: []
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Base técnica para todo lo demás: migración V6__agenda.sql (esquema scheduling, btree_gist, patient_link), crate crates/scheduling, nuevas ProtectedAction en app_core y en el proto, puertos y adaptadores. tasks.md 1.1–1.3, 2.1–2.2, 4.1–4.3, 8.1–8.2, 9.1.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
