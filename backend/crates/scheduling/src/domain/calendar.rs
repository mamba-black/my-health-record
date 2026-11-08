//! Tiempo de la agenda. Único lugar del dominio que usa `jiff` (regla 2 de la
//! arquitectura cebolla): el resto del crate usa los alias de este módulo.
//!
//! - Todo instante es UTC ([`Instant`]).
//! - Las franjas del horario son horas locales de la clínica ([`LocalTime`]) y se
//!   expanden en su zona ([`Zone`]), así el horario de verano se resuelve bien.
//! - La zona la carga `infrastructure`: aquí solo se usa, sin I/O.

pub type Instant = jiff::Timestamp;
pub type LocalTime = jiff::civil::Time;
pub type LocalDate = jiff::civil::Date;
pub type Zone = jiff::tz::TimeZone;

/// Día ISO de la semana: 1 = lunes … 7 = domingo.
pub type Weekday = u8;

/// Hora local `HH:MM`.
pub fn local_time(hour: i8, minute: i8) -> Option<LocalTime> {
    LocalTime::new(hour, minute, 0, 0).ok()
}

/// Lee una hora local `HH:MM`.
pub fn parse_local_time(text: &str) -> Option<LocalTime> {
    let (hour, minute) = text.trim().split_once(':')?;
    local_time(hour.parse().ok()?, minute.get(..2)?.parse().ok()?)
}

/// `HH:MM` de una hora local.
pub fn format_local_time(time: LocalTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

/// Minutos desde la medianoche.
pub fn minutes_of(time: LocalTime) -> i32 {
    i32::from(time.hour()) * 60 + i32::from(time.minute())
}

/// Suma minutos a un instante.
pub fn plus_minutes(instant: Instant, minutes: i64) -> Instant {
    instant
        .checked_add(jiff::SignedDuration::from_mins(minutes))
        .unwrap_or(instant)
}

/// Suma horas a un instante.
pub fn plus_hours(instant: Instant, hours: i64) -> Instant {
    plus_minutes(instant, hours * 60)
}

/// Fecha local de un instante en la zona.
pub fn local_date(instant: Instant, zone: &Zone) -> LocalDate {
    instant.to_zoned(zone.clone()).date()
}

/// Día ISO de la semana de una fecha.
pub fn weekday_of(date: LocalDate) -> Weekday {
    // `to_monday_one_offset` devuelve 1..=7.
    date.weekday().to_monday_one_offset().unsigned_abs()
}

/// Instante en que empieza `time` del día `date` en la zona. En un salto de horario
/// de verano se toma la interpretación compatible (la hora posterior al salto).
pub fn at(date: LocalDate, time: LocalTime, zone: &Zone) -> Option<Instant> {
    zone.to_ambiguous_zoned(date.to_datetime(time))
        .compatible()
        .ok()
        .map(|zoned| zoned.timestamp())
}

/// Fechas locales desde la de `from` hasta la de `to`, inclusive.
pub fn dates_between(from: Instant, to: Instant, zone: &Zone) -> Vec<LocalDate> {
    let (mut date, last) = (local_date(from, zone), local_date(to, zone));
    let mut dates = Vec::new();
    while date <= last {
        dates.push(date);
        match date.tomorrow() {
            Ok(next) => date = next,
            Err(_) => break,
        }
    }
    dates
}

/// Fecha y hora local legible: `jueves 12/03/2026 10:00`.
pub fn describe(instant: Instant, zone: &Zone) -> String {
    let zoned = instant.to_zoned(zone.clone());
    const DAYS: [&str; 7] = [
        "lunes",
        "martes",
        "miércoles",
        "jueves",
        "viernes",
        "sábado",
        "domingo",
    ];
    let day = DAYS[usize::from(weekday_of(zoned.date()).saturating_sub(1))];
    format!(
        "{day} {:02}/{:02}/{} {:02}:{:02}",
        zoned.day(),
        zoned.month(),
        zoned.year(),
        zoned.hour(),
        zoned.minute()
    )
}

/// Los rangos `[a1, a2)` y `[b1, b2)` se cruzan.
pub fn overlaps(a: (Instant, Instant), b: (Instant, Instant)) -> bool {
    a.0 < b.1 && b.0 < a.1
}
