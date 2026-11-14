---
schema_version: 9
id: my-health-record-vxm
title: Servidor de correo para notificaciones
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-07T23:22:25Z
deps: []
tags:
  - task
acceptance: []
---

Falta un servidor/proveedor SMTP (dominio de envío, SPF/DKIM/DMARC, credenciales SMTP_*) para los recordatorios de citas y demás avisos. Sin él, el sistema solo registra los avisos en el log. Relacionado con el change scheduling-appointments.

## Notes

scheduling-appointments deja listo: el puerto NotificationSender y SmtpNotificationSender (lettre, STARTTLS) en bin/clickcare/src/infrastructure/scheduling.rs, activado con SMTP_HOST/SMTP_PORT/SMTP_USER/SMTP_PASSWORD/SMTP_FROM. Sin SMTP_HOST solo se registra en el log. Falta: el servidor o proveedor de correo, el dominio remitente (SPF/DKIM) y el texto definitivo.
