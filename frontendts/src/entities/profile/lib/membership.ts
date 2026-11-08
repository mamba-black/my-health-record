import { ClinicStatus, MembershipRole, MembershipStatus } from "$shared/api";

export const MEMBERSHIP_ROLE_LABELS: Record<MembershipRole, string> = {
  [MembershipRole.UNSPECIFIED]: "Miembro",
  [MembershipRole.OWNER]: "Dueño",
  [MembershipRole.ADMIN]: "Administrador",
  [MembershipRole.PRACTITIONER]: "Médico",
  [MembershipRole.RECEPTIONIST]: "Recepcionista",
};

export const MEMBERSHIP_STATUS_LABELS: Record<MembershipStatus, string> = {
  [MembershipStatus.UNSPECIFIED]: "",
  [MembershipStatus.INVITED]: "Invitación pendiente",
  [MembershipStatus.REQUESTED]: "Solicitud pendiente de aprobación",
  [MembershipStatus.PENDING]: "Pendiente de activación",
  [MembershipStatus.ACTIVE]: "Activo",
  [MembershipStatus.SUSPENDED]: "Suspendido",
};

export const CLINIC_STATUS_LABELS: Record<ClinicStatus, string> = {
  [ClinicStatus.UNSPECIFIED]: "",
  [ClinicStatus.PENDING_SETUP]: "Clínica pendiente de activación",
  [ClinicStatus.ACTIVE]: "Clínica activa",
  [ClinicStatus.SUSPENDED]: "Clínica suspendida",
};
