import { expect, test } from "./support/fixtures";

// Salvo que se indique, estas pruebas requieren el backend. La sesión es de un médico
// activo en la clínica e2e, cuyos pacientes siembra `e2e/support/seed.ts`.
test.describe("búsqueda de pacientes", () => {
  test.beforeEach(async ({ signedInPage: page }) => {
    await page.goto("/search");
  });

  const search = async (page: import("@playwright/test").Page, name: string) => {
    await page.getByPlaceholder("Busqueda de pacientes, nombre...").fill(name);
    await page.getByRole("button", { name: "Busqueda" }).click();
  };

  test("filtra los resultados por nombre", async ({ signedInPage: page }) => {
    await search(page, "picard");

    const results = page.getByRole("list").last().getByRole("listitem");
    await expect(results).toHaveCount(1);
    await expect(results.first()).toContainText("Picard");
    await expect(page.getByText("Crusher")).toBeHidden();
  });

  test("no muestra resultados si nadie coincide", async ({
    signedInPage: page,
  }) => {
    await search(page, "zzz-sin-coincidencias");

    await expect(page.getByRole("button", { name: "Busqueda" })).toBeEnabled();
    await expect(page.getByRole("link", { name: /Cita programada/ })).toHaveCount(
      0,
    );
    await expect(page.getByRole("alert")).toBeHidden();
  });

  test("abre el perfil del paciente seleccionado", async ({
    signedInPage: page,
  }) => {
    await search(page, "crusher");

    await page.getByRole("link", { name: /Crusher/ }).click();

    await expect(page).toHaveURL(/\/patients\/[0-9a-f-]{36}$/);
    await expect(page.getByText("Crusher").first()).toBeVisible();
  });

  // No requiere backend: la respuesta se intercepta.
  test("informa cuando el servidor falla", async ({ signedInPage: page }) => {
    await page.route("**/grpc.PatientApi/SearchPatient", (route) =>
      route.abort(),
    );

    await search(page, "picard");

    await expect(page.getByRole("alert")).toContainText(
      "Ocurrió un error al buscar pacientes",
    );
  });
});
