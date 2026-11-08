// Siembra una cita de este momento (para registrar la llegada sin depender de la hora).
// Uso: bun --env-file=../backend/.env e2e/support/seed-appointment.ts <correo-médico> <modo> <correo-paciente>
// Modos:
// - account: la cita es del registro de la cuenta del paciente en la clínica e2e.
// - walk-in: la cita es de un registro sin cuenta con el mismo DNI que la cuenta del paciente.
// Imprime `{ "appointmentId": ..., "patientId": ... }`.
const [doctorEmail, mode, patientEmail] = process.argv.slice(2);
if (!doctorEmail || !mode || !patientEmail) throw new Error("Uso: seed-appointment.ts <médico> <modo> <paciente>");

const NETWORK_ID = "0191eb44-4860-7000-8000-000000000001";
const CLINIC_ID = "0191eb44-4860-7000-8000-0000000e2e02";
const sql = new Bun.SQL(process.env.PG_URL as string);

const [doctor] = await sql`select id from identity.user_account where email = ${doctorEmail}`;
const [account] = await sql`
  select id, given_name, family_name, document_value from identity.user_account where email = ${patientEmail}`;
const person = (dni: string, given: string, family: string) =>
  JSON.stringify({
    id: Bun.randomUUIDv7(),
    name: { given: [given], family, second_family: null, text: `${given} ${family}` },
    telecom: [{ system: "Phone", value: "987654321", use_type: "Mobile", rank: null }],
    identifier: { DNI: dni },
    birth_date: null,
    links: [],
  });

let patientId: string;
if (mode === "account") {
  const [existing] = await sql`
    select id from administration.patient where organization_id = ${CLINIC_ID} and user_id = ${account.id}`;
  patientId = existing?.id ?? Bun.randomUUIDv7();
  if (!existing) {
    await sql`
      insert into administration.patient (id, organization_id, user_id, active, person)
      values (${patientId}, ${CLINIC_ID}, ${account.id}, true,
        ${person(account.document_value, account.given_name, account.family_name ?? "")})`;
  }
} else {
  patientId = Bun.randomUUIDv7();
  await sql`
    insert into administration.patient (id, organization_id, user_id, active, person)
    values (${patientId}, ${CLINIC_ID}, null, true, ${person(account.document_value, "Rosa", "Quispe")})`;
}
const appointmentId = Bun.randomUUIDv7();
await sql`
  insert into scheduling.appointment
    (id, network_id, organization_id, practitioner_user_id, patient_id, period, status, booked_by)
  values (${appointmentId}, ${NETWORK_ID}, ${CLINIC_ID}, ${doctor.id}, ${patientId},
    tstzrange(now() - interval '1 minute', now() + interval '29 minutes', '[)'), 'BOOKED', ${doctor.id})`;
await sql.close();
console.log(JSON.stringify({ appointmentId, patientId }));
