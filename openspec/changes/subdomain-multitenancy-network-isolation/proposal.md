## Why

En el modelo B2B de ClickCare / My Health Record, los clientes acceden a través de subdominios específicos (ej. `clinica-sanborja.clickcare.com`, `clinica-miraflores.clickcare.com`). Actualmente, la cuenta `User` en `crates/user` está concebida a nivel global sin noción de red (`network_id`), y no existe un mecanismo para aislar cuentas ni para federar o asociar clínicas en una red médica.
Este cambio introduce una arquitectura multi-inquilino gobernada por subdominios planos y redes de clínicas (`ClinicalNetwork`): dos clínicas no asociadas tendrán aislamiento total de usuarios, sesiones y datos; mientras que clínicas pertenecientes a una misma red asociada compartirán el directorio de usuarios, el inicio de sesión y la información clínica/operativa.

## What Changes

- **Resolución y Caché de Subdominio en Capa gRPC (`bin/clickcare`)**:
  - Extracción del subdominio plano desde el encabezado `Host` / `Origin` o metadata gRPC (`x-subdomain`).
  - Caché en memoria en Rust (TTL / invalidación reactiva) que mapea `subdomain -> (network_id, organization_id)` con resolución ultra-rápida.
- **Aislamiento de Identidad en `crates/user` mediante `network_id`**:
  - Incorporar `network_id: Uuid` en el agregador `User` y en la tabla `identity.user_account`.
  - Reemplazar la unicidad global de correo por unicidad compuesta por red: `UNIQUE (network_id, email)`, permitiendo que un mismo correo exista independientemente en redes no asociadas.
  - Comandos `SignUp` y `SignIn` acotados estrictamente al `network_id` resuelto.
- **Redes y Clínicas en `crates/administration`**:
  - Creación obligatoria de una red (`ClinicalNetwork`) desde el día cero (con `is_default = true` para clínicas solitarias).
  - Elección explícita de subdominio al registrar la clínica con validación de formato `[a-z0-9-]` y nombres reservados.
  - Sede física representada por `Organization` (`id`, `network_id`, `subdomain`).
- **Absorción de Clínicas e Historial Unificado en `Patient`**:
  - Incorporar `referenced_patient_ids: Vec<Uuid>` con índice GIN en PostgreSQL para consultar el historial completo unificado sin alterar registros pasados.
- **Frontend y Sesiones**:
  - Cookies `Host-Only` estrictas por subdominio (sin comodín `.clickcare.com`) y traspaso de sesión (SSO) entre sedes asociadas mediante ticket temporal de un solo uso o login directo en la red.
- **Particionamiento en Base de Datos**:
  - Particionamiento estático `PARTITION BY HASH (network_id)` para tablas transaccionales de alto volumen (`appointment`), eliminando bloqueos DDL en runtime.

## Capabilities

### New Capabilities
- `multi-tenancy/subdomain-network-isolation`: Aislamiento multi-inquilino por subdominio plano y compartición federada de usuarios y datos exclusivamente entre clínicas de una misma red asociada.

### Modified Capabilities
<!-- Ninguna previa registrada en openspec/specs -->

## Impact

- **Crates afectados**:
  - `backend/crates/user`: `User` agregador, comandos `CreateUserCommand`/`SignUpCommand`, repositorio Toasty `UserAccount`.
  - `backend/crates/administration`: entidades `ClinicalNetwork` y `Organization` (con `subdomain` y `network_id`), entidad `Patient` (con `referenced_patient_ids`).
  - `backend/bin/clickcare`: interceptor gRPC y caché en memoria para resolución de subdominio a `(network_id, organization_id)`.
  - Esquema DDL de PostgreSQL: tabla `administration.clinical_network`, tabla `administration.organization` con `subdomain` y `network_id`, índice `UNIQUE (network_id, email)` en `identity.user_account`, e índice GIN en `administration.patient`.
