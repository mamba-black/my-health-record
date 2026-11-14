---
schema_version: 9
id: my-health-record-md1
title: Canal de notificaciones por WhatsApp (y SMS)
priority: P3
status: open
owner: miuler@gmail.com
created_at: 2026-11-07T23:22:26Z
deps: []
tags:
  - task
acceptance: []
---

Agregar un proveedor de WhatsApp Business (y opcionalmente SMS) detrás del puerto NotificationSender, para pacientes sin correo o que prefieran ese canal. Requiere elegir proveedor, plantillas aprobadas y consentimiento del paciente (Ley 29733).

## Notes

scheduling-appointments deja el puerto NotificationSender (crates/scheduling/src/application/integration.rs): un canal nuevo (WhatsApp/SMS) es otro adaptador que se elige en scheduling_adapters::notification_sender(). Falta resolver el teléfono del paciente (hoy el aviso usa el correo de la cuenta) y el proveedor.
