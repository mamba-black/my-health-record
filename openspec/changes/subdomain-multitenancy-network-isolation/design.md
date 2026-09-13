## Context

Ver `proposal.md` y `specs/multi-tenancy/subdomain-network-isolation/spec.md`.
En el estado actual del backend de My Health Record:
1. `User` (`crates/user`) no posee un identificador de red (`network_id`), y la tabla `identity.user_account` impone una unicidad global sobre el correo electrónico.
2. `Organization` (`crates/administration`) carece de subdominio (`subdomain`) y de punteros hacia una red (`network_id`).
3. El servidor gRPC (`bin/clickcare`) no inspecciona el encabezado `Host` ni los metadatos gRPC para resolver el contexto de inquilino/red antes de despachar a los casos de uso.

## Goals / Non-Goals

**Goals:**
- Crear la entidad `ClinicalNetwork` (`administration.clinical_network`) con `is_default = true` para redes implícitas creadas en el alta.
- Extender `Organization` con `subdomain: String` (único, validado con `[a-z0-9-]`) y `network_id: Uuid`.
- Incorporar `network_id: Uuid` en la entidad `User` y en la tabla `identity.user_account`.
- Modificar la restricción en PostgreSQL a `UNIQUE (network_id, email)`.
- Extender `Patient` con `referenced_patient_ids: Vec<Uuid>` e índice GIN para soportar absorción e historial unificado.
- Diseñar un extractor/middleware gRPC en `clickcare` con caché en memoria (`subdomain -> (network_id, organization_id)`) que resuelva el subdominio desde el encabezado `Host` (o `x-subdomain` en testing).
- Establecer particionamiento estático `PARTITION BY HASH (network_id)` con 8 particiones para tablas transaccionales de alto volumen (`appointment`).

**Non-Goals:**
- Automatización de DNS o aprovisionamiento dinámico de certificados SSL/TLS (responsabilidad del reverse proxy / Cloudflare con wildcard `*.clickcare.com`).
- Particionamiento por listas (`PARTITION BY LIST`), el cual genera bloqueos DDL `AccessExclusiveLock` en runtime.

## Decisions

1. **Lenguaje Ubicuo: `network_id` vs. `organization_id`**:
   - *Decisión*: Se destierra el término ambiguo `tenant_id`. Se usa `network_id` para la Red (ámbito de autenticación, directorio de usuarios y federación) y `organization_id` para la Clínica/Sede física específica (donde residen consultorios y citas).
   - *Razón*: Refleja fielmente el dominio DDD y evita confusiones sobre si un ID refiere a una clínica o a una red.

2. **Red Obligatoria desde el Día Cero (`is_default`)**:
   - *Decisión*: Toda clínica pertenece obligatoriamente a una `ClinicalNetwork`. Para clínicas solitarias se crea una red implícita con `is_default = true`.
   - *Razón*: Elimina código condicional `if clinic.has_network()`. Todo el sistema opera de forma homogénea bajo `network_id`.

3. **Elección Explícita del Subdominio (Opción A)**:
   - *Decisión*: El usuario ingresa el subdominio en el formulario de alta o creación de clínica. Se valida con la regex `^[a-z0-9-]+$` y se prohíben nombres reservados (`app`, `api`, `admin`, `www`, `static`).
   - *Razón*: Brinda branding profesional y predecible a cada clínica cliente.

4. **Resolución y Caché de Subdominio en Capa gRPC**:
   - *Decisión*: El servidor extrae el subdominio del encabezado `Host` (ej. `san-borja.clickcare.com` -> `san-borja`) o del metadato gRPC `x-subdomain`. Consulta una caché en memoria en Rust (`moka` o `RwLock<HashMap>`) con TTL corto e invalidación reactiva.
   - *Razón*: Resolución sub-microsegundo sin impactar la base de datos PostgreSQL en cada frame gRPC.

5. **Cookies Host-Only y SSO Inter-Sede Plano**:
   - *Decisión*: Las cookies de sesión se emiten sin el atributo `Domain=.clickcare.com` (Host-Only). El salto entre sedes asociadas usa un ticket efímero de un solo uso (30s) o login directo con el mismo correo/contraseña en la red.
   - *Razón*: Aislamiento de seguridad total en el navegador entre clínicas no asociadas.

6. **Absorción de Historial Inmutable vía `referenced_patient_ids`**:
   - *Decisión*: En caso de absorción o fusión de clínicas, el expediente activo del paciente en la red mantiene `referenced_patient_ids: Vec<Uuid>`. Las consultas buscan `WHERE patient_id = ANY(ARRAY[id] || referenced_patient_ids)` respaldadas por un índice GIN.
   - *Razón*: Alineado al estándar HL7 FHIR R4 (`Patient.link`); evita migraciones de actualización destructivas sobre filas de consultas pasadas.

## Risks / Trade-offs

- [Colisiones de subdominio al crearse] → Mitigado: Restricción `UNIQUE` en PostgreSQL y verificación en el caso de uso antes de persistir.
- [Pruebas locales en `localhost`] → Mitigado: Soporte de metadato gRPC `x-subdomain: <nombre>` como fallback oficial de desarrollo.
