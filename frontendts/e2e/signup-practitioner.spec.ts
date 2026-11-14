import type { Page } from "@playwright/test";
import { expect, readAccount, test } from "./support/fixtures";
import { googleToken, signInWithGoogle, stubGoogleIdentity } from "./support/google";

/** Lleva una cuenta nueva al alta como médico, con los datos obligatorios. */
async function startPractitionerSignUp(page: Page, email: string) {
  await stubGoogleIdentity(page);
  await signInWithGoogle(page, await googleToken({ email }));
  await expect(page).toHaveURL("/signup");

  await page.locator("#signup-intent").selectOption({ label: "Profesional de Salud (Médico / Especialista)" });
  await page.locator("#signup-given-name").fill("Médica");
  await page.locator("#signup-family-name").fill("Fundadora");
  await page.locator("#signup-phone").fill("+51 999 888 777");
  await page.locator("#signup-birth-date").fill("1985-04-12");
}

test("el médico elige entre unirse a una clínica o crear la suya", async ({ page }) => {
  await startPractitionerSignUp(page, `e2e.medico.opciones.${crypto.randomUUID()}@clickcare.test`);

  // Por defecto se une por enlace.
  await expect(page.getByLabel("Unirme a una clínica")).toBeChecked();
  await expect(page.locator("#signup-clinic")).toBeVisible();

  await page.getByLabel("Crear mi propia clínica").check();
  await expect(page.locator("#signup-clinic")).toBeHidden();
});

test("el médico que crea su propia clínica queda como dueño y orientado como médico", async ({
  page,
}) => {
  const email = `e2e.medico.fundador.${crypto.randomUUID()}@clickcare.test`;
  await startPractitionerSignUp(page, email);

  await page.getByLabel("Crear mi propia clínica").check();
  await page.getByRole("button", { name: "Crear cuenta" }).click();

  await expect(page).toHaveURL(/\/my-clinics/);
  const account = readAccount(email);
  expect(account.is_owner).toBe(true);
  expect(account.preferred_role).toBe("PRACTITIONER");
});
