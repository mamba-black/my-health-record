---
schema_version: 9
id: my-health-record-cvk.10
title: "Recepción: reservar a nombre de un paciente (con o sin cuenta)"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:23:10Z
completed_at: 2026-11-08T04:28:36Z
deps:
  - my-health-record-cvk.1
  - my-health-record-cvk.17
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Recepción saca citas por teléfono a nombre de un paciente de la clínica o registra uno nuevo sin cuenta (nombres, apellido paterno, documento, teléfono). Hoy NO existe formulario para registrar pacientes desde la clínica: el RPC PatientApi.Save es un stub que no persiste y no hay pantalla. Este issue incluye ese formulario. tasks.md 5.4, 9.5.

## Notes

El formulario de paciente sin cuenta ya existe (cvk.17, /patients/new y PatientApi.RegisterPatient). Falta reservar la cita a su nombre.

## Close Reason

Implementado en el cambio scheduling-appointments (36/36 tareas): backend, API gRPC, worker de recordatorios, frontend y e2e. Ver openspec/changes/scheduling-appointments/verification.md.
