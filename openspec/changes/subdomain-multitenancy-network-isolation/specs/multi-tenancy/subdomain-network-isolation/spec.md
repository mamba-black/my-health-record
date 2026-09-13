## Purpose

Garantiza el aislamiento estricto de datos y cuentas entre clínicas no asociadas mediante subdominios planos, permitiendo la compartición federada de identidades de usuario, sesiones y datos operativos exclusivamente entre clínicas pertenecientes a una misma red médica (`network_id`).

## ADDED Requirements

### Requirement: Asignación y validación de subdominio en la clínica
El sistema SHALL permitir que el usuario elija explícitamente el subdominio al crear una clínica, validando que cumpla el patrón `^[a-z0-9-]+$` y no coincida con nombres reservados de la plataforma.

#### Scenario: Elección de subdominio válido
- **WHEN** un usuario registra una clínica con subdominio `san-borja`
- **THEN** el sistema valida el formato, verifica que no esté ocupado y registra la clínica vinculada a dicho subdominio

#### Scenario: Rechazo de subdominio reservado o inválido
- **WHEN** un usuario intenta registrar una clínica con un subdominio reservado (`app`, `api`, `admin`, `www`, `static`) o con caracteres no permitidos (`san_borja!`)
- **THEN** el sistema rechaza la operación retornando un error `INVALID_ARGUMENT`

### Requirement: Resolución y caché de subdominio en capa gRPC
El sistema SHALL resolver el `network_id` y el `organization_id` a partir del subdominio provisto en el encabezado `Host` / `Origin` o el metadato gRPC `x-subdomain`, utilizando una caché en memoria para optimizar el rendimiento.

#### Scenario: Resolución mediante encabezado Host HTTP/2
- **WHEN** un cliente gRPC-Web envía una solicitud con el encabezado `Host: san-borja.clickcare.com`
- **THEN** el sistema extrae el subdominio `san-borja`, consulta la caché o el repositorio, y contextualiza la solicitud con su `network_id` y `organization_id`

#### Scenario: Resolución mediante metadato gRPC de desarrollo
- **WHEN** un cliente de pruebas envía una solicitud con el metadato gRPC `x-subdomain: san-borja`
- **THEN** el sistema resuelve el contexto de la clínica de forma idéntica al encabezado Host

#### Scenario: Petición con subdominio no registrado
- **WHEN** un cliente envía una solicitud con un subdominio inexistente en la plataforma
- **THEN** el sistema rechaza la solicitud retornando un error gRPC `NOT_FOUND`

### Requirement: Aislamiento estricto de cuentas de usuario entre redes no asociadas
El sistema SHALL restringir la unicidad y el ciclo de vida de las cuentas `User` al ámbito de su `network_id`, impidiendo el cruce de autenticación o visibilidad entre clínicas que no pertenezcan a la misma red.

#### Scenario: Registro con el mismo correo en redes independientes
- **WHEN** un usuario registra una cuenta con `email = doctor@example.com` en `clinica-a.clickcare.com` y posteriormente se registra con el mismo correo en `clinica-b.clickcare.com` (red independiente)
- **THEN** el sistema procesa ambos registros como cuentas independientes con diferentes `user_id`, cada una perteneciente al `network_id` de su respectiva red

#### Scenario: Rechazo de autenticación en red no asociada
- **WHEN** un usuario registrado en `clinica-a.clickcare.com` intenta autenticarse mediante `SignIn` en `clinica-b.clickcare.com` (no asociada a la clínica A)
- **THEN** el sistema rechaza la autenticación con error gRPC `UNAUTHENTICATED`, impidiendo el acceso a la clínica ajena

### Requirement: Compartición de identidad entre clínicas de una misma red asociada
El sistema SHALL permitir que las clínicas vinculadas bajo un mismo `network_id` compartan el directorio de usuarios y el contexto de autenticación.

#### Scenario: Autenticación unificada entre clínicas asociadas
- **WHEN** un usuario registrado en `sede-norte.clickcare.com` intenta autenticarse en `sede-sur.clickcare.com` (ambas asociadas bajo el mismo `network_id`)
- **THEN** el sistema valida satisfactoriamente sus credenciales y emite una sesión válida para la sede consultada

#### Scenario: Detección de duplicidad de correo dentro de la misma red
- **WHEN** un cliente intenta registrar un usuario con un correo ya existente dentro del mismo `network_id`
- **THEN** el sistema rechaza la operación con error `ALREADY_EXISTS` garantizando la unicidad dentro de la red

### Requirement: Absorción de clínicas e historial médico unificado
El sistema SHALL permitir que un paciente mantenga una lista de identificadores previos absorbidos (`referenced_patient_ids`), y consolidar la búsqueda de registros médicos a través de dicha lista.

#### Scenario: Consulta de historia clínica con expedientes absorbidos
- **WHEN** un profesional médico consulta la historia clínica de un paciente que posee identificadores absorbidos en `referenced_patient_ids`
- **THEN** el sistema consulta y retorna las atenciones, diagnósticos y recetas asociadas tanto al identificador principal como a cualquiera de los identificadores absorbidos
