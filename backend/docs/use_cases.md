# Casos de Uso del Ciclo de Vida de Cuentas e Identidad

## Caso de Uso 1: Registro Progresivo y Cita Confirmada
El usuario se registra con datos mínimos (Google OIDC/Email). Al agendar una cita médica, el DNI y Teléfono se vuelven obligatorios. La cita se guarda **100% CONFIRMADA** en la agenda de la clínica.

```mermaid
sequenceDiagram
    autonumber
    actor Patient as Paciente / App
    participant App as App Móvil/Web
    participant UserDomain as crates/user
    participant SchedDomain as crates/scheduling
    participant DB as Base de Datos PostgreSQL

    Patient->>App: 1. Registro inicial (Email / OIDC)
    App->>UserDomain: Crear Usuario (Person Mínima)
    UserDomain->>DB: Guardar Usuario (active=true, DNI=None)

    Patient->>App: 2. Reservar Cita Médica
    App->>Patient: Solicitar DNI y Teléfono (Obligatorio Ley 30024)
    Patient->>App: Ingresa DNI y Teléfono
    App->>UserDomain: Actualizar Person (identifier=DNI, telecom=Teléfono)
    App->>SchedDomain: Reservar Cita (Médico, Horario)
    SchedDomain-->>App: Estado de Cita: CONFIRMADA
    App-->>Patient: Mostrar Confirmación de Cita
```

## Caso de Uso 2: Recuperación de Credenciales y Re-vinculación Presencial
El usuario perdió acceso a su correo/teléfono antiguo. Registra una nueva cuenta con su DNI. La cuenta se crea con `LinkAssuranceLevel::Level1` (Pendiente). La cita se guarda **100% CONFIRMADA**. El día de la cita, la recepcionista verifica el DNI físico al hacer Check-in, elevando la certeza a `Level3`/`Level4` (Verificado).

```mermaid
sequenceDiagram
    autonumber
    actor Patient as Paciente
    actor Receptionist as Recepcionista de Clínica
    participant App as App / Sistema
    participant UserDomain as crates/user
    participant AdminDomain as crates/administration
    participant DB as Base de Datos PostgreSQL

    Patient->>App: Registro con Nuevo Correo + DNI Existente
    App->>UserDomain: Detectar Historial de DNI Existente
    UserDomain->>UserDomain: Crear Usuario + PersonLink (Assurance: Level1 Pendiente)
    App->>App: Reservar Cita Médica (Estado: CONFIRMADA)

    Note over Patient, Receptionist: Día de la Cita Médica (Check-in Presencial)
    Patient->>Receptionist: Llega a la clínica y presenta DNI Físico
    Receptionist->>AdminDomain: Realizar Check-in (DNI 10000001)
    AdminDomain->>AdminDomain: Detectar Solicitud de Vinculación Pendiente (carlos_nuevo@gmail.com)
    Receptionist->>AdminDomain: Verificar DNI Físico y clic en "Aprobar Vinculación"
    AdminDomain->>UserDomain: Elevar Assurance de PersonLink (Level3/Level4 Verificado)
    UserDomain->>DB: Actualizar Assurance y Desactivar Usuario Antiguo
    AdminDomain-->>Patient: Paciente con Check-in completo y App totalmente vinculada
```

## Caso de Uso 3: Corrección de DNI Registrado por Error y Conversión a Perfil Familiar
El usuario registró por error el DNI de un familiar (ej. hijo o padre) en su cuenta principal. El usuario convierte el DNI del familiar en un perfil gestionado de `Patient` (`PersonLinkTarget::Patient`) e ingresa su propio DNI en la cuenta principal.

```mermaid
sequenceDiagram
    autonumber
    actor User as Titular de la Cuenta
    participant App as App Móvil
    participant UserDomain as crates/user
    participant AdminDomain as crates/administration
    participant DB as Base de Datos PostgreSQL

    User->>App: Seleccionar "Corregir DNI / Mover DNI a Dependiente"
    App->>UserDomain: Iniciar Conversión de Perfil (DNI 77777777)
    UserDomain->>AdminDomain: Crear Perfil de Paciente Dependiente (DNI 77777777)
    AdminDomain->>DB: Guardar Paciente (Gestionado por Titular)
    UserDomain->>UserDomain: Agregar PersonLinkTarget::Patient(dependent_id)

    User->>App: Ingresar DNI Real del Titular (10000001)
    App->>UserDomain: Actualizar User.person.identifier = DNI 10000001
    UserDomain->>DB: Guardar Identidad del Titular
    App-->>User: Perfil Reorganizado (Titular + Paciente Dependiente)
```

