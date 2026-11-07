import { mapGrpcPatientToDomain, type Patient } from "$entities/patient";
import { patientApiClient, withSession } from "$shared/api";
import { log } from "$shared/lib";

export async function searchPatients(name: string): Promise<Patient[]> {
  const trimmed = name.trim();
  log.debug("Calling SearchPatient with filter:", trimmed);
  const { response } = await withSession(() =>
    patientApiClient.searchPatient({
      name: trimmed.length > 0 ? trimmed : undefined,
    }),
  );
  log.debug("SearchPatient response received:", response);
  return response.patients.map(mapGrpcPatientToDomain);
}
