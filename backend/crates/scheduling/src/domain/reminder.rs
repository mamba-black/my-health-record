//! Recordatorio de cita. La tabla es la fuente de verdad del envío; la cola solo
//! despierta al worker. Antes repetido que perdido.

use crate::domain::calendar::{Instant, Zone, describe, plus_hours};
use uuid::Uuid;

/// Anticipación por defecto del recordatorio.
pub const DEFAULT_LEAD_HOURS: i64 = 24;
/// Intentos de envío antes de darlo por fallido.
pub const DEFAULT_MAX_ATTEMPTS: i16 = 3;
/// Tiempo tras el cual un envío en curso se da por caído y se reintenta.
pub const DEFAULT_SENDING_TIMEOUT_MINUTES: i64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReminderStatus {
    Pending,
    Sending,
    Sent,
    Failed,
    NoChannel,
    Cancelled,
}

impl ReminderStatus {
    pub fn code(self) -> &'static str {
        match self {
            ReminderStatus::Pending => "PENDING",
            ReminderStatus::Sending => "SENDING",
            ReminderStatus::Sent => "SENT",
            ReminderStatus::Failed => "FAILED",
            ReminderStatus::NoChannel => "NO_CHANNEL",
            ReminderStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "PENDING" => Some(ReminderStatus::Pending),
            "SENDING" => Some(ReminderStatus::Sending),
            "SENT" => Some(ReminderStatus::Sent),
            "FAILED" => Some(ReminderStatus::Failed),
            "NO_CHANNEL" => Some(ReminderStatus::NoChannel),
            "CANCELLED" => Some(ReminderStatus::Cancelled),
            _ => None,
        }
    }
}

/// Recordatorio de una cita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reminder {
    pub id: Uuid,
    pub network_id: Uuid,
    pub appointment_id: Uuid,
    pub due_at: Instant,
    pub status: ReminderStatus,
    pub attempts: i16,
}

impl Reminder {
    /// Recordatorio pendiente de una cita nueva.
    pub fn for_appointment(
        network_id: Uuid,
        appointment_id: Uuid,
        start: Instant,
        lead_hours: i64,
        now: Instant,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            network_id,
            appointment_id,
            due_at: due_at(start, lead_hours, now),
            status: ReminderStatus::Pending,
            attempts: 0,
        }
    }
}

/// Cuándo se envía: `inicio − anticipación`, o ya mismo si eso quedó en el pasado.
pub fn due_at(start: Instant, lead_hours: i64, now: Instant) -> Instant {
    plus_hours(start, -lead_hours).max(now)
}

/// Espera antes del siguiente intento tras un fallo: 1, 4, 9… minutos.
pub fn retry_delay_minutes(attempts: i16) -> i64 {
    let attempts = i64::from(attempts.max(1));
    attempts * attempts
}

/// Datos del aviso. No lleva motivo de consulta ni datos clínicos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderMessage {
    pub clinic_name: String,
    pub clinic_address: Option<String>,
    pub practitioner_name: String,
    pub start: Instant,
    pub zone: Zone,
}

impl ReminderMessage {
    pub fn subject(&self) -> String {
        format!("Recordatorio de tu cita en {}", self.clinic_name)
    }

    /// Texto fijo e idempotente: no cambia entre envíos repetidos.
    pub fn body(&self) -> String {
        let address = self
            .clinic_address
            .as_deref()
            .map(|address| format!("\nDirección: {address}"))
            .unwrap_or_default();
        format!(
            "Te recordamos tu cita.\n\nFecha y hora: {}\nMédico: {}\nClínica: {}{address}\n\n\
             Si no puedes asistir, cancélala o cámbiala desde «Mis citas».",
            describe(self.start, &self.zone),
            self.practitioner_name,
            self.clinic_name,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::calendar::{LocalDate, at, local_time};

    fn zone() -> Zone {
        Zone::get("America/Lima").unwrap()
    }

    fn instant(day: i8, hour: i8) -> Instant {
        at(
            LocalDate::new(2026, 3, day).unwrap(),
            local_time(hour, 0).unwrap(),
            &zone(),
        )
        .unwrap()
    }

    /// «Cita reservada con días de anticipación»: el miércoles a las 10:00.
    #[test]
    fn un_dia_antes() {
        assert_eq!(due_at(instant(5, 10), 24, instant(1, 9)), instant(4, 10));
    }

    /// «Cita para dentro de 3 horas»: de inmediato.
    #[test]
    fn de_inmediato_si_falta_poco() {
        let now = instant(5, 7);
        assert_eq!(due_at(instant(5, 10), 24, now), now);
    }

    /// El texto lleva fecha, hora local, médico, clínica y dirección, y no lleva motivo.
    #[test]
    fn texto_del_correo() {
        let message = ReminderMessage {
            clinic_name: "Clínica San Borja".into(),
            clinic_address: Some("Av. Aviación 123".into()),
            practitioner_name: "Dra. Elena García".into(),
            start: instant(5, 10),
            zone: zone(),
        };
        let body = message.body();
        assert!(body.contains("jueves 05/03/2026 10:00"), "{body}");
        assert!(body.contains("Dra. Elena García"));
        assert!(body.contains("Clínica San Borja"));
        assert!(body.contains("Av. Aviación 123"));
        assert!(!body.to_lowercase().contains("motivo"));
        assert_eq!(body, message.body(), "idempotente");
    }
}
