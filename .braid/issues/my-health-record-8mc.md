---
schema_version: 9
id: my-health-record-8mc
title: La validación de id_token de Google acepta firmas inválidas y rechaza tokens reales
priority: P0
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T23:45:34Z
started_at: 2026-11-05T23:45:34Z
completed_at: 2026-11-06T04:58:16Z
deps: []
tags:
  - bug
acceptance: []
---

google-jwt-verify 0.3.0 descarta el booleano de Verifier::verify (jwk.rs), por lo que acepta firmas falsas (verificado con token sintético y kid real). Además IdPayload exige locale y otros campos, y falla con InvalidToken si Google no los envía. Reemplazar por jsonwebtoken validando firma contra JWKS, aud, iss y exp.

## Notes

Reemplazado google-jwt-verify por jsonwebtoken-google2 dentro de TokenServiceImpl, con pruebas. Falta probar el login con token real de Google tras reiniciar el backend. Limitación: la descarga de claves de la librería no tiene timeout y mantiene un mutex, y una clave desconocida se reporta como UnknownError (INTERNAL).

## Close Reason

jsonwebtoken-google2 verifica firma, aud, iss y exp. Validado con cuenta real de Google y con pruebas e2e.
