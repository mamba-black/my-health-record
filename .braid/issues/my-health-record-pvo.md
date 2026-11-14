---
schema_version: 9
id: my-health-record-pvo
title: Validación legal (Ley 29733) del recordatorio de citas y de la vinculación de pacientes
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-08T04:28:29Z
deps: []
tags:
  - task
acceptance: []
---

El recordatorio por correo no lleva motivo ni datos clínicos, pero fecha, médico y clínica revelan que hay una cita (dato de salud sensible). La vinculación de un paciente registrado sin cuenta con una cuenta expone sus citas y su historia a esa cuenta: hoy la confirma recepción con el documento físico a la vista, queda auditada (patient_link: linked_by/linked_at, unlinked_by/unlinked_at/unlink_reason) y el dueño puede deshacerla. Validar con asesoría legal: consentimiento para el canal de correo, contenido mínimo del aviso, base legal y registro de la vinculación.
