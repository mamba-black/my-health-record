# Verificación: escenario → prueba

Pruebas de integración en `backend/bin/clickcare/tests/user_api_test.rs` (módulo `access_control`, salvo indicación), pruebas unitarias en el crate indicado y e2e en `frontendts/e2e/`.

## access-control/action-authorization

| Escenario | Prueba |
|---|---|
| Petición sin credencial | `sin_credencial_valida_no_autenticado`; `clinic_api::rejects_creating_a_clinic_without_session` |
| Credencial vencida | `sin_credencial_valida_no_autenticado`; e2e `role-access` «una sesión vencida termina en el login» |
| Las tres condiciones | core `access::tests::matriz_de_acciones::atencion_confirmada` |
| Usuario recién registrado | `completar_perfil_y_dni_verificado` (perfil editable sin datos previos) |
| Clínicas pendientes no se exploran | core `matriz_de_acciones::confirmar_cita_clinica_pendiente` (regla). **Sin API de exploración aún** |
| Paciente busca pacientes | `sign_in_emite_la_sesion_y_conserva_el_rol`; core `paciente_busca_pacientes` |
| Paciente con datos completos confirma cita | core `confirmar_cita_ok` (regla). **Sin API de citas aún** |
| Médico de clínica pendiente | `aislamiento_por_clinica`; core `buscar_pacientes_clinica_pendiente` |
| Médico con colegiatura sin confirmar | core `receta_sin_confirmar` (regla). **Sin API de recetas aún** |
| Dueño configura antes de activar | `flujo_de_clinica_y_medico` (invita con la clínica pendiente) |
| Administrador intenta activar | core `admin_activa_clinica` |
| Error distinguible | `flujo_de_clinica_y_medico` (detalle `ActionRequirements`) |
| Sin rol no se listan datos | `flujo_de_clinica_y_medico` (médico activa la clínica: sin detalle) |
| Paciente de otra clínica | `aislamiento_por_clinica` |
| Misma persona en dos clínicas | `aislamiento_por_clinica` |
| Expediente ajeno | `aislamiento_por_clinica` (dueño de otra clínica → `NOT_FOUND`) |
| Historial previo sin verificar | **Pendiente**: no existe la vista de historia clínica del paciente |
| Paciente entra por URL a la búsqueda | e2e `role-access` |
| Médico activo | e2e `role-access` |

## identity/progressive-profiling

| Escenario | Prueba |
|---|---|
| Un dato no se exige si ninguna acción en curso lo requiere | user `guarda_sin_exigir_datos_si_no_hay_accion_en_curso`; e2e `incomplete-profile` (Ahora no) |
| Un dato que no aplica al rol no se pide | e2e `role-access` «Un dato que no aplica al rol no se pide» |
| Correo no editable | e2e `incomplete-profile` (`#profile-email` deshabilitado) |
| Extranjero sin DNI | user `acepta_carne_de_extranjeria` |
| Confirmar cita | core `confirmar_cita_sin_documento` (regla). **Sin API de citas aún** |
| Primera atención | **Pendiente**: no existe la API de atenciones |
| Activar sin colegiatura | `flujo_de_clinica_y_medico`; administration `activacion_sin_colegiatura` |
| Especialidad opcional | core `activar_perfil_ok` (sin especialidad) |
| Crear clínica sin RUC | `flujo_de_clinica_y_medico` |
| Activar clínica sin RUC | `flujo_de_clinica_y_medico`; administration `configurar_y_activar_clinica` |
| Médico que también es paciente | e2e `role-access` |
| Datos opcionales siempre | e2e `role-access` |
| Paciente sin DNI intenta confirmar una cita | core `confirmar_cita_sin_documento` (regla); flujo equivalente e2e con «Activar mi perfil médico» |
| Contenido de la ventana | e2e `requirements-dialog` |
| Datos ya registrados no se piden | e2e `requirements-dialog` |
| Validación dentro de la ventana | e2e `requirements-dialog` |
| Pasar al formulario completo | e2e `requirements-dialog` |
| Lo escrito no se pierde al cambiar de vista | e2e `requirements-dialog` |
| Editar datos opcionales y continuar | e2e `requirements-dialog` |
| Faltan los obligatorios en el formulario completo | e2e `requirements-dialog` |
| Volver a la vista reducida | e2e `requirements-dialog` |
| Completar y continuar | e2e `requirements-dialog` |
| Advertencia al rechazar | e2e `requirements-dialog` |
| Volver a completar | e2e `requirements-dialog` |
| Salir sin continuar | e2e `requirements-dialog` |
| Cierre de la ventana por otros medios | e2e `requirements-dialog` |
| Aviso ignorable | e2e `incomplete-profile` |
| Sin datos faltantes no hay aviso | e2e `incomplete-profile`; e2e `google-login` |
| Dato opcional mal formado | user `rechaza_datos_mal_formados`; e2e `incomplete-profile` |
| RUC con dígito verificador inválido | `flujo_de_clinica_y_medico`; core `ruc_modulo_11` |
| DNI verificado | `completar_perfil_y_dni_verificado`; user `un_dni_verificado_no_se_cambia_y_uno_sin_verificar_si` |
| DNI no verificado | user `un_dni_verificado_no_se_cambia_y_uno_sin_verificar_si` |

## identity/role-membership

| Escenario | Prueba |
|---|---|
| Rol preferido no autoriza | `sign_in_emite_la_sesion_y_conserva_el_rol` |
| Cuenta sin intención declarada | user `la_intencion_define_el_rol_preferido` |
| Dueño que también atiende | administration `dueno_que_tambien_atiende` |
| Médico en dos clínicas | `aislamiento_por_clinica` (membresía por clínica) |
| Creación de clínica | `clinic_api::creates_the_clinic_and_the_owner_practitioner` |
| Activación sin colegiatura | `flujo_de_clinica_y_medico` |
| Activación completa | `flujo_de_clinica_y_medico`; administration `activacion_completa_del_medico` |
| Médico suspendido | `flujo_de_clinica_y_medico`; administration `medico_suspendido_pierde_la_membresia_activa` |
| Último dueño | administration `no_se_suspende_al_ultimo_dueno` |

## identity/signup-intent

| Escenario | Prueba |
|---|---|
| La intención se conserva como rol preferido | `sign_in_emite_la_sesion_y_conserva_el_rol` |
| Escenarios previos del requisito | `sign_in`, `clinic_api` y `multi_tenancy` existentes |

## Pendientes

Los escenarios marcados **Pendiente** o **Sin API** dependen de agenda, citas, atenciones, recetas o la vista del paciente, que este cambio declaró fuera de alcance: su regla de acceso ya existe y está probada en `authorize`, pero falta la prueba de punta a punta cuando exista cada API.
