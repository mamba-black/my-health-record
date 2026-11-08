#![allow(clippy::too_many_lines)]
//! Escenarios de los specs de agenda con dobles en memoria.

use crate::application::booking_usecase::{BookingUseCases, PatientChoice};
use crate::application::integration::{ClinicInfo, NewPatientData, PractitionerInfo};
use crate::application::reception_usecase::ReceptionUseCases;
use crate::application::reminder_usecase::{Delivery, ReminderUseCases};
use crate::application::schedule_usecase::ScheduleUseCases;
use crate::domain::appointment::{AppointmentStatus, DocumentCheck};
use crate::domain::calendar::{Instant, LocalDate, Zone, at, local_time, plus_minutes};
use crate::domain::error::SchedulingError;
use crate::domain::reminder::ReminderStatus;
use crate::domain::schedule::TimeBlock;
use crate::testing::Doubles;
use app_core::domain::access::{
    AccessDenied, ClinicStatus, MembershipRole, MembershipStatus, MembershipView, ProfileField,
    RequestContext,
};
use uuid::Uuid;

fn lima() -> Zone {
    Zone::get("America/Lima").unwrap()
}

/// Lunes 2 de marzo de 2026, a la hora local indicada.
fn monday(hour: i8, minute: i8) -> Instant {
    at(
        LocalDate::new(2026, 3, 2).unwrap(),
        local_time(hour, minute).unwrap(),
        &lima(),
    )
    .unwrap()
}

fn days_later(instant: Instant, days: i64) -> Instant {
    plus_minutes(instant, days * 24 * 60)
}

fn all_week(from: i8, to: i8) -> Vec<TimeBlock> {
    (1..=7)
        .map(|weekday| TimeBlock {
            weekday,
            start: local_time(from, 0).unwrap(),
            end: local_time(to, 0).unwrap(),
        })
        .collect()
}

struct World {
    d: Doubles,
    network: Uuid,
    clinic: ClinicInfo,
    doctor: Uuid,
}

impl World {
    fn new() -> Self {
        let d = Doubles::new(monday(8, 0));
        let network = Uuid::now_v7();
        let clinic = add_clinic(&d, network, "san-borja", ClinicStatus::Active);
        let doctor = Uuid::now_v7();
        add_doctor(&d, clinic.id, doctor, "Dra. García");
        World {
            d,
            network,
            clinic,
            doctor,
        }
    }

    fn schedules(&self) -> ScheduleUseCases {
        ScheduleUseCases::new(self.d.state())
    }

    fn booking(&self) -> BookingUseCases {
        BookingUseCases::new(self.d.state())
    }

    fn reception(&self) -> ReceptionUseCases {
        ReceptionUseCases::new(self.d.state())
    }

    fn reminders(&self) -> ReminderUseCases {
        ReminderUseCases::new(self.d.state())
    }

    fn doctor_ctx(&self) -> RequestContext {
        member(
            self.network,
            self.doctor,
            self.clinic.id,
            MembershipRole::Practitioner,
            true,
        )
    }

    fn admin(&self) -> RequestContext {
        member(
            self.network,
            Uuid::now_v7(),
            self.clinic.id,
            MembershipRole::Admin,
            false,
        )
    }

    fn patient(&self) -> RequestContext {
        patient(self.network, Uuid::now_v7())
    }

    async fn publish_schedule(&self) {
        self.schedules()
            .set_weekly_schedule(&self.doctor_ctx(), self.clinic.id, 30, all_week(9, 13))
            .await
            .expect("horario");
    }

    async fn book_monday(
        &self,
        ctx: &RequestContext,
        hour: i8,
    ) -> crate::domain::appointment::Appointment {
        self.booking()
            .book(ctx, self.clinic.id, self.doctor, monday(hour, 0))
            .await
            .expect("reserva")
    }
}

fn add_clinic(d: &Doubles, network: Uuid, subdomain: &str, status: ClinicStatus) -> ClinicInfo {
    let clinic = ClinicInfo {
        id: Uuid::now_v7(),
        network_id: network,
        subdomain: subdomain.into(),
        name: format!("Clínica {subdomain}"),
        status,
        zone: lima(),
        address: Some("Av. Aviación 123".into()),
    };
    d.clinics.clinics.lock().unwrap().push(clinic.clone());
    clinic
}

