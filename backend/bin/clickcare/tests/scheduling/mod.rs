//! Pruebas de integración de la agenda y las citas (`scheduling-appointments`).
//!
//! Usan el mismo contenedor `clickcare-test` que el resto de `user_api_test`.

use crate::{TestEnv, test_env};
use rstest::*;

/// Ejecuta una consulta de un solo valor en la base de pruebas.
pub(crate) async fn scalar<T>(env: &TestEnv, sql: &'static str) -> T
where
    T: for<'a> ::postgres::types::FromSql<'a> + Send + 'static,
{
    let conn = env.pg_connection_string.clone();
    tokio::task::spawn_blocking(move || {
        ::postgres::Client::connect(&conn, ::postgres::NoTls)
            .expect("conexión")
            .query_one(sql, &[])
            .expect("consulta")
            .get::<_, T>(0)
    })
    .await
    .expect("tarea bloqueante")
}

/// Tarea 1.2: el contenedor queda migrado hasta `V8__agenda.sql`, con la exclusión de
/// citas que se cruzan dentro de la organización.
#[rstest]
#[tokio::test]
async fn migra_la_agenda_hasta_v8(#[future(awt)] test_env: &'static TestEnv) {
    let version: i32 = scalar(
        test_env,
        "select max(version) from public.refinery_schema_history",
    )
    .await;
    assert!(version >= 8, "versión aplicada: {version}");
    let exclusion: i64 = scalar(
        test_env,
        "select count(*) from pg_constraint where conname = 'ex_appointment_practitioner_overlap'",
    )
    .await;
    assert_eq!(exclusion, 1);
}

/// Repositorios de la agenda contra el contenedor de pruebas.
mod repositories {
    use crate::{TestEnv, test_env};
    use rstest::*;
    use scheduling::domain::appointment::Appointment;
    use scheduling::domain::calendar::{Instant, plus_minutes};
    use scheduling::domain::error::SchedulingError;
    use scheduling::domain::reminder::Reminder;
    use scheduling::infrastructure::config::{Repositories, repositories};
    use uuid::Uuid;

    async fn repos(env: &TestEnv) -> Repositories {
        repositories(&env.pg_connection_string)
            .await
            .expect("repositorios de la agenda")
    }

    /// Instante futuro distinto por prueba, alineado al minuto.
    fn future_start() -> Instant {
        let now = Instant::now().as_second();
        let offset = i64::try_from(Uuid::now_v7().as_u128() % 100_000).unwrap_or(0) * 3600;
        Instant::from_second(now - now % 60 + 86_400 + offset).expect("instante")
    }

    fn appointment(
        network: Uuid,
        clinic: Uuid,
        practitioner: Uuid,
        start: Instant,
    ) -> (Appointment, Reminder) {
        let appointment = Appointment::book(
            network,
            clinic,
            practitioner,
            Uuid::now_v7(),
            (start, plus_minutes(start, 30)),
            Uuid::now_v7(),
        );
        let reminder =
            Reminder::for_appointment(network, appointment.id, start, 24, Instant::now());
        (appointment, reminder)
    }

    /// «Reservas simultáneas»: exactamente una de las dos tiene éxito.
    #[rstest]
    #[tokio::test]
    async fn reservas_simultaneas(#[future(awt)] test_env: &'static TestEnv) {
        let r = repos(test_env).await;
        let (network, clinic, practitioner) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
        let start = future_start();
        let (a, ra) = appointment(network, clinic, practitioner, start);
        let (b, rb) = appointment(network, clinic, practitioner, start);
        let (first, second) = tokio::join!(
            r.appointments.insert(&a, &ra),
            r.appointments.insert(&b, &rb)
        );
        let ok = [first.is_ok(), second.is_ok()];
        assert_eq!(
            ok.iter().filter(|ok| **ok).count(),
            1,
            "{first:?} {second:?}"
        );
        assert!(
            matches!(first, Err(SchedulingError::SlotUnavailable))
                || matches!(second, Err(SchedulingError::SlotUnavailable))
        );
    }

    /// «Nuevo cupo ocupado»: la cita conserva su cupo original.
    #[rstest]
    #[tokio::test]
    async fn reprogramar_a_un_cupo_ocupado(#[future(awt)] test_env: &'static TestEnv) {
        let r = repos(test_env).await;
        let (network, clinic, practitioner) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
        let start = future_start();
        let later = plus_minutes(start, 60);
        let (mine, rm) = appointment(network, clinic, practitioner, start);
        let (other, ro) = appointment(network, clinic, practitioner, later);
        r.appointments.insert(&mine, &rm).await.expect("mía");
        r.appointments.insert(&other, &ro).await.expect("otra");

        let mut moved = mine.clone();
        moved
            .reschedule(
                practitioner,
                clinic,
                (later, plus_minutes(later, 30)),
                Instant::now(),
            )
            .expect("regla de dominio");
        let result = r.appointments.update(&moved).await;
        assert!(
            matches!(result, Err(SchedulingError::SlotUnavailable)),
            "{result:?}"
        );
        let stored = r
            .appointments
            .find(&network, &mine.id)
            .await
            .expect("leer")
            .expect("existe");
        assert_eq!(stored.start, start, "conserva su cupo");
    }

    /// Un médico no tiene citas que se crucen entre clínicas de su red, pero dos redes
    /// distintas no chocan aunque compartan base.
    #[rstest]
    #[tokio::test]
    async fn cruce_entre_clinicas_y_redes(#[future(awt)] test_env: &'static TestEnv) {
        let r = repos(test_env).await;
        let (network, practitioner) = (Uuid::now_v7(), Uuid::now_v7());
        let start = future_start();
        let (a, ra) = appointment(network, Uuid::now_v7(), practitioner, start);
        r.appointments.insert(&a, &ra).await.expect("clínica A");
        let (b, rb) = appointment(
            network,
            Uuid::now_v7(),
            practitioner,
            plus_minutes(start, 15),
        );
        assert!(matches!(
            r.appointments.insert(&b, &rb).await,
            Err(SchedulingError::SlotUnavailable)
        ));
        let (c, rc) = appointment(Uuid::now_v7(), Uuid::now_v7(), practitioner, start);
        r.appointments
            .insert(&c, &rc)
            .await
            .expect("otra red no se compara");
    }
}

/// Flujo completo por gRPC contra el servidor de pruebas (grupos 4.3, 5, 6, 7 y 8).
mod api {
    use crate::support::{Account, authed, sign_up, sign_up_in};
    use crate::{TestEnv, test_env};
    use clickcare::infrastructure::grpc::clinic_api_client::ClinicApiClient;
    use clickcare::infrastructure::grpc::public_scheduling_api_client::PublicSchedulingApiClient;
    use clickcare::infrastructure::grpc::scheduling_api_client::SchedulingApiClient;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::*;
    use prost::Message;
    use rstest::*;
    use std::time::Duration;
    use tonic::Code;
    use tonic::transport::Channel;
    use uuid::Uuid;

    async fn api(env: &TestEnv) -> SchedulingApiClient<Channel> {
        SchedulingApiClient::connect(env.grpc_addr.clone())
            .await
            .expect("conexión")
    }

    async fn public(env: &TestEnv) -> PublicSchedulingApiClient<Channel> {
        PublicSchedulingApiClient::connect(env.grpc_addr.clone())
            .await
            .expect("conexión")
    }

    async fn sql(env: &TestEnv, statement: String) {
        let conn = env.pg_connection_string.clone();
        tokio::task::spawn_blocking(move || {
            ::postgres::Client::connect(&conn, ::postgres::NoTls)
                .expect("conexión")
                .batch_execute(&statement)
                .expect("SQL de preparación");
        })
        .await
        .expect("tarea");
    }

    async fn text(env: &TestEnv, query: String) -> Option<String> {
        let conn = env.pg_connection_string.clone();
        tokio::task::spawn_blocking(move || {
            ::postgres::Client::connect(&conn, ::postgres::NoTls)
                .expect("conexión")
                .query_opt(query.as_str(), &[])
                .expect("consulta")
                .map(|row| row.get::<_, String>(0))
        })
        .await
        .expect("tarea")
    }

    fn dni() -> String {
        format!("{:08}", Uuid::now_v7().as_u128() % 100_000_000)
    }

    async fn complete(env: &TestEnv, account: &Account, document: &str) {
        UserApiClient::connect(env.grpc_addr.clone())
            .await
            .expect("conexión")
            .complete_profile(authed(
                CompleteProfileRequest {
                    given_name: "Nombre".into(),
                    family_name: Some("Apellido".into()),
                    phone: "999888777".into(),
                    identifier: Some(Identifier {
                        identifier_type: Some(identifier::IdentifierType::Dni(document.into())),
                    }),
                    ..Default::default()
                },
                &account.access_token,
            ))
            .await
            .expect("completar perfil");
    }

    fn new_patient(document: &str, phone: &str) -> book_for_patient_request::Patient {
        book_for_patient_request::Patient::NewPatient(NewPatientInfo {
            given_name: "Rosa".into(),
            family_name: "Quispe".into(),
            identifier: Some(Identifier {
                identifier_type: Some(identifier::IdentifierType::Dni(document.into())),
            }),
            phone: phone.into(),
            ..Default::default()
        })
    }

    /// Cupos del médico entre `from` y `to` (RFC 3339).
    async fn slots(
        env: &TestEnv,
        subdomain: &str,
        doctor: &Account,
        from: String,
        to: String,
    ) -> Vec<String> {
        public(env)
            .await
            .get_availability(tonic::Request::new(AvailabilityRequest {
                subdomain: subdomain.into(),
                practitioner_user_id: doctor.user_id.to_string(),
                from,
                to,
            }))
            .await
            .expect("disponibilidad")
            .into_inner()
            .slots
            .into_iter()
            .map(|slot| slot.start)
            .collect()
    }

    #[rstest]
    #[tokio::test]
    async fn flujo_de_agenda_por_grpc(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        // ── Clínica activa con un médico activo de colegiatura confirmada ──
        let owner = sign_up(env, "DuenaAgenda", SignUpIntent::IntentClinicOwner).await;
        let subdomain = format!("agenda-{}", owner.user_id.simple());
        let clinic = ClinicApiClient::connect(env.grpc_addr.clone())
            .await
            .unwrap()
            .create_clinic(authed(
                CreateClinicRequest {
                    name: "Clínica Agenda".into(),
                    subdomain: subdomain.clone(),
                    given_name: "Dueña".into(),
                    ..Default::default()
                },
                &owner.access_token,
            ))
            .await
            .expect("crear clínica")
            .into_inner();
        let organization_id = clinic.organization_id.clone();
        let doctor = sign_up_in(
            env,
            "Medica",
            SignUpIntent::IntentPractitioner,
            Some(&subdomain),
        )
        .await;
        sql(
            env,
            format!(
                "update administration.organization set status = 'ACTIVE' where id = '{organization_id}';
                 insert into administration.membership
                   (id, organization_id, user_id, role, status, license_confirmed_at)
                 values ('{}', '{organization_id}', '{}', 'PRACTITIONER', 'ACTIVE', now());",
                Uuid::now_v7(),
                doctor.user_id
            ),
        )
        .await;

        // ── 8.1: la API con sesión exige credencial; la pública no ──
        let unauthenticated = api(env)
            .await
            .list_my_appointments(tonic::Request::new(ListMyAppointmentsRequest {}))
            .await
            .expect_err("sin credencial");
        assert_eq!(unauthenticated.code(), Code::Unauthenticated);
        let public_clinic = public(env)
            .await
            .get_clinic(tonic::Request::new(ClinicLinkRequest {
                subdomain: subdomain.clone(),
            }))
            .await
            .expect("pública sin credencial")
            .into_inner();
        assert_eq!(public_clinic.organization_id, organization_id);

        // ── 5.1: horario de todos los días, citas de 30 min ──
        let blocks = (1..=7)
            .map(|weekday| TimeBlockInfo {
                weekday,
                start: "00:00".into(),
                end: "23:30".into(),
            })
            .collect();
        let schedule = api(env)
            .await
            .set_weekly_schedule(authed(
                SetWeeklyScheduleRequest {
                    organization_id: organization_id.clone(),
                    slot_minutes: 30,
                    blocks,
                },
                &doctor.access_token,
            ))
            .await
            .expect("horario")
            .into_inner();
        assert!(schedule.defined);
        let practitioners = public(env)
            .await
            .list_practitioners(tonic::Request::new(ClinicLinkRequest {
                subdomain: subdomain.clone(),
            }))
            .await
            .unwrap()
            .into_inner()
            .practitioners;
        assert!(
            practitioners
                .iter()
                .any(|p| p.user_id == doctor.user_id.to_string())
        );

        let now = jiff::Timestamp::now();
        let in_two_days = now + jiff::SignedDuration::from_hours(48);
        let later = slots(
            env,
            &subdomain,
            &doctor,
            in_two_days.to_string(),
            (in_two_days + jiff::SignedDuration::from_hours(6)).to_string(),
        )
        .await;
        assert!(later.len() >= 4, "cupos: {later:?}");

        // ── 5.3: reserva del paciente con datos faltantes y luego completos ──
        let patient = sign_up_in(
            env,
            "Paciente",
            SignUpIntent::IntentPatient,
            Some(&subdomain),
        )
        .await;
        let book = |start: &str| BookAppointmentRequest {
            organization_id: organization_id.clone(),
            practitioner_user_id: doctor.user_id.to_string(),
            start: start.to_string(),
        };
        let missing = api(env)
            .await
            .book_appointment(authed(book(&later[0]), &patient.access_token))
            .await
            .expect_err("faltan datos");
        assert_eq!(missing.code(), Code::FailedPrecondition);
        let requirements = ActionRequirements::decode(missing.details()).expect("detalle");
        assert_eq!(
            requirements.action(),
            ProtectedAction::ActionConfirmAppointment
        );
        complete(env, &patient, &dni()).await;
        let first = api(env)
            .await
            .book_appointment(authed(book(&later[0]), &patient.access_token))
            .await
            .expect("reserva")
            .into_inner();
        assert_eq!(first.status(), AppointmentStatus::Booked);
        assert_eq!(first.clinic_name, "Clínica Agenda");

        // «Cupo ocupado»: ABORTED.
        let other = sign_up_in(env, "Otro", SignUpIntent::IntentPatient, Some(&subdomain)).await;
        complete(env, &other, &dni()).await;
        let taken = api(env)
            .await
            .book_appointment(authed(book(&later[0]), &other.access_token))
            .await
            .expect_err("ocupado");
        assert_eq!(taken.code(), Code::Aborted);

        // «Segunda cita»: el mismo registro de paciente.
        let second = api(env)
            .await
            .book_appointment(authed(book(&later[1]), &patient.access_token))
            .await
            .expect("segunda")
            .into_inner();
        assert_eq!(first.patient_id, second.patient_id);

        // ── 5.5 y 5.6 ──
        let mine = api(env)
            .await
            .list_my_appointments(authed(ListMyAppointmentsRequest {}, &patient.access_token))
            .await
            .unwrap()
            .into_inner()
            .appointments;
        assert_eq!(mine.len(), 2);
        let foreign = api(env)
            .await
            .cancel_appointment(authed(
                AppointmentIdRequest {
                    appointment_id: first.id.clone(),
                },
                &other.access_token,
            ))
            .await
            .expect_err("ajena");
        assert_eq!(foreign.code(), Code::NotFound);
        let cancelled = api(env)
            .await
            .cancel_appointment(authed(
                AppointmentIdRequest {
                    appointment_id: second.id.clone(),
                },
                &patient.access_token,
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(cancelled.status(), AppointmentStatus::Cancelled);
        let moved = api(env)
            .await
            .reschedule_appointment(authed(
                RescheduleAppointmentRequest {
                    appointment_id: first.id.clone(),
                    practitioner_user_id: doctor.user_id.to_string(),
                    start: later[2].clone(),
                },
                &patient.access_token,
            ))
            .await
            .expect("reprogramar")
            .into_inner();
        assert_eq!(moved.start, later[2]);

        // ── 5.4: recepción reserva para un paciente sin cuenta ──
        let walk_in_dni = dni();
        let no_phone = api(env)
            .await
            .book_for_patient(authed(
                BookForPatientRequest {
                    organization_id: organization_id.clone(),
                    practitioner_user_id: doctor.user_id.to_string(),
                    start: later[3].clone(),
                    patient: Some(new_patient(&walk_in_dni, "")),
                },
                &owner.access_token,
            ))
            .await
            .expect_err("sin teléfono");
        assert_eq!(no_phone.code(), Code::InvalidArgument);
        let walk_in = api(env)
            .await
            .book_for_patient(authed(
                BookForPatientRequest {
                    organization_id: organization_id.clone(),
                    practitioner_user_id: doctor.user_id.to_string(),
                    start: later[3].clone(),
                    patient: Some(new_patient(&walk_in_dni, "987654321")),
                },
                &owner.access_token,
            ))
            .await
            .expect("reserva por recepción")
            .into_inner();
        assert_eq!(walk_in.patient_name.as_deref(), Some("Rosa Quispe"));

        // ── 6.1: llegada en otro día, y hoy con el DNI verificado ──
        let not_today = api(env)
            .await
            .check_in(authed(
                CheckInRequest {
                    appointment_id: moved.id.clone(),
                    document_check: DocumentCheck::Match as i32,
                },
                &owner.access_token,
            ))
            .await
            .expect_err("otro día");
        assert_eq!(not_today.code(), Code::FailedPrecondition);
        let doctor_check_in = api(env)
            .await
            .check_in(authed(
                CheckInRequest {
                    appointment_id: moved.id.clone(),
                    document_check: DocumentCheck::Unspecified as i32,
                },
                &doctor.access_token,
            ))
            .await
            .expect_err("el médico no registra llegadas");
        assert_eq!(doctor_check_in.code(), Code::PermissionDenied);

        // El recordatorio de una cita para dentro de menos de 24 h sale de inmediato.
        let soon = slots(
            env,
            &subdomain,
            &doctor,
            now.to_string(),
            (now + jiff::SignedDuration::from_hours(20)).to_string(),
        )
        .await;
        let soon_appointment = api(env)
            .await
            .book_appointment(authed(book(&soon[0]), &other.access_token))
            .await
            .expect("cita próxima")
            .into_inner();
        let mut status = None;
        for _ in 0..40 {
            status = text(
                env,
                format!(
                    "select status from scheduling.reminder where appointment_id = '{}'",
                    soon_appointment.id
                ),
            )
            .await;
            if status.as_deref() == Some("SENT") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        assert_eq!(status.as_deref(), Some("SENT"), "el worker envía el aviso");

        // Cita de este momento (creada en la base para no depender de la hora del día):
        // recepción registra la llegada con el DNI a la vista.
        let today_id = Uuid::now_v7();
        sql(
            env,
            format!(
                "insert into scheduling.appointment
                   (id, network_id, organization_id, practitioner_user_id, patient_id, period, status, booked_by)
                 values ('{today_id}', '{}', '{organization_id}', '{}', '{}',
                         tstzrange(now() - interval '1 minute', now() + interval '29 minutes', '[)'),
                         'BOOKED', '{}')",
                clinic.network_id,
                Uuid::now_v7(),
                soon_appointment.patient_id,
                owner.user_id
            ),
        )
        .await;
        let arrived = api(env)
            .await
            .check_in(authed(
                CheckInRequest {
                    appointment_id: today_id.to_string(),
                    document_check: DocumentCheck::Match as i32,
                },
                &owner.access_token,
            ))
            .await
            .expect("llegada")
            .into_inner();
        assert!(arrived.account_verified);
        assert_eq!(
            arrived.appointment.as_ref().map(|a| a.status()),
            Some(AppointmentStatus::Arrived)
        );
        let verified = text(
            env,
            format!(
                "select document_verified::text from identity.user_account where id = '{}'",
                other.user_id
            ),
        )
        .await;
        assert_eq!(verified.as_deref(), Some("true"), "documento verificado");

        // ── 6.3 a 6.5: vincular al paciente sin cuenta con la cuenta que crea ──
        let account = sign_up_in(env, "Rosa", SignUpIntent::IntentPatient, Some(&subdomain)).await;
        complete(env, &account, &walk_in_dni).await;
        // La cuenta ya tiene su propio registro en la clínica (reservó ella misma).
        let own = api(env)
            .await
            .book_appointment(authed(book(&later[4]), &account.access_token))
            .await
            .expect("reserva propia")
            .into_inner();
        let claim = api(env)
            .await
            .suggest_patient_links(authed(
                PatientLinkRequest {
                    organization_id: organization_id.clone(),
                    patient_id: walk_in.patient_id.clone(),
                },
                &account.access_token,
            ))
            .await
            .expect_err("el paciente no reclama historiales");
        assert_eq!(claim.code(), Code::PermissionDenied);
        let suggestions = api(env)
            .await
            .suggest_patient_links(authed(
                PatientLinkRequest {
                    organization_id: organization_id.clone(),
                    patient_id: walk_in.patient_id.clone(),
                },
                &owner.access_token,
            ))
            .await
            .unwrap()
            .into_inner()
            .suggestions;
        assert!(
            suggestions
                .iter()
                .any(|s| s.user_id == account.user_id.to_string())
        );
        let link = api(env)
            .await
            .link_patient_account(authed(
                LinkPatientAccountRequest {
                    organization_id: organization_id.clone(),
                    patient_id: walk_in.patient_id.clone(),
                    user_id: account.user_id.to_string(),
                    document_seen: true,
                },
                &owner.access_token,
            ))
            .await
            .expect("vincular")
            .into_inner();
        let linked = api(env)
            .await
            .list_my_appointments(authed(ListMyAppointmentsRequest {}, &account.access_token))
            .await
            .unwrap()
            .into_inner()
            .appointments;
        assert!(
            linked.iter().any(|a| a.id == walk_in.id),
            "ve su cita anterior"
        );
        assert!(linked.iter().any(|a| a.id == own.id), "y la suya");
        // «La cuenta ya tenía registro en la clínica»: la búsqueda la muestra una vez.
        let found = clickcare::infrastructure::grpc::patient_api_client::PatientApiClient::connect(
            env.grpc_addr.clone(),
        )
        .await
        .unwrap()
        .search_patient(authed(
            SearchPatientRequest { name: None },
            &doctor.access_token,
        ))
        .await
        .expect("búsqueda del médico")
        .into_inner()
        .patients;
        assert!(found.iter().any(|p| p.id == own.patient_id));
        assert!(
            !found.iter().any(|p| p.id == walk_in.patient_id),
            "el registro vinculado no se repite"
        );
        api(env)
            .await
            .unlink_patient_account(authed(
                UnlinkPatientAccountRequest {
                    organization_id: organization_id.clone(),
                    link_id: link.link_id,
                    reason: "Persona equivocada".into(),
                },
                &owner.access_token,
            ))
            .await
            .expect("deshacer");
        let after = api(env)
            .await
            .list_my_appointments(authed(ListMyAppointmentsRequest {}, &account.access_token))
            .await
            .unwrap()
            .into_inner()
            .appointments;
        assert_eq!(after.len(), 1, "solo queda su propia cita");
        assert_eq!(after[0].id, own.id);
    }
}
