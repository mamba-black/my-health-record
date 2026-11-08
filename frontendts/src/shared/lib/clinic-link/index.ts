import { browser } from "$app/environment";

const CLINIC_KEY = "clinic_link";

/**
 * Recuerda la clínica por cuyo enlace (`/c/<subdominio>`) entró la persona: si crea
 * su cuenta, queda en la organización de esa clínica.
 */
export function rememberClinicLink(subdomain: string) {
  if (browser) sessionStorage.setItem(CLINIC_KEY, subdomain);
}

export function rememberedClinicLink(): string {
  return browser ? (sessionStorage.getItem(CLINIC_KEY) ?? "") : "";
}
