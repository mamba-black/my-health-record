import type { Page } from "@playwright/test";
import { E2E_GOOGLE_CLIENT_ID, FAKE_GOOGLE_URL } from "./config";

type GoogleWindow = Window & {
  google?: unknown;
  completeGoogleSignIn?: (response: { credential: string }) => void;
};

interface TokenOptions {
  email: string;
  claims?: Record<string, unknown>;
  signedByAttacker?: boolean;
}

/** Firma un `id_token` como lo haría Google; `claims` sobrescribe los valores válidos. */
export const googleToken = async ({
  email,
  claims,
  signedByAttacker,
}: TokenOptions) => {
  const now = Math.floor(Date.now() / 1000);
  const response = await fetch(`${FAKE_GOOGLE_URL}/sign`, {
    method: "POST",
    body: JSON.stringify({
      signedByAttacker,
      claims: {
        iss: "https://accounts.google.com",
        aud: E2E_GOOGLE_CLIENT_ID,
        sub: "e2e-subject",
        iat: now,
        exp: now + 3600,
        email,
        given_name: "Usuario",
        family_name: "Prueba",
        ...claims,
      },
    }),
  });
  return response.text();
};

/** Reemplaza Google Identity Services: `LoginForm` no carga el script si ya existe. */
export const stubGoogleIdentity = (page: Page) =>
  page.addInitScript(() => {
    const googleWindow = window as GoogleWindow;
    googleWindow.google = {
      accounts: {
        id: {
          initialize: ({
            callback,
          }: {
            callback: GoogleWindow["completeGoogleSignIn"];
          }) => {
            googleWindow.completeGoogleSignIn = callback;
          },
          renderButton: () => {},
          prompt: () => {},
          disableAutoSelect: () => {},
        },
      },
    };
  });

/** Abre el login y entrega el token al formulario, como hace Google al elegir una cuenta. */
export const signInWithGoogle = async (page: Page, idToken: string) => {
  await page.goto("/login");
  await page.waitForFunction(
    () => !!(window as GoogleWindow).completeGoogleSignIn,
  );
  await page.evaluate((credential) => {
    (window as GoogleWindow).completeGoogleSignIn?.({ credential });
  }, idToken);
};
