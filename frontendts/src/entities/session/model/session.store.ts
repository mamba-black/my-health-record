import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { base } from "$app/paths";
import { page } from "$app/state";
import { LOGIN, type Page } from "$shared/config/routes";
import { log } from "$shared/lib/logger";

export interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  user: string | null;
}

const initialAuth: AuthState = {
  isAuthenticated: false,
  token: null,
  user: null,
};

if (browser) {
  const storedToken = localStorage.getItem("auth_token");
  const storedUser = localStorage.getItem("auth_user");
  if (storedToken) {
    initialAuth.isAuthenticated = true;
    initialAuth.token = storedToken;
    initialAuth.user = storedUser;
  }
}

export const authStore = writable<AuthState>(initialAuth);

export const setAuthToken = (token: string, user: string | null = null) => {
  if (browser) {
    localStorage.setItem("auth_token", token);
    if (user) {
      localStorage.setItem("auth_user", user);
    } else {
      localStorage.removeItem("auth_user");
    }
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
    localStorage.removeItem("auth_user");

    const win = window as Window & {
      google?: {
        accounts?: {
          id?: {
            disableAutoSelect?: () => void;
          };
        };
      };
    };
    win.google?.accounts?.id?.disableAutoSelect?.();
    navigator.credentials?.preventSilentAccess?.().catch(() => {});
  }

  authStore.set({ isAuthenticated: false, token: null, user: null });
  log.info("Sesión cerrada correctamente");
  goto(LOGIN);
};
