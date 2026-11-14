---
schema_version: 9
id: my-health-record-uka
title: Eliminar tablas legadas duplicadas clinic/patient/patient_search
priority: P3
status: open
owner: miuler@gmail.com
created_at: 2026-11-07T14:43:25Z
deps: []
tags:
  - chore
acceptance: []
---

En Neon (base clickcare) las tablas legadas clinic, patient y patient_search existen en tres esquemas: clickcare, identity y public. Las de public probablemente las creó la migración V1 de refinery (2026-11-07); las de clickcare e identity vienen de ejecuciones manuales del antiguo ddl/table.sql. Ningún código Rust las usa (administration.patient las reemplazó). Hacer: confirmar que están vacías o que sus datos no importan, y quitarlas con una migración nueva (V3__...) con DROP TABLE IF EXISTS para los tres esquemas; nunca editando V1. Revisar también si V1 debe dejar de crearlas en bases nuevas (solo vía una migración posterior, no editando V1).
