---
schema_version: 9
id: my-health-record-99t
title: Validar en el backend los datos obligatorios de SignUp
priority: P2
status: open
owner: miuler@gmail.com
created_at: 2026-11-07T04:03:34Z
deps: []
tags:
  - bug
acceptance: []
---

SignUp acepta nombre y teléfono vacíos; la obligatoriedad solo existe en SignUpForm. Por eso existen cuentas vacías. Reutilizar la validación de CompleteProfileUseCase.
