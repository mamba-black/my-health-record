//! Horario semanal de un médico en una clínica (FHIR `Schedule`) y sus cupos
//! (FHIR `Slot`). Los cupos no se guardan: se calculan con
//! `horario − excepciones − citas activas − pasado − ventana de reserva`.

use crate::domain::calendar::{
    self, Instant, LocalTime, Weekday, Zone, at, dates_between, format_local_time, minutes_of,
    overlaps, plus_minutes, weekday_of,
};
use crate::domain::error::SchedulingError;
use uuid::Uuid;

/// Ventana máxima de reserva por defecto.
pub const BOOKING_WINDOW_DAYS: i64 = 60;

/// Nombre del día ISO.
pub fn weekday_name(weekday: Weekday) -> &'static str {
    match weekday {
        1 => "lunes",
        2 => "martes",
        3 => "miércoles",
        4 => "jueves",
        5 => "viernes",
        6 => "sábado",
        _ => "domingo",
    }
}

/// Franja semanal en hora local de la clínica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBlock {
    pub weekday: Weekday,
    pub start: LocalTime,
    pub end: LocalTime,
}

/// Horario de un médico en una clínica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    pub id: Uuid,
    pub network_id: Uuid,
    pub organization_id: Uuid,
    pub practitioner_user_id: Uuid,
    pub slot_minutes: u16,
    pub blocks: Vec<TimeBlock>,
}

impl Schedule {
    /// Valida y arma el horario: duración entre 5 y 480 minutos, franjas con inicio
    /// antes del fin, que admitan al menos una cita y que no se crucen el mismo día.
    pub fn new(
        id: Uuid,
        network_id: Uuid,
        organization_id: Uuid,
        practitioner_user_id: Uuid,
        slot_minutes: u16,
        mut blocks: Vec<TimeBlock>,
    ) -> Result<Self, SchedulingError> {
        if !(5..=480).contains(&slot_minutes) {
            return Err(SchedulingError::Invalid(
                "La duración de la cita debe estar entre 5 y 480 minutos".to_string(),
            ));
        }
        for block in &blocks {
            if !(1..=7).contains(&block.weekday) {
                return Err(SchedulingError::Invalid(
                    "El día de la semana debe estar entre 1 (lunes) y 7 (domingo)".to_string(),
                ));
            }
            if block.start >= block.end {
                return Err(SchedulingError::Invalid(format!(
                    "La franja del {} debe empezar antes de terminar",
                    weekday_name(block.weekday)
                )));
            }
            if minutes_of(block.end) - minutes_of(block.start) < i32::from(slot_minutes) {
                return Err(SchedulingError::Invalid(format!(
                    "La franja del {} de {} a {} no alcanza para una cita",
                    weekday_name(block.weekday),
                    format_local_time(block.start),
                    format_local_time(block.end)
                )));
            }
        }
        blocks.sort_by_key(|block| (block.weekday, block.start));
        for pair in blocks.windows(2) {
            if pair[0].weekday == pair[1].weekday && pair[1].start < pair[0].end {
                return Err(SchedulingError::Invalid(format!(
                    "Dos franjas del {} se cruzan",
                    weekday_name(pair[0].weekday)
                )));
            }
        }
        Ok(Self {
            id,
            network_id,
            organization_id,
            practitioner_user_id,
            slot_minutes,
            blocks,
        })
    }

    /// Primer cruce con el horario del mismo médico en otra clínica de su red: día y
    /// franja del conflicto, sin nada más de la otra clínica.
    pub fn first_overlap_with(&self, other: &Schedule) -> Option<(Weekday, LocalTime, LocalTime)> {
        for mine in &self.blocks {
            for theirs in other.blocks.iter().filter(|b| b.weekday == mine.weekday) {
                let start = mine.start.max(theirs.start);
                let end = mine.end.min(theirs.end);
                if start < end {
                    return Some((mine.weekday, start, end));
                }
            }
        }
        None
    }

