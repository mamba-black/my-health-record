//! Cita (FHIR `Appointment`). Nace confirmada (`booked`) y solo cambia por las
//! transiciones de este módulo, cada una con su auditoría.

use crate::domain::calendar::{Instant, Zone, local_date};
use crate::domain::error::SchedulingError;
use uuid::Uuid;

/// Estado de la cita (FHIR `booked`, `arrived`, `cancelled`, `noshow`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppointmentStatus {
    Booked,
    Arrived,
    Cancelled,
    NoShow,
}

impl AppointmentStatus {
    pub fn code(self) -> &'static str {
        match self {
            AppointmentStatus::Booked => "BOOKED",
            AppointmentStatus::Arrived => "ARRIVED",
            AppointmentStatus::Cancelled => "CANCELLED",
            AppointmentStatus::NoShow => "NOSHOW",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "BOOKED" => Some(AppointmentStatus::Booked),
            "ARRIVED" => Some(AppointmentStatus::Arrived),
            "CANCELLED" => Some(AppointmentStatus::Cancelled),
            "NOSHOW" => Some(AppointmentStatus::NoShow),
            _ => None,
        }
    }

    /// Ocupa el horario del médico (lo cuenta la exclusión de la base).
    pub fn is_active(self) -> bool {
        matches!(self, AppointmentStatus::Booked | AppointmentStatus::Arrived)
    }
}

/// Resultado de comparar el documento físico con el registrado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentCheck {
    Match,
    Mismatch,
}

impl DocumentCheck {
    pub fn code(self) -> &'static str {
        match self {
            DocumentCheck::Match => "MATCH",
            DocumentCheck::Mismatch => "MISMATCH",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "MATCH" => Some(DocumentCheck::Match),
            "MISMATCH" => Some(DocumentCheck::Mismatch),
            _ => None,
        }
    }
}

/// Cita de un paciente con un médico en una clínica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appointment {
    pub id: Uuid,
    pub network_id: Uuid,
    pub organization_id: Uuid,
    pub practitioner_user_id: Uuid,
    /// Registro del paciente en la clínica (`administration.patient.id`).
    pub patient_id: Uuid,
    pub start: Instant,
    pub end: Instant,
    pub status: AppointmentStatus,
    /// Versión para el control de concurrencia optimista.
    pub version: i32,
    pub booked_by: Uuid,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<Instant>,
    pub arrived_by: Option<Uuid>,
    pub arrived_at: Option<Instant>,
    pub document_check: Option<DocumentCheck>,
    pub noshow_by: Option<Uuid>,
    pub noshow_at: Option<Instant>,
}

