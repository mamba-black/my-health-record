---
schema_version: 9
id: my-health-record-suz
title: Endpoints sin autenticación
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:19Z
started_at: 2026-11-08T06:21:17Z
completed_at: 2026-11-08T06:25:36Z
deps: []
tags:
  - bug
acceptance: []
---

SearchPatient y /api/v1/queues (apalis-board) responden sin credenciales, con CORS * y escuchando en 0.0.0.0:50051. Definir autenticación antes de desplegar con datos reales.

## Notes

Avance en 18a7ef6: ClinicApi y PatientApi exigen sesión propia (Bearer) y autorizan por membresía. Queda pendiente: /api/v1/queues (apalis-board) sin credenciales, CORS * y escucha en 0.0.0.0:50051.

## Close Reason

Tablero de colas (apalis-board) detrás de HTTP Basic (QUEUE_BOARD_USER/QUEUE_BOARD_PASSWORD) y sin publicar si no hay credenciales; CORS con lista de orígenes (CORS_ALLOWED_ORIGINS, admite https://*.dominio), sin '*'; escucha en 127.0.0.1 por defecto (CLICKCARE_HOST); GET /health sin credenciales. bin/clickcare/src/infrastructure/http.rs. La reflexión gRPC sigue pública.
