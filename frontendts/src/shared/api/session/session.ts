import { browser } from "$app/environment";
import { RpcError } from "@protobuf-ts/runtime-rpc";
import { UserApiClient } from "../grpc/generated/api.client";
import type { SessionTokens } from "../grpc/generated/api";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { grpcBaseUrl } from "../grpc/base-url";

/** Sesión emitida por el backend. El token de acceso viaja en `authorization: Bearer`. */
export interface StoredSession {
  accessToken: string;
  refreshToken: string;
  /** Vencimiento del token de acceso, en segundos Unix. */
  accessExpiresAt: number;
}

const ACCESS_KEY = "auth_token";
const REFRESH_KEY = "auth_refresh";
const EXPIRES_KEY = "auth_expires";

/** Sesión tal como llega del backend (el vencimiento es `int64`) o ya guardada. */
export type SessionInput = SessionTokens | StoredSession;

/** Renovar si al token de acceso le quedan menos de estos segundos. */
const REFRESH_MARGIN_SECONDS = 30;

export const sessionTokens = {
  get(): StoredSession | null {
    if (!browser) return null;
    const accessToken = localStorage.getItem(ACCESS_KEY);
    const refreshToken = localStorage.getItem(REFRESH_KEY);
    if (!accessToken || !refreshToken) return null;
    return {
      accessToken,
      refreshToken,
      accessExpiresAt: Number(localStorage.getItem(EXPIRES_KEY) ?? 0),
    };
  },
  save(session: SessionInput) {
    if (!browser) return;
    localStorage.setItem(ACCESS_KEY, session.accessToken);
    localStorage.setItem(REFRESH_KEY, session.refreshToken);
    localStorage.setItem(EXPIRES_KEY, String(Number(session.accessExpiresAt)));
  },
  clear() {
    if (!browser) return;
    localStorage.removeItem(ACCESS_KEY);
    localStorage.removeItem(REFRESH_KEY);
    localStorage.removeItem(EXPIRES_KEY);
  },
};

// Cliente sin interceptor: la renovación no lleva token de acceso.
const publicUserClient = new UserApiClient(
  new GrpcWebFetchTransport({ baseUrl: grpcBaseUrl, format: "binary" }),
);

let refreshing: Promise<boolean> | null = null;
let expiredHandler: (() => void) | null = null;

/** Registra qué hacer cuando la sesión ya no se puede renovar (p. ej. ir al login). */
export function onSessionExpired(handler: () => void) {
  expiredHandler = handler;
}

/** Canjea el token de renovación. Las llamadas concurrentes comparten la misma renovación. */
export function refreshSession(): Promise<boolean> {
  if (refreshing) return refreshing;
  const current = sessionTokens.get();
  if (!current) return Promise.resolve(false);
  refreshing = publicUserClient
    .refreshSession({ refreshToken: current.refreshToken })
    .then(({ response }) => {
      sessionTokens.save(response);
      return true;
    })
    .catch(() => false)
    .finally(() => {
      refreshing = null;
    });
  return refreshing;
}

export const isUnauthenticated = (error: unknown) =>
  error instanceof RpcError &&
  (error.code === "UNAUTHENTICATED" || error.code === "16");

/**
 * Ejecuta una llamada protegida: renueva la sesión si está por vencer y, si el servidor
 * la rechaza por no autenticada, renueva una vez y reintenta. Si no se puede renovar,
 * avisa con `onSessionExpired`.
 */
export async function withSession<T>(call: () => PromiseLike<T>): Promise<T> {
  const current = sessionTokens.get();
  const now = Math.floor(Date.now() / 1000);
  if (current && current.accessExpiresAt - now < REFRESH_MARGIN_SECONDS) {
    await refreshSession();
  }
  try {
    return await call();
  } catch (error) {
    if (!isUnauthenticated(error)) throw error;
    if (await refreshSession()) return call();
    expiredHandler?.();
    throw error;
  }
}