## Caso de Uso 4: Consulta Previa por DNI y Opciones Dinámicas
La App consulta a la API antes o durante el registro. El backend verifica la existencia y estado del DNI para guiar las opciones de interfaz.

```mermaid
sequenceDiagram
    autonumber
    actor Client as App / Cliente
    participant API as UserApi (gRPC)
    participant UserDomain as crates/user
    participant DB as Base de Datos PostgreSQL

    Client->>API: Consultar Estado de DNI (DNI 10000001)
    API->>UserDomain: Verificar Existencia de DNI
    UserDomain->>DB: SELECT FROM users/persons WHERE identifier = DNI 10000001

    alt DNI No Encontrado
        DB-->>UserDomain: No Encontrado
        UserDomain-->>API: Disponible
        API-->>Client: Estado: OK (DNI Disponible para registro normal)
    else DNI Existe en Cuenta Activa (Level3/Level4 Verificado)
        DB-->>UserDomain: Usuario Activo (carlos@gmail.com)
        UserDomain-->>API: Conflicto (Cuenta Verificada)
        API-->>Client: Estado: ALREADY_EXISTS (Sugerir Iniciar Sesión / Recuperar Cuenta)
    else DNI Existe en Historia Clínica / Cuenta No Verificada
        DB-->>UserDomain: Historial de Paciente Encontrado
        UserDomain-->>API: Historial Encontrado (Vinculación Disponible)
        API-->>Client: Estado: LINK_AVAILABLE (Sugerir Solicitar Vinculación Presencial)
    end
```

## Caso de Uso 5: Independización de Perfil Dependiente (Hijo cumple 18 años)
Un hijo registrado como dependiente (`Patient` vinculado a la cuenta del padre) registra su propia cuenta `User` autónoma con su correo y DNI.

```mermaid
sequenceDiagram
    autonumber
    actor Child as Dependiente (Ahora Adulto)
    participant App as App Móvil
    participant UserDomain as crates/user
    participant AdminDomain as crates/administration
    participant DB as Base de Datos PostgreSQL

    Child->>App: Registrar Cuenta Autónoma (email + DNI 77777777)
    App->>UserDomain: SignUpRequest (confirm_pending_presencial_link=true)
    UserDomain->>UserDomain: Crear Usuario + PersonLink (Assurance: Level1 Pendiente)
    UserDomain->>DB: Guardar Cuenta de Usuario
    App-->>Child: Mostrar Aviso de Verificación Pendiente

    Note over Child, AdminDomain: Cita Presencial en la Clínica
    Child->>AdminDomain: Presentar DNI Físico en Check-in
    AdminDomain->>UserDomain: Aprobar Vinculación y Transferir Registro (Assurance: Level3/Level4)
    UserDomain->>DB: Actualizar Vinculación y Desvincular Estado de Dependiente
    AdminDomain-->>Child: App Desbloqueada e Historial Totalmente Independiente
```

## Caso de Uso 6: Actualización de Datos de Perfil y Controles por Nivel de Certeza
El usuario intenta actualizar campos de identidad (Nombre, DNI, Teléfono). Las actualizaciones se permiten libremente en cuentas no verificadas (`Level1`) y se restringen/auditan en cuentas verificadas (`Level3`/`Level4`).

```mermaid
sequenceDiagram
    autonumber
    actor User as Usuario / Paciente
    participant App as App Móvil
    participant UserDomain as crates/user
    participant DB as Base de Datos PostgreSQL

    User->>App: Solicitud de Edición de Perfil (Nombre / DNI)
    App->>UserDomain: UpdateProfile(UserCommand)

    alt Assurance es Level1 (No Verificado / Pendiente)
        UserDomain->>UserDomain: Actualizar Identifier / Nombre de Person
        UserDomain->>DB: Guardar Person Actualizado
        UserDomain-->>App: Éxito (Perfil Actualizado)
    else Assurance es Level3/Level4 (Verificado en Clínica)
        UserDomain-->>App: Error / Restringido (Edición de DNI requiere aprobación en clínica)
        App-->>User: Mostrar Aviso ("Contacta a Recepción para actualizar un DNI verificado")
    end
```
