//! Dobles en memoria de los puertos de la agenda (pruebas unitarias e integración).

use crate::application::integration::{
    ClinicDirectory, ClinicInfo, Clock, IdentityVerification, LinkSuggestion, NewPatientData,
    NotificationSender, PatientRef, PatientRegistry, PractitionerInfo, ReminderScheduler,
};
use crate::application::state::{SchedulingConfig, SchedulingState};
use crate::domain::appointment::Appointment;
use crate::domain::calendar::{Instant, overlaps};
use crate::domain::error::SchedulingError;
use crate::domain::reminder::{Reminder, ReminderMessage, ReminderStatus};
use crate::domain::repository::{
    AppointmentRepository, ClaimedReminder, OverlapCheck, ReminderRepository, ScheduleRepository,
};
use crate::domain::schedule::{Schedule, ScheduleException};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Default)]
pub struct InMemorySchedules {
    pub schedules: Mutex<Vec<Schedule>>,
    pub exceptions: Mutex<Vec<ScheduleException>>,
}

#[async_trait]
impl ScheduleRepository for InMemorySchedules {
    async fn find(&self, o: &Uuid, p: &Uuid) -> Result<Option<Schedule>, SchedulingError> {
        Ok(lock(&self.schedules)
            .iter()
            .find(|s| s.organization_id == *o && s.practitioner_user_id == *p)
            .cloned())
    }

    async fn by_clinic(&self, o: &Uuid) -> Result<Vec<Schedule>, SchedulingError> {
        Ok(lock(&self.schedules)
            .iter()
            .filter(|s| s.organization_id == *o)
            .cloned()
            .collect())
    }

    async fn save(&self, s: &Schedule, check: OverlapCheck<'_>) -> Result<(), SchedulingError> {
        let mut schedules = lock(&self.schedules);
        let others: Vec<Schedule> = schedules
            .iter()
            .filter(|o| {
                o.network_id == s.network_id
                    && o.practitioner_user_id == s.practitioner_user_id
                    && o.organization_id != s.organization_id
            })
            .cloned()
            .collect();
        check(&others)?;
        schedules.retain(|o| {
            !(o.organization_id == s.organization_id
                && o.practitioner_user_id == s.practitioner_user_id)
        });
        schedules.push(s.clone());
        Ok(())
    }

    async fn exceptions(&self, id: &Uuid) -> Result<Vec<ScheduleException>, SchedulingError> {
        Ok(lock(&self.exceptions)
            .iter()
            .filter(|e| e.schedule_id == *id)
            .cloned()
            .collect())
    }

    async fn add_exception(&self, e: &ScheduleException) -> Result<(), SchedulingError> {
        lock(&self.exceptions).push(e.clone());
        Ok(())
    }

    async fn remove_exception(&self, s: &Uuid, e: &Uuid) -> Result<bool, SchedulingError> {
        let mut exceptions = lock(&self.exceptions);
        let before = exceptions.len();
        exceptions.retain(|x| !(x.id == *e && x.schedule_id == *s));
        Ok(exceptions.len() < before)
    }
}

/// Citas y recordatorios en memoria, con la misma exclusión que la base.
#[derive(Default)]
pub struct InMemoryAppointments {
    pub appointments: Mutex<Vec<Appointment>>,
    pub reminders: Mutex<Vec<(Reminder, Option<Instant>)>>,
}

impl InMemoryAppointments {
    fn clashes(all: &[Appointment], a: &Appointment) -> bool {
        a.status.is_active()
            && all.iter().any(|o| {
                o.id != a.id
                    && o.status.is_active()
                    && o.network_id == a.network_id
                    && o.practitioner_user_id == a.practitioner_user_id
                    && overlaps((o.start, o.end), (a.start, a.end))
            })
    }

    pub fn reminder_of(&self, appointment_id: &Uuid) -> Option<Reminder> {
        lock(&self.reminders)
            .iter()
            .find(|(r, _)| r.appointment_id == *appointment_id)
            .map(|(r, _)| r.clone())
    }

