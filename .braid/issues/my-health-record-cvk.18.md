---
schema_version: 9
id: my-health-record-cvk.18
title: "Rol recepcionista: solicitud de ingreso a clínica existente y aprobación"
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:40:56Z
started_at: 2026-11-08T02:40:57Z
completed_at: 2026-11-08T02:51:35Z
deps: []
tags:
  - feature
  - parent:my-health-record-cvk
acceptance: []
---

Nuevo MembershipRole Receptionist e intención de registro Recepcionista. El recepcionista no crea clínicas: al registrarse indica el enlace (subdominio) de una clínica existente y queda con membresía PENDING hasta que el dueño o un administrador la apruebe. Puede registrar pacientes sin cuenta, buscarlos y ver sus datos de contacto; no ve historia clínica ni configura la clínica. Incluye lista de personal de la clínica con botón Aprobar.

## Close Reason

Implementado: rol recepcionista con solicitud y aprobación, invitación ampliada, lista de personal y formulario de registro de pacientes sin cuenta. Pruebas unitarias e integración en verde.
