---
schema_version: 9
id: cc-e9mv
title: Eliminación o desactivación de clínicas demo y política de inactividad
priority: P2
status: open
type: design
deps:
- cc-xs9h
tags:
- clinic
- administration
owner: null
created_at: 2026-11-11T04:54:30.403035527Z
acceptance:
- 'Solo el OWNER cierra, reabre o fuerza (nuevas ProtectedAction CloseClinic/ReopenClinic en app_core::domain::access; ni ADMIN ni recepción), desde /my-clinics'
- 'Clínica vacía (sin pacientes, citas, horarios ni más miembros que el dueño): borrado físico inmediato en una transacción que también borra scheduling.* (hoy sin FK a organization) y el Person.link del dueño; el subdominio queda libre al instante'
- 'Clínica con datos: baja lógica CLOSED (closed_at, closed_by, close_reason, status previo); sin acceso operativo, oculta en /c/<subdominio>; las citas futuras BOOKED se cancelan y se avisa al paciente; sus recordatorios pendientes se cancelan'
- 'Subdominio de una clínica CLOSED en cuarentena (CLINIC_SUBDOMAIN_QUARANTINE_DAYS, 180 por defecto): nadie más puede tomarlo hasta que venza; al vencer se libera'
- 'Reapertura: el dueño reabre una clínica CLOSED y vuelve a su estado previo (PENDING_SETUP o ACTIVE) con sus miembros; las citas canceladas no se restauran; si el subdominio ya se liberó, debe elegir uno nuevo'
- 'Forzado: el dueño puede saltarse los plazos con confirmación explícita (escribir el subdominio): liberar ya el subdominio de una clínica CLOSED y borrar ya una demo sin esperar el TTL; queda auditado (forced, quién, cuándo); nunca borra datos con retención legal'
- 'TTL de demos: worker programado que borra solo clínicas PENDING_SETUP vacías a los 90 días de creadas, con avisos al dueño a los 60 y 83 días (depende de my-health-record-vxm); una clínica activa o con datos nunca se borra sola'
- 'La Organization raíz del dueño (clinical_network) nunca se borra con la clínica, y las cuentas de su red no se mueven ni se borran'
- 'Pruebas: unitarias en administration y core (matriz de authorize), integración contra Postgres (borrado sin huérfanos, cuarentena, forzado) y e2e de /my-clinics'
---

Hoy no existe un flujo para eliminar o desactivar clínicas, ni en el backend ni en el frontend.

## Validación contra el código (2026-11-11)

**La premisa «UNIQUE (owner_user_id) impide crear otra clínica» no es exacta.**

- `administration.organization` **no** tiene `UNIQUE (owner_user_id)` (V1). Lo único único por dueño es la Organization raíz: `uq_clinical_network_owner` sobre `administration.clinical_network` (V7), y por diseño el dueño tiene una sola raíz con muchas clínicas colgando de ella.
- El bloqueo real está en `CreateClinicUseCaseImpl::execute` (`crates/administration/src/application/create_clinic_usecase.rs:47`). Es idempotente **por dueño**: `find_id_by_owner_user_id` devuelve la primera clínica del usuario con `already_existed: true` en lugar de crear otra. Su comentario dice que «la base lo impide con UNIQUE (owner_user_id)», pero eso ya no es cierto.
- Además, el frontend solo crea clínicas al registrarse (`SignUpForm.svelte:114`, `createClinic`). `/my-clinics` no tiene el botón «Nueva clínica».

**Consecuencia:** borrar la clínica demo **no** sirve para crear otra. Con la idempotencia actual, al borrar la demo, `CreateClinic` sí crearía una nueva, pero eso sería un efecto lateral. Lo correcto es permitir varias clínicas por dueño, que es lo que el modelo de Organization raíz ya soporta. **Propuesta: separarlo en otro issue** (quitar la idempotencia por dueño o hacerla por subdominio, y agregar «Nueva clínica» en /my-clinics). Ese issue resuelve el problema de fondo y es independiente de este.

## Hechos que condicionan el diseño

- **Estados actuales:** `ClinicStatus` = `PENDING_SETUP | ACTIVE | SUSPENDED` (`crates/core/src/domain/access.rs:188`). `SUSPENDED` existe, pero ninguna RPC lo asigna.
- **Borrar por la base deja huérfanos.** Al borrar la `organization` sí se borran en cascada `practitioner`, `patient`, `membership` y, a través de `patient`, `patient_link`. En cambio, `scheduling.schedule`, `scheduling.appointment` y `scheduling.reminder` **no tienen FK** a `organization` (V8; `patient_id` es una referencia débil). Un `DELETE` directo los dejaría huérfanos, y el worker seguiría enviando recordatorios de una clínica que ya no existe.
- **Otras referencias:**
  - `identity.person_link` (`target_type = 'ORGANIZATION'`) no tiene FK. Hay que borrar el vínculo del dueño con la clínica.
  - Los trabajos de apalis pendientes (`administration` y `scheduling-reminders`) referencian ids.
  - La caché del `AccessContext` debe invalidarse.
