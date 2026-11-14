---
schema_version: 9
id: my-health-record-l6m
title: Guardar la fecha de nacimiento y la dirección del registro
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-07T04:03:34Z
completed_at: 2026-11-07T04:42:17Z
deps: []
tags:
  - bug
acceptance: []
---

SignUpForm exige la fecha de nacimiento y pide la dirección, pero identity.user_account no tiene esas columnas y save_user no las persiste en esa tabla. Por eso el aviso de datos faltantes solo cubre nombre y teléfono. Decidir dónde se guardan (migración) y agregarlas a missing_required_fields.

## Close Reason

birth_date y address en user_account; se guardan en registro y al completar
