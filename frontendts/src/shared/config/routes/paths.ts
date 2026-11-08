import { resolve } from "$app/paths";

export const ROOT = resolve(`/`);
export const HISTORY = resolve(`/patients`);
export const LOGIN = resolve(`/login`);
export const SIGNUP = resolve(`/signup`);
export const COMPLETE_PROFILE = resolve(`/complete-profile`);
export const CALLBACK = resolve(`/auth/google/callback`);
export const SEARCH = resolve(`/search`);
export const REGISTER_PATIENT = resolve(`/patients/new`);
export const MY_APPOINTMENTS = resolve(`/appointments`);
export const MY_SCHEDULE = resolve(`/schedule`);
export const CLINIC_AGENDA = resolve(`/clinic-agenda`);
/** Prefijo de la página pública de cada clínica (`/c/<subdominio>`). */
export const CLINIC_PUBLIC_PREFIX = resolve(`/c/`);
export const MY_CLINICS = resolve(`/my-clinics`);
export const ACCESS_DENIED = resolve(`/access-denied`);

export class Page {
  constructor(public path: string) {}
}

export default Page;
