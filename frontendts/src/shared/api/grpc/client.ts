import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcInterceptor } from "@protobuf-ts/runtime-rpc";
import { sessionTokens } from "../session/session";
import { grpcBaseUrl } from "./base-url";
import {
  ClinicApiClient,
  PatientApiClient,
  PublicSchedulingApiClient,
  SchedulingApiClient,
  UserApiClient,
} from "./generated/api.client";

/** Operaciones que no llevan token de acceso. */
const PUBLIC_METHODS = new Set([
  "SignUp",
  "SignIn",
  "RefreshSession",
  // `PublicSchedulingApi`: la agenda pública de una clínica.
  "GetClinic",
  "ListPractitioners",
  "GetAvailability",
]);

/** Agrega `authorization: Bearer <token>` a toda llamada protegida. */
const sessionInterceptor: RpcInterceptor = {
  interceptUnary(next, method, input, options) {
    const token = sessionTokens.get()?.accessToken;
    if (!token || PUBLIC_METHODS.has(method.name)) {
      return next(method, input, options);
    }
    return next(method, input, {
      ...options,
      meta: { ...options.meta, authorization: `Bearer ${token}` },
    });
  },
};

export const grpcTransport = new GrpcWebFetchTransport({
  baseUrl: grpcBaseUrl,
  format: "binary",
  interceptors: [sessionInterceptor],
});

export const patientApiClient = new PatientApiClient(grpcTransport);
export const userApiClient = new UserApiClient(grpcTransport);
export const clinicApiClient = new ClinicApiClient(grpcTransport);
export const schedulingApiClient = new SchedulingApiClient(grpcTransport);
export const publicSchedulingApiClient = new PublicSchedulingApiClient(grpcTransport);
