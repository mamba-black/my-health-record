---
schema_version: 9
id: my-health-record-cvk.6
title: "Paciente: reservar cita con «Completa tus datos»"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:09Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
  - my-health-record-cvk.5
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

El paciente elige un horario y reserva. Si le faltan datos (p. ej. DNI) se abre la ventana «Completa tus datos» y la reserva se reintenta sola. La cita nace confirmada. tasks.md 3.3, 5.3, 9.2.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
