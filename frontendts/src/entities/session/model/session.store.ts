import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { base } from "$app/paths";
import { page } from "$app/state";
import type { Page } from "$shared/config/routes";
import { log } from "$shared/lib/logger";

export interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  user: unknown | null;
}

const initialAuth: AuthState = {
  isAuthenticated: false,
  token: null,
  user: null,
};

if (browser) {
  const storedToken = localStorage.getItem("auth_token");
  if (storedToken) {
    initialAuth.isAuthenticated = true;
    initialAuth.token = storedToken;
  }
}

export const authStore = writable<AuthState>(initialAuth);

export const setAuthToken = (token: string, user: unknown = null) => {
  if (browser) {
    localStorage.setItem("auth_token", token);
  }
  authStore.set({ isAuthenticated: true, token, user });

  const previousPage = page.state as Partial<Page> | undefined;
  log.debug(`AuthService: previous path ${previousPage?.path}`);

  const targetPath = previousPage?.path;
  const url = targetPath
    ? targetPath.startsWith(base)
      ? targetPath
      : `${base}${targetPath}`
    : `${base}/`;

  log.debug(`AuthService: redirecting to ${url}`);
  goto(url);
};

export const clearAuth = () => {
  if (browser) {
    localStorage.removeItem("auth_token");
  }
  authStore.set({ isAuthenticated: false, token: null, user: null });
};