    /// Todos los cupos del horario entre `from` y `to`, sin descontar nada.
    pub fn expand(&self, zone: &Zone, from: Instant, to: Instant) -> Vec<(Instant, Instant)> {
        let mut slots = Vec::new();
        let step = i32::from(self.slot_minutes);
        for date in dates_between(from, to, zone) {
            let weekday = weekday_of(date);
            for block in self.blocks.iter().filter(|b| b.weekday == weekday) {
                let mut minute = minutes_of(block.start);
                while minute + step <= minutes_of(block.end) {
                    let time = calendar::local_time((minute / 60) as i8, (minute % 60) as i8);
                    if let Some(start) = time.and_then(|time| at(date, time, zone)) {
                        let end = plus_minutes(start, i64::from(self.slot_minutes));
                        if start >= from && start < to {
                            slots.push((start, end));
                        }
                    }
                    minute += step;
                }
            }
        }
        slots.sort();
        slots.dedup();
        slots
    }
}

/// Ausencia del médico en su agenda de una clínica (feriado, vacaciones…).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleException {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub start: Instant,
    pub end: Instant,
    pub reason: Option<String>,
}

impl ScheduleException {
    pub fn new(
        id: Uuid,
        schedule_id: Uuid,
        start: Instant,
        end: Instant,
        reason: Option<String>,
    ) -> Result<Self, SchedulingError> {
        if start >= end {
            return Err(SchedulingError::Invalid(
                "La excepción debe empezar antes de terminar".to_string(),
            ));
        }
        Ok(Self {
            id,
            schedule_id,
            start,
            end,
            reason: reason
                .map(|reason| reason.trim().to_string())
                .filter(|reason| !reason.is_empty()),
        })
    }
}

/// Cupo libre (FHIR `Slot`). Su id se deriva de clínica, médico e inicio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub organization_id: Uuid,
    pub practitioner_user_id: Uuid,
    pub start: Instant,
    pub end: Instant,
}

impl Slot {
    pub fn id(&self) -> String {
        format!(
            "{}.{}.{}",
            self.organization_id.simple(),
            self.practitioner_user_id.simple(),
            self.start.as_second()
        )
    }
}

/// Lo que descuenta el cálculo de cupos.
pub struct Availability<'a> {
    pub schedule: &'a Schedule,
    pub zone: &'a Zone,
    pub exceptions: &'a [ScheduleException],
    /// Citas activas del médico en toda su red (también las de otras clínicas).
    pub busy: &'a [(Instant, Instant)],
    pub now: Instant,
    pub window_days: i64,
}