fn add_doctor(d: &Doubles, clinic: Uuid, user_id: Uuid, name: &str) {
    d.clinics.practitioners.lock().unwrap().push((
        clinic,
        PractitionerInfo {
            user_id,
            name: name.into(),
            specialty: Some("Medicina general".into()),
        },
    ));
}

fn full_data() -> Vec<ProfileField> {
    vec![
        ProfileField::GivenName,
        ProfileField::FamilyName,
        ProfileField::Phone,
        ProfileField::Document,
    ]
}

fn patient(network: Uuid, user_id: Uuid) -> RequestContext {
    RequestContext {
        user_id,
        network_id: network,
        present_fields: full_data().into_iter().collect(),
        ..RequestContext::default()
    }
}

fn member(
    network: Uuid,
    user_id: Uuid,
    clinic: Uuid,
    role: MembershipRole,
    license_confirmed: bool,
) -> RequestContext {
    RequestContext {
        memberships: vec![MembershipView {
            id: Uuid::now_v7(),
            organization_id: clinic,
            role,
            status: MembershipStatus::Active,
            license_confirmed,
        }],
        ..patient(network, user_id)
    }
}

fn new_patient(phone: &str) -> NewPatientData {
    NewPatientData {
        given_name: "Rosa".into(),
        family_name: "Quispe".into(),
        document_type: "DNI".into(),
        document_value: "12345678".into(),
        phone: phone.into(),
        ..NewPatientData::default()
    }
}

// ─── 5.1 Horario ────────────────────────────────────────────────────────────

/// «Médico define horario sin colegiatura confirmada».
#[tokio::test]
async fn horario_sin_colegiatura() {
    let w = World::new();
    let ctx = member(
        w.network,
        w.doctor,
        w.clinic.id,
        MembershipRole::Practitioner,
        false,
    );
    let result = w
        .schedules()
        .set_weekly_schedule(&ctx, w.clinic.id, 30, all_week(9, 13))
        .await;
    assert!(matches!(
        result,
        Err(SchedulingError::Denied(AccessDenied::MissingData { fields, .. }))
            if fields == vec![ProfileField::LicenseConfirmed]
    ));
}

/// «Cruce con otra clínica de la red» (solo día y franja) y «Clínicas de redes distintas».
#[tokio::test]
async fn cruce_con_otra_clinica_de_la_red() {
    let w = World::new();
    let monday_9_13 = vec![TimeBlock {
        weekday: 1,
        start: local_time(9, 0).unwrap(),
        end: local_time(13, 0).unwrap(),
    }];
    w.schedules()
        .set_weekly_schedule(&w.doctor_ctx(), w.clinic.id, 30, monday_9_13)
        .await
        .expect("clínica A");

    let b = add_clinic(&w.d, w.network, "miraflores", ClinicStatus::Active);
    let ctx_b = member(
        w.network,
        w.doctor,
        b.id,
        MembershipRole::Practitioner,
        true,
    );
    let monday_12_15 = vec![TimeBlock {
        weekday: 1,
        start: local_time(12, 0).unwrap(),
        end: local_time(15, 0).unwrap(),
    }];
    let result = w
        .schedules()
        .set_weekly_schedule(&ctx_b, b.id, 30, monday_12_15.clone())
        .await;
    match result {
        Err(SchedulingError::CrossClinicOverlap {
            weekday,
            start,
            end,
        }) => assert_eq!(
            (weekday.as_str(), start.as_str(), end.as_str()),
            ("lunes", "12:00", "13:00")
        ),
        other => panic!("se esperaba el cruce: {other:?}"),
    }

    let other_network = Uuid::now_v7();
    let c = add_clinic(&w.d, other_network, "arequipa", ClinicStatus::Active);
    let ctx_c = member(
        other_network,
        w.doctor,
        c.id,
        MembershipRole::Practitioner,
        true,
    );
    w.schedules()
        .set_weekly_schedule(&ctx_c, c.id, 30, monday_12_15)
        .await
        .expect("otra red no se compara");
}

