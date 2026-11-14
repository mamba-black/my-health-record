---
schema_version: 9
id: my-health-record-vrw
title: La clínica nueva crea su propia red, distinta de la red de la cuenta de su dueño
priority: P1
status: done
owner: miuler@gmail.com
created_at: 2026-11-08T02:50:53Z
started_at: 2026-11-08T03:37:08Z
completed_at: 2026-11-08T03:47:12Z
deps: []
tags:
  - bug
acceptance: []
---

CreateClinicUseCase (sin network_id) y el worker user_created_handler crean siempre una ClinicalNetwork nueva para la clínica, mientras que la cuenta del dueño y del personal queda en la red resuelta por el subdominio al registrarse (DEFAULT_DEV_NETWORK_ID sin subdominio). Efectos: (1) RequestMembership responde NotFound en local porque la cuenta del recepcionista está en otra red que la clínica; (2) InviteMember busca al invitado en la red de la cuenta del dueño, no en la de la clínica; (3) el frontend no envía x-subdomain, así que en desarrollo todas las cuentas caen en la red por defecto. Decidir: la clínica nace en la red de su dueño, o las cuentas del personal se crean/inician sesión siempre por el subdominio de la clínica. Ver memoria aislamiento-por-red-clinica.

## Design

Decisión del usuario (2026-11-07): la red es la Organization raíz del dueño (FHIR R4), no un atributo suelto. 1) Cada dueño tiene una única Organization raíz (empresa/grupo), creada al registrarse como dueño o, a más tardar, al crear su primera clínica; no depende de la red del enlace ni de una red por defecto. 2) Cada clínica es una Organization con partOf = raíz del dueño. 3) La cuenta del dueño se vincula a la raíz y a sus clínicas con Person.link. 4) El personal se vincula a la clínica con PractitionerRole (membresía). 5) La red de una clínica se deriva de su raíz; invitaciones y solicitudes buscan solo dentro de esa raíz. 6) Aislamiento también en local: sin red por defecto compartida entre dueños. 7) No se mueven cuentas entre redes: la raíz existe desde el inicio y cuenta y clínicas apuntan a ella.

## Close Reason

vrw: Organization raíz por dueño (partOf, Person.link, sin red compartida, sign-in sin subdominio), con pruebas de integración. s6g: clippy -D warnings en 0 errores.
