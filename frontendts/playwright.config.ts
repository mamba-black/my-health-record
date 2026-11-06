import { defineConfig, devices } from "@playwright/test";

const baseURL = "http://localhost:5173";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL,
    locale: "es-PE",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "bun run dev --port 5173 --strictPort",
    url: baseURL,
    reuseExistingServer: !process.env.CI,
  },
});
