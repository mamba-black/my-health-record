import { publicSchedulingApiClient, schedulingApiClient } from "../grpc";
import type {
  AppointmentInfo,
  BookForPatientRequest,
  CheckInResponse,
  DocumentCheck,
  LinkSuggestionInfo,
  PublicClinic,
  PublicPractitioner,
  ScheduleResponse,
  SetWeeklyScheduleRequest,
  SlotInfo,
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

const open = async <T>(run: () => PromiseLike<{ response: T }>): Promise<T> => {
  try {
    return (await run()).response;
  } catch (error) {
    throw toUserApiError(error);
  }
};

/** Agenda pública de una clínica: no exige sesión. Fechas RFC 3339 (UTC). */
export const publicSchedulingApi = {
  getClinic: (subdomain: string): Promise<PublicClinic> =>
    open(() => publicSchedulingApiClient.getClinic({ subdomain })),
  listPractitioners: async (subdomain: string): Promise<PublicPractitioner[]> =>
    (await open(() => publicSchedulingApiClient.listPractitioners({ subdomain }))).practitioners,
  getAvailability: async (
    subdomain: string,
    practitionerUserId: string,
    from: Date,
    to: Date,
  ): Promise<SlotInfo[]> =>
    (
      await open(() =>
        publicSchedulingApiClient.getAvailability({
          subdomain,
          practitionerUserId,
          from: from.toISOString(),
          to: to.toISOString(),
        }),
      )
    ).slots,
};

/** Agenda y citas con sesión. */
export const schedulingApi = {
  getSchedule: (organizationId: string): Promise<ScheduleResponse> =>
    call(() => schedulingApiClient.getSchedule({ organizationId })),
  setWeeklySchedule: (request: SetWeeklyScheduleRequest): Promise<ScheduleResponse> =>
    call(() => schedulingApiClient.setWeeklySchedule(request)),
  addException: (
    organizationId: string,
    start: Date,
    end: Date,
    reason?: string,
  ): Promise<ScheduleResponse> =>
    call(() =>
      schedulingApiClient.addScheduleException({
        organizationId,
        start: start.toISOString(),
        end: end.toISOString(),
        reason,
      }),
    ),
  removeException: (organizationId: string, exceptionId: string): Promise<ScheduleResponse> =>
    call(() => schedulingApiClient.removeScheduleException({ organizationId, exceptionId })),
  book: (organizationId: string, practitionerUserId: string, start: string): Promise<AppointmentInfo> =>
    call(() => schedulingApiClient.bookAppointment({ organizationId, practitionerUserId, start })),
  bookForPatient: (request: BookForPatientRequest): Promise<AppointmentInfo> =>
    call(() => schedulingApiClient.bookForPatient(request)),
  myAppointments: async (): Promise<AppointmentInfo[]> =>
    (await call(() => schedulingApiClient.listMyAppointments({}))).appointments,
  clinicAppointments: async (organizationId: string, from: Date, to: Date): Promise<AppointmentInfo[]> =>
    (
      await call(() =>
        schedulingApiClient.listClinicAppointments({
          organizationId,
          from: from.toISOString(),
          to: to.toISOString(),
        }),
      )
    ).appointments,
  cancel: (appointmentId: string): Promise<AppointmentInfo> =>
    call(() => schedulingApiClient.cancelAppointment({ appointmentId })),
  reschedule: (appointmentId: string, practitionerUserId: string, start: string): Promise<AppointmentInfo> =>
    call(() => schedulingApiClient.rescheduleAppointment({ appointmentId, practitionerUserId, start })),
  checkIn: (appointmentId: string, documentCheck: DocumentCheck): Promise<CheckInResponse> =>
    call(() => schedulingApiClient.checkIn({ appointmentId, documentCheck })),
  markNoShow: (appointmentId: string): Promise<AppointmentInfo> =>
    call(() => schedulingApiClient.markNoShow({ appointmentId })),
  suggestLinks: async (organizationId: string, patientId: string): Promise<LinkSuggestionInfo[]> =>
    (await call(() => schedulingApiClient.suggestPatientLinks({ organizationId, patientId }))).suggestions,
  linkAccount: (organizationId: string, patientId: string, userId: string): Promise<string> =>
    call(() =>
      schedulingApiClient.linkPatientAccount({ organizationId, patientId, userId, documentSeen: true }),
    ).then((response) => response.linkId),
};
