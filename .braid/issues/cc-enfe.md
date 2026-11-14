---
schema_version: 9
id: cc-enfe
title: Model de toasty en patient_link_repository_impl
priority: P2
status: done
deps: []
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T07:53:52.306148728Z
completed_at: 2026-11-14T02:35:34.045072349Z
acceptance:
- Model PatientLink; active_link, insert y close con el Model
- mark_document_verified con update() tipado del Model de patient
- Joins documentados como SQL a mano por necesidad
- cargo test --workspace, clippy y e2e en verde
---

No tiene Model. active_link, insert, close y mark_document_verified mapean a mano. records_of_account (join) y account_of (left join + coalesce) se quedan en SQL.