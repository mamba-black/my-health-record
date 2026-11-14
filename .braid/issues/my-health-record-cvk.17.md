---
schema_version: 9
id: my-health-record-cvk.17
title: "Recepción: formulario para registrar paciente sin cuenta"
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:33:45Z
started_at: 2026-11-08T02:33:46Z
completed_at: 2026-11-08T02:51:35Z
deps: []
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Hoy no existe forma de que la clínica registre un paciente: PatientApi.Save es un stub y no hay pantalla. Agregar RPC RegisterPatient (dueño o administrador activo, clínica activa), user_id nulo en administration.patient, rechazo por documento repetido en la clínica, y el formulario en el frontend. Requisito «Paciente sin cuenta» de identity/patient-account-linking.

## Close Reason

Implementado: rol recepcionista con solicitud y aprobación, invitación ampliada, lista de personal y formulario de registro de pacientes sin cuenta. Pruebas unitarias e integración en verde.