/// «Excepción con citas reservadas».
#[tokio::test]
async fn excepcion_con_citas() {
    let w = World::new();
    w.publish_schedule().await;
    for hour in [9, 10, 11] {
        w.book_monday(&w.patient(), hour).await;
    }
    let result = w
        .schedules()
        .add_exception(
            &w.doctor_ctx(),
            w.clinic.id,
            monday(0, 0),
            monday(23, 0),
            Some("Feriado".into()),
        )
        .await;
    assert!(matches!(
        result,
        Err(SchedulingError::ExceptionHasAppointments(3))
    ));
}

// ─── 5.2 Disponibilidad pública ─────────────────────────────────────────────

/// «Paciente abre el enlace», «Clínica pendiente» y «Médico invitado sin activar».
#[tokio::test]
async fn disponibilidad_publica() {
    let w = World::new();
    w.publish_schedule().await;
    let practitioners = w
        .schedules()
        .public_practitioners("san-borja")
        .await
        .unwrap();
    assert_eq!(practitioners.len(), 1);
    assert_eq!(practitioners[0].name, "Dra. García");
    let slots = w
        .schedules()
        .public_availability("san-borja", w.doctor, monday(0, 0), monday(23, 0))
        .await
        .unwrap();
    assert_eq!(slots.len(), 8);

    // Un médico invitado (sin membresía activa) no aparece aunque tenga horario.
    let invited = Uuid::now_v7();
    let ctx = member(
        w.network,
        invited,
        w.clinic.id,
        MembershipRole::Practitioner,
        true,
    );
    w.schedules()
        .set_weekly_schedule(&ctx, w.clinic.id, 30, all_week(15, 16))
        .await
        .unwrap();
    assert_eq!(
        w.schedules()
            .public_practitioners("san-borja")
            .await
            .unwrap()
            .len(),
        1
    );

    let pending = add_clinic(&w.d, w.network, "pendiente", ClinicStatus::PendingSetup);
    add_doctor(&w.d, pending.id, w.doctor, "Dra. García");
    assert!(
        w.schedules()
            .public_practitioners("pendiente")
            .await
            .unwrap()
            .is_empty()
    );
}

// ─── 5.3 Reserva del paciente ───────────────────────────────────────────────

/// «Paciente con datos completos» e «Identidad sin verificar»: nace confirmada.
#[tokio::test]
async fn paciente_reserva() {
    let w = World::new();
    w.publish_schedule().await;
    let appointment = w.book_monday(&w.patient(), 10).await;
    assert_eq!(appointment.status, AppointmentStatus::Booked);
    let reminder = w.d.appointments.reminder_of(&appointment.id).unwrap();
    assert_eq!(
        reminder.due_at,
        monday(8, 0),
        "falta menos de 24 h: de inmediato"
    );
    assert_eq!(w.d.scheduler.scheduled.lock().unwrap().len(), 1);
    // El cupo deja de ofrecerse.
    let slots = w
        .schedules()
        .public_availability("san-borja", w.doctor, monday(10, 0), monday(10, 30))
        .await
        .unwrap();
    assert!(slots.is_empty());
}

/// «Paciente sin documento»: la regla responde los datos que faltan.
#[tokio::test]
async fn paciente_sin_documento() {
    let w = World::new();
    w.publish_schedule().await;
    let mut ctx = w.patient();
    ctx.present_fields.remove(&ProfileField::Document);
    let result = w
        .booking()
        .book(&ctx, w.clinic.id, w.doctor, monday(10, 0))
        .await;
    assert!(matches!(
        result,
        Err(SchedulingError::Denied(AccessDenied::MissingData { fields, .. }))
            if fields == vec![ProfileField::Document]
    ));
}

