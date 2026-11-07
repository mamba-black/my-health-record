import { clinicApiClient } from "../grpc";
import type {
  ClinicSummary,
  Membership,
  UpdateClinicSetupRequest,
} from "../grpc/generated/api";
import { withSession } from "../session";
import { toUserApiError } from "../user/user-api";

const call = async <T>(run: () => PromiseLike<{ response: T }>): Promise<T> => {
  try {
    const { response } = await withSession(run);
    return response;
  } catch (error) {
    throw toUserApiError(error);
  }
};

/** Operaciones de clínica y membresía. Todas exigen sesión. */
export const clinicApi = {
  joinClinic: (membershipId: string): Promise<Membership> =>
    call(() => clinicApiClient.joinClinic({ membershipId })),
  activatePractitioner: (membershipId: string): Promise<Membership> =>
    call(() => clinicApiClient.activatePractitioner({ membershipId })),
  activateClinic: (organizationId: string): Promise<ClinicSummary> =>
    call(() => clinicApiClient.activateClinic({ organizationId })),
  updateClinicSetup: (request: UpdateClinicSetupRequest): Promise<ClinicSummary> =>
    call(() => clinicApiClient.updateClinicSetup(request)),
};
