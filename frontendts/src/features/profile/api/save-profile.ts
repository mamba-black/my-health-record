import { clinicApi, type Role, type SignInResponse, userApi } from "$shared/api";
import { type ProfileDraft, toCompleteProfileRequest } from "../model/draft";

/**
 * Guarda el perfil personal y, si se indica la clínica, sus datos legales.
 * Un dato vacío conserva el valor actual en el servidor.
 */
export async function saveProfile(
  draft: ProfileDraft,
  options: { organizationId?: string; preferredRole?: Role } = {},
): Promise<SignInResponse> {
  const account = await userApi.completeProfile(
    toCompleteProfileRequest(draft, options.preferredRole),
  );
  const clinicValues = [
    draft.clinicTaxId,
    draft.clinicLegalName,
    draft.clinicTradeName,
    draft.ipressCode,
  ].map((value) => value.trim());
  if (options.organizationId && clinicValues.some(Boolean)) {
    const [taxId, legalName, tradeName, ipressCode] = clinicValues;
    await clinicApi.updateClinicSetup({
      organizationId: options.organizationId,
      taxId: taxId || undefined,
      legalName: legalName || undefined,
      tradeName: tradeName || undefined,
      ipressCode: ipressCode || undefined,
    });
  }
  return account;
}
