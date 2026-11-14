---
schema_version: 9
id: cc-g8bf
title: Model de toasty en clinical_network_repository_impl
priority: P2
status: done
deps: []
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T07:53:52.164734026Z
completed_at: 2026-11-14T02:21:47.934482956Z
acceptance:
- find_by_id y find_by_owner usan los buscadores tipados del Model
- Sin column_types ni mapeo manual de filas
- cargo test --workspace, clippy y e2e en verde
---

find_one usa toasty::sql::query con column_types y mapeo a mano. Pasar al Model (filter_by_id, owner_user_id único) y TryFrom.