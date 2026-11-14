---
schema_version: 9
id: my-health-record-8hv
title: Quitar credenciales de BD de logs y errores
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:19Z
deps: []
tags:
  - task
acceptance: []
---

La URL con contraseña sigue en nivel trace (administration y user config) y en el mensaje de error de administration/src/infrastructure/config/mod.rs:169. Decidir si enmascararla.
