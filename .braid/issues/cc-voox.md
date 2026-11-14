---
schema_version: 9
id: cc-voox
title: Model de toasty en person_link_repository_impl
priority: P2
status: done
deps: []
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T07:53:52.283229005Z
completed_at: 2026-11-14T02:32:53.880921683Z
acceptance:
- links_of con el Model (PK compuesta user_id, target_type, target_id)
- link sigue siendo idempotente
- cargo test --workspace, clippy y e2e en verde
---

links_of lee filas a mano; link usa insert ... on conflict do nothing.