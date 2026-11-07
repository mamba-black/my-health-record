import type { Page } from "@playwright/test";
import { expect, SESSION_USER, test } from "./support/fixtures";

// Las páginas públicas llegan pre-renderizadas: hay que esperar la hidratación
// antes de interactuar o los manejadores de Svelte aún no están conectados.
const gotoHydrated = async (page: Page, path: string) => {
  await page.goto(path, { waitUntil: "networkidle" });
};

test.describe("sin sesión", () => {
  test("la página de inicio es pública", async ({ page }) => {
    await page.goto("/");

    await expect(page).toHaveURL("/");
    await expect(
      page.getByRole("link", { name: "Iniciar sesión" }),
    ).toBeVisible();
  });

  for (const path of ["/search", "/patients/123"]) {
    test(`${path} redirige a /login`, async ({ page }) => {
      await page.goto(path);

      await expect(page).toHaveURL("/login");
      await expect(page.getByRole("heading", { name: "Login" })).toBeVisible();
    });
  }

  test("el registro permite volver al login", async ({ page }) => {
    await gotoHydrated(page, "/signup");
    await expect(
      page.getByRole("heading", { name: "Completa tu Registro" }),
    ).toBeVisible();

    await page.getByRole("button", { name: "Volver al Login" }).click();

    await expect(page).toHaveURL("/login");
  });

  // El servidor aún no autentica con contraseña: el formulario no abre sesión.
  test("el login rechaza credenciales inválidas", async ({ page }) => {
    await gotoHydrated(page, "/login");

    await page.getByLabel("Email Address").fill("nadie@example.com");
    await page.getByLabel("Password").fill("contraseña-incorrecta");
    await page.getByRole("button", { name: "Submit" }).click();

    await expect(
      page.getByRole("button", { name: "Cerrar sesión" }),
    ).toBeHidden();
    await expect(page).toHaveURL("/login");
  });
});

test.describe("con sesión", () => {
  test("muestra al usuario y da acceso a la búsqueda", async ({
    signedInPage: page,
  }) => {
    await page.goto("/search");

    await expect(
      page.getByRole("heading", { name: "Busqueda de pacientes" }),
    ).toBeVisible();
    await expect(page.getByText(SESSION_USER)).toBeVisible();
  });

  test("cerrar sesión limpia el token y vuelve al login", async ({
    signedInPage: page,
  }) => {
    await page.goto("/search");

    await page.getByRole("button", { name: "Cerrar sesión" }).click();

    await expect(page).toHaveURL("/login");
    expect(
      await page.evaluate(() => localStorage.getItem("auth_token")),
    ).toBeNull();
  });
});
