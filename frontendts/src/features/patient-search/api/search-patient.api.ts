import { mapGrpcPatientToDomain, type Patient } from "$entities/patient";
import { patientApiClient } from "$shared/api/grpc";
import { log } from "$shared/lib/logger";

export async function searchPatients(name: string): Promise<Patient[]> {
  const trimmed = name.trim();
  log.debug("Calling SearchPatient with filter:", trimmed);
  const { response } = await patientApiClient.searchPatient({
    name: trimmed.length > 0 ? trimmed : undefined,
  });
  log.debug("SearchPatient response received:", response);
  return response.patients.map(mapGrpcPatientToDomain);
}
