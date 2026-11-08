import { type Patient, patientApiClient, type RegisterPatientRequest, toUserApiError, withSession } from "$shared/api";

/** Registra en la clínica a un paciente sin cuenta. */
export async function registerPatient(request: RegisterPatientRequest): Promise<Patient> {
  try {
    const { response } = await withSession(() => patientApiClient.registerPatient(request));
    return response;
  } catch (error) {
    throw toUserApiError(error);
  }
}
