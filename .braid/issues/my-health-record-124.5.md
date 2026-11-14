---
schema_version: 9
id: my-health-record-124.5
title: "Frontend FSD (Features, Pages & Routes): Flujo LoginForm, SignUpForm y página /signup"
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-10-18T23:41:56Z
started_at: 2026-10-19T04:29:47Z
completed_at: 2026-10-19T04:31:30Z
deps:
  - my-health-record-124.4
tags:
  - task
  - parent:my-health-record-124
acceptance:
  - "bun run check y bun run build en frontendts/ finalizan con éxito."
---

Integrar la llamada a userApi.signIn en LoginForm.svelte y construir el formulario de Perfilado Progresivo (SignUpForm.svelte) y la página /signup.

## Design

Archivos: frontendts/src/features/auth/ui/LoginForm.svelte (invocar userApi.signIn al recibir credential de Google, iniciar sesión si OK o guardar pendingRegistration y redirigir a /signup si NOT_FOUND), frontendts/src/features/auth/ui/SignUpForm.svelte (pre-llenar con datos de Google, elegir SignUpIntent, teléfono, fecha de nacimiento, DNI opcional, enviar userApi.signUp con UUIDv7), frontendts/src/pages/signup/{ui/SignUpPage.svelte,index.ts}, frontendts/src/routes/signup/+page.svelte y frontendts/src/app/ui/AppLayout.svelte (PUBLIC_ROUTES).

## Close Reason

LoginForm, SignUpForm, SignUpPage y ruta /signup implementados; bun run check y bun run build en verde
