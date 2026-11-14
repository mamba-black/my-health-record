---
schema_version: 9
id: cc-4d1k
title: Model de toasty en patient_repository_impl
priority: P2
status: open
deps: []
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T07:53:52.261238081Z
acceptance:
- exists_by_user_id y find_by_id con el Model; find_by_document si no filtra dentro del texto de person
- search documentado como SQL a mano por necesidad
- cargo test --workspace, clippy y e2e en verde
---

exists_by_user_id, find_by_id y find_by_document mapean filas a mano (map_patient_row). search se queda en SQL (like + not exists con join).