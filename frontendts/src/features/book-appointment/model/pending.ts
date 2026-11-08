import { browser } from "$app/environment";

/** Reserva que espera a que el paciente inicie sesión para continuar con el mismo cupo. */
export interface PendingBooking {
  subdomain: string;
  organizationId: string;
  practitionerUserId: string;
  start: string;
}

const KEY = "pending_booking";

export function savePendingBooking(booking: PendingBooking) {
  if (browser) sessionStorage.setItem(KEY, JSON.stringify(booking));
}

export function takePendingBooking(subdomain: string): PendingBooking | null {
  if (!browser) return null;
  const raw = sessionStorage.getItem(KEY);
  if (!raw) return null;
  sessionStorage.removeItem(KEY);
  try {
    const booking = JSON.parse(raw) as PendingBooking;
    return booking.subdomain === subdomain ? booking : null;
  } catch {
    return null;
  }
}
