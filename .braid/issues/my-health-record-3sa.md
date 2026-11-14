---
schema_version: 9
id: my-health-record-3sa
title: PatientApi usa datos mock en vez de la base
priority: P2
status: done
owner: miuler@gmail.com
created_at: 2026-11-05T19:12:19Z
completed_at: 2026-11-07T22:44:06Z
deps: []
tags:
  - task
acceptance: []
---

PatientApiImpl::default() no recibe repositorio; SearchPatient y GetPatientById devuelven datos de get_mock_patients/demo. Conectar a persistencia.

## Close Reason

Resuelto en 18a7ef6: PatientApi usa PatientRepository por clínica, con autorización
