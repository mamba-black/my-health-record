---
schema_version: 9
id: my-health-record-cvk.5
title: "Paciente: página pública de la clínica con cupos libres"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:09Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.2
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Ruta /c/[subdomain]: médicos activos de la clínica y sus horarios libres, sin necesidad de cuenta. Solo clínicas activas; ventana de 60 días. tasks.md 5.2, 9.2.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
