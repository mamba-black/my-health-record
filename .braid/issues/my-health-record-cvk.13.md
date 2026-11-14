---
schema_version: 9
id: my-health-record-cvk.13
title: "Regla: nunca dos citas en el mismo horario de un médico"
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

Garantizado en la base con EXCLUDE USING gist, también con reservas simultáneas: una tiene éxito y la otra recibe «el cupo ya no está disponible». tasks.md 1.2, 4.1, 5.3.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
