# Verificación

Cada escenario de los specs del cambio y la prueba que lo cubre. Abreviaturas:

- **U-dom**: `backend/crates/scheduling/src/domain/*.rs` (pruebas unitarias del dominio).
- **U-app**: `backend/crates/scheduling/src/application/tests.rs` (casos de uso con dobles).
- **U-acc**: `backend/crates/core/src/domain/access.rs`, matriz `matriz_de_acciones`.
- **I-repo**: `backend/bin/clickcare/tests/scheduling/mod.rs`, módulo `repositories` (Postgres real).
- **I-grpc**: `backend/bin/clickcare/tests/scheduling/mod.rs`, `api::flujo_de_agenda_por_grpc` (gRPC, Postgres y worker reales).
- **I-ac**: `backend/bin/clickcare/tests/user_api_test.rs`, módulo `access_control`.
- **E2E**: `frontendts/e2e/scheduling.spec.ts` y `frontendts/e2e/role-access.spec.ts`.

Comandos: `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bun run check` y `bun run test:e2e --repeat-each=2` (98/98 contra un Postgres local).

## scheduling/practitioner-availability

| Escenario | Prueba |
|---|---|
| Médico define su horario | U-dom `medico_define_su_horario`; E2E «El médico publica su horario…» |
| Médico sin colegiatura confirmada | U-app `horario_sin_colegiatura`; U-acc `horario_sin_colegiatura` |
| Franjas que se cruzan | U-dom `franjas_que_se_cruzan` |
| Cruce con otra clínica de la red | U-app `cruce_con_otra_clinica_de_la_red`; U-dom `cruce_con_otra_clinica` |
| Clínicas de redes distintas | U-app `cruce_con_otra_clinica_de_la_red` (última parte) |
| Vacaciones sin citas | U-dom `vacaciones_sin_citas` |
| Excepción con citas reservadas | U-app `excepcion_con_citas` |
| Se acorta el horario | U-dom `se_acorta_el_horario` |
| Paciente abre el enlace de la clínica | U-app `disponibilidad_publica`; I-grpc (ListPractitioners) |
| Clínica pendiente | U-app `disponibilidad_publica` |
| Médico invitado sin activar | U-app `disponibilidad_publica` |
| Cupo de hoy ya pasado | U-dom `cupo_de_hoy_ya_pasado` |
| Fuera de la ventana de reserva | U-dom `fuera_de_la_ventana_de_reserva` |

## scheduling/appointment-booking

| Escenario | Prueba |
|---|---|
| Paciente con datos completos | U-app `paciente_reserva`; I-grpc |
| Paciente sin documento | U-app `paciente_sin_documento`; I-grpc (`FAILED_PRECONDITION` con `ActionRequirements`) |
| Completa sus datos y continúa | E2E «Paciente sin documento reserva y completa sus datos…» |
| Identidad sin verificar | U-app `paciente_reserva` |
| Reservas simultáneas | I-repo `reservas_simultaneas` |
| Cupo ocupado | U-app `cupo_ocupado_y_clinica_pendiente`; I-grpc (`ABORTED`) |
| Recepción reserva para un paciente sin cuenta | U-app `recepcion_reserva_sin_cuenta`; I-grpc; E2E «Recepción reserva por un paciente sin cuenta…» |
| Recepción sin los datos del paciente | U-app `recepcion_reserva_sin_cuenta`; I-grpc (`INVALID_ARGUMENT`) |
| Médico reserva en la agenda de otro médico | U-app `medico_reserva_en_otra_agenda` |
| Primera cita en la clínica | U-app `un_solo_registro_por_paciente`; I-grpc |
| Segunda cita | U-app `un_solo_registro_por_paciente`; I-grpc (mismo `patient_id`) |
| Paciente revisa sus citas | U-app `consultas_de_citas` (dos clínicas de la red); I-grpc |
| Médico revisa su agenda | U-app `consultas_de_citas` |
| Paciente cancela | U-app `paciente_cancela`; I-grpc; E2E «Cancelar libera el cupo…» |
| Cita ya empezada | U-dom `cancelar_antes_y_despues_del_inicio` |
| Cita ajena | U-app `citas_ajenas`; I-grpc (`NOT_FOUND`) |
| Reprogramación exitosa | U-app `reprogramar`; I-grpc; E2E «…reprogramar mueve la cita» |
| Nuevo cupo ocupado | U-app `reprogramar`; I-repo `reprogramar_a_un_cupo_ocupado` |
| Cambio de médico | U-dom `reprogramar_con_otro_medico`; U-app `reprogramar` |

## scheduling/check-in

| Escenario | Prueba |
|---|---|
| Paciente llega | U-app `llegada` |
| Llegada en otro día | U-app `llegada`; I-grpc |
| Médico intenta registrar la llegada | U-app `llegada`; U-acc `llegada_medico`; I-grpc (`PERMISSION_DENIED`) |
| Recepción verifica el DNI | U-app `verificacion_del_documento`; I-grpc; E2E «Recepción registra la llegada con el DNI…» (el perfil lo muestra no editable) |
| El documento no coincide | U-app `verificacion_del_documento` |
| Paciente sin cuenta | U-app `verificacion_del_documento` |
| Paciente no llega | U-app `no_asistio`; U-dom `no_asistio_solo_despues_del_inicio` |
| Antes de la hora | U-app `no_asistio` |

