---
schema_version: 9
id: my-health-record-cy9
title: Podman falla con overlay sobre btrfs
priority: P3
status: open
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:20Z
deps: []
tags:
  - chore
acceptance: []
---

podman ps falla: 'overlay' is not supported over btrfs. DOCKER_HOST apunta a Podman, así que probablemente afecte a las pruebas con testcontainers (no verificado). Es del entorno local.
