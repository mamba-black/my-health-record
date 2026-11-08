import { ProfileField, ProtectedAction, Role } from "$shared/api";

/** Nombre de cada dato, en minúsculas, para armar frases. */
export const FIELD_LABELS: Record<ProfileField, string> = {
  [ProfileField.UNSPECIFIED]: "otros datos",
  [ProfileField.GIVEN_NAME]: "nombres",
  [ProfileField.FAMILY_NAME]: "apellido paterno",
  [ProfileField.SECOND_FAMILY_NAME]: "apellido materno",
  [ProfileField.EMAIL]: "correo",
  [ProfileField.PHONE]: "teléfono",
  [ProfileField.DOCUMENT]: "documento",
  [ProfileField.BIRTH_DATE]: "fecha de nacimiento",
  [ProfileField.SEX]: "sexo",
  [ProfileField.ADDRESS]: "dirección",
  [ProfileField.PROFESSIONAL_LICENSE]: "colegiatura",
  [ProfileField.SPECIALTY]: "especialidad",
  [ProfileField.LICENSE_CONFIRMED]: "confirmación de tu colegiatura por la clínica",
  [ProfileField.CLINIC_TAX_ID]: "RUC",
  [ProfileField.CLINIC_LEGAL_NAME]: "razón social",
  [ProfileField.CLINIC_TRADE_NAME]: "nombre comercial",
  [ProfileField.IPRESS_CODE]: "código IPRESS",
};

/** Para qué sirve cada acción, como complemento de «Para … necesitamos estos datos». */
export const ACTION_PURPOSE: Record<ProtectedAction, string> = {
  [ProtectedAction.ACTION_UNSPECIFIED]: "continuar",
  [ProtectedAction.ACTION_CONFIRM_APPOINTMENT]: "confirmar tu cita",
  [ProtectedAction.ACTION_JOIN_CLINIC]: "unirte a la clínica",
  [ProtectedAction.ACTION_ACTIVATE_PRACTITIONER]: "activar tu perfil médico",
  [ProtectedAction.ACTION_SEARCH_PATIENTS]: "buscar pacientes",
  [ProtectedAction.ACTION_VIEW_PATIENT]: "ver al paciente",
  [ProtectedAction.ACTION_RECORD_ENCOUNTER]: "registrar la atención",
  [ProtectedAction.ACTION_CREATE_CLINIC]: "crear tu clínica",
  [ProtectedAction.ACTION_CONFIGURE_CLINIC]: "configurar tu clínica",
  [ProtectedAction.ACTION_INVITE_STAFF]: "invitar a tu equipo",
  [ProtectedAction.ACTION_ACTIVATE_CLINIC]: "activar tu clínica",
  [ProtectedAction.ACTION_OPEN_FIRST_ENCOUNTER]: "abrir la historia clínica",
  [ProtectedAction.ACTION_ISSUE_PRESCRIPTION]: "emitir recetas",
  [ProtectedAction.ACTION_SUSPEND_MEMBER]: "suspender a un miembro",
  [ProtectedAction.ACTION_CONFIRM_LICENSE]: "confirmar una colegiatura",
  [ProtectedAction.ACTION_REGISTER_PATIENT]: "registrar pacientes",
  [ProtectedAction.ACTION_REQUEST_MEMBERSHIP]: "pedir entrar a una clínica",
  [ProtectedAction.ACTION_APPROVE_MEMBER]: "aprobar solicitudes de ingreso",
  [ProtectedAction.ACTION_MANAGE_SCHEDULE]: "publicar tu horario",
  [ProtectedAction.ACTION_VIEW_CLINIC_AGENDA]: "ver la agenda de la clínica",
  [ProtectedAction.ACTION_BOOK_FOR_PATIENT]: "reservar por un paciente",
  [ProtectedAction.ACTION_MANAGE_APPOINTMENT]: "cambiar citas de la clínica",
  [ProtectedAction.ACTION_CHECK_IN]: "registrar llegadas",
  [ProtectedAction.ACTION_LINK_PATIENT_ACCOUNT]: "vincular pacientes con su cuenta",
  [ProtectedAction.ACTION_UNLINK_PATIENT_ACCOUNT]: "deshacer vínculos de pacientes",
};

/** Lo que el rol hace habitualmente: alimenta el aviso al iniciar sesión. */
export const ROLE_HABITUAL_PURPOSE: Record<Role, string> = {
  [Role.UNSPECIFIED]: "confirmar citas",
  [Role.PATIENT]: "confirmar citas",
  [Role.PRACTITIONER]: "activar tu perfil médico",
  [Role.CLINIC_OWNER]: "activar tu clínica",
  [Role.RECEPTIONIST]: "pedir entrar a una clínica",
};

/** «a», «a y b», «a, b y c». */
export function joinWithAnd(items: string[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")} y ${items[items.length - 1]}`;
}

export const fieldList = (fields: ProfileField[]): string =>
  joinWithAnd(fields.map((field) => FIELD_LABELS[field]));

/** Texto «Obligatorio para …» u «Opcional» de un dato. */
export function requirementHint(requiredFor: ProtectedAction[]): string {
  if (requiredFor.length === 0) return "Opcional";
  return `Obligatorio para ${joinWithAnd(requiredFor.map((action) => ACTION_PURPOSE[action]))}`;
}