/// «Cupo ocupado» y «Paciente reserva en una clínica pendiente».
#[tokio::test]
async fn cupo_ocupado_y_clinica_pendiente() {
    let w = World::new();
    w.publish_schedule().await;
    w.book_monday(&w.patient(), 10).await;
    let again = w
        .booking()
        .book(&w.patient(), w.clinic.id, w.doctor, monday(10, 0))
        .await;
    assert!(matches!(again, Err(SchedulingError::SlotUnavailable)));

    let pending = add_clinic(&w.d, w.network, "pendiente", ClinicStatus::PendingSetup);
    let result = w
        .booking()
        .book(&w.patient(), pending.id, w.doctor, monday(10, 0))
        .await;
    assert!(matches!(
        result,
        Err(SchedulingError::Denied(AccessDenied::ClinicNotActive))
    ));
}

#[tokio::test]
async fn cupo_inventado() {
    let w = World::new();
    w.publish_schedule().await;
    let result = w
        .booking()
        .book(&w.patient(), w.clinic.id, w.doctor, monday(10, 10))
        .await;
    assert!(matches!(result, Err(SchedulingError::SlotNotOffered)));
}

/// «Primera cita en la clínica» y «Segunda cita»: un solo registro.
#[tokio::test]
async fn un_solo_registro_por_paciente() {
    let w = World::new();
    w.publish_schedule().await;
    let ctx = w.patient();
    let a = w.book_monday(&ctx, 9).await;
    let b = w.book_monday(&ctx, 11).await;
    assert_eq!(a.patient_id, b.patient_id);
    assert_eq!(w.d.patients.patients.lock().unwrap().len(), 1);
}

// ─── 5.4 Reserva por la clínica ─────────────────────────────────────────────

/// «Recepción reserva para un paciente sin cuenta» y «Recepción sin los datos».
#[tokio::test]
async fn recepcion_reserva_sin_cuenta() {
    let w = World::new();
    w.publish_schedule().await;
    let admin = w.admin();
    let a = w
        .booking()
        .book_for_patient(
            &admin,
            w.clinic.id,
            w.doctor,
            monday(10, 0),
            PatientChoice::New(new_patient("987654321")),
        )
        .await
        .expect("reserva");
    assert_eq!(a.status, AppointmentStatus::Booked);
    let result = w
        .booking()
        .book_for_patient(
            &admin,
            w.clinic.id,
            w.doctor,
            monday(11, 0),
            PatientChoice::New(new_patient("")),
        )
        .await;
    assert!(
        matches!(result, Err(SchedulingError::MissingPatientData(m)) if m.contains("teléfono"))
    );
}

/// «Médico reserva en la agenda de otro médico».
#[tokio::test]
async fn medico_reserva_en_otra_agenda() {
    let w = World::new();
    w.publish_schedule().await;
    let other = member(
        w.network,
        Uuid::now_v7(),
        w.clinic.id,
        MembershipRole::Practitioner,
        true,
    );
    let result = w
        .booking()
        .book_for_patient(
            &other,
            w.clinic.id,
            w.doctor,
            monday(10, 0),
            PatientChoice::New(new_patient("9")),
        )
        .await;
    assert!(matches!(
        result,
        Err(SchedulingError::Denied(AccessDenied::Forbidden))
    ));
}

// ─── 5.5 Consultas ──────────────────────────────────────────────────────────

/// «Paciente revisa sus citas» (dos clínicas de la red) y «Médico revisa su agenda».
#[tokio::test]
async fn consultas_de_citas() {
    let w = World::new();
    w.publish_schedule().await;
    let b = add_clinic(&w.d, w.network, "miraflores", ClinicStatus::Active);
    let other_doctor = Uuid::now_v7();
    add_doctor(&w.d, b.id, other_doctor, "Dr. Ruiz");
    let ctx_b = member(
        w.network,
        other_doctor,
        b.id,
        MembershipRole::Practitioner,
        true,
    );
    w.schedules()
        .set_weekly_schedule(&ctx_b, b.id, 30, all_week(9, 13))
        .await
        .unwrap();

    let me = w.patient();
    w.book_monday(&me, 9).await;
    w.booking()
        .book(&me, b.id, other_doctor, monday(11, 0))
        .await
        .unwrap();
    let mine = w.booking().my_appointments(&me).await.unwrap();
    assert_eq!(mine.len(), 2);
    assert!(mine.iter().any(|v| v.clinic_name == "Clínica miraflores"));

    w.book_monday(&w.patient(), 10).await;
    let agenda = w
        .booking()
        .clinic_appointments(&w.doctor_ctx(), w.clinic.id, monday(0, 0), monday(23, 0))
        .await
        .unwrap();
    assert_eq!(agenda.len(), 2);
    assert!(
        agenda
            .iter()
            .all(|v| v.appointment.practitioner_user_id == w.doctor)
    );
}

