import { test as base, type Page } from "@playwright/test";

export const SESSION_USER = "Usuario E2E";

/** Siembra la sesión que `session.store.ts` lee de `localStorage` al iniciar. */
const seedSession = (page: Page) =>
  page.addInitScript((user) => {
    localStorage.setItem("auth_token", "e2e-token");
    localStorage.setItem("auth_user", user);
  }, SESSION_USER);

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
  signedInPage: async ({ page }, use) => {
    await seedSession(page);
    await use(page);
  },
});

export { expect } from "@playwright/test";
