import { derived, writable } from "svelte/store";
import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { base } from "$app/paths";
import { page } from "$app/state";
import {
  ClinicStatus,
  type Membership,
  MembershipRole,
  MembershipStatus,
  onSessionExpired,
  type ProfileRequirements,
  Role,
  type SessionInput,
  sessionTokens,
  userApi,
} from "$shared/api";
import { LOGIN, type Page } from "$shared/config";
import { log } from "$shared/lib";

export interface PendingRegistration {
  idToken: string;
  email: string;
  givenName: string;
  familyName?: string;
  secondFamilyName?: string;
  dni?: string;
  phone?: string;
  birthDate?: string;
  address?: string;
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
  const stored = sessionTokens.get();
  const storedUser = localStorage.getItem("auth_user");
  if (stored) {
    initialAuth.isAuthenticated = true;
    initialAuth.token = stored.accessToken;
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

/** Abre la sesión con los tokens que emitió el backend y navega. */
export const setAuthSession = (
  session: SessionInput,
  user: string | null = null,
  redirectTo?: string,
) => {
  sessionTokens.save(session);
  if (browser) {
    if (user) {
      localStorage.setItem("auth_user", user);
    } else {
      localStorage.removeItem("auth_user");
    }
    sessionStorage.removeItem(PENDING_REGISTRATION_STORAGE_KEY);
  }
  authStore.set({
    isAuthenticated: true,
    token: session.accessToken,
    user,
    pendingRegistration: null,
  });
  void loadProfileContext();

  const previousPage = page.state as Partial<Page> | undefined;
  log.debug(`AuthService: previous path ${previousPage?.path}`);

  const targetPath = previousPage?.path;
  const url = targetPath
    ? targetPath.startsWith(base)
      ? targetPath
      : `${base}${targetPath}`
    : `${base}/`;

  log.debug(`AuthService: redirecting to ${redirectTo ?? url}`);
  goto(redirectTo ?? url);
};

export const clearAuth = () => {
  sessionTokens.clear();
  profileContext.set(null);
  if (browser) {
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

/** Actualiza el nombre que se muestra del usuario de la sesión. */
export const setSessionUser = (user: string) => {
  if (browser) localStorage.setItem("auth_user", user);
  authStore.update((state) => ({ ...state, user }));
};

/** Rol preferido, membresías y requisitos de datos del usuario de la sesión. */
export const profileContext = writable<ProfileRequirements | null>(null);

/** Vuelve a leer del backend el rol, las membresías y los datos que faltan. */
export async function loadProfileContext(): Promise<ProfileRequirements | null> {
  if (!sessionTokens.get()) {
    profileContext.set(null);
    return null;
  }
  try {
    const context = await userApi.getProfileRequirements();
    profileContext.set(context);
    return context;
  } catch (error) {
    log.warn("No se pudo leer el contexto del perfil", error);
    return null;
  }
}

const isActiveClinicPractitioner = (membership: Membership) =>
  membership.role === MembershipRole.PRACTITIONER &&
  membership.status === MembershipStatus.ACTIVE &&
  membership.organizationStatus === ClinicStatus.ACTIVE;

/** Puede buscar y ver pacientes: es médico activo en una clínica activa. */
export const canSearchPatients = derived(
  profileContext,
  ($context) => $context?.memberships.some(isActiveClinicPractitioner) ?? false,
);

/** Tiene alguna membresía (no suspendida) en una clínica. */
export const hasClinicMemberships = derived(
  profileContext,
  ($context) =>
    $context?.memberships.some(
      (membership) => membership.status !== MembershipStatus.SUSPENDED,
    ) ?? false,
);

export const preferredRole = derived(
  profileContext,
  ($context) => $context?.preferredRole ?? Role.PATIENT,
);

// Sin forma de renovar la sesión, se cierra y se vuelve al login.
onSessionExpired(() => {
  log.info("La sesión venció y no se pudo renovar");
  clearAuth();
});

if (browser && sessionTokens.get()) {
  void loadProfileContext();
}
