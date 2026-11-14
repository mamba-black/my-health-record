---
schema_version: 9
id: cc-1lpo
title: Fechas con zona (TIMESTAMPTZ) y repositorios con el Model de toasty en vez de mapeo manual
priority: P2
status: done
deps:
- cc-g8bf
- cc-1ldq
- cc-gip9
- cc-3278
- cc-4d1k
- cc-voox
- cc-enfe
tags:
- backend
- persistence
owner: null
created_at: 2026-11-11T06:42:06.051628477Z
completed_at: 2026-11-14T03:16:39.928278954Z
acceptance:
- V9 convierte a TIMESTAMPTZ todas las columnas TIMESTAMP (user_account, clinical_network, organization, practitioner, patient, membership, person_link), interpretando los valores existentes como UTC
- Los toasty::Model leen filas sin pánico (hoy jiff::Timestamp choca con TIMESTAMP sin zona)
- Las lecturas de user_repository_impl usan el modelo UserAccount (get/filter tipados) y TryFrom<UserAccount> for User; desaparecen USER_COLUMNS, user_column_types, map_row_to_user y next_*_column
- 'Los atributos #[index]/#[unique] del modelo reflejan los índices de las migraciones (p. ej. #[unique(network_id, email)])'
- Se mantiene SQL a mano solo donde toasty no alcanza (búsqueda por texto, rangos/exclusión y bloqueos de scheduling, actualizaciones parciales si el update tipado no sirve)
- cargo test --workspace, clippy y e2e en verde
---

Hallado al revisar user_repository_impl.rs (2026-11-11): una consulta tipada de toasty 0.8 sobre user_account entra en pánico en value_stream.rs porque created_at/updated_at son TIMESTAMP y el modelo los declara jiff::Timestamp; con TIMESTAMPTZ devuelve el modelo bien. Por eso el código mapeaba filas a mano. Después seguir con los repositorios de administration.