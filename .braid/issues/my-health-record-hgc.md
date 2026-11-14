---
schema_version: 9
id: my-health-record-hgc
title: sqlx ignora channel_binding=require en PG_URL
priority: P3
status: open
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:20Z
deps: []
tags:
  - task
acceptance: []
---

sqlx_postgres avisa 'ignoring unrecognized connect parameter channel_binding', de modo que no se aplica en sus conexiones. Revisar si hace falta o quitarlo de la URL.