// ─── 5.6 Cancelar y reprogramar ─────────────────────────────────────────────

/// «Paciente cancela»: queda cancelada, el recordatorio se anula y el cupo vuelve.
#[tokio::test]
async fn paciente_cancela() {
    let w = World::new();
    w.publish_schedule().await;
    let me = w.patient();
    let a = w.book_monday(&me, 10).await;
    let cancelled = w.booking().cancel(&me, a.id).await.unwrap();
    assert_eq!(cancelled.status, AppointmentStatus::Cancelled);
    assert_eq!(cancelled.cancelled_by, Some(me.user_id));
    assert_eq!(
        w.d.appointments.reminder_of(&a.id).unwrap().status,
        ReminderStatus::Cancelled
    );
    w.book_monday(&w.patient(), 10).await;
}

/// «Cita ajena», «Paciente reprograma la cita de otro» y «Médico cancela una cita de
/// otro médico».
#[tokio::test]
async fn citas_ajenas() {
    let w = World::new();
    w.publish_schedule().await;
    let a = w.book_monday(&w.patient(), 10).await;
    let stranger = w.patient();
    assert!(matches!(
        w.booking().cancel(&stranger, a.id).await,
        Err(SchedulingError::NotFound(_))
    ));
    assert!(matches!(
        w.booking()
            .reschedule(&stranger, a.id, w.doctor, monday(11, 0))
            .await,
        Err(SchedulingError::NotFound(_))
    ));
    let other_doctor = member(
        w.network,
        Uuid::now_v7(),
        w.clinic.id,
        MembershipRole::Practitioner,
        true,
    );
    assert!(matches!(
        w.booking().cancel(&other_doctor, a.id).await,
        Err(SchedulingError::Denied(AccessDenied::Forbidden))
    ));
}

/// «Reprogramación exitosa» y «Nuevo cupo ocupado».
#[tokio::test]
async fn reprogramar() {
    let w = World::new();
    w.publish_schedule().await;
    let me = w.patient();
    let a = w.book_monday(&me, 10).await;
    w.book_monday(&w.patient(), 12).await;

    let tuesday_11 = days_later(monday(11, 0), 1);
    let moved = w
        .booking()
        .reschedule(&me, a.id, w.doctor, tuesday_11)
        .await
        .unwrap();
    assert_eq!(moved.start, tuesday_11);
    let reminder = w.d.appointments.reminder_of(&a.id).unwrap();
    assert_eq!(
        reminder.due_at,
        monday(11, 0),
        "24 h antes del nuevo horario"
    );
    assert_eq!(reminder.status, ReminderStatus::Pending);
    w.book_monday(&w.patient(), 10).await; // el lunes 10:00 vuelve a estar libre

    let busy = w
        .booking()
        .reschedule(&me, a.id, w.doctor, monday(12, 0))
        .await;
    assert!(matches!(busy, Err(SchedulingError::SlotUnavailable)));
    let kept = w.booking().my_appointments(&me).await.unwrap();
    assert_eq!(kept[0].appointment.start, tuesday_11, "conserva el cupo");

    let other = w
        .booking()
        .reschedule(&me, a.id, Uuid::now_v7(), monday(11, 0))
        .await;
    assert!(matches!(other, Err(SchedulingError::DifferentPractitioner)));
}

// ─── 6. Recepción ───────────────────────────────────────────────────────────

