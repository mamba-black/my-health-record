---
schema_version: 9
id: my-health-record-cvk.3
title: "Médico: excepciones (vacaciones y feriados)"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:08Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

El médico marca vacaciones o feriados y esos días no se ofrecen cupos. Se rechaza si ya hay citas en el rango. tasks.md 3.2, 5.1, 9.4.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