    /// Simula que el reclamo de un envío ocurrió en `at` (para probar la caída).
    pub fn set_claimed_at(&self, id: &Uuid, at: Instant) {
        if let Some(entry) = lock(&self.reminders).iter_mut().find(|(r, _)| r.id == *id) {
            entry.1 = Some(at);
        }
    }
}

#[async_trait]
impl AppointmentRepository for InMemoryAppointments {
    async fn insert(&self, a: &Appointment, r: &Reminder) -> Result<(), SchedulingError> {
        let mut all = lock(&self.appointments);
        if Self::clashes(&all, a) {
            return Err(SchedulingError::SlotUnavailable);
        }
        all.push(a.clone());
        lock(&self.reminders).push((r.clone(), None));
        Ok(())
    }

    async fn find(&self, n: &Uuid, id: &Uuid) -> Result<Option<Appointment>, SchedulingError> {
        Ok(lock(&self.appointments)
            .iter()
            .find(|a| a.network_id == *n && a.id == *id)
            .cloned())
    }

    async fn update(&self, a: &Appointment) -> Result<(), SchedulingError> {
        let mut all = lock(&self.appointments);
        if Self::clashes(&all, a) {
            return Err(SchedulingError::SlotUnavailable);
        }
        let stored = all
            .iter_mut()
            .find(|o| o.id == a.id && o.network_id == a.network_id && o.version == a.version)
            .ok_or(SchedulingError::Conflict)?;
        *stored = Appointment {
            version: a.version + 1,
            ..a.clone()
        };
        Ok(())
    }