/// «Paciente llega», «Llegada en otro día», «Médico intenta registrar la llegada» y
/// «Administrador de otra clínica».
#[tokio::test]
async fn llegada() {
    let w = World::new();
    w.publish_schedule().await;
    let today = w.book_monday(&w.patient(), 10).await;
    let next_week = w
        .booking()
        .book(
            &w.patient(),
            w.clinic.id,
            w.doctor,
            days_later(monday(10, 0), 7),
        )
        .await
        .unwrap();
    let admin = w.admin();

    assert!(matches!(
        w.reception().check_in(&admin, next_week.id, None).await,
        Err(SchedulingError::NotToday)
    ));
    assert!(matches!(
        w.reception()
            .check_in(&w.doctor_ctx(), today.id, None)
            .await,
        Err(SchedulingError::Denied(AccessDenied::Forbidden))
    ));
    let elsewhere = add_clinic(&w.d, w.network, "otra", ClinicStatus::Active);
    let other_admin = member(
        w.network,
        Uuid::now_v7(),
        elsewhere.id,
        MembershipRole::Admin,
        false,
    );
    assert!(matches!(
        w.reception().check_in(&other_admin, today.id, None).await,
        Err(SchedulingError::NotFound(_))
    ));

    let result = w
        .reception()
        .check_in(&admin, today.id, None)
        .await
        .unwrap();
    assert_eq!(result.appointment.status, AppointmentStatus::Arrived);
    assert_eq!(result.appointment.arrived_by, Some(admin.user_id));
}

/// «Recepción verifica el DNI», «El documento no coincide» y «Paciente sin cuenta».
#[tokio::test]
async fn verificacion_del_documento() {
    let w = World::new();
    w.publish_schedule().await;
    let me = w.patient();
    let mine = w.book_monday(&me, 9).await;
    let admin = w.admin();
    let verified = w
        .reception()
        .check_in(&admin, mine.id, Some(DocumentCheck::Match))
        .await
        .unwrap();
    assert!(verified.account_verified);
    assert_eq!(*w.d.identity.verified.lock().unwrap(), vec![me.user_id]);

    let other = w.book_monday(&w.patient(), 10).await;
    let mismatch = w
        .reception()
        .check_in(&admin, other.id, Some(DocumentCheck::Mismatch))
        .await
        .unwrap();
    assert!(!mismatch.account_verified);
    assert_eq!(w.d.identity.verified.lock().unwrap().len(), 1);

    let walk_in = w
        .booking()
        .book_for_patient(
            &admin,
            w.clinic.id,
            w.doctor,
            monday(11, 0),
            PatientChoice::New(new_patient("9")),
        )
        .await
        .unwrap();
    let account = Uuid::now_v7();
    w.d.patients
        .documents
        .lock()
        .unwrap()
        .push((account, "12345678".into()));
    let result = w
        .reception()
        .check_in(&admin, walk_in.id, Some(DocumentCheck::Match))
        .await
        .unwrap();
    assert!(!result.account_verified);
    assert!(
        w.d.patients
            .verified_records
            .lock()
            .unwrap()
            .contains(&walk_in.patient_id)
    );
    assert_eq!(
        result.link_suggestions.len(),
        1,
        "sugiere la cuenta con su DNI"
    );
}

/// «Paciente no llega» y «Antes de la hora».
#[tokio::test]
async fn no_asistio() {
    let w = World::new();
    w.publish_schedule().await;
    let a = w.book_monday(&w.patient(), 10).await;
    let admin = w.admin();
    assert!(matches!(
        w.reception().mark_no_show(&admin, a.id).await,
        Err(SchedulingError::NotStarted)
    ));
    w.d.clock.set(monday(10, 40));
    let marked = w.reception().mark_no_show(&admin, a.id).await.unwrap();
    assert_eq!(marked.status, AppointmentStatus::NoShow);
}

