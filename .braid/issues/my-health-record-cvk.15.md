---
schema_version: 9
id: my-health-record-cvk.15
title: "Regla: aislamiento por red clínica"
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

Ninguna consulta ni restricción de agenda cruza redes (network_id en tablas y EXCLUDE). Ver memoria aislamiento-por-red-clinica. tasks.md 1.2, 4.1, 10.2.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
