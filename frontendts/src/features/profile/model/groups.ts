import { type FieldRequirement, ProfileField, type ProtectedAction } from "$shared/api";

export interface FieldGroup {
  title: string;
  fields: ProfileField[];
}

const GROUPS: FieldGroup[] = [
  {
    title: "Datos personales",
    fields: [
      ProfileField.EMAIL,
      ProfileField.GIVEN_NAME,
      ProfileField.FAMILY_NAME,
      ProfileField.SECOND_FAMILY_NAME,
      ProfileField.DOCUMENT,
      ProfileField.PHONE,
    ],
  },
  {
    title: "Para tu atención como paciente",
    fields: [ProfileField.BIRTH_DATE, ProfileField.SEX, ProfileField.ADDRESS],
  },
  {
    title: "Datos profesionales",
    fields: [
      ProfileField.PROFESSIONAL_LICENSE,
      ProfileField.SPECIALTY,
      ProfileField.LICENSE_CONFIRMED,
    ],
  },
  {
    title: "Datos de tu clínica",
    fields: [
      ProfileField.CLINIC_TAX_ID,
      ProfileField.CLINIC_LEGAL_NAME,
      ProfileField.CLINIC_TRADE_NAME,
      ProfileField.IPRESS_CODE,
    ],
  },
];

/** Agrupa los datos que aplican al usuario, en el orden del formulario. */
export function groupFields(
  available: ProfileField[],
  options: { includeClinic?: boolean } = {},
): FieldGroup[] {
  return GROUPS.filter((group) => options.includeClinic || group.title !== "Datos de tu clínica")
    .map((group) => ({
      title: group.title,
      fields: group.fields.filter((field) => available.includes(field)),
    }))
    .filter((group) => group.fields.length > 0);
}

/** Acciones que vuelven obligatorio cada dato, según el servidor. */
export function requiredForMap(
  requirements: FieldRequirement[],
): Partial<Record<ProfileField, ProtectedAction[]>> {
  return Object.fromEntries(
    requirements.map((requirement) => [requirement.field, requirement.requiredFor]),
  );
}
