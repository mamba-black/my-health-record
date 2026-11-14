---
schema_version: 9
id: my-health-record-cvk
title: Agenda y citas (scheduling-appointments)
priority: P1
status: done
type: meta
owner: miuler@gmail.com
created_at: 2026-11-08T02:22:51Z
completed_at: 2026-11-08T04:28:37Z
deps: []
tags: []
acceptance: []
---

Implementar el cambio OpenSpec openspec/changes/scheduling-appointments: horario del médico, reserva de citas por paciente y recepción, check-in, recordatorios y vínculo de pacientes sin cuenta, aislado por red clínica. Las tareas detalladas están en tasks.md; cada hijo indica qué tareas cubre.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