/// «Paciente intenta reclamar un historial», «Documentos distintos», «Recepción vincula
/// con el DNI a la vista», «Citas anteriores visibles» y «Vinculación por error».
#[tokio::test]
async fn vincular_pacientes() {
    let w = World::new();
    w.publish_schedule().await;
    let admin = w.admin();
    let walk_in = w
        .booking()
        .book_for_patient(
            &admin,
            w.clinic.id,
            w.doctor,
            monday(10, 0),
            PatientChoice::New(new_patient("9")),
        )
        .await
        .unwrap();
    let account = patient(w.network, Uuid::now_v7());
    w.d.patients
        .documents
        .lock()
        .unwrap()
        .push((account.user_id, "12345678".into()));

    assert!(matches!(
        w.reception()
            .suggest_links(&account, w.clinic.id, walk_in.patient_id)
            .await,
        Err(SchedulingError::Denied(AccessDenied::Forbidden))
    ));
    let suggestions = w
        .reception()
        .suggest_links(&admin, w.clinic.id, walk_in.patient_id)
        .await
        .unwrap();
    assert_eq!(suggestions[0].user_id, account.user_id);

    let unrelated = Uuid::now_v7();
    w.d.patients
        .documents
        .lock()
        .unwrap()
        .push((unrelated, "87654321".into()));
    assert!(matches!(
        w.reception()
            .link_account(&admin, w.clinic.id, walk_in.patient_id, unrelated, true)
            .await,
        Err(SchedulingError::Invalid(_))
    ));
    assert!(matches!(
        w.reception()
            .link_account(
                &admin,
                w.clinic.id,
                walk_in.patient_id,
                account.user_id,
                false
            )
            .await,
        Err(SchedulingError::Invalid(_))
    ));

    let link = w
        .reception()
        .link_account(
            &admin,
            w.clinic.id,
            walk_in.patient_id,
            account.user_id,
            true,
        )
        .await
        .unwrap();
    assert!(
        w.d.identity
            .verified
            .lock()
            .unwrap()
            .contains(&account.user_id)
    );
    assert_eq!(
        w.booking().my_appointments(&account).await.unwrap().len(),
        1
    );

    assert!(matches!(
        w.reception()
            .unlink_account(&admin, w.clinic.id, link, "Persona equivocada")
            .await,
        Err(SchedulingError::Denied(AccessDenied::Forbidden))
    ));
    let owner = member(
        w.network,
        Uuid::now_v7(),
        w.clinic.id,
        MembershipRole::Owner,
        false,
    );
    w.reception()
        .unlink_account(&owner, w.clinic.id, link, "Persona equivocada")
        .await
        .unwrap();
    assert!(
        w.booking()
            .my_appointments(&account)
            .await
            .unwrap()
            .is_empty()
    );
}

// ─── 7. Recordatorios ───────────────────────────────────────────────────────

async fn booked_with_email(
    w: &World,
    hour: i8,
) -> (
    crate::domain::appointment::Appointment,
    crate::domain::reminder::Reminder,
) {
    let me = w.patient();
    w.d.identity
        .emails
        .lock()
        .unwrap()
        .push((me.user_id, "paciente@example.com".into()));
    let a = w
        .booking()
        .book(&me, w.clinic.id, w.doctor, days_later(monday(hour, 0), 3))
        .await
        .unwrap();
    let r = w.d.appointments.reminder_of(&a.id).unwrap();
    (a, r)
}

/// «Cita reservada con días de anticipación» y «Aviso entregado dos veces».
#[tokio::test]
async fn recordatorio_enviado_una_vez() {
    let w = World::new();
    w.publish_schedule().await;
    let (_, r) = booked_with_email(&w, 10).await;
    assert_eq!(r.due_at, days_later(monday(10, 0), 2), "un día antes");
    w.d.clock.set(r.due_at);
    assert_eq!(
        w.reminders().deliver(r.id, r.due_at).await.unwrap(),
        Delivery::Sent
    );
    assert_eq!(
        w.reminders().deliver(r.id, r.due_at).await.unwrap(),
        Delivery::Skipped
    );
    let sent = w.d.notifier.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, "paciente@example.com");
    assert_eq!(sent[0].1.practitioner_name, "Dra. García");
}