impl Availability<'_> {
    /// Fin de la ventana de reserva.
    fn horizon(&self) -> Instant {
        calendar::plus_hours(self.now, self.window_days * 24)
    }

    fn offered(&self, slot: (Instant, Instant)) -> bool {
        slot.0 > self.now
            && slot.0 < self.horizon()
            && !self
                .exceptions
                .iter()
                .any(|e| overlaps(slot, (e.start, e.end)))
    }

    /// Cupos libres entre `from` y `to`: solo futuros, dentro de la ventana, fuera de
    /// las excepciones y sin citas activas que se crucen.
    pub fn free_slots(&self, from: Instant, to: Instant) -> Vec<Slot> {
        let from = from.max(self.now);
        let to = to.min(self.horizon());
        if from >= to {
            return Vec::new();
        }
        self.schedule
            .expand(self.zone, from, to)
            .into_iter()
            .filter(|slot| self.offered(*slot))
            .filter(|slot| !self.busy.iter().any(|busy| overlaps(*slot, *busy)))
            .map(|(start, end)| Slot {
                organization_id: self.schedule.organization_id,
                practitioner_user_id: self.schedule.practitioner_user_id,
                start,
                end,
            })
            .collect()
    }

    /// El cupo que empieza en `start`, si el horario lo ofrece (sin mirar si está
    /// ocupado: eso lo garantiza la base al reservar).
    pub fn offered_slot(&self, start: Instant) -> Result<(Instant, Instant), SchedulingError> {
        let day_end = calendar::plus_hours(start, 24);
        self.schedule
            .expand(self.zone, start, day_end)
            .into_iter()
            .find(|slot| slot.0 == start)
            .filter(|slot| self.offered(*slot))
            .ok_or(SchedulingError::SlotNotOffered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::calendar::{LocalDate, local_time};

    fn lima() -> Zone {
        Zone::get("America/Lima").expect("zona de Lima")
    }

    fn t(hour: i8, minute: i8) -> LocalTime {
        local_time(hour, minute).unwrap()
    }

    fn block(weekday: Weekday, from: (i8, i8), to: (i8, i8)) -> TimeBlock {
        TimeBlock {
            weekday,
            start: t(from.0, from.1),
            end: t(to.0, to.1),
        }
    }

    fn schedule(slot_minutes: u16, blocks: Vec<TimeBlock>) -> Result<Schedule, SchedulingError> {
        Schedule::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            slot_minutes,
            blocks,
        )
    }

    fn instant(date: LocalDate, hour: i8, minute: i8, zone: &Zone) -> Instant {
        at(date, t(hour, minute), zone).unwrap()
    }

    /// Lunes 2 de marzo de 2026.
    fn monday() -> LocalDate {
        LocalDate::new(2026, 3, 2).unwrap()
    }

    fn availability<'a>(
        schedule: &'a Schedule,
        zone: &'a Zone,
        exceptions: &'a [ScheduleException],
        busy: &'a [(Instant, Instant)],
        now: Instant,
    ) -> Availability<'a> {
        Availability {
            schedule,
            zone,
            exceptions,
            busy,
            now,
            window_days: BOOKING_WINDOW_DAYS,
        }
    }

    /// «Médico define su horario»: lunes y miércoles de 09:00 a 13:00, citas de 30 min.
    #[test]
    fn medico_define_su_horario() {
        let zone = lima();
        let s = schedule(
            30,
            vec![block(1, (9, 0), (13, 0)), block(3, (9, 0), (13, 0))],
        )
        .unwrap();
        let day = monday();
        let slots = s.expand(
            &zone,
            instant(day, 0, 0, &zone),
            instant(day.tomorrow().unwrap(), 0, 0, &zone),
        );
        assert_eq!(slots.len(), 8);
        assert_eq!(slots[0].0, instant(day, 9, 0, &zone));
        assert_eq!(slots[7].0, instant(day, 12, 30, &zone));
        let tuesday = day.tomorrow().unwrap();
        assert!(
            s.expand(
                &zone,
                instant(tuesday, 0, 0, &zone),
                instant(tuesday.tomorrow().unwrap(), 0, 0, &zone)
            )
            .is_empty()
        );
    }

    /// «Franjas que se cruzan».
    #[test]
    fn franjas_que_se_cruzan() {
        let result = schedule(
            30,
            vec![block(1, (9, 0), (12, 0)), block(1, (11, 0), (14, 0))],
        );
        assert!(matches!(result, Err(SchedulingError::Invalid(_))));
    }

    #[test]
    fn franja_invertida_o_corta() {
        assert!(schedule(30, vec![block(1, (12, 0), (9, 0))]).is_err());
        assert!(schedule(60, vec![block(1, (9, 0), (9, 30))]).is_err());
        assert!(schedule(2, vec![block(1, (9, 0), (10, 0))]).is_err());
    }

    /// «Cruce con otra clínica de la red»: solo el día y la franja del conflicto.
    #[test]
    fn cruce_con_otra_clinica() {
        let a = schedule(30, vec![block(1, (9, 0), (13, 0))]).unwrap();
        let b = schedule(30, vec![block(1, (12, 0), (15, 0))]).unwrap();
        assert_eq!(b.first_overlap_with(&a), Some((1, t(12, 0), t(13, 0))));
        let c = schedule(30, vec![block(2, (12, 0), (15, 0))]).unwrap();
        assert_eq!(c.first_overlap_with(&a), None);
    }

    /// «Cupo de hoy ya pasado»: a las 11:10 no se ofrecen los de las 11:00 o antes.
    #[test]
    fn cupo_de_hoy_ya_pasado() {
        let zone = lima();
        let s = schedule(30, vec![block(1, (9, 0), (13, 0))]).unwrap();
        let day = monday();
        let now = instant(day, 11, 10, &zone);
        let free = availability(&s, &zone, &[], &[], now)
            .free_slots(instant(day, 0, 0, &zone), instant(day, 23, 0, &zone));
        let starts: Vec<_> = free.iter().map(|slot| slot.start).collect();
        assert_eq!(
            starts,
            vec![
                instant(day, 11, 30, &zone),
                instant(day, 12, 0, &zone),
                instant(day, 12, 30, &zone)
            ]
        );
    }

    /// «Fuera de la ventana de reserva»: nada más allá de 60 días.
    #[test]
    fn fuera_de_la_ventana_de_reserva() {
        let zone = lima();
        let s = schedule(30, (1..=7).map(|d| block(d, (9, 0), (10, 0))).collect()).unwrap();
        let now = instant(monday(), 8, 0, &zone);
        let free = availability(&s, &zone, &[], &[], now)
            .free_slots(now, calendar::plus_hours(now, 90 * 24));
        let horizon = calendar::plus_hours(now, BOOKING_WINDOW_DAYS * 24);
        assert!(!free.is_empty());
        assert!(free.iter().all(|slot| slot.start < horizon));
    }

    /// «Vacaciones sin citas»: no se ofrecen cupos en el rango de la excepción.
    #[test]
    fn vacaciones_sin_citas() {
        let zone = lima();
        let s = schedule(30, vec![block(1, (9, 0), (13, 0))]).unwrap();
        let day = monday();
        let now = instant(day, 7, 0, &zone);
        let vacation = ScheduleException::new(
            Uuid::now_v7(),
            s.id,
            instant(day, 0, 0, &zone),
            instant(day.tomorrow().unwrap(), 0, 0, &zone),
            Some("Vacaciones".into()),
        )
        .unwrap();
        let free = availability(&s, &zone, &[vacation], &[], now)
            .free_slots(now, instant(day, 23, 0, &zone));
        assert!(free.is_empty());
    }

    /// «Se acorta el horario»: la cita de las 12:30 se conserva (no se toca) y el lunes
    /// ya no ofrece cupos después de las 12:00.
    #[test]
    fn se_acorta_el_horario() {
        let zone = lima();
        let day = monday();
        let now = instant(day, 7, 0, &zone);
        let shorter = schedule(30, vec![block(1, (9, 0), (12, 0))]).unwrap();
        let booked = [(instant(day, 12, 30, &zone), instant(day, 13, 0, &zone))];
        let free = availability(&shorter, &zone, &[], &booked, now)
            .free_slots(now, instant(day, 23, 0, &zone));
        assert_eq!(free.len(), 6);
        assert!(
            free.iter()
                .all(|slot| slot.end <= instant(day, 12, 0, &zone))
        );
    }

    /// Un cupo ocupado (aunque sea en otra clínica de la red) no se ofrece.
    #[test]
    fn cupo_ocupado_no_se_ofrece() {
        let zone = lima();
        let day = monday();
        let now = instant(day, 7, 0, &zone);
        let s = schedule(30, vec![block(1, (9, 0), (10, 0))]).unwrap();
        let busy = [(instant(day, 9, 15, &zone), instant(day, 9, 45, &zone))];
        let free =
            availability(&s, &zone, &[], &busy, now).free_slots(now, instant(day, 23, 0, &zone));
        assert!(
            free.is_empty(),
            "ambos cupos se cruzan con la cita: {free:?}"
        );
    }

    /// Día con cambio de horario de verano (Nueva York, 8 de marzo de 2026): los cupos
    /// siguen a las 09:00 locales y duran 30 minutos reales.
    #[test]
    fn dia_con_cambio_de_horario_de_verano() {
        let zone = Zone::get("America/New_York").expect("zona de Nueva York");
        let sunday = LocalDate::new(2026, 3, 8).unwrap();
        let s = schedule(
            30,
            vec![block(7, (1, 0), (4, 0)), block(7, (9, 0), (10, 0))],
        )
        .unwrap();
        let slots = s.expand(
            &zone,
            instant(sunday, 0, 0, &zone),
            instant(sunday, 23, 0, &zone),
        );
        let nine = slots
            .iter()
            .find(|slot| slot.0.to_zoned(zone.clone()).hour() == 9)
            .expect("cupo de las 09:00");
        assert_eq!(nine.0.to_zoned(zone.clone()).minute(), 0);
        assert_eq!(nine.0.duration_until(nine.1).as_mins(), 30);
        // 02:00–03:00 no existe ese día: los cupos de esa hora caen después del salto.
        assert!(slots.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn el_cupo_ofrecido_debe_existir_en_el_horario() {
        let zone = lima();
        let day = monday();
        let now = instant(day, 7, 0, &zone);
        let s = schedule(30, vec![block(1, (9, 0), (10, 0))]).unwrap();
        let a = availability(&s, &zone, &[], &[], now);
        assert!(a.offered_slot(instant(day, 9, 30, &zone)).is_ok());
        assert!(matches!(
            a.offered_slot(instant(day, 9, 10, &zone)),
            Err(SchedulingError::SlotNotOffered)
        ));
    }
}
