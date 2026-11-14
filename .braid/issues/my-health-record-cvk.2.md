---
schema_version: 9
id: my-health-record-cvk.2
title: "Médico: horario semanal por clínica"
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

El médico arma su horario en «Mi agenda» (días, franjas, duración de la cita) y los cupos salen solos de ese horario. Requiere membresía activa y colegiatura confirmada. tasks.md 3.1, 3.2, 5.1, 9.4.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
