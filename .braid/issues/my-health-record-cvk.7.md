---
schema_version: 9
id: my-health-record-cvk.7
title: "Paciente: «Mis citas», cancelar y reprogramar"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:09Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Ruta /appointments: citas de todas las clínicas de la red, cancelar antes del inicio y reprogramar a otro cupo del mismo médico en un solo paso. tasks.md 5.5, 5.6, 9.3.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