## scheduling/appointment-reminders

| Escenario | Prueba |
|---|---|
| Cita reservada con días de anticipación | U-dom `un_dia_antes`; U-app `recordatorio_enviado_una_vez` |
| Cita para dentro de 3 horas | U-dom `de_inmediato_si_falta_poco`; I-grpc (el worker real lo deja `SENT`) |
| Paciente con cuenta | U-app `recordatorio_enviado_una_vez` (correo de la cuenta) |
| Paciente sin correo | U-app `recordatorio_sin_canal` |
| Cita cancelada | U-app `recordatorio_de_cita_cambiada` |
| Cita reprogramada | U-app `recordatorio_de_cita_cambiada`, `reprogramar` |
| Falla temporal del correo | U-app `reintentos` |
| Aviso entregado dos veces | U-app `recordatorio_enviado_una_vez` |
| Caída a mitad del envío | U-app `caida_a_mitad_del_envio` |
| Reintentos agotados | U-app `reintentos` |

El texto del correo (fecha, hora local, médico, clínica, dirección; sin motivo) lo cubre U-dom `texto_del_correo`.

## identity/patient-account-linking

| Escenario | Prueba |
|---|---|
| Paciente que llamó por teléfono | U-app `recepcion_reserva_sin_cuenta`; I-grpc |
| Paciente crea su cuenta y vuelve a la clínica | U-app `verificacion_del_documento` (sugerencia al llegar); E2E «Recepción vincula…» |
| Mismo documento en otra red | I-grpc (cuenta de otra organización con el mismo DNI no se sugiere) |
| Recepción vincula con el DNI a la vista | U-app `vincular_pacientes`; I-grpc; E2E «Recepción vincula…» |
| Paciente intenta reclamar un historial | U-app `vincular_pacientes`; I-grpc (`PERMISSION_DENIED`) |
| Documentos distintos | U-app `vincular_pacientes` |
| Citas anteriores visibles | U-app `vincular_pacientes`; I-grpc; E2E «…él ve su cita anterior» |
| La cuenta ya tenía registro en la clínica | I-grpc (la búsqueda de pacientes no repite el registro vinculado) |
| Vinculación por error | U-app `vincular_pacientes`; I-grpc (deshacer) |

## access-control/action-authorization (requisitos modificados)

| Escenario | Prueba |
|---|---|
| Paciente busca pacientes | U-acc `paciente_busca_pacientes` |
| Paciente con datos completos confirma cita | U-acc `confirmar_cita_ok` |
| Paciente reserva en una clínica pendiente | U-acc `confirmar_cita_clinica_pendiente`; U-app `cupo_ocupado_y_clinica_pendiente` |
| Paciente reprograma la cita de otro | U-app `citas_ajenas` |
| Médico de clínica pendiente | U-acc `buscar_pacientes_clinica_pendiente` |
| Médico con colegiatura sin confirmar | U-acc `receta_sin_confirmar` |
| Médico define horario sin colegiatura confirmada | U-acc `horario_sin_colegiatura`; U-app `horario_sin_colegiatura` |
| Médico cancela una cita de otro médico | U-app `citas_ajenas` |
| Dueño configura antes de activar | U-acc `invitar_clinica_pendiente`; I-ac `flujo_de_clinica_y_medico` |
| Administrador intenta activar | U-acc `admin_activa_clinica` |
| Administrador reserva por un paciente | U-acc `reserva_admin`; U-app `recepcion_reserva_sin_cuenta` |
| Administrador de otra clínica | U-app `llegada` (`NOT_FOUND`) |
| Recepción vincula un paciente a su cuenta | U-acc `vincular_recepcion`; U-app `vincular_pacientes` |
| Recepcionista registra un paciente sin cuenta | I-ac `recepcionista_pide_entrar_y_registra_pacientes_sin_cuenta` |
| Solicitud sin aprobar | U-acc `recepcion_pendiente_no_registra`; I-ac (idem) |
| Recepcionista intenta registrar una atención | U-acc `recepcion_no_abre_historia` |
| Recepcionista registra una llegada | U-acc `llegada_recepcion` |

## Desviaciones del plan

- **Tarea 1.1**: no se creó el índice único parcial de `administration.patient`: `uq_patient_org_user UNIQUE (organization_id, user_id)` de `V1` ya lo garantiza, porque en Postgres los `NULL` no chocan. En Neon había 0 duplicados.
- **Tarea 7.2**: la prueba del worker real comprueba que el recordatorio queda `SENT` con el `LogNotificationSender` (sin SMTP), en lugar de un `NotificationSender` espía inyectado en el servidor. El espía se usa en las pruebas de casos de uso (U-app).
- **Migraciones**: la agenda quedó en `V8__agenda.sql` (la `V6` y la `V7` las ocuparon el rol recepcionista y la Organization raíz). La `V8` además agrega a `administration` la dirección de la clínica y la verificación del documento en el paciente sin cuenta.
- **E2E**: corren contra un Postgres local (`PG_URL=… bun run test:e2e`); el sembrado deja la clínica e2e en la misma organización que las cuentas sembradas.
