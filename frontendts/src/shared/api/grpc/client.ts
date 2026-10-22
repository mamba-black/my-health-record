import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { PatientApiClient, UserApiClient } from "./generated/api.client";

const baseUrl =
  import.meta.env.VITE_GRPC_API_URL ?? import.meta.env.VITE_API_URL ?? "";

export const grpcTransport = new GrpcWebFetchTransport({
  baseUrl,
  format: "binary",
});

export const patientApiClient = new PatientApiClient(grpcTransport);
export const userApiClient = new UserApiClient(grpcTransport);