/// «Paciente sin correo».
#[tokio::test]
async fn recordatorio_sin_canal() {
    let w = World::new();
    w.publish_schedule().await;
    let a = w
        .booking()
        .book_for_patient(
            &w.admin(),
            w.clinic.id,
            w.doctor,
            monday(10, 0),
            PatientChoice::New(new_patient("9")),
        )
        .await
        .unwrap();
    let r = w.d.appointments.reminder_of(&a.id).unwrap();
    assert_eq!(
        w.reminders().deliver(r.id, r.due_at).await.unwrap(),
        Delivery::NoChannel
    );
    assert_eq!(
        w.d.appointments.reminder_of(&a.id).unwrap().status,
        ReminderStatus::NoChannel
    );
}

/// «Cita cancelada» y «Cita reprogramada»: el aviso viejo no se envía.
#[tokio::test]
async fn recordatorio_de_cita_cambiada() {
    let w = World::new();
    w.publish_schedule().await;
    let (a, old) = booked_with_email(&w, 10).await;
    let patient_ctx = patient(
        w.network,
        w.d.patients.patients.lock().unwrap()[0].user_id.unwrap(),
    );
    w.booking()
        .reschedule(&patient_ctx, a.id, w.doctor, days_later(monday(11, 0), 4))
        .await
        .unwrap();
    assert_eq!(
        w.reminders().deliver(old.id, old.due_at).await.unwrap(),
        Delivery::Skipped
    );
    let new = w.d.appointments.reminder_of(&a.id).unwrap();
    w.booking().cancel(&patient_ctx, a.id).await.unwrap();
    assert_eq!(
        w.reminders().deliver(new.id, new.due_at).await.unwrap(),
        Delivery::Skipped
    );
    assert!(w.d.notifier.sent.lock().unwrap().is_empty());
}

/// «Falla temporal del correo» y «Reintentos agotados».
#[tokio::test]
async fn reintentos() {
    let w = World::new();
    w.publish_schedule().await;
    let (_, r) = booked_with_email(&w, 10).await;
    *w.d.notifier.fail_first.lock().unwrap() = 1;
    assert_eq!(
        w.reminders().deliver(r.id, r.due_at).await.unwrap(),
        Delivery::Retrying
    );
    let retry = *w.d.scheduler.scheduled.lock().unwrap().last().unwrap();
    assert_eq!(
        w.reminders().deliver(retry.0, retry.1).await.unwrap(),
        Delivery::Sent
    );
    assert_eq!(w.d.notifier.sent.lock().unwrap().len(), 1);

    let (a2, r2) = booked_with_email(&w, 11).await;
    *w.d.notifier.fail_first.lock().unwrap() = 10;
    let mut due = r2.due_at;
    let mut outcome = Delivery::Retrying;
    for _ in 0..3 {
        outcome = w.reminders().deliver(r2.id, due).await.unwrap();
        due = w.d.appointments.reminder_of(&a2.id).unwrap().due_at;
    }
    assert_eq!(outcome, Delivery::Failed);
    assert_eq!(
        w.d.appointments.reminder_of(&a2.id).unwrap().status,
        ReminderStatus::Failed
    );
}

/// «Caída a mitad del envío»: un envío colgado vuelve a `PENDING` y se reenvía.
#[tokio::test]
async fn caida_a_mitad_del_envio() {
    let w = World::new();
    w.publish_schedule().await;
    let (a, r) = booked_with_email(&w, 10).await;
    {
        let mut reminders = w.d.appointments.reminders.lock().unwrap();
        let entry = reminders.iter_mut().find(|(x, _)| x.id == r.id).unwrap();
        entry.0.status = ReminderStatus::Sending;
    }
    w.d.appointments.set_claimed_at(&r.id, r.due_at);
    w.d.clock.set(plus_minutes(r.due_at, 10));
    assert_eq!(
        w.reminders().sweep().await.unwrap(),
        1,
        "se vuelve a encolar"
    );
    assert_eq!(
        w.d.appointments.reminder_of(&a.id).unwrap().status,
        ReminderStatus::Pending
    );
    assert_eq!(
        w.reminders().deliver(r.id, r.due_at).await.unwrap(),
        Delivery::Sent
    );
}
