// Deja la cuenta indicada sin datos obligatorios, como las cuentas antiguas sin perfil.
// Corre con Bun porque Playwright usa Node, y Bun trae cliente de Postgres.
// Uso: bun --env-file=../backend/.env e2e/support/reset-incomplete-user.ts <correo>
const [email] = process.argv.slice(2);
// `DEFAULT_DEV_NETWORK_ID` del backend: la red que se usa sin subdominio.
const networkId = "0191eb44-4860-7000-8000-000000000001";

const sql = new Bun.SQL(process.env.PG_URL as string);
await sql`
  insert into identity.user_account (id, network_id, given_name, email)
  values (${Bun.randomUUIDv7()}, ${networkId}, '', ${email})
  on conflict (network_id, email)
  do update set given_name = '', family_name = null, second_family_name = null,
    document_type = null, document_value = null, phone = null, birth_date = null,
    address = null`;
await sql.close();
