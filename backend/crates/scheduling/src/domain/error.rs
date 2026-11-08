use app_core::domain::access::AccessDenied;
use app_core::domain::error::ClickCareError;
use thiserror::Error;

/// Errores de la agenda.
#[derive(Debug, Error)]
pub enum SchedulingError {
    #[error("Acceso denegado: {0:?}")]
    Denied(AccessDenied),
    /// Lo ajeno o lo inexistente: no se revela cuál de los dos es.
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Invalid(String),
    /// Datos de un paciente nuevo que faltan (los escribe recepción).
    #[error("Faltan datos del paciente: {0}")]
    MissingPatientData(String),
    #[error("El horario se cruza con tu agenda en otra clínica el {weekday} entre {start} y {end}")]
    CrossClinicOverlap {
        weekday: String,
        start: String,
        end: String,
    },
    #[error("Hay {0} citas activas en ese rango: reprográmalas o cancélalas primero")]
    ExceptionHasAppointments(usize),
    #[error("El cupo ya no está disponible")]
    SlotUnavailable,
    #[error("Ese horario no es un cupo de la agenda")]
    SlotNotOffered,
    #[error("La cita ya empezó")]
    AlreadyStarted,
    #[error("La cita todavía no empieza")]
    NotStarted,
    #[error("La llegada solo se registra el día de la cita")]
    NotToday,
    #[error("Solo se reprograma con el mismo médico: cancela y reserva de nuevo")]
    DifferentPractitioner,
    #[error("La cita no está en un estado que permita esta acción")]
    InvalidState,
    #[error("La cita cambió mientras la editabas: vuelve a intentarlo")]
    Conflict,
    #[error(transparent)]
    Unknown(#[from] ClickCareError),
}

impl From<AccessDenied> for SchedulingError {
    fn from(denied: AccessDenied) -> Self {
        SchedulingError::Denied(denied)
    }
}
