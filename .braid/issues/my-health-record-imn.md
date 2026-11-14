---
schema_version: 9
id: my-health-record-imn
title: rolldown-vite rompe la navegación en bun run dev
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:19Z
completed_at: 2026-11-07T04:07:27Z
deps: []
tags:
  - bug
acceptance: []
---

Con rolldown-vite 7.1.16 el navegador carga dos instancias de @sveltejs/kit client.js (una con ?v=hash sin inicializar) y goto lanza "Cannot read properties of undefined (reading 'hash')": no funcionan redirección a /login, logout ni Volver al Login. Con node node_modules/vite/bin/vite.js (7.1.5) funciona. bun run test:e2e usa bun run dev si no hay servidor en :5173.

## Close Reason

bun run dev usa Vite normal; las e2e usan el mismo comando