- **Raíz y cuentas:** las cuentas creadas con el enlace de la clínica viven en la red raíz del dueño (`user_account.network_id`), no en la clínica. Al borrar o cerrar la clínica **no se borran ni se mueven** (regla «no mover cuentas entre redes»). Siguen pudiendo iniciar sesión sin subdominio.
- **Subdominio:** `organization.subdomain` es `UNIQUE` global. Liberarlo tiene un riesgo: un paciente con el enlace viejo caería en la clínica de otra persona.
- **Retención legal:**
  - Pacientes y citas son datos personales (Ley 29733) y la cita es un dato de salud sensible.
  - Cuando exista la historia clínica (`crates/clinical`), su conservación es obligatoria por años (NTS 139-MINSA). No podrá borrarse físicamente al cerrar la clínica.
  - Validar con asesoría legal junto con `my-health-record-pvo`.

## Opciones

| Opción | Cuándo aplica | Qué se libera | Riesgo |
|---|---|---|---|
| **A. Borrado físico** | Clínica vacía: sin pacientes, citas, horarios ni más miembros que el dueño. | Subdominio y filas. | Bajo, porque no hay datos de terceros. |
| **B. Baja lógica `CLOSED`** | Clínica con datos. | Nada al inicio; el subdominio queda reservado. | Bajo, porque se puede revertir y se conserva lo exigido. |
| **C. Suspensión `SUSPENDED`** | Pausa temporal (impago, decisión de la plataforma). | Nada. | Hoy no tiene RPC. |

**Elegido:** A + B, con forzado (ver «Decisiones»).

- La RPC `CloseClinic` decide sola entre las dos opciones:
  - si la clínica está vacía, la borra (A);
  - si tiene datos, la deja `CLOSED` (B), con `closed_at`, `closed_by` y `close_reason`.
- `SUSPENDED` queda reservado para la plataforma, fuera de este issue.
- En `authorize`, una clínica `CLOSED` no permite ninguna acción salvo la lectura del dueño, para exportar.

**TTL de demos:** borrar automáticamente **solo** clínicas `PENDING_SETUP` vacías creadas hace más de N días. El dueño recibe dos avisos por correo antes del borrado, lo que depende de `my-health-record-vxm`. Una clínica activa o con datos nunca se borra automáticamente.

## Decisiones (2026-11-11, con el humano)

1. **Varias clínicas por dueño:** van en otro issue, `cc-xs9h`, y se hacen primero. Este issue depende de él.
2. **Subdominio de una clínica `CLOSED`:** queda en cuarentena. Propuesta: 180 días, configurable con `CLINIC_SUBDOMAIN_QUARANTINE_DAYS`. Mientras dura, nadie más puede tomarlo y el enlace viejo muestra «Clínica cerrada» en lugar de la página pública.
3. **Citas futuras al cerrar:** se cancelan y se avisa al paciente. Mientras no exista el correo (`my-health-record-vxm`), el aviso solo queda en el log. Siguiendo la política de notificaciones, ante la duda el aviso se repite.
4. **Reapertura:** sí. La clínica vuelve a su estado previo (`PENDING_SETUP` o `ACTIVE`) con sus miembros. Las citas canceladas no se restauran. Si el subdominio ya se liberó, el dueño debe elegir uno nuevo.
5. **TTL de demos:** 90 días, con avisos a los 60 y 83 días.

### Forzado (pedido del humano)

El dueño puede saltarse los plazos máximos:

- **Liberar ya el subdominio** de una clínica `CLOSED`, sin esperar la cuarentena. Después ya no puede reabrirla con ese subdominio.
- **Borrar ya una demo** vacía, sin esperar el TTL. Es el mismo borrado físico de una clínica vacía.

Condiciones:

- Pide confirmación explícita: escribir el subdominio, como el borrado de un repositorio en GitHub. La pantalla advierte que los enlaces viejos podrían llevar a otra clínica.
- Queda auditado: `forced`, quién y cuándo.
- **No se salta la retención legal.** Una clínica con datos nunca se borra físicamente, ni forzando: solo pasa a `CLOSED` y se le libera el subdominio. La purga de datos personales vencidos, cuando corresponda, es otro tema (`my-health-record-pvo`).

## Fuera de alcance

- Borrar la Organization raíz o la cuenta del dueño.
- Exportar los datos de la clínica, que podría ser un issue aparte.
