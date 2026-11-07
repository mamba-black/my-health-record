import type { Page } from "@playwright/test";
import {
  expect,
  readAccount,
  type SeedScenario,
  signInAs,
  test,
  workerEmail,
} from "./support/fixtures";

// `identity/progressive-profiling`: ventana «Completa tus datos». La acción que la
// dispara es «Activar mi perfil médico» de un médico invitado: el servidor responde
// FAILED_PRECONDITION con la lista exacta de datos que faltan.
const dialog = (page: Page) => page.getByRole("dialog");
const field = (page: Page, id: string) => page.locator(`#requirement-${id}`);
const membershipStatus = (page: Page) => page.locator("[data-status]").first();

let email: string;

async function openForActivation(page: Page, scenario: SeedScenario = "practitioner-invited") {
  await signInAs(page, scenario, email);
  await page.goto("/my-clinics");
  await page.getByRole("button", { name: "Activar mi perfil médico" }).click();
  await expect(dialog(page)).toBeVisible();
}

const fillRequired = async (page: Page, dni: string) => {
  await field(page, "family-name").fill("Ramírez");
  await field(page, "phone").fill("+51988777666");
  await field(page, "document").fill(dni);
  await field(page, "professional-body").selectOption("CMP");
  await field(page, "license-number").fill("054321");
};

const uniqueDni = () =>
  `${String(Date.now()).slice(-6)}${String(test.info().parallelIndex).padStart(2, "0")}`;

test.beforeEach(async ({}, testInfo) => {
  email = workerEmail("doctor", testInfo.parallelIndex);
});

test("Contenido de la ventana", async ({ page }) => {
  await openForActivation(page);

  await expect(dialog(page).getByRole("heading", { name: "Completa tus datos" })).toBeVisible();
  await expect(dialog(page)).toContainText("Para activar tu perfil médico necesitamos estos datos");
  for (const id of ["family-name", "phone", "document", "license-number"]) {
    await expect(field(page, id)).toBeVisible();
  }
  await expect(field(page, "given-name")).toHaveCount(0);
  await expect(dialog(page).getByRole("button", { name: "Completar" })).toBeVisible();
  await expect(dialog(page).getByRole("button", { name: "Rechazar" })).toBeVisible();
});

test("Datos ya registrados no se piden", async ({ page }) => {
  await openForActivation(page, "practitioner-invited-phone");

  await expect(field(page, "document")).toBeVisible();
  await expect(field(page, "phone")).toHaveCount(0);
});

test("Validación dentro de la ventana", async ({ page }) => {
  await openForActivation(page);
  await fillRequired(page, "1234567");
  await dialog(page).getByRole("button", { name: "Completar" }).click();

  await expect(dialog(page)).toBeVisible();
  await expect(dialog(page).getByText("El DNI debe tener 8 dígitos")).toBeVisible();
  await expect(membershipStatus(page)).toHaveText("Invitación pendiente");
});

test("Completar y continuar", async ({ page }) => {
  await openForActivation(page);
  await fillRequired(page, uniqueDni());
  await dialog(page).getByRole("button", { name: "Completar" }).click();

  await expect(dialog(page)).toBeHidden();
  await expect(page.getByRole("status")).toHaveText("Listo: activar mi perfil médico.");
  await expect(membershipStatus(page)).toHaveText("Pendiente de activación");
  expect(readAccount(email)).toMatchObject({
    family_name: "Ramírez",
    phone: "+51988777666",
    professional_body: "CMP",
    license_number: "054321",
  });
});

