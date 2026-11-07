import { test, expect, chromium } from "@playwright/test";

// Correr chromium con el puerto de debugging abierto en 9222
// $ chromium --remote-debugging-port=9222
test.describe("Page loaded", () => {

  test("Page loaded", async ({}) => {
    const browser = await chromium.connectOverCDP("http://localhost:9222");
    const context = browser.contexts()[0] ?? (await browser.newContext());
    // const page = await context.newPage();
    const page = await context.pages()[0];

    await page.goto("http://localhost:5174/login");
    await expect(page).toHaveTitle("ClickCare");
    await expect(page.getByRole("link", { name: "Iniciar sesión" })).toBeVisible();

    // await page.close();
  });
});

