---
schema_version: 9
id: my-health-record-cvk.12
title: "Recepción: vincular paciente sin cuenta con su cuenta"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:11Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.10
  - my-health-record-cvk.1
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Sugerir el vínculo cuando coincide el documento en la misma red (nunca automático); recepción confirma en persona con el DNI a la vista; la cuenta ve citas e historia anteriores. El dueño puede deshacerlo con motivo. tasks.md 6.3–6.5, 9.5.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