    async fn busy(
        &self,
        n: &Uuid,
        p: &Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<(Instant, Instant)>, SchedulingError> {
        Ok(lock(&self.appointments)
            .iter()
            .filter(|a| {
                a.network_id == *n
                    && a.practitioner_user_id == *p
                    && a.status.is_active()
                    && overlaps((a.start, a.end), (from, to))
            })
            .map(|a| (a.start, a.end))
            .collect())
    }

    async fn count_active(
        &self,
        o: &Uuid,
        p: &Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<usize, SchedulingError> {
        Ok(lock(&self.appointments)
            .iter()
            .filter(|a| {
                a.organization_id == *o
                    && a.practitioner_user_id == *p
                    && a.status.is_active()
                    && overlaps((a.start, a.end), (from, to))
            })
            .count())
    }

    async fn by_patients(
        &self,
        n: &Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<Appointment>, SchedulingError> {
        Ok(lock(&self.appointments)
            .iter()
            .filter(|a| a.network_id == *n && ids.contains(&a.patient_id))
            .cloned()
            .collect())
    }

    async fn by_clinic(
        &self,
        o: &Uuid,
        p: Option<&Uuid>,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<Appointment>, SchedulingError> {
        Ok(lock(&self.appointments)
            .iter()
            .filter(|a| {
                a.organization_id == *o
                    && p.is_none_or(|p| a.practitioner_user_id == *p)
                    && overlaps((a.start, a.end), (from, to))
            })
            .cloned()
            .collect())
    }
}

#[async_trait]
impl ReminderRepository for InMemoryAppointments {
    async fn of_appointment(&self, id: &Uuid) -> Result<Option<Reminder>, SchedulingError> {
        Ok(self.reminder_of(id))
    }

    async fn reset(
        &self,
        id: &Uuid,
        due_at: Instant,
        status: ReminderStatus,
    ) -> Result<Option<Reminder>, SchedulingError> {
        let mut reminders = lock(&self.reminders);
        let Some(entry) = reminders
            .iter_mut()
            .find(|(r, _)| r.appointment_id == *id && r.status != ReminderStatus::Sent)
        else {
            return Ok(None);
        };
        entry.0.due_at = due_at;
        entry.0.status = status;
        entry.0.attempts = 0;
        entry.1 = None;
        Ok(Some(entry.0.clone()))
    }

    async fn claim(
        &self,
        id: &Uuid,
        due_at: Instant,
        now: Instant,
    ) -> Result<Option<ClaimedReminder>, SchedulingError> {
        let reminder = {
            let mut reminders = lock(&self.reminders);
            let Some(entry) = reminders.iter_mut().find(|(r, _)| {
                r.id == *id && r.status == ReminderStatus::Pending && r.due_at == due_at
            }) else {
                return Ok(None);
            };
            entry.0.status = ReminderStatus::Sending;
            entry.0.attempts += 1;
            entry.1 = Some(now);
            entry.0.clone()
        };
        let appointment = lock(&self.appointments)
            .iter()
            .find(|a| a.id == reminder.appointment_id)
            .cloned()
            .ok_or_else(|| SchedulingError::NotFound("cita".into()))?;
        Ok(Some(ClaimedReminder {
            reminder,
            appointment,
        }))
    }

    async fn finish(
        &self,
        id: &Uuid,
        status: ReminderStatus,
        due_at: Instant,
        _error: Option<String>,
        _now: Instant,
    ) -> Result<(), SchedulingError> {
        if let Some(entry) = lock(&self.reminders)
            .iter_mut()
            .find(|(r, _)| r.id == *id && r.status == ReminderStatus::Sending)
        {
            entry.0.status = status;
            entry.0.due_at = due_at;
            entry.1 = None;
        }
        Ok(())
    }

    async fn release_stale(&self, before: Instant) -> Result<usize, SchedulingError> {
        let mut count = 0;
        for entry in lock(&self.reminders).iter_mut() {
            if entry.0.status == ReminderStatus::Sending && entry.1.is_some_and(|at| at < before) {
                entry.0.status = ReminderStatus::Pending;
                entry.1 = None;
                count += 1;
            }
        }
        Ok(count)
    }

    async fn pending_due(&self, now: Instant) -> Result<Vec<Reminder>, SchedulingError> {
        Ok(lock(&self.reminders)
            .iter()
            .filter(|(r, _)| r.status == ReminderStatus::Pending && r.due_at <= now)
            .map(|(r, _)| r.clone())
            .collect())
    }
}

/// Clínicas y médicos fijos.
#[derive(Default)]
pub struct FakeClinics {
    pub clinics: Mutex<Vec<ClinicInfo>>,
    /// (clínica, médico activo)
    pub practitioners: Mutex<Vec<(Uuid, PractitionerInfo)>>,
}

#[async_trait]
impl ClinicDirectory for FakeClinics {
    async fn by_subdomain(&self, s: &str) -> Result<Option<ClinicInfo>, SchedulingError> {
        Ok(lock(&self.clinics)
            .iter()
            .find(|c| c.subdomain == s)
            .cloned())
    }

    async fn by_id(&self, id: &Uuid) -> Result<Option<ClinicInfo>, SchedulingError> {
        Ok(lock(&self.clinics).iter().find(|c| c.id == *id).cloned())
    }

    async fn active_practitioners(
        &self,
        o: &Uuid,
    ) -> Result<Vec<PractitionerInfo>, SchedulingError> {
        Ok(lock(&self.practitioners)
            .iter()
            .filter(|(c, _)| c == o)
            .map(|(_, p)| p.clone())
            .collect())
    }

    async fn person_name(&self, u: &Uuid) -> Result<Option<String>, SchedulingError> {
        Ok(lock(&self.practitioners)
            .iter()
            .find(|(_, p)| p.user_id == *u)
            .map(|(_, p)| p.name.clone()))
    }
}

/// Pacientes, vínculos y documentos en memoria.
#[derive(Default)]
pub struct FakePatients {
    pub patients: Mutex<Vec<PatientRef>>,
    /// (id, registro, cuenta, activo)
    pub links: Mutex<Vec<(Uuid, Uuid, Uuid, bool)>>,
    /// Documento de cada registro y de cada cuenta.
    pub documents: Mutex<Vec<(Uuid, String)>>,
    pub verified_records: Mutex<Vec<Uuid>>,
}

#[async_trait]
impl PatientRegistry for FakePatients {
    async fn ensure_for_account(&self, o: &Uuid, u: &Uuid) -> Result<PatientRef, SchedulingError> {
        let mut patients = lock(&self.patients);
        if let Some(p) = patients
            .iter()
            .find(|p| p.organization_id == *o && p.user_id == Some(*u))
        {
            return Ok(p.clone());
        }
        let p = PatientRef {
            id: Uuid::now_v7(),
            organization_id: *o,
            user_id: Some(*u),
            name: "Paciente".into(),
        };
        patients.push(p.clone());
        Ok(p)
    }

    async fn register(
        &self,
        o: &Uuid,
        data: NewPatientData,
    ) -> Result<PatientRef, SchedulingError> {
        if data.phone.trim().is_empty() {
            return Err(SchedulingError::MissingPatientData("teléfono".into()));
        }
        let p = PatientRef {
            id: Uuid::now_v7(),
            organization_id: *o,
            user_id: None,
            name: format!("{} {}", data.given_name, data.family_name),
        };
        lock(&self.patients).push(p.clone());
        lock(&self.documents).push((p.id, data.document_value));
        Ok(p)
    }

    async fn find(&self, o: &Uuid, id: &Uuid) -> Result<Option<PatientRef>, SchedulingError> {
        Ok(lock(&self.patients)
            .iter()
            .find(|p| p.organization_id == *o && p.id == *id)
            .cloned())
    }

    async fn records_of_account(&self, _n: &Uuid, u: &Uuid) -> Result<Vec<Uuid>, SchedulingError> {
        let mut ids: Vec<Uuid> = lock(&self.patients)
            .iter()
            .filter(|p| p.user_id == Some(*u))
            .map(|p| p.id)
            .collect();
        ids.extend(
            lock(&self.links)
                .iter()
                .filter(|(_, _, user, active)| user == u && *active)
                .map(|(_, record, _, _)| *record),
        );
        Ok(ids)
    }

    async fn account_of(&self, id: &Uuid) -> Result<Option<Uuid>, SchedulingError> {
        if let Some(user) = lock(&self.patients)
            .iter()
            .find(|p| p.id == *id)
            .and_then(|p| p.user_id)
        {
            return Ok(Some(user));
        }
        Ok(lock(&self.links)
            .iter()
            .find(|(_, record, _, active)| record == id && *active)
            .map(|(_, _, user, _)| *user))
    }

    async fn mark_document_verified(&self, id: &Uuid, _by: &Uuid) -> Result<(), SchedulingError> {
        lock(&self.verified_records).push(*id);
        Ok(())
    }

    async fn suggest_links(
        &self,
        _n: &Uuid,
        _o: &Uuid,
        id: &Uuid,
    ) -> Result<Vec<LinkSuggestion>, SchedulingError> {
        let documents = lock(&self.documents);
        let Some((_, document)) = documents.iter().find(|(owner, _)| owner == id) else {
            return Ok(Vec::new());
        };
        Ok(documents
            .iter()
            .filter(|(owner, d)| d == document && owner != id)
            .filter(|(owner, _)| !lock(&self.patients).iter().any(|p| p.id == *owner))
            .map(|(owner, _)| LinkSuggestion {
                user_id: *owner,
                name: "Cuenta".into(),
                email: "cuenta@example.com".into(),
            })
            .collect())
    }

    async fn link(
        &self,
        _n: &Uuid,
        _o: &Uuid,
        id: &Uuid,
        u: &Uuid,
        _by: &Uuid,
    ) -> Result<Uuid, SchedulingError> {
        let documents = lock(&self.documents);
        let doc = |owner: &Uuid| documents.iter().find(|(o, _)| o == owner).map(|(_, d)| d);
        if doc(id).is_none() || doc(id) != doc(u) {
            return Err(SchedulingError::Invalid("El documento no coincide".into()));
        }
        let link = Uuid::now_v7();
        lock(&self.links).push((link, *id, *u, true));
        Ok(link)
    }

    async fn unlink(
        &self,
        _o: &Uuid,
        link: &Uuid,
        _by: &Uuid,
        _reason: &str,
    ) -> Result<Uuid, SchedulingError> {
        let mut links = lock(&self.links);
        let entry = links
            .iter_mut()
            .find(|(id, _, _, active)| id == link && *active)
            .ok_or_else(|| SchedulingError::NotFound("El vínculo no existe".into()))?;
        entry.3 = false;
        Ok(entry.2)
    }
}

/// Identidad: cuentas verificadas y correos.
#[derive(Default)]
pub struct FakeIdentity {
    pub verified: Mutex<Vec<Uuid>>,
    pub emails: Mutex<Vec<(Uuid, String)>>,
}

#[async_trait]
impl IdentityVerification for FakeIdentity {
    async fn verify_document(&self, u: &Uuid) -> Result<(), SchedulingError> {
        lock(&self.verified).push(*u);
        Ok(())
    }

    async fn email_of(&self, u: &Uuid) -> Result<Option<String>, SchedulingError> {
        Ok(lock(&self.emails)
            .iter()
            .find(|(user, _)| user == u)
            .map(|(_, e)| e.clone()))
    }
}

/// Espía de envíos. Falla las primeras `fail_first` veces.
#[derive(Default)]
pub struct SpyNotifier {
    pub sent: Mutex<Vec<(String, ReminderMessage)>>,
    pub fail_first: Mutex<usize>,
}

#[async_trait]
impl NotificationSender for SpyNotifier {
    async fn send(&self, to: &str, message: &ReminderMessage) -> Result<(), String> {
        let mut failures = lock(&self.fail_first);
        if *failures > 0 {
            *failures -= 1;
            return Err("servidor de correo no disponible".into());
        }
        lock(&self.sent).push((to.to_string(), message.clone()));
        Ok(())
    }
}

/// Despertador que solo anota lo programado.
#[derive(Default)]
pub struct RecordingScheduler {
    pub scheduled: Mutex<Vec<(Uuid, Instant)>>,
}

#[async_trait]
impl ReminderScheduler for RecordingScheduler {
    async fn schedule(&self, id: &Uuid, due_at: Instant) -> Result<(), SchedulingError> {
        lock(&self.scheduled).push((*id, due_at));
        Ok(())
    }
}

/// Reloj que se mueve a mano.
pub struct FixedClock(pub Mutex<Instant>);

impl FixedClock {
    pub fn set(&self, now: Instant) {
        *lock(&self.0) = now;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Instant {
        *lock(&self.0)
    }
}

/// Estado con todos los dobles, y acceso a ellos para afirmar.
pub struct Doubles {
    pub schedules: Arc<InMemorySchedules>,
    pub appointments: Arc<InMemoryAppointments>,
    pub clinics: Arc<FakeClinics>,
    pub patients: Arc<FakePatients>,
    pub identity: Arc<FakeIdentity>,
    pub notifier: Arc<SpyNotifier>,
    pub scheduler: Arc<RecordingScheduler>,
    pub clock: Arc<FixedClock>,
}

impl Doubles {
    pub fn new(now: Instant) -> Self {
        Self {
            schedules: Arc::default(),
            appointments: Arc::default(),
            clinics: Arc::default(),
            patients: Arc::default(),
            identity: Arc::default(),
            notifier: Arc::default(),
            scheduler: Arc::default(),
            clock: Arc::new(FixedClock(Mutex::new(now))),
        }
    }

    pub fn state(&self) -> SchedulingState {
        SchedulingState {
            schedules: self.schedules.clone(),
            appointments: self.appointments.clone(),
            reminders: self.appointments.clone(),
            clinics: self.clinics.clone(),
            patients: self.patients.clone(),
            identity: self.identity.clone(),
            notifier: self.notifier.clone(),
            scheduler: self.scheduler.clone(),
            clock: self.clock.clone(),
            config: SchedulingConfig::default(),
        }
    }
}
