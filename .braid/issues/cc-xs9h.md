---
schema_version: 9
id: cc-xs9h
title: 'Varias clínicas por dueño: quitar la idempotencia por dueño de CreateClinic y agregar «Nueva clínica» en /my-clinics'
priority: P1
status: open
deps: []
tags:
- clinic
- administration
owner: null
created_at: 2026-11-11T05:18:20.168747083Z
acceptance:
- CreateClinic crea una clínica nueva bajo la Organization raíz del dueño aunque ya tenga otras; ya no devuelve la primera con already_existed
- 'La idempotencia ante reintentos se mantiene por subdominio: si el mismo dueño repite la creación con el mismo subdominio, se devuelve esa clínica; si el subdominio es de otro, SubdomainAlreadyExists'
- El worker de administración (signup con createClinic) sigue siendo idempotente ante entregas repetidas de la cola
- Se corrige el comentario de create_clinic_usecase.rs que menciona un UNIQUE (owner_user_id) inexistente; organization_repository.find_id_by_owner_user_id se elimina si queda sin uso
- /my-clinics tiene «Nueva clínica» (nombre, subdominio, RUC opcional) solo para quien ya es dueño; la clínica nace PENDING_SETUP con su membresía OWNER y su Person.link
- 'Pruebas: unitarias del caso de uso (segunda clínica, reintento con el mismo subdominio), integración gRPC y e2e del formulario'
---

Separado de cc-e9mv (2026-11-11). El modelo ya permite varias clínicas por dueño: la única unicidad por dueño es la Organization raíz (uq_clinical_network_owner, V7); administration.organization no tiene UNIQUE (owner_user_id). Lo que lo impide es CreateClinicUseCaseImpl::execute (crates/administration/src/application/create_clinic_usecase.rs:47), idempotente por dueño vía find_id_by_owner_user_id, y que el frontend solo crea clínicas en el registro (SignUpForm.svelte:114). Revisar quién llama a CreateClinic (RPC ClinicApi y worker de signup) antes de cambiar la idempotencia.