test.describe("rechazar", () => {
  test("Advertencia al rechazar y Volver a completar", async ({ page }) => {
    await openForActivation(page);
    await field(page, "phone").fill("+51988777666");
    await dialog(page).getByRole("button", { name: "Rechazar" }).click();

    await expect(dialog(page)).toContainText(
      "Sin tu apellido paterno, teléfono, documento y colegiatura no podrás activar tu perfil médico. Si sales, perderás los datos que ingresaste",
    );
    await dialog(page).getByRole("button", { name: "Volver a completar" }).click();
    await expect(field(page, "phone")).toHaveValue("+51988777666");
  });

  test("Salir sin continuar", async ({ page }) => {
    await openForActivation(page);
    await field(page, "phone").fill("+51988777666");
    await dialog(page).getByRole("button", { name: "Rechazar" }).click();
    await dialog(page).getByRole("button", { name: "Salir sin continuar" }).click();

    await expect(dialog(page)).toBeHidden();
    await expect(page).toHaveURL("/my-clinics");
    await expect(membershipStatus(page)).toHaveText("Invitación pendiente");
    expect(readAccount(email).phone).toBeNull();

    // Al volver a intentarlo, la ventana empieza vacía: lo escrito se perdió.
    await page.getByRole("button", { name: "Activar mi perfil médico" }).click();
    await expect(field(page, "phone")).toHaveValue("");
  });

  test("Cierre de la ventana por otros medios", async ({ page }) => {
    await openForActivation(page);
    await page.keyboard.press("Escape");
    await expect(dialog(page)).toContainText("¿Salir sin completar?");
    await dialog(page).getByRole("button", { name: "Volver a completar" }).click();

    await page.getByTestId("requirements-backdrop").click({ position: { x: 5, y: 5 } });
    await expect(dialog(page)).toContainText("¿Salir sin completar?");
  });
});

test.describe("formulario completo", () => {
  test("Pasar al formulario completo y lo escrito no se pierde", async ({ page }) => {
    await openForActivation(page);
    await field(page, "phone").fill("+51988777666");
    await dialog(page).getByRole("button", { name: "Editar todos mis datos" }).click();

    await expect(field(page, "given-name")).toHaveValue("Doctor");
    await expect(field(page, "address")).toBeVisible();
    await expect(field(page, "phone")).toHaveValue("+51988777666");
    await expect(
      field(page, "address").locator("xpath=..").locator("[data-hint]"),
    ).toHaveText("Opcional");
    await expect(dialog(page).getByText("Teléfono *")).toBeVisible();
  });

  test("Volver a la vista reducida", async ({ page }) => {
    await openForActivation(page);
    await dialog(page).getByRole("button", { name: "Editar todos mis datos" }).click();
    await field(page, "phone").fill("+51988777666");
    await dialog(page).getByRole("button", { name: "Ver solo lo necesario" }).click();

    await expect(field(page, "given-name")).toHaveCount(0);
    await expect(field(page, "phone")).toHaveValue("+51988777666");
  });

  test("Faltan los obligatorios en el formulario completo", async ({ page }) => {
    await openForActivation(page);
    await dialog(page).getByRole("button", { name: "Editar todos mis datos" }).click();
    await field(page, "address").fill("Av. Arequipa 100");
    await dialog(page).getByRole("button", { name: "Completar" }).click();

    await expect(dialog(page)).toBeVisible();
    await expect(
      field(page, "document").locator("xpath=..").getByRole("alert"),
    ).toHaveText("Este dato es obligatorio para continuar");
  });

  test("Editar datos opcionales y continuar", async ({ page }) => {
    await openForActivation(page);
    await dialog(page).getByRole("button", { name: "Editar todos mis datos" }).click();
    await fillRequired(page, uniqueDni());
    await field(page, "specialty").fill("Pediatría");
    await field(page, "address").fill("Av. Arequipa 100");
    await dialog(page).getByRole("button", { name: "Completar" }).click();

    await expect(dialog(page)).toBeHidden();
    await expect(membershipStatus(page)).toHaveText("Pendiente de activación");
    expect(readAccount(email)).toMatchObject({
      specialty: "Pediatría",
      address: "Av. Arequipa 100",
      license_number: "054321",
    });
  });
});
