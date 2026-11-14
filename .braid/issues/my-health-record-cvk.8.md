---
schema_version: 9
id: my-health-record-cvk.8
title: "Paciente: recordatorio un día antes"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:10Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Worker scheduling-reminders con apalis: un aviso por cita (REMINDER_LEAD_HOURS, 24 h por defecto), al menos una vez (antes repetido que perdido). Mientras no exista servidor de correo (my-health-record-vxm) solo se registra en el log. tasks.md 7.1–7.3.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
