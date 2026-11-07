import type { Page } from "@playwright/test";
import { expect, readAccount, seed, test, workerEmail } from "./support/fixtures";
import {
  googleToken,
  signInWithGoogle,
  stubGoogleIdentity,
} from "./support/google";

const warning = (page: Page) => page.getByRole("alertdialog");
const logoutButton = (page: Page) =>
  page.getByRole("button", { name: "Cerrar sesión" });

// Recordatorio al iniciar sesión (`identity/progressive-profiling`): según el rol,
// informa lo que falta para su acción habitual y siempre se puede ignorar.
test.describe("recordatorio al iniciar sesión", () => {
  let email: string;

  test.beforeEach(async ({ page }, testInfo) => {
    await stubGoogleIdentity(page);
    email = workerEmail("incompleto", testInfo.parallelIndex);
    seed("patient-incomplete", email);
  });

  test("avisa qué falta según el rol y no abre la sesión hasta elegir", async ({ page }) => {
    await signInWithGoogle(page, await googleToken({ email }));

    await expect(warning(page)).toContainText(
      "Te falta registrar: nombres, apellido paterno, teléfono y documento. Los necesitarás para confirmar citas.",
    );
    await expect(page).toHaveURL("/login");
    await expect(logoutButton(page)).toBeHidden();
  });

  test("Aviso ignorable: Ignorar entra sin guardar y vuelve a avisar", async ({ page }) => {
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Ignorar" }).click();

    await expect(page).toHaveURL("/");
    await expect(logoutButton(page)).toBeVisible();

    await logoutButton(page).click();
    await signInWithGoogle(page, await googleToken({ email }));
    await expect(warning(page)).toBeVisible();
  });

  test("Completar ahora guarda los datos y deja de avisar", async ({ page }) => {
    const dni = `${String(Date.now()).slice(-6)}${String(test.info().parallelIndex).padStart(2, "0")}`;
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Completar ahora" }).click();

    await expect(page).toHaveURL("/complete-profile");
    await expect(page.locator("#profile-email")).toHaveValue(email);
    // Los datos que Google ya conoce llegan escritos.
    await expect(page.locator("#profile-given-name")).toHaveValue("Usuario");
    await expect(page.locator("#profile-family-name")).toHaveValue("Prueba");

    await page.locator("#profile-given-name").fill("Elena");
    await page.locator("#profile-family-name").fill("García");
    await page.locator("#profile-second-family-name").fill("Pérez");
    await page.locator("#profile-document").fill(dni);
    await page.locator("#profile-phone").fill("+51999888777");
    await page.locator("#profile-birth-date").fill("1990-05-10");
    await page.locator("#profile-address").fill("Av. Principal 123");
    await page.getByRole("button", { name: "Guardar datos" }).click();

    await expect(page).toHaveURL("/");
    await expect(page.getByText("Elena", { exact: true })).toBeVisible();
    expect(readAccount(email)).toMatchObject({
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

  test("el perfil no exige datos pero valida su formato; Ahora no entra sin guardar", async ({
    page,
  }) => {
    await signInWithGoogle(page, await googleToken({ email }));
    await warning(page).getByRole("button", { name: "Completar ahora" }).click();
    await expect(page).toHaveURL("/complete-profile");

    await page.locator("#profile-document").fill("1234567");
    await page.getByRole("button", { name: "Guardar datos" }).click();
    await expect(page.getByText("El DNI debe tener 8 dígitos")).toBeVisible();
    await expect(page).toHaveURL("/complete-profile");

    await page.getByRole("button", { name: "Ahora no" }).click();
    await expect(page).toHaveURL("/");
    expect(readAccount(email).document_value).toBeNull();
  });
});

test("Sin datos faltantes no hay aviso", async ({ page }, testInfo) => {
  const email = workerEmail("completo", testInfo.parallelIndex);
  seed("patient-complete", email);
  await stubGoogleIdentity(page);
  await signInWithGoogle(page, await googleToken({ email }));

  await expect(logoutButton(page)).toBeVisible();
  await expect(warning(page)).toBeHidden();
});
