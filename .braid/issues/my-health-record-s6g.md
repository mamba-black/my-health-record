---
schema_version: 9
id: my-health-record-s6g
title: Corregir warnings que rompen clippy -D warnings
priority: P3
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:20Z
completed_at: 2026-11-08T03:47:12Z
deps: []
tags:
  - chore
acceptance: []
---

Unused import ContactPointSystem en crates/core detiene cargo clippy con -D warnings; además clickcare tiene 5 dependencias sin usar (clinic_admin, fake, patient, regex, entre otras).

## Close Reason

vrw: Organization raíz por dueño (partOf, Person.link, sin red compartida, sign-in sin subdominio), con pruebas de integración. s6g: clippy -D warnings en 0 errores.
