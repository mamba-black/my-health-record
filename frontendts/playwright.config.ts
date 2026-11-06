import { defineConfig, devices } from "@playwright/test";
import {
  BACKEND_PORT,
  E2E_GOOGLE_CLIENT_ID,
  FAKE_GOOGLE_URL,
} from "./e2e/support/config";

const baseURL = "http://localhost:5174";
const backendURL = `http://localhost:${BACKEND_PORT}`;

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
  webServer: [
    {
      command: "bun e2e/support/fake-google.ts",
      url: `${FAKE_GOOGLE_URL}/certs`,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: "cargo run -p clickcare --features test-support",
      cwd: "../backend",
      url: `${backendURL}/api/v1/queues`,
      reuseExistingServer: !process.env.CI,
      timeout: 300_000,
      env: {
        CLICKCARE_PORT: String(BACKEND_PORT),
        GOOGLE_JWKS_URL: `${FAKE_GOOGLE_URL}/certs`,
        PUBLIC_GOOGLE_CLIENT_ID: E2E_GOOGLE_CLIENT_ID,
        // En `debug` el backend imprime credenciales.
        RUST_LOG: "info",
      },
    },
    {
      // Vite normal: `rolldown-vite` carga dos veces el cliente de SvelteKit y rompe `goto`.
      command: "node node_modules/vite/bin/vite.js --port 5174 --strictPort",
      url: baseURL,
      reuseExistingServer: !process.env.CI,
      env: {
        VITE_CACHE_DIR: "node_modules/.vite-e2e",
        VITE_GRPC_API_URL: backendURL,
      },
    },
  ],
});
