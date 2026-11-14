---
schema_version: 9
id: my-health-record-cvk.14
title: "Regla: sin horarios cruzados del médico entre clínicas de su red"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:12Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Un médico no puede tener franjas que se crucen entre clínicas de la misma red clínica. tasks.md 3.1, 5.1.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
