import dayjs from "dayjs";
import {
  ContactPoint_System,
  type Patient as GrpcPatient,
} from "$shared/api/grpc";
import Address from "./Address";
import MedicalConditions from "./MedicalConditions";
import Patient from "./Patient";

export function mapGrpcPatientToDomain(grpcPatient: GrpcPatient): Patient {
  const firstName = grpcPatient.name[0]?.given.join(" ") ?? "";
  const lastName = grpcPatient.name[0]?.family[0] ?? "";
  const secondLastName = grpcPatient.name[0]?.family[1];

  let age = 0;
  if (grpcPatient.birthDate) {
    const parsed = dayjs(grpcPatient.birthDate);
    if (parsed.isValid()) {
      age = dayjs().diff(parsed, "year");
    }
  }

  const phone = grpcPatient.telecom.find(
    (t) => t.system === ContactPoint_System.PHONE,
  )?.value;

  const email = grpcPatient.telecom.find(
    (t) => t.system === ContactPoint_System.EMAIL,
  )?.value;

  return new Patient(
    grpcPatient.id,
    firstName,
    lastName,
    age,
    secondLastName,
    phone,
    email,
    new Address(),
    new MedicalConditions(true),
  );
}
