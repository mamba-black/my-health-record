import { RpcError } from "@protobuf-ts/runtime-rpc";
import {
  ActionRequirements,
  type ProfileField,
  ProtectedAction,
} from "../grpc/generated/api";

/** La acción tiene permiso, pero faltan datos (FAILED_PRECONDITION con `ActionRequirements`). */
export class MissingDataError extends Error {
  constructor(
    public readonly action: ProtectedAction,
    public readonly missing: ProfileField[],
    public readonly organizationId?: string,
  ) {
    super("Faltan datos para realizar esta acción");
    this.name = "MissingDataError";
  }
}

const decodeBase64 = (value: string): Uint8Array => {
  const normalized = value.replace(/-/g, "+").replace(/_/g, "/");
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "=");
  return Uint8Array.from(atob(padded), (char) => char.charCodeAt(0));
};

/** Devuelve el `MissingDataError` que trae un error gRPC, o `null` si no es de ese tipo. */
export function toMissingDataError(error: unknown): MissingDataError | null {
  if (!(error instanceof RpcError)) return null;
  if (error.code !== "FAILED_PRECONDITION" && error.code !== "9") return null;
  const raw = error.meta["grpc-status-details-bin"];
  const encoded = Array.isArray(raw) ? raw[0] : raw;
  if (!encoded) return null;
  try {
    const details = ActionRequirements.fromBinary(decodeBase64(encoded));
    if (details.missing.length === 0) return null;
    return new MissingDataError(
      details.action,
      details.missing,
      details.organizationId,
    );
  } catch {
    return null;
  }
}
