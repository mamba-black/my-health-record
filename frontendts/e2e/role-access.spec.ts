import type { Page } from "@playwright/test";
import { expect, signInAs, test, workerEmail } from "./support/fixtures";

const search = async (page: Page, name: string) => {
  await page.getByPlaceholder("Busqueda de pacientes, nombre...").fill(name);
  await page.getByRole("button", { name: "Busqueda" }).click();
};

// `access-control/action-authorization`: interfaz coherente con los permisos.
test.describe("rutas según el rol", () => {
  test("Paciente entra por URL a la búsqueda", async ({ page }, testInfo) => {
    await signInAs(page, "patient-complete", workerEmail("paciente", testInfo.parallelIndex));
    await expect(page.getByRole("link", { name: "Busqueda" })).toBeHidden();

    await page.goto("/search");

    await expect(page).toHaveURL("/access-denied");
    await expect(page.getByRole("heading", { name: "Acceso denegado" })).toBeVisible();
    await expect(page.getByPlaceholder("Busqueda de pacientes, nombre...")).toBeHidden();
  });

  test("Médico activo", async ({ signedInPage: page }) => {
    await expect(page.getByRole("link", { name: "Busqueda" })).toBeVisible();
    await page.getByRole("link", { name: "Busqueda" }).click();
    await search(page, "picard");
    await expect(page.getByRole("list").last().getByRole("listitem")).toHaveCount(1);
  });
});

// Tarea 1.3: la sesión del backend viaja en cada llamada y se renueva.
test.describe("sesión", () => {
  test("una sesión vencida termina en el login", async ({ signedInPage: page }) => {
    await page.evaluate(() => {
      localStorage.setItem("auth_token", "token-vencido");
      localStorage.setItem("auth_refresh", "renovacion-invalida");
    });
    await page.goto("/search");

    await expect(page).toHaveURL("/login");
    expect(await page.evaluate(() => localStorage.getItem("auth_token"))).toBeNull();
  });

  test("si el token de acceso no sirve, se renueva y la llamada sigue", async ({
    signedInPage: page,
  }) => {
    await page.evaluate(() => localStorage.setItem("auth_token", "token-vencido"));
    await page.goto("/search");
    await search(page, "picard");

    await expect(page.getByRole("list").last().getByRole("listitem")).toHaveCount(1);
    expect(await page.evaluate(() => localStorage.getItem("auth_token"))).not.toBe(
      "token-vencido",
    );
  });
});

// `identity/progressive-profiling`: formulario de perfil según rol.
test.describe("perfil según el rol", () => {
  test("Médico que también es paciente", async ({ page }, testInfo) => {
    await signInAs(
      page,
      "patient-with-practitioner-pending",
      workerEmail("paciente-medico", testInfo.parallelIndex),
    );
    await page.getByRole("link", { name: "Mi perfil" }).click();

    await expect(page.getByRole("heading", { name: "Para tu atención como paciente" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Datos profesionales" })).toBeVisible();
    await expect(
      page.locator("#profile-license-number").locator("xpath=..").locator("[data-hint]"),
    ).toHaveText("Obligatorio para activar tu perfil médico");
  });

  test("Datos opcionales siempre", async ({ page }, testInfo) => {
    await signInAs(
      page,
      "patient-with-practitioner-pending",
      workerEmail("opcionales", testInfo.parallelIndex),
    );
    await page.goto("/complete-profile");

    for (const id of ["second-family-name", "address", "specialty"]) {
      await expect(
        page.locator(`#profile-${id}`).locator("xpath=..").locator("[data-hint]"),
      ).toHaveText("Opcional");
    }
  });

  test("Un dato que no aplica al rol no se pide", async ({ page }, testInfo) => {
    await signInAs(page, "patient-complete", workerEmail("solo-paciente", testInfo.parallelIndex));
    await page.goto("/complete-profile");

    await expect(page.locator("#profile-given-name")).toBeVisible();
    await expect(page.locator("#profile-license-number")).toHaveCount(0);
    await expect(page.locator("#profile-clinic-tax-id")).toHaveCount(0);
  });
});

// `scheduling-appointments`: cada rol ve solo sus entradas de agenda.
test.describe("menú de agenda según el rol", () => {
  const link = (page: Page, name: string) => page.getByRole("link", { name, exact: true });

  test("Paciente", async ({ page }, testInfo) => {
    await signInAs(page, "patient-complete", workerEmail("menu-paciente", testInfo.parallelIndex));
    await expect(link(page, "Mis citas")).toBeVisible();
    await expect(link(page, "Mi agenda")).toBeHidden();
    await expect(link(page, "Agenda de la clínica")).toBeHidden();
  });

  test("Médico activo", async ({ page }, testInfo) => {
    await signInAs(page, "practitioner-active", workerEmail("menu-medico", testInfo.parallelIndex));
    await expect(link(page, "Mi agenda")).toBeVisible();
    await expect(link(page, "Agenda de la clínica")).toBeVisible();
    await expect(link(page, "Registrar paciente")).toBeHidden();
  });

  test("Recepcionista", async ({ page }, testInfo) => {
    await signInAs(page, "receptionist-active", workerEmail("menu-recepcion", testInfo.parallelIndex));
    await expect(link(page, "Agenda de la clínica")).toBeVisible();
    await expect(link(page, "Registrar paciente")).toBeVisible();
    await expect(link(page, "Mi agenda")).toBeHidden();
  });
});
