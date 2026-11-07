import { writable } from "svelte/store";
import {
  MissingDataError,
  type ProfileField,
  type ProtectedAction,
  toMissingDataError,
} from "$shared/api";

/** Acción detenida por falta de datos, a la espera de la ventana «Completa tus datos». */
export interface PendingRequirement {
  action: ProtectedAction;
  missing: ProfileField[];
  organizationId?: string;
  /** Reintenta la acción original. Puede volver a lanzar `MissingDataError`. */
  retry: () => Promise<unknown>;
  /** Termina el flujo: `true` si la acción se completó, `false` si el usuario salió. */
  finish: (completed: boolean) => void;
}

export const pendingRequirement = writable<PendingRequirement | null>(null);

/**
 * Ejecuta una acción protegida. Si el servidor responde que faltan datos, abre la
 * ventana «Completa tus datos»; al completarlos reintenta la acción.
 *
 * Devuelve el resultado de la acción, o `undefined` si el usuario salió sin continuar.
 */
export async function runWithRequirements<T>(
  run: () => Promise<T>,
): Promise<T | undefined> {
  try {
    return await run();
  } catch (error) {
    const missing =
      error instanceof MissingDataError ? error : toMissingDataError(error);
    if (!missing) throw error;

    let result: T | undefined;
    const completed = await new Promise<boolean>((resolve) => {
      pendingRequirement.set({
        action: missing.action,
        missing: missing.missing,
        organizationId: missing.organizationId,
        retry: async () => {
          result = await run();
        },
        finish: (done) => {
          pendingRequirement.set(null);
          resolve(done);
        },
      });
    });
    return completed ? result : undefined;
  }
}
