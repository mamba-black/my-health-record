---
schema_version: 9
id: my-health-record-7em
title: El worker de administration crea la clínica sin la membresía de dueño
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-07T22:41:46Z
started_at: 2026-11-07T22:41:49Z
completed_at: 2026-11-07T22:43:39Z
deps: []
tags:
  - bug
acceptance: []
---

handle_user_created_event (create_clinic=true) crea organization, practitioner y patient, pero no la membresía OWNER ACTIVE. Desde role-based-access-by-profile-data los permisos salen de administration.membership, así que el dueño queda sin acceso a su clínica. CreateClinic sí la crea; V4 solo rellenó las existentes.

## Close Reason

Implementado y probado
