import { clinicApiClient } from "../grpc";
import type {
  ClinicMember,
  ClinicSummary,
  Membership,
  MembershipRole,
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
  /** Pide entrar a una clínica existente (por su enlace) como médico o recepcionista. */
  requestMembership: (subdomain: string, role: MembershipRole): Promise<Membership> =>
    call(() => clinicApiClient.requestMembership({ subdomain, role })),
  approveMembership: (membershipId: string): Promise<Membership> =>
    call(() => clinicApiClient.approveMembership({ membershipId })),
  /** Suspende una membresía; sobre una solicitud, la rechaza. */
  suspendMembership: (membershipId: string): Promise<Membership> =>
    call(() => clinicApiClient.suspendMembership({ membershipId })),
  inviteMember: (organizationId: string, email: string, role: MembershipRole): Promise<Membership> =>
    call(() => clinicApiClient.inviteMember({ organizationId, email, role })),
  listClinicMembers: async (organizationId: string): Promise<ClinicMember[]> =>
    (await call(() => clinicApiClient.listClinicMembers({ organizationId }))).members,
};
