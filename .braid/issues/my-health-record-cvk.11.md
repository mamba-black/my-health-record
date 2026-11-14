---
schema_version: 9
id: my-health-record-cvk.11
title: "Recepción: check-in, DNI verificado y no asistió"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:11Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Marcar «llegó» el mismo día; si mostró el DNI físico queda verificado (MATCH/MISMATCH). Marcar «no asistió» después del inicio. tasks.md 6.1, 6.2, 9.5.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
