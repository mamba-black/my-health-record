## 1. Esquema DDL y Entidades de Dominio

- [x] 1.1 Actualizar el esquema DDL en `backend/ddl/table.sql` para crear la tabla `administration.clinical_network`, incorporar `subdomain` (VARCHAR UNIQUE) y `network_id` (UUID) en `administration.organization`, añadir `network_id` con índice compuesto `UNIQUE (network_id, email)` en `identity.user_account`, y agregar `referenced_patient_ids uuid[]` con índice GIN en `administration.patient`.
- [x] 1.2 Implementar la entidad `ClinicalNetwork` y extender `Organization` (con `subdomain: String` y `network_id: Uuid`) y `Patient` (con `referenced_patient_ids: Vec<Uuid>`) en `crates/administration`, verificando con `cargo check -p administration`.
- [x] 1.3 Incorporar `network_id: Uuid` en la entidad `User` y en el modelo Toasty `UserAccount` en `crates/user`, verificando con `cargo check -p user`.

## 2. Casos de Uso y Scoping de Red en crates/user

- [x] 2.1 Actualizar `CreateUserCommand` / `SignUpCommand` en `crates/user` para exigir `network_id: Uuid` y restringir la verificación de existencia de email y documento al ámbito de dicho `network_id`.
- [x] 2.2 Agregar pruebas unitarias en `crates/user` verificando que dos usuarios con el mismo correo electrónico en diferentes `network_id` se registran exitosamente, y que colisiones dentro del mismo `network_id` son rechazadas.

## 3. Capa gRPC, Extractor y Caché de Subdominio en bin/clickcare

- [x] 3.1 Implementar el resolvedor de subdominio con caché en memoria en `bin/clickcare` que extraiga el subdominio desde el encabezado HTTP/2 `Host` (o metadata gRPC `x-subdomain`) y retorne `(network_id, organization_id)`.
- [x] 3.2 Conectar la resolución de subdominio en `UserApiImpl` para inyectar `network_id` en el comando de registro y autenticación.
- [x] 3.3 Verificar que toda la suite del workspace compile limpiamente con `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace`.
