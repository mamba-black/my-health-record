// Imprime los datos de perfil guardados de la cuenta indicada.
// Uso: bun --env-file=../backend/.env e2e/support/read-account.ts <correo>
const [email] = process.argv.slice(2);

const sql = new Bun.SQL(process.env.PG_URL as string);
const [account] = await sql`
  select given_name, family_name, second_family_name, document_type, document_value,
    phone, birth_date::text as birth_date, address, professional_body, license_number,
    specialty, document_verified
  from identity.user_account where email = ${email}`;
await sql.close();
console.log(JSON.stringify(account));
