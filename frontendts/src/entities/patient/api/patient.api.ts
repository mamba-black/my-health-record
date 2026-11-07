import { patientApiClient, withSession } from "$shared/api";
import { log } from "$shared/lib";
import { mapGrpcPatientToDomain } from "../model/patient.mapper";
import type Patient from "../model/Patient";

export async function getPatientById(id: string): Promise<Patient> {
  log.debug("Calling GetPatientById for id:", id);
  const { response } = await withSession(() =>
    patientApiClient.getPatientById({ id }),
  );
  log.debug("GetPatientById response received:", response);
  return mapGrpcPatientToDomain(response);
}
