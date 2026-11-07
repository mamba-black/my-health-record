import { execFileSync } from "node:child_process";
import type { Page } from "@playwright/test";
import { expect, test } from "./support/fixtures";
import {
  googleToken,
  signInWithGoogle,
  stubGoogleIdentity,
} from "./support/google";

const runDbScript = (script: string, email: string) =>
  execFileSync("bun", ["--env-file=../backend/.env", script, email], {
    encoding: "utf8",
  });
const readAccount = (email: string) =>
  JSON.parse(runDbScript("e2e/support/read-account.ts", email));

const warning = (page: Page) => page.getByRole("alertdialog");
const logoutButton = (page: Page) =>
  page.getByRole("button", { name: "Cerrar sesión" });

test.describe("cuenta sin datos obligatorios", () => {
  let email: string;

  test.beforeEach(async ({ page }, testInfo) => {
    await stubGoogleIdentity(page);
    // Una cuenta por worker: se reutiliza entre corridas sin chocar en paralelo.
    email = `e2e.incompleto.${testInfo.parallelIndex}@clickcare.test`;
    runDbScript("e2e/support/reset-incomplete-user.ts", email);
  });

  test("avisa qué falta y no abre la sesión hasta elegir", async ({ page }) => {
    await signInWithGoogle(page, await googleToken({ email }));

    await expect(warning(page)).toContainText("Te falta registrar: nombres, teléfono, fecha de nacimiento.");
    await expect(page).toHaveURL("/login");
    await expect(logoutButton(page)).toBeHidden();
  });

  test("Ignorar entra sin guardar nada y vuelve a avisar", async ({ page }) => {
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Ignorar" }).click();

    await expect(page).toHaveURL("/");
    await expect(logoutButton(page)).toBeVisible();

    await logoutButton(page).click();
    await signInWithGoogle(page, await googleToken({ email }));
    await expect(warning(page)).toBeVisible();
  });

  test("Registrar guarda los datos y deja de avisar", async ({ page }) => {
    const dni = `${String(Date.now()).slice(-6)}${String(test.info().parallelIndex).padStart(2, "0")}`;
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Registrar" }).click();

    await expect(page).toHaveURL("/complete-profile");
    await expect(page.locator("#profile-email")).toHaveValue(email);
    // Los datos que Google ya conoce llegan escritos.
    await expect(page.locator("#profile-given-name")).toHaveValue("Usuario");
    await expect(page.locator("#profile-family-name")).toHaveValue("Prueba");

    await page.locator("#profile-given-name").fill("Elena");
    await page.locator("#profile-family-name").fill("García");
    await page.locator("#profile-second-family-name").fill("Pérez");
    await page.locator("#profile-dni").fill(dni);
    await page.locator("#profile-phone").fill("+51999888777");
    await page.locator("#profile-birth-date").fill("1990-05-10");
    await page.locator("#profile-address").fill("Av. Principal 123");
    await page.getByRole("button", { name: "Guardar datos" }).click();

    await expect(page).toHaveURL("/");
    await expect(page.getByText("Elena", { exact: true })).toBeVisible();
    expect(readAccount(email)).toEqual({
      given_name: "Elena",
      family_name: "García",
      second_family_name: "Pérez",
      document_type: "DNI",
      document_value: dni,
      phone: "+51999888777",
      birth_date: "1990-05-10",
      address: "Av. Principal 123",
    });

    await logoutButton(page).click();
    await signInWithGoogle(page, await googleToken({ email }));
    await expect(logoutButton(page)).toBeVisible();
    await expect(warning(page)).toBeHidden();
  });

  test("el formulario exige los obligatorios y Ahora no entra sin guardar", async ({
    page,
  }) => {
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Registrar" }).click();
    await expect(page).toHaveURL("/complete-profile");

    await page.locator("#profile-phone").fill("   ");
    await page.locator("#profile-birth-date").fill("1990-05-10");
    await page.getByRole("button", { name: "Guardar datos" }).click();
    await expect(page.getByText(/campos obligatorios/)).toBeVisible();

    await page.getByRole("button", { name: "Ahora no" }).click();
    await expect(page).toHaveURL("/");
  });

  test("el token decide la cuenta: sin cuenta propia no guarda nada", async ({
    page,
  }) => {
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Registrar" }).click();
    await expect(page).toHaveURL("/complete-profile");

    const stranger = await googleToken({
      email: `e2e.desconocido.${crypto.randomUUID()}@clickcare.test`,
    });
    await page.evaluate((token) => {
      localStorage.setItem("auth_token", token);
    }, stranger);
    await page.reload();

    await page.locator("#profile-given-name").fill("Elena");
    await page.locator("#profile-phone").fill("+51999888777");
    await page.locator("#profile-birth-date").fill("1990-05-10");
    await page.getByRole("button", { name: "Guardar datos" }).click();

    await expect(page.getByText(/No existe cuenta registrada/)).toBeVisible();
    await expect(page).toHaveURL("/complete-profile");
  });
});
