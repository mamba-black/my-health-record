import type { Page } from "@playwright/test";
import { expect, seed, test } from "./support/fixtures";
import {
  googleToken,
  signInWithGoogle,
  stubGoogleIdentity,
} from "./support/google";

// Correo fijo: la prueba deja la cuenta en un estado conocido antes de usarla.
const REGISTERED_EMAIL = "e2e.usuario@clickcare.test";

const logoutButton = (page: Page) =>
  page.getByRole("button", { name: "Cerrar sesión" });

test.beforeEach(async ({ page }) => {
  await stubGoogleIdentity(page);
});

test.describe("tokens rechazados por el backend", () => {
  const rejected = async (page: Page, idToken: string, reason: string) => {
    await signInWithGoogle(page, idToken);

    await expect(page.getByText(reason)).toBeVisible();
    await expect(page).toHaveURL("/login");
    await expect(logoutButton(page)).toBeHidden();
  };

  test("firma de una clave que Google no publicó", async ({ page }) => {
    const idToken = await googleToken({
      email: REGISTERED_EMAIL,
      signedByAttacker: true,
    });

    await rejected(page, idToken, /InvalidSignature/);
  });

  test("token expirado", async ({ page }) => {
    const idToken = await googleToken({
      email: REGISTERED_EMAIL,
      claims: { exp: Math.floor(Date.now() / 1000) - 3600 },
    });

    await rejected(page, idToken, /ExpiredSignature/);
  });

  test("token para otra aplicación", async ({ page }) => {
    const idToken = await googleToken({
      email: REGISTERED_EMAIL,
      claims: { aud: "otro-client-id" },
    });

    await rejected(page, idToken, /InvalidAudience/);
  });

  test("token de otro emisor", async ({ page }) => {
    const idToken = await googleToken({
      email: REGISTERED_EMAIL,
      claims: { iss: "https://evil.example.com" },
    });

    await rejected(page, idToken, /InvalidIssuer/);
  });
});

test("un usuario desconocido pasa al registro con su correo", async ({
  page,
}) => {
  const email = `e2e.desconocido.${crypto.randomUUID()}@clickcare.test`;

  await signInWithGoogle(page, await googleToken({ email }));

  await expect(page).toHaveURL("/signup");
  await expect(page.locator("#signup-email")).toHaveValue(email);
  await expect(logoutButton(page)).toBeHidden();
});

test("un usuario registrado entra directo sin campos opcionales de Google", async ({
  page,
}) => {
  // Cuenta de paciente con los datos que pide su acción habitual: entra sin aviso.
  seed("patient-complete", REGISTERED_EMAIL);

  // Google no garantiza `locale`, `name` ni `picture`: `googleToken` no los incluye.
  await signInWithGoogle(
    page,
    await googleToken({ email: REGISTERED_EMAIL }),
  );

  await expect(logoutButton(page)).toBeVisible();
  await expect(page).toHaveURL("/");
  await expect(page.getByText("Usuario", { exact: true })).toBeVisible();
  await expect(page.getByRole("alertdialog")).toBeHidden();
});
