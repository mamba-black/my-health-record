---
schema_version: 9
id: cc-1ldq
title: Model de toasty en organization_repository_impl
priority: P2
status: open
deps: []
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T07:53:52.189370344Z
acceptance:
- Lecturas con filter_by_id / filter_by_subdomain (#[unique]) / owner_user_id
- update_setup con update() tipado
- Sin column_types ni mapeo manual
- cargo test --workspace, clippy y e2e en verde
---

find_by_id mapea 12 columnas con coalesce; find_id_by_owner_user_id y find_organization_and_network_by_subdomain usan SQL; update_setup es un UPDATE a mano.