import { AppointmentStatus } from "$shared/api";

export const APPOINTMENT_STATUS_LABELS: Record<AppointmentStatus, string> = {
  [AppointmentStatus.UNSPECIFIED]: "",
  [AppointmentStatus.BOOKED]: "Confirmada",
  [AppointmentStatus.ARRIVED]: "Llegó",
  [AppointmentStatus.CANCELLED]: "Cancelada",
  [AppointmentStatus.NOSHOW]: "No asistió",
};

/** Zona por defecto de las clínicas. */
export const DEFAULT_TIMEZONE = "America/Lima";

/** «jueves 5 de marzo, 10:00» en la zona de la clínica. */
export function formatDateTime(iso: string, timeZone = DEFAULT_TIMEZONE): string {
  return new Intl.DateTimeFormat("es-PE", {
    timeZone,
    weekday: "long",
    day: "numeric",
    month: "long",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(iso));
}

/** «10:00» en la zona de la clínica. */
export function formatTime(iso: string, timeZone = DEFAULT_TIMEZONE): string {
  return new Intl.DateTimeFormat("es-PE", {
    timeZone,
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(iso));
}

/** Clave de día local («2026-03-05») para agrupar cupos. */
export function dayKey(iso: string, timeZone = DEFAULT_TIMEZONE): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date(iso));
}

/** «jueves 5 de marzo» para el encabezado de un día. */
export function formatDay(iso: string, timeZone = DEFAULT_TIMEZONE): string {
  return new Intl.DateTimeFormat("es-PE", {
    timeZone,
    weekday: "long",
    day: "numeric",
    month: "long",
  }).format(new Date(iso));
}

/** La cita todavía no empieza. */
export const isUpcoming = (iso: string) => new Date(iso).getTime() > Date.now();
