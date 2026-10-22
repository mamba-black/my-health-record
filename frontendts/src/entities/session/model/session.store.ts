import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { base } from "$app/paths";
import { page } from "$app/state";
import { LOGIN, type Page } from "$shared/config/routes";
import { log } from "$shared/lib/logger";

export interface PendingRegistration {
  idToken: string;
  email: string;
  givenName: string;
  familyName?: string;
  picture?: string;
  providerId: string;
}

export interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  user: string | null;
  pendingRegistration: PendingRegistration | null;
}

const PENDING_REGISTRATION_STORAGE_KEY = "auth_pending_registration";

const initialAuth: AuthState = {
  isAuthenticated: false,
  token: null,
  user: null,
  pendingRegistration: null,
};

if (browser) {
  const storedToken = localStorage.getItem("auth_token");
  const storedUser = localStorage.getItem("auth_user");
  if (storedToken) {
    initialAuth.isAuthenticated = true;
    initialAuth.token = storedToken;
    initialAuth.user = storedUser;
  }

  const storedPending = sessionStorage.getItem(PENDING_REGISTRATION_STORAGE_KEY);
  if (storedPending) {
    try {
      initialAuth.pendingRegistration = JSON.parse(
        storedPending,
      ) as PendingRegistration;
    } catch {
      sessionStorage.removeItem(PENDING_REGISTRATION_STORAGE_KEY);
    }
  }
}

export const authStore = writable<AuthState>(initialAuth);

export const setPendingRegistration = (
  pendingRegistration: PendingRegistration,
) => {
  if (browser) {
    sessionStorage.setItem(
      PENDING_REGISTRATION_STORAGE_KEY,
      JSON.stringify(pendingRegistration),
    );
  }
  authStore.update((currentState) => ({
    ...currentState,
    pendingRegistration,
  }));
};

export const clearPendingRegistration = () => {
  if (browser) {
    sessionStorage.removeItem(PENDING_REGISTRATION_STORAGE_KEY);
  }
  authStore.update((currentState) => ({
    ...currentState,
    pendingRegistration: null,
  }));
};

export const setAuthToken = (token: string, user: string | null = null) => {
  if (browser) {
    localStorage.setItem("auth_token", token);
    if (user) {
      localStorage.setItem("auth_user", user);
    } else {
      localStorage.removeItem("auth_user");
    }
    sessionStorage.removeItem(PENDING_REGISTRATION_STORAGE_KEY);
  }
  authStore.set({
    isAuthenticated: true,
    token,
    user,
    pendingRegistration: null,
  });

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
    sessionStorage.removeItem(PENDING_REGISTRATION_STORAGE_KEY);

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

  authStore.set({
    isAuthenticated: false,
    token: null,
    user: null,
    pendingRegistration: null,
  });
  log.info("Sesión cerrada correctamente");
  goto(LOGIN);
};
