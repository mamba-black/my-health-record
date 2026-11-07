// Prepara en la base los datos de una prueba e2e y deja la cuenta en un estado conocido.
// Corre con Bun porque Playwright usa Node, y Bun trae cliente de Postgres.
// Uso: bun --env-file=../backend/.env e2e/support/seed.ts <escenario> <correo>
// Imprime `{ "userId": ..., "membershipId": ... }`.
//
// Escenarios:
// - patient-incomplete: cuenta de paciente sin datos.
// - patient-complete: paciente con los datos que pide confirmar una cita.
// - practitioner-active: médico activo en la clínica e2e (activa) con pacientes.
// - practitioner-invited: médico invitado a la clínica e2e, solo con nombres.
// - practitioner-invited-phone: igual, pero ya tiene teléfono.
// - patient-with-practitioner-pending: rol preferido paciente y membresía de médico pendiente.
const [scenario, email] = process.argv.slice(2);
if (!scenario || !email) throw new Error("Uso: seed.ts <escenario> <correo>");

// `DEFAULT_DEV_NETWORK_ID` del backend: la red que se usa sin subdominio.
const NETWORK_ID = "0191eb44-4860-7000-8000-000000000001";
const CLINIC_NETWORK_ID = "0191eb44-4860-7000-8000-0000000e2e01";
const CLINIC_ID = "0191eb44-4860-7000-8000-0000000e2e02";
const CLINIC_OWNER_ID = "0191eb44-4860-7000-8000-0000000e2e03";
const PATIENTS = [
  { id: "0191eb44-4860-7000-8000-0000000e2e11", given: ["Hector", "Miuler"], family: "Malpica" },
  { id: "0191eb44-4860-7000-8000-0000000e2e12", given: ["Jan", "Luc"], family: "Picard" },
  { id: "0191eb44-4860-7000-8000-0000000e2e13", given: ["Beverly"], family: "Crusher" },
];

const sql = new Bun.SQL(process.env.PG_URL as string);
const uniqueDni = () => String(Math.floor(10_000_000 + Math.random() * 89_999_999));

interface Account {
  givenName: string;
  familyName?: string;
  phone?: string;
  dni?: string;
  birthDate?: string;
  preferredRole: "PATIENT" | "PRACTITIONER" | "CLINIC_OWNER";
  license?: boolean;
}

async function upsertAccount(account: Account): Promise<string> {
  const [row] = await sql`
    insert into identity.user_account (id, network_id, given_name, email, preferred_role)
    values (${Bun.randomUUIDv7()}, ${NETWORK_ID}, ${account.givenName}, ${email}, ${account.preferredRole})
    on conflict (network_id, email) do update set given_name = excluded.given_name
    returning id`;
  const userId = row.id as string;
  await sql`
    update identity.user_account set
      given_name = ${account.givenName},
      family_name = ${account.familyName ?? null},
      second_family_name = null,
      document_type = ${account.dni ? "DNI" : null},
      document_value = ${account.dni ?? null},
      phone = ${account.phone ?? null},
      birth_date = ${account.birthDate ?? null},
      address = null, sex = null, specialty = null, rne = null,
      professional_body = ${account.license ? "CMP" : null},
      license_number = ${account.license ? "012345" : null},
      preferred_role = ${account.preferredRole},
      document_verified = false,
      license_verified = ${account.license ?? false}
    where id = ${userId}`;
  await sql`delete from administration.membership where user_id = ${userId}`;
  return userId;
}

/** Clínica e2e activa con sus pacientes. Idempotente. */
async function ensureClinic() {
  await sql`
    insert into administration.clinical_network (id, name) values (${CLINIC_NETWORK_ID}, 'Red E2E')
    on conflict (id) do nothing`;
  await sql`
    insert into administration.organization (id, network_id, subdomain, name, owner_user_id, status, active)
    values (${CLINIC_ID}, ${CLINIC_NETWORK_ID}, 'e2e-clinica', 'Clínica E2E', ${CLINIC_OWNER_ID}, 'ACTIVE', true)
    on conflict (id) do update set status = 'ACTIVE', active = true`;
  for (const patient of PATIENTS) {
    const person = JSON.stringify({
      id: patient.id,
      name: {
        given: patient.given,
        family: patient.family,
        second_family: null,
        text: `${patient.given.join(" ")} ${patient.family}`,
      },
      telecom: [],
      identifier: null,
      birth_date: "1980-05-15",
      links: [],
    });
    await sql`
      insert into administration.patient (id, organization_id, user_id, active, person)
      values (${patient.id}, ${CLINIC_ID}, ${patient.id}, true, ${person})
      on conflict (id) do nothing`;
  }
}

async function addMembership(
  userId: string,
  status: "INVITED" | "PENDING" | "ACTIVE",
  licenseConfirmed = false,
): Promise<string> {
  const id = Bun.randomUUIDv7();
  await sql`
    insert into administration.membership (id, organization_id, user_id, role, status, license_confirmed_at)
    values (${id}, ${CLINIC_ID}, ${userId}, 'PRACTITIONER', ${status},
      ${licenseConfirmed ? new Date() : null})`;
  return id;
}

const completePerson = {
  familyName: "Prueba",
  phone: "+51999888777",
  birthDate: "1990-01-01",
};

let userId: string;
let membershipId: string | undefined;
switch (scenario) {
  case "patient-incomplete":
    userId = await upsertAccount({ givenName: "", preferredRole: "PATIENT" });
    break;
  case "patient-complete":
    userId = await upsertAccount({
      givenName: "Usuario",
      ...completePerson,
      dni: uniqueDni(),
      preferredRole: "PATIENT",
    });
    break;
  case "practitioner-active":
    await ensureClinic();
    userId = await upsertAccount({
      givenName: "Usuario E2E",
      ...completePerson,
      dni: uniqueDni(),
      preferredRole: "PRACTITIONER",
      license: true,
    });
    membershipId = await addMembership(userId, "ACTIVE", true);
    break;
  case "practitioner-invited":
  case "practitioner-invited-phone":
    await ensureClinic();
    userId = await upsertAccount({
      givenName: "Doctor",
      phone: scenario === "practitioner-invited-phone" ? "+51911222333" : undefined,
      preferredRole: "PRACTITIONER",
    });
    membershipId = await addMembership(userId, "INVITED");
    break;
  case "patient-with-practitioner-pending":
    await ensureClinic();
    userId = await upsertAccount({ givenName: "Paciente Médico", preferredRole: "PATIENT" });
    membershipId = await addMembership(userId, "PENDING");
    break;
  default:
    throw new Error(`Escenario desconocido: ${scenario}`);
}

await sql.close();
console.log(JSON.stringify({ userId, membershipId }));
