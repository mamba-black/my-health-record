---
schema_version: 9
id: cc-gip9
title: Model de toasty en practitioner_repository_impl
priority: P2
status: done
deps: []
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T07:53:52.211279443Z
completed_at: 2026-11-14T02:26:18.955963664Z
acceptance:
- find_id_by_user_id con buscador tipado
- cargo test --workspace, clippy y e2e en verde
---

find_id_by_user_id usa SQL a mano.