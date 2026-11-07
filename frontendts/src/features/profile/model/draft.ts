import {
  type CompleteProfileRequest,
  ProfileField,
  Role,
  Sex,
  type SignInResponse,
} from "$shared/api";

export type DocumentType = "DNI" | "CE" | "PASSPORT";

/** Borrador del formulario de perfil. Lo comparten la vista reducida y la completa. */
export interface ProfileDraft {
  email: string;
  givenName: string;
  familyName: string;
  secondFamilyName: string;
  documentType: DocumentType;
  documentNumber: string;
  phone: string;
  birthDate: string;
  sex: Sex;
  address: string;
  professionalBody: string;
  licenseNumber: string;
  specialty: string;
  rne: string;
  clinicTaxId: string;
  clinicLegalName: string;
  clinicTradeName: string;
  ipressCode: string;
}

export const emptyDraft = (): ProfileDraft => ({
  email: "",
  givenName: "",
  familyName: "",
  secondFamilyName: "",
  documentType: "DNI",
  documentNumber: "",
  phone: "",
  birthDate: "",
  sex: Sex.UNSPECIFIED,
  address: "",
  professionalBody: "",
  licenseNumber: "",
  specialty: "",
  rne: "",
  clinicTaxId: "",
  clinicLegalName: "",
  clinicTradeName: "",
  ipressCode: "",
});

/** Borrador con los valores actuales de la cuenta. */
export function draftFromAccount(account: SignInResponse | undefined): ProfileDraft {
  const draft = emptyDraft();
  if (!account) return draft;
  const document = account.identifier?.identifierType;
  return {
    ...draft,
    email: account.email,
    givenName: account.givenName,
    familyName: account.familyName ?? "",
    secondFamilyName: account.secondFamilyName ?? "",
    documentType:
      document?.oneofKind === "ce"
        ? "CE"
        : document?.oneofKind === "passport"
          ? "PASSPORT"
          : "DNI",
    documentNumber:
      document?.oneofKind === "dni"
        ? document.dni
        : document?.oneofKind === "ce"
          ? document.ce
          : document?.oneofKind === "passport"
            ? document.passport
            : "",
    phone: account.phone ?? "",
    birthDate: account.birthDate ?? "",
    sex: account.sex,
    address: account.address ?? "",
    professionalBody: account.professionalBody ?? "",
    licenseNumber: account.licenseNumber ?? "",
    specialty: account.specialty ?? "",
    rne: account.rne ?? "",
  };
}

const trimmed = (value: string) => value.trim() || undefined;

/** Datos personales a guardar. Un dato vacío conserva el valor actual en el servidor. */
export function toCompleteProfileRequest(
  draft: ProfileDraft,
  preferredRole: Role = Role.UNSPECIFIED,
): CompleteProfileRequest {
  const number = draft.documentNumber.trim();
  const identifier = !number
    ? undefined
    : draft.documentType === "CE"
      ? { identifierType: { oneofKind: "ce" as const, ce: number } }
      : draft.documentType === "PASSPORT"
        ? { identifierType: { oneofKind: "passport" as const, passport: number } }
        : { identifierType: { oneofKind: "dni" as const, dni: number } };
  return {
    idToken: "",
    givenName: draft.givenName.trim(),
    familyName: trimmed(draft.familyName),
    secondFamilyName: trimmed(draft.secondFamilyName),
    identifier,
    phone: draft.phone.trim(),
    birthDate: draft.birthDate.trim(),
    address: trimmed(draft.address),
    sex: draft.sex,
    professionalBody: trimmed(draft.professionalBody),
    licenseNumber: trimmed(draft.licenseNumber),
    specialty: trimmed(draft.specialty),
    rne: trimmed(draft.rne),
    preferredRole,
  };
}

const filled = (value: string) => value.trim().length > 0;

/** Indica si el borrador ya tiene el dato. */
export function hasField(draft: ProfileDraft, field: ProfileField): boolean {
  switch (field) {
    case ProfileField.GIVEN_NAME:
      return filled(draft.givenName);
    case ProfileField.FAMILY_NAME:
      return filled(draft.familyName);
    case ProfileField.SECOND_FAMILY_NAME:
      return filled(draft.secondFamilyName);
    case ProfileField.EMAIL:
      return filled(draft.email);
    case ProfileField.PHONE:
      return filled(draft.phone);
    case ProfileField.DOCUMENT:
      return filled(draft.documentNumber);
    case ProfileField.BIRTH_DATE:
      return filled(draft.birthDate);
    case ProfileField.SEX:
      return draft.sex !== Sex.UNSPECIFIED;
    case ProfileField.ADDRESS:
      return filled(draft.address);
    case ProfileField.PROFESSIONAL_LICENSE:
      return filled(draft.professionalBody) && filled(draft.licenseNumber);
    case ProfileField.SPECIALTY:
      return filled(draft.specialty);
    case ProfileField.CLINIC_TAX_ID:
      return filled(draft.clinicTaxId);
    case ProfileField.CLINIC_LEGAL_NAME:
      return filled(draft.clinicLegalName);
    case ProfileField.CLINIC_TRADE_NAME:
      return filled(draft.clinicTradeName);
    case ProfileField.IPRESS_CODE:
      return filled(draft.ipressCode);
    default:
      return true;
  }
}

/** Datos de la clínica, que se guardan aparte del perfil personal. */
export const CLINIC_FIELDS = new Set<ProfileField>([
  ProfileField.CLINIC_TAX_ID,
  ProfileField.CLINIC_LEGAL_NAME,
  ProfileField.CLINIC_TRADE_NAME,
  ProfileField.IPRESS_CODE,
]);

/** Datos que no se llenan en un formulario. */
export const NON_FILLABLE_FIELDS = new Set<ProfileField>([
  ProfileField.EMAIL,
  ProfileField.LICENSE_CONFIRMED,
]);

export type FieldErrors = Partial<Record<ProfileField, string>>;

/** Valida formato de lo ingresado y presencia de los obligatorios. */
export function validateDraft(
  draft: ProfileDraft,
  required: ProfileField[],
): FieldErrors {
  const errors: FieldErrors = {};
  for (const field of required) {
    if (!NON_FILLABLE_FIELDS.has(field) && !hasField(draft, field)) {
      errors[field] = "Este dato es obligatorio para continuar";
    }
  }
  const number = draft.documentNumber.trim();
  if (number && draft.documentType === "DNI" && !/^\d{8}$/.test(number)) {
    errors[ProfileField.DOCUMENT] = "El DNI debe tener 8 dígitos";
  }
  if (number && draft.documentType !== "DNI" && !/^[A-Za-z0-9]{6,12}$/.test(number)) {
    errors[ProfileField.DOCUMENT] = "El documento debe tener entre 6 y 12 letras o dígitos";
  }
  if (filled(draft.professionalBody) !== filled(draft.licenseNumber)) {
    errors[ProfileField.PROFESSIONAL_LICENSE] =
      "Indica el colegio profesional y el número de colegiatura";
  }
  const ruc = draft.clinicTaxId.trim();
  if (ruc && !/^\d{11}$/.test(ruc)) {
    errors[ProfileField.CLINIC_TAX_ID] = "El RUC debe tener 11 dígitos";
  }
  if (draft.birthDate && draft.birthDate > new Date().toISOString().slice(0, 10)) {
    errors[ProfileField.BIRTH_DATE] = "La fecha de nacimiento no puede ser futura";
  }
  return errors;
}
