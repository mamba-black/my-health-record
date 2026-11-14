---
schema_version: 9
id: my-health-record-c2p
title: Pruebas de punta a punta pendientes del control de acceso por rol
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-07T22:44:18Z
deps: []
tags:
  - task
acceptance: []
---

Tarea 7.2 del change role-based-access-by-profile-data (archivado). Estos escenarios tienen la regla probada en authorize (core) pero no prueba de punta a punta porque su API no existe: Clínicas pendientes no se exploran (exploración), Paciente con datos completos confirma cita y Confirmar cita (citas), Médico con colegiatura sin confirmar (recetas), Historial previo sin verificar (vista de historia del paciente), Primera atención (atenciones). Agregar cada prueba cuando exista su API. Detalle en openspec/changes/archive/2026-11-07-role-based-access-by-profile-data/verification.md.

## Notes

Cubiertos por scheduling-appointments (ver openspec/changes/scheduling-appointments/verification.md): «Clínicas pendientes no se exploran» (disponibilidad pública vacía para clínica pendiente), «Paciente con datos completos confirma cita» y «Confirmar cita» con datos faltantes (I-grpc flujo_de_agenda_por_grpc y E2E scheduling.spec.ts «Paciente sin documento reserva y completa sus datos»). Siguen pendientes porque su API no existe: «Médico con colegiatura sin confirmar» (recetas), «Historial previo sin verificar» y «Primera atención» (historia clínica).
