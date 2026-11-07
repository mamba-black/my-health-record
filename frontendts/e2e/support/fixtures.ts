import { execFileSync } from "node:child_process";
import { test as base, expect, type Page } from "@playwright/test";
import { googleToken, signInWithGoogle, stubGoogleIdentity } from "./google";

/** Nombre que muestra la cuenta de `signedInPage`. */
export const SESSION_USER = "Usuario E2E";

export type SeedScenario =
  | "patient-incomplete"
  | "patient-complete"
  | "practitioner-active"
  | "practitioner-invited"
  | "practitioner-invited-phone"
  | "patient-with-practitioner-pending";

export interface SeedResult {
  userId: string;
  membershipId?: string;
}

const runDbScript = (script: string, ...args: string[]) =>
  execFileSync("bun", ["--env-file=../backend/.env", script, ...args], {
    encoding: "utf8",
  });

/** Deja la cuenta `email` en el estado del escenario (ver `e2e/support/seed.ts`). */
export const seed = (scenario: SeedScenario, email: string): SeedResult =>
  JSON.parse(runDbScript("e2e/support/seed.ts", scenario, email));

/** Datos guardados de la cuenta (ver `e2e/support/read-account.ts`). */
export const readAccount = (email: string) =>
  JSON.parse(runDbScript("e2e/support/read-account.ts", email));

/** Prepara la cuenta, entra con Google simulado y espera la sesión abierta. */
export async function signInAs(
  page: Page,
  scenario: SeedScenario,
  email: string,
): Promise<SeedResult> {
  const result = seed(scenario, email);
  await stubGoogleIdentity(page);
  await signInWithGoogle(page, await googleToken({ email }));
  // Si a la cuenta le faltan datos para su acción habitual, el aviso se ignora.
  const logout = page.getByRole("button", { name: "Cerrar sesión" });
  const warning = page.getByRole("alertdialog");
  await expect(logout.or(warning)).toBeVisible();
  if (await warning.isVisible()) {
    await warning.getByRole("button", { name: "Ignorar" }).click();
  }
  await expect(logout).toBeVisible();
  return result;
}

/** Correo propio de cada worker: las pruebas en paralelo no se pisan. */
export const workerEmail = (kind: string, index: number) =>
  `e2e.${kind}.${index}@clickcare.test`;

export const test = base.extend<{ blockGoogle: void; signedInPage: Page }>({
  // Las pruebas no deben depender de Google Identity Services.
  blockGoogle: [
    async ({ page }, use) => {
      await page.route("https://accounts.google.com/**", (route) =>
        route.abort(),
      );
      await use();
    },
    { auto: true },
  ],
  // Médico activo en la clínica e2e: puede buscar pacientes.
  signedInPage: async ({ page }, use, testInfo) => {
    await signInAs(page, "practitioner-active", workerEmail("medico", testInfo.parallelIndex));
    await use(page);
  },
});

export { expect };
