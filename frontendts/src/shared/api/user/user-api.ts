import { RpcError } from "@protobuf-ts/runtime-rpc";
import {
  type SignInRequest,
  type SignInResponse,
  SignUpIntent,
  type SignUpRequest,
  type SignUpResponse,
  SignUpStatus,
  userApiClient,
} from "../grpc";

export type GrpcStatusName =
  | "OK"
  | "NOT_FOUND"
  | "UNAUTHENTICATED"
  | "ALREADY_EXISTS"
  | "INVALID_ARGUMENT"
  | "INTERNAL"
  | "UNKNOWN";

export class UserApiError extends Error {
  constructor(
    public readonly code: GrpcStatusName,
    message: string,
  ) {
    super(message);
    this.name = "UserApiError";
  }
}

function normalizeGrpcCode(rawCode: string | number | undefined): GrpcStatusName {
  const normalized = String(rawCode ?? "")
    .trim()
    .toUpperCase();

  switch (normalized) {
    case "5":
    case "NOT_FOUND":
      return "NOT_FOUND";
    case "16":
    case "UNAUTHENTICATED":
      return "UNAUTHENTICATED";
    case "6":
    case "ALREADY_EXISTS":
      return "ALREADY_EXISTS";
    case "3":
    case "INVALID_ARGUMENT":
      return "INVALID_ARGUMENT";
    case "13":
    case "INTERNAL":
      return "INTERNAL";
    default:
      return "UNKNOWN";
  }
}

function toUserApiError(error: unknown): UserApiError {
  if (error instanceof UserApiError) {
    return error;
  }
  if (error instanceof RpcError) {
    return new UserApiError(
      normalizeGrpcCode(error.code),
      decodeURIComponent(error.message || "Error de comunicación gRPC-Web"),
    );
  }
  if (error instanceof Error) {
    return new UserApiError("UNKNOWN", error.message);
  }
  return new UserApiError("UNKNOWN", "Error desconocido al invocar UserApi");
}

export const userApi = {
  async signIn(request: SignInRequest): Promise<SignInResponse> {
    try {
      const { response } = await userApiClient.signIn(request);
      return response;
    } catch (error) {
      throw toUserApiError(error);
    }
  },

  async signUp(request: SignUpRequest): Promise<SignUpResponse> {
    try {
      const { response } = await userApiClient.signUp(request);
      return response;
    } catch (error) {
      throw toUserApiError(error);
    }
  },
};

export {
  type SignInRequest,
  type SignInResponse,
  SignUpIntent,
  type SignUpRequest,
  type SignUpResponse,
  SignUpStatus,
};