impl Appointment {
    /// Cita recién reservada: siempre confirmada, aunque la identidad del paciente aún
    /// no esté verificada en persona.
    pub fn book(
        network_id: Uuid,
        organization_id: Uuid,
        practitioner_user_id: Uuid,
        patient_id: Uuid,
        slot: (Instant, Instant),
        booked_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            network_id,
            organization_id,
            practitioner_user_id,
            patient_id,
            start: slot.0,
            end: slot.1,
            status: AppointmentStatus::Booked,
            version: 1,
            booked_by,
            cancelled_by: None,
            cancelled_at: None,
            arrived_by: None,
            arrived_at: None,
            document_check: None,
            noshow_by: None,
            noshow_at: None,
        }
    }

    fn require_future_booking(&self, now: Instant) -> Result<(), SchedulingError> {
        if self.status != AppointmentStatus::Booked {
            return Err(SchedulingError::InvalidState);
        }
        if now >= self.start {
            return Err(SchedulingError::AlreadyStarted);
        }
        Ok(())
    }

    /// Cancela una cita que todavía no empieza. El cupo vuelve a ofrecerse.
    pub fn cancel(&mut self, by: Uuid, now: Instant) -> Result<(), SchedulingError> {
        self.require_future_booking(now)?;
        self.status = AppointmentStatus::Cancelled;
        self.cancelled_by = Some(by);
        self.cancelled_at = Some(now);
        Ok(())
    }

    /// Mueve la cita a otro cupo del mismo médico en la misma clínica.
    pub fn reschedule(
        &mut self,
        practitioner_user_id: Uuid,
        organization_id: Uuid,
        slot: (Instant, Instant),
        now: Instant,
    ) -> Result<(), SchedulingError> {
        self.require_future_booking(now)?;
        if practitioner_user_id != self.practitioner_user_id
            || organization_id != self.organization_id
        {
            return Err(SchedulingError::DifferentPractitioner);
        }
        self.start = slot.0;
        self.end = slot.1;
        Ok(())
    }

    /// Registra la llegada, solo el día local de la cita.
    pub fn check_in(
        &mut self,
        by: Uuid,
        now: Instant,
        zone: &Zone,
        document_check: Option<DocumentCheck>,
    ) -> Result<(), SchedulingError> {
        if self.status != AppointmentStatus::Booked {
            return Err(SchedulingError::InvalidState);
        }
        if local_date(now, zone) != local_date(self.start, zone) {
            return Err(SchedulingError::NotToday);
        }
        self.status = AppointmentStatus::Arrived;
        self.arrived_by = Some(by);
        self.arrived_at = Some(now);
        self.document_check = document_check;
        Ok(())
    }

    /// Marca la inasistencia, solo después de la hora de inicio. No libera el cupo.
    pub fn mark_no_show(&mut self, by: Uuid, now: Instant) -> Result<(), SchedulingError> {
        if self.status != AppointmentStatus::Booked {
            return Err(SchedulingError::InvalidState);
        }
        if now < self.start {
            return Err(SchedulingError::NotStarted);
        }
        self.status = AppointmentStatus::NoShow;
        self.noshow_by = Some(by);
        self.noshow_at = Some(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::calendar::{LocalDate, at, local_time, plus_minutes};

    fn zone() -> Zone {
        Zone::get("America/Lima").unwrap()
    }

    fn day() -> LocalDate {
        LocalDate::new(2026, 3, 5).unwrap()
    }

    fn time(hour: i8, minute: i8) -> Instant {
        at(day(), local_time(hour, minute).unwrap(), &zone()).unwrap()
    }

    fn appointment() -> Appointment {
        let start = time(10, 0);
        Appointment::book(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            (start, plus_minutes(start, 30)),
            Uuid::now_v7(),
        )
    }

    #[test]
    fn nace_confirmada() {
        assert_eq!(appointment().status, AppointmentStatus::Booked);
    }

    /// «Paciente cancela» y «Cita ya empezada».
    #[test]
    fn cancelar_antes_y_despues_del_inicio() {
        let mut a = appointment();
        let by = Uuid::now_v7();
        a.cancel(by, time(9, 0)).expect("antes del inicio");
        assert_eq!(a.status, AppointmentStatus::Cancelled);
        assert_eq!(a.cancelled_by, Some(by));

        let mut b = appointment();
        assert!(matches!(
            b.cancel(by, time(10, 0)),
            Err(SchedulingError::AlreadyStarted)
        ));
    }

    /// «Cambio de médico»: se reprograma solo con el mismo médico y clínica.
    #[test]
    fn reprogramar_con_otro_medico() {
        let mut a = appointment();
        let slot = (time(11, 0), time(11, 30));
        assert!(matches!(
            a.reschedule(Uuid::now_v7(), a.organization_id, slot, time(8, 0)),
            Err(SchedulingError::DifferentPractitioner)
        ));
        let (practitioner, organization) = (a.practitioner_user_id, a.organization_id);
        a.reschedule(practitioner, organization, slot, time(8, 0))
            .expect("mismo médico");
        assert_eq!(a.start, time(11, 0));
    }

    /// «Antes de la hora» y «Paciente no llega».
    #[test]
    fn no_asistio_solo_despues_del_inicio() {
        let mut a = appointment();
        let by = Uuid::now_v7();
        assert!(matches!(
            a.mark_no_show(by, time(9, 50)),
            Err(SchedulingError::NotStarted)
        ));
        a.mark_no_show(by, time(10, 40))
            .expect("después del inicio");
        assert_eq!(a.status, AppointmentStatus::NoShow);
        // «no asistió» no se cancela ni se reprograma.
        assert!(a.cancel(by, time(9, 0)).is_err());
        assert!(
            a.clone()
                .reschedule(
                    a.practitioner_user_id,
                    a.organization_id,
                    (time(11, 0), time(11, 30)),
                    time(9, 0)
                )
                .is_err()
        );
    }

    #[test]
    fn llegada_solo_el_dia_de_la_cita() {
        let zone = zone();
        let mut a = appointment();
        let next_week = plus_minutes(time(9, 0), 7 * 24 * 60);
        assert!(matches!(
            a.check_in(Uuid::now_v7(), next_week, &zone, None),
            Err(SchedulingError::NotToday)
        ));
        a.check_in(
            Uuid::now_v7(),
            time(9, 55),
            &zone,
            Some(DocumentCheck::Match),
        )
        .expect("mismo día");
        assert_eq!(a.status, AppointmentStatus::Arrived);
        assert_eq!(a.document_check, Some(DocumentCheck::Match));
    }
}
