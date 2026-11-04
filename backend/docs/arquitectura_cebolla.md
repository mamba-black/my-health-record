# Arquitectura Cebolla: reglas innegociables (Java y Rust)

Este documento es a la vez un manifiesto técnico y una **instrucción normativa estricta
para agentes de inteligencia artificial**. No contiene sugerencias flexibles: son reglas
sobre cómo estructurar un proyecto con Arquitectura Cebolla (_Onion Architecture_).

Las reglas valen para cualquier lenguaje. Cuando hace falta un ejemplo se muestra en
**Java** y en **Rust**.

---

## 1. Las tres capas y sus nombres

El proyecto se divide en tres capas concéntricas: **`domain`**, **`application`** e
**`infrastructure`**.

### 1.1. Por qué `application` y no `application services`

No todo lo que interviene en la coordinación del sistema es un servicio:

1. El **caso de uso** (`UseCase`) no es un servicio: es una **orquestación** del flujo de
   una acción del sistema.
2. Escuchar un evento externo no es un servicio de aplicación: es **otro punto de entrada**,
   igual que un controlador.
3. Solo los **servicios externos** (pasarelas de pago, APIs de terceros, procesos fuera de
   nuestro control) son servicios.

Llamar "servicios" a la capa cuyo corazón son los casos de uso promueve clases gigantes
con métodos inconexos (`UserService`, `GitUseCases`). Por eso la capa se llama
`application` y sus piezas se llaman casos de uso.

### 1.2. `domain` frente a `application`

- **`domain`:** el núcleo inmutable del negocio. Entidades, su estado, sus reglas puras y
  los contratos de salida que el negocio necesita para existir (repositorios y
  publicadores de eventos).
- **`application`:** la capa de orquestación. Define los flujos de trabajo y coordina qué
  pasos se ejecutan con las piezas del dominio para resolver una tarea.

---

## 2. Reglas de oro

```text
[ INFRAESTRUCTURA ]  única capa CON frameworks
        │            (controladores, suscriptores, base de datos, clientes, serialización)
        ▼
[ APLICACIÓN ]       CERO frameworks
        │            (casos de uso, DTOs, contratos de servicios externos)
        ▼
[ DOMINIO ]          CERO frameworks
                     (entidades, objetos de valor, eventos, contratos de repositorio y publicador)
```

### Regla 1: dependencia unidireccional hacia adentro

- `infrastructure` conoce a `application` y a `domain`.
- `application` conoce únicamente a `domain`.
- `domain` no conoce a nadie.

### Regla 2: ni `domain` ni `application` tienen frameworks

Ambas capas internas están libres de frameworks. Todo se escribe en código puro del
lenguaje. Queda prohibido importar en ellas:

- **Java:** Spring, Quarkus, JPA/Hibernate, Jackson, clientes HTTP, clientes de mensajería.
- **Rust:** `tauri`, `serde` (derives de serialización), `sqlx`/`rusqlite`, `reqwest`,
  `tantivy` y similares, además de `std::fs` y `std::process` (el acceso a disco y a procesos
  se hace mediante un contrato).

**Librerías permitidas.** Se aceptan en `domain` y `application` solo dos tipos de
librería, y únicamente si figuran en la lista explícita del test de arquitectura:

1. Las que solo eliminan código repetitivo sin cambiar el significado del modelo
   (Lombok en Java, `thiserror` en Rust).
2. Las librerías puras que implementan un estándar y rara vez cambian (un parser de
   CommonMark, expresiones regulares, UUID), sin I/O y sin obligar a anotar el modelo. Su uso
   queda encerrado en un solo lugar, para que reemplazarlas toque un solo archivo.

3. `log`, la fachada de logging: no hace I/O por sí misma (lo hace el backend que configura
   `infrastructure`), así que un caso de uso puede registrar avisos.

Lista actual de Synapse:

| Librería         | Tipo                  | Dónde                                        | Para qué                                  |
| :--------------- | :-------------------- | :------------------------------------------- | :---------------------------------------- |
| `log`            | Fachada de logging    | casos de uso                                 | Avisos y trazas                           |
| `pulldown_cmark` | Estándar (CommonMark) | `domain/graph/tag.rs`, `domain/note/note.rs` | Separar texto real de sintaxis Markdown   |
| `sha2`           | Estándar (SHA-256)    | `domain/note/note_content.rs`                | Huella `sha256:` del contenido para e control de concurrencia optimista del MCP; no se cambia el algoritmo porque los agentes devuelven ese valor |

Lista de My Health Record (backend):

| Librería         | Tipo                         | Dónde                      | Para qué                                                    |
| :--------------- | :--------------------------- | :------------------------- | :---------------------------------------------------------- |
| `log`            | Fachada de logging           | casos de uso               | Avisos y trazas                                             |
| `tracing`        | Fachada de logging con spans | casos de uso               | Trazas estructuradas; se permite como alternativa a `log`   |
| `uuid`           | Estándar (RFC 4122)          | entidades, identificadores | Identidad de entidades                                      |
| `uuid_v7`        | Estándar (RFC 9562 / UUIDv7) | entidades, casos de uso    | Identidad y generación de UUIDv7 (`Uuid::now_v7()`) para claves primarias ordenables temporalmente |
| `derive_getters` | Quita código repetitivo      | entidades                  | Equivalente a `@Getter` de Lombok                           |
| `bon`            | Quita código repetitivo      | entidades                  | Equivalente a `@Builder` de Lombok. El builder debe pasar por las validaciones de invariantes                                                |
| `strum_macros`   | Quita código repetitivo      | enums de dominio           | `Display` para enums, sin cambiar el significado del modelo |
| `async_trait`    | Quita código repetitivo      | contratos asíncronos       | Solo mientras los traits se usen como `dyn`; si no, usar `async fn` nativo                                                                   |
| `thiserror`      | Quita código repetitivo      | errores de casos de uso    | Deriva `Display` y `Error` para los errores de aplicación (equivalente a una excepción tipada); no cambia el significado del modelo          |

Entrar en la lista es una decisión, no un descuido. Siguen prohibidas las que obligan a
anotar el modelo (`serde`, JPA, Jackson) y las que contactan servicios externos o hacen I/O
(`jsonwebtoken` para verificar tokens de terceros, `google_jwt_verify`). Estas últimas van
detrás de un contrato de servicio externo, con la implementación en `infrastructure`.

La única capa que conoce y usa frameworks es `infrastructure`. La serialización
(`serde`, Jackson) vive allí, en modelos de transporte con _mappers_ hacia el dominio.

### Regla 3: los eventos

1. **Los suscriptores de eventos (`subscriber`) viven en `infrastructure`.** Escuchar un
   mensaje (Kafka, RabbitMQ, SQS, o el vigilante de archivos de una aplicación de
   escritorio) es igual que recibir una petición HTTP: es otro punto de entrada. El
   suscriptor deserializa el mensaje e invoca al **caso de uso** correspondiente.
   No existe un `application/handler`.
2. **Cada publicador va junto a los eventos que publica.** Hay dos tipos:
    - **Interno:** los eventos de dominio y su publicador (`AccountEventPublisher`) viven en
      `domain/<agregado>/`. Los consume el mismo microservicio.
    - **Externo (integración):** los eventos de integración y su publicador
      (`IntegrationEventPublisher`) viven en `application/integration/event/`. Los consumen otros
      microservicios.
      La implementación técnica de ambos, con el framework de mensajería, vive en `infrastructure`.

### Regla 4: los casos de uso no tienen contrato de entrada

El caso de uso es una clase concreta. Los puntos de entrada la invocan directamente.
No se declara una interfaz de entrada porque no va a cambiar.

### Regla 5: el arranque y el ensamblado viven en `infrastructure`

Qué se ejecuta al iniciar (servidor, CLI, aplicación de escritorio) y qué implementación
se pasa a cada caso de uso se decide en `infrastructure/config`.

---

## 3. Repository o servicio externo: la regla de propiedad

La pregunta no es "¿está en mi máquina?" sino **"¿yo soy dueño de estos datos y de su forma?"**

|                         | Repository                                              | Servicio externo           |
| :---------------------- | :------------------------------------------------------ | :------------------------- |
| Dueño de los datos      | Yo                                                      | Un tercero                 |
| Quién dicta el contrato | Yo (esquema, formato)                                   | Ellos (API, protocolo)     |
| Dónde vive el contrato  | `domain/<agregado>/`                                    | `application/integration/service/` |
| Ubicación física        | Indiferente (disco, SQLite, Postgres propio en la nube) | Indiferente                |

Prueba rápida: si mañana cambio la tecnología y mi dominio no se entera, es un
**repository**. Si tengo que adaptarme a lo que otro decide, es un **servicio externo**.

| Pieza                                                    | Clasificación    |
| :------------------------------------------------------- | :--------------- |
| Cuentas o notas en disco, SQLite o Postgres propio       | Repository       |
| Mi repositorio de versiones local                        | Repository       |
| Un índice de búsqueda que construyo yo                   | Repository       |
| API de pagos, correo SMTP, catálogo de plugins en GitHub | Servicio externo |
| Abrir un archivo con el sistema operativo                | Servicio externo |
| Un agente o proceso externo                              | Servicio externo |

### Capacidades intercambiables

Aunque una capacidad sea un cálculo en memoria (por ejemplo una búsqueda difusa), si
**podría cambiar de implementación** (otra librería, o un servicio externo), es un
**contrato**. La prueba: "¿alguna vez querría sustituirla sin tocar el negocio?". Se declara
según la regla de propiedad: sobre mis entidades con una librería local es un repository
(`NoteSearchRepository`); si se delega a un tercero que impone su API, es un servicio externo.

Solo es lógica propia, sin contrato, lo que es regla de negocio y nunca cambiaría de
implementación (por ejemplo, calcular el total de un pedido).

---

## 4. Catálogo de arquetipos

### Cuándo existe un servicio de dominio

Solo cuando una regla de negocio involucra **más de una entidad** y no pertenece de forma
natural a ninguna. Además debe cumplir tres condiciones:

1. **Sin estado:** recibe las entidades y devuelve un resultado.
2. **Puro:** sin I/O ni repositorios. Cargar y guardar es trabajo del caso de uso; el
   servicio recibe las entidades ya cargadas.
3. **Nombrado por la operación de negocio** (`LinkResolver`, `FundsTransferer`), nunca
   `NoteService` ni `AccountService`, para no confundirlo con los servicios externos de
   `application/integration`.

Si la lógica toca una sola entidad o un solo concepto, es un método de la entidad o del
objeto de valor, no un servicio de dominio.

| Capa                 | Arquetipo                       | Responsabilidad y regla de                                                                                                             |
| :------------------- | :------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------- |
| **`domain`**         | **Entity**                      | Objeto con identidad, estado y métodos que protegen sus invariantes. Prohibidos los modelos anémicos.                                  |
|                      | **Value Object**                | Inmutable, sin identidad. Valida su propia consistencia al crearse.                                                                    |
|                      | **Domain Event**                | Hecho de negocio ya ocurrido, inmutable.                                                                                               |
|                      | **Repository**                  | **Solo contrato**, junto a su entidad en `domain/<agregado>/`. Guardar y recuperar las entidades de las que soy dueño.                 |
|                      | **Event Publisher**             | **Solo contrato**, junto a sus eventos en `domain/<agregado>/`. Emitir eventos de dominio dentro del mismo microservicio.              |
|                      | **Domain Service**              | Regla de negocio que involucra **más de una entidad** y no pertenece a ninguna. Sin estado, sin I/O ni repositorios (recibe las entidades ya cargadas), nombrado por la operación. Excepcional. |
|                      | **Domain Error**                | Errores puros de negocio y de los contratos, sin códigos HTTP ni detalles técnicos.                                                    |
| **`application`**    | **Use Case**                    | **Orquestador atómico**, uno por intención de negocio, con un único método `execute`. Llama al repository, ejecuta métodos de la entidad, invoca servicios externos o dispara el publicador. |
|                      | **Command / Query / DTO**       | Objetos inmutables de datos puros. Cada caso de uso recibe un `Command` (escritura) o un `Query` (lectura), aunque lleve un solo dato. |
|                      | **Integration Event Publisher** | **Solo contrato**, junto a sus eventos en `application/integration/event/`. Emitir eventos de integración hacia otros microservicios.  |
|                      | **External Service**            | **Solo contrato** de un servicio que no controlamos, en `application/integration/service/`.                                            |
| **`infrastructure`** | **Controller**                  | Punto de entrada síncrono (REST, gRPC, comandos de escritorio). Recibe, convierte y llama al caso de uso.                              |
|                      | **Subscriber**                  | Punto de entrada asíncrono (colas, vigilante de archivos). Escucha e invoca al caso de uso.                                            |
|                      | **Repository Impl**             | Implementa el repository con frameworks y mapea el modelo de persistencia a la entidad.                                                |
|                      | **Event Publisher Impl**        | Implementa el publicador con el framework de mensajería.                                                                               |
|                      | **External Service Impl**       | Implementación técnica (HTTP, SDK, procesos) del contrato de servicio externo.                                                         |
|                      | **Configuration**               | Ensambla los casos de uso con sus implementaciones.                                                                                    |

---

## 5. Nombres

| Elemento                     | Regla                                        | Ejemplo                           |
| :--------------------------- | :------------------------------------------- | :-------------------------------- |
| Contrato de repository       | Agregado + `Repository`                      | `AccountRepository`               |
| Contrato de servicio externo | Nombre de la **capacidad**, no de la entidad | `WelcomeEmailSender`              |
| Contrato de publicador       | Capacidad                                    | `AccountEventPublisher`           |
| Servicio de dominio          | Operación de negocio, nunca `…Service`       | `FundsTransferer`, `LinkResolver` |
| Implementación               | Contrato + `Impl`                            | `AccountRepositoryImpl`           |
| Segunda implementación       | Calificador delante; `Impl` siempre al final | `MockAccountRepositoryImpl`       |
| Caso de uso                  | Intención de negocio + `UseCase`             | `RegisterAccountUseCase`          |
| Entrada del caso de uso      | Intención + `Command` o `Query`              | `RegisterAccountCommand`          |
| Modelo de persistencia       | Entidad + sufijo técnico                     | `AccountRow`, `AccountJpaEntity`  |

Reglas de apoyo:

- Si un contrato de servicio y una entidad empiezan con la misma palabra, algo está mal
  nombrado. El nombre de una herramienta (Git, Tantivy, Postgres) solo aparece en
  `infrastructure`; las capas internas hablan en conceptos de negocio.
- La primera implementación no lleva calificador. Si aparece una segunda de producción, la
  nueva se califica (`PostgresAccountRepositoryImpl`) y la original se conserva.
- Los dobles de prueba (`Mock…Impl`, `InMemory…Impl`) viven en el código de pruebas, no en
  `infrastructure`.
- El archivo sigue el nombre de la clase: `account_repository_impl.rs` en Rust,
  `AccountRepositoryImpl.java` en Java.
- Un campo que guarda una dependencia se nombra por su rol (`accounts`, `mailer`), nunca
  por el patrón.
- Sin lenguaje técnico en las capas internas: se usa el lenguaje ubicuo del negocio.

---

## 6. Dos destinos para los mismos datos

Cuando hay más de un destino (disco local y nube, SQLite y Postgres), la pregunta es
**quién necesita saber que uno falló**.

- **Solo la mecánica** (reintentar y listo): se queda dentro de **una** implementación, con
  piezas internas privadas que no implementan el contrato.
- **El negocio reacciona distinto** (avisar, marcar como pendiente, elegir otro destino):
  el caso de uso recibe **dos dependencias del mismo contrato**, con nombres que expresan su
  rol (`local`, `remote`), y decide con el resultado.
- Solo si los destinos tienen operaciones distintas se crean **dos contratos**.

El error es un tipo del dominio (`SaveError::Unavailable`), nunca el de la base de datos.
La re-subida cuando vuelve la red es otro caso de uso (`SyncPendingAccountsUseCase`), y la
política de conflictos se escribe en el dominio.

---

## 7. Estructura de directorios

### Java

```text
src/main/java/com/empresa/proyecto/
├── domain/                              # NÚCLEO PURO, agrupado por agregado
│   ├── account/                         # Account, Email, AccountRepository, AccountEventPublisher, AccountRegistered
│   ├── service/                         # Servicios de dominio (excepcionales)
│   └── exception/                       # Errores de negocio
│
├── application/                         # ORQUESTACIÓN
│   ├── usecase/                         # Un archivo por caso de uso
│   ├── dto/                             # Commands, Queries y respuestas
│   └── integration/                     # Contratos externos
│       ├── event/                       # IntegrationEventPublisher, AccountRegisteredIntegrationEvent
│       └── service/                     # WelcomeEmailSender
│
└── infrastructure/                      # ÚNICA CAPA CON FRAMEWORKS
    ├── entrypoint/
    │   ├── controller/                  # REST, gRPC y sus DTOs de transporte
    │   └── subscriber/                  # Kafka, RabbitMQ
    ├── repository/
    │   └── jpa/                         # AccountRepositoryImpl, AccountJpaEntity, AccountMapper
    ├── integration/
    │   ├── event/
    │   │   └── kafka/                   # AccountIntegrationEventPublisherImpl
    │   └── service/
    │       └── smtp/                    # WelcomeEmailSenderImpl
    └── config/                          # Beans e inyección
```

### Rust

```text
src/
├── domain/
│   ├── account/                         # account.rs, email.rs, account_repository.rs (trait),
│   │                                    # account_event_publisher.rs (trait), account_registered.rs
│   ├── service/                         # servicios de dominio (excepcionales)
│   └── error/                           # errores de negocio
│
├── application/
│   ├── use_case/                        # register_account_use_case.rs
│   ├── dto/                             # register_account_command.rs
│   └── integration/
│       ├── event/                       # integration_event_publisher.rs (trait), account_registered_integration_event.rs
│       └── service/                     # welcome_email_sender.rs (trait)
│
└── infrastructure/
    ├── entrypoint/
    │   ├── controller/                  # REST o comandos Tauri, con sus DTOs serde
    │   └── subscriber/                  # vigilante de archivos, colas
    ├── repository/
    │   └── sqlite/                      # account_repository_impl.rs, account_row.rs, account_mapper.rs
    ├── integration/
    │   ├── event/                       # apalis_integration_event_publisher_impl.rs
    │   └── service/
    │       └── smtp/                    # welcome_email_sender_impl.rs
    └── config/                          # ensamblado de dependencias
```

En `infrastructure`, la primera subcarpeta (`repository`, `publisher`, `integration`) indica qué
contrato se cumple y la segunda indica la tecnología. `entrypoint` agrupa todo lo que
invoca casos de uso.

---

## 8. Ejemplo completo: registrar una cuenta

Usa un repository, un servicio externo (correo) y un publicador de eventos.

### Java

```java
// domain/account/AccountRepository.java
public interface AccountRepository {
    void save(Account account) throws SaveError;
    Optional<Account> findByEmail(Email email) throws SaveError;
}

// domain/account/AccountEventPublisher.java
public interface AccountEventPublisher {
    void publish(AccountRegistered event);
}

// application/integration/service/WelcomeEmailSender.java   (servicio externo)
public interface WelcomeEmailSender {
    void send(Email to) throws SendError;
}

// application/dto/RegisterAccountCommand.java
public record RegisterAccountCommand(String email) {}

// application/usecase/RegisterAccountUseCase.java
public class RegisterAccountUseCase {
    private final AccountRepository accounts;
    private final WelcomeEmailSender mailer;
    private final AccountEventPublisher events;

    public RegisterAccountUseCase(AccountRepository accounts, WelcomeEmailSender mailer,
                                  AccountEventPublisher events) {
        this.accounts = accounts;
        this.mailer = mailer;
        this.events = events;
    }

    public AccountId execute(RegisterAccountCommand command) {
        Email email = new Email(command.email());             // el objeto de valor valida
        if (accounts.findByEmail(email).isPresent()) {
            throw new AccountAlreadyExists(email);
        }
        Account account = Account.register(email);             // regla en el dominio
        accounts.save(account);
        events.publish(new AccountRegistered(account.id()));
        try {
            mailer.send(email);
        } catch (SendError e) {
            // decisión del caso de uso: la cuenta existe aunque el correo falle
        }
        return account.id();
    }
}

// infrastructure/repository/jpa/AccountRepositoryImpl.java
@Repository
class AccountRepositoryImpl implements AccountRepository {
    private final AccountJpaRepository jpa;
    private final AccountMapper mapper;
    // save: mapper.toJpa(account) -> jpa.save(...)
    // findByEmail: jpa.findByEmail(...).map(mapper::toDomain)
}

// infrastructure/config/UseCaseConfig.java
@Configuration
class UseCaseConfig {
    @Bean
    RegisterAccountUseCase registerAccountUseCase(AccountRepository accounts,
            WelcomeEmailSender mailer, AccountEventPublisher events) {
        return new RegisterAccountUseCase(accounts, mailer, events);
    }
}
```

### Rust

```rust
// domain/account/account_repository.rs
pub trait AccountRepository: Send + Sync {
    fn save(&self, account: &Account) -> Result<(), SaveError>;
    fn find_by_email(&self, email: &Email) -> Result<Option<Account>, SaveError>;
}

// domain/account/account_event_publisher.rs
pub trait AccountEventPublisher: Send + Sync {
    fn publish(&self, event: AccountRegistered);
}

// application/integration/service/welcome_email_sender.rs   (servicio externo)
pub trait WelcomeEmailSender: Send + Sync {
    fn send(&self, to: &Email) -> Result<(), SendError>;
}

// application/dto/register_account_command.rs   (sin serde: es código puro)
pub struct RegisterAccountCommand {
    pub email: String,
}

// application/use_case/register_account_use_case.rs
pub struct RegisterAccountUseCase<A, M, E>
where
    A: AccountRepository,
    M: WelcomeEmailSender,
    E: AccountEventPublisher,
{
    accounts: A,
    mailer: M,
    events: E,
}

impl<A, M, E> RegisterAccountUseCase<A, M, E>
where
    A: AccountRepository,
    M: WelcomeEmailSender,
    E: AccountEventPublisher,
{
    pub fn new(accounts: A, mailer: M, events: E) -> Self {
        Self { accounts, mailer, events }
    }

    pub fn execute(&self, command: RegisterAccountCommand) -> Result<AccountId, RegisterAccountError> {
        let email = Email::parse(&command.email)?;                // el objeto de valor valida
        if self.accounts.find_by_email(&email)?.is_some() {
            return Err(RegisterAccountError::AlreadyExists(email));
        }
        let account = Account::register(email.clone());            // regla en el dominio
        self.accounts.save(&account)?;
        self.events.publish(AccountRegistered::new(account.id()));
        // decisión del caso de uso: la cuenta existe aunque el correo falle
        let _ = self.mailer.send(&email);
        Ok(account.id())
    }
}

// infrastructure/repository/sqlite/account_repository_impl.rs
pub struct AccountRepositoryImpl { /* conexión */ }

impl AccountRepository for AccountRepositoryImpl {
    fn save(&self, account: &Account) -> Result<(), SaveError> {
        let row = AccountRow::from(account);      // modelo con serde / sqlx
        // INSERT ...; los errores de SQLite se convierten en SaveError
        Ok(())
    }
    fn find_by_email(&self, email: &Email) -> Result<Option<Account>, SaveError> {
        // SELECT ... -> AccountRow -> Account (mapper)
        Ok(None)
    }
}

// infrastructure/entrypoint/controller/register_account_controller.rs
// Recibe un DTO de transporte (con serde), lo convierte en RegisterAccountCommand,
// llama a execute y traduce el resultado a la respuesta (códigos, JSON).
```

El ensamblado vive en `infrastructure/config`. En Java suele hacerlo el contenedor de
dependencias; en Rust se hace a mano al arrancar, o con `Arc<dyn Trait>` si la implementación
se elige en ejecución.

---

## 9. Beneficios

1. **Pruebas ultrarrápidas del núcleo.** Sin frameworks en `domain` ni `application`, todas las
   reglas y los flujos se prueban en milisegundos con dobles en memoria.
2. **Bajo impacto ante cambios técnicos.** Migrar de base de datos o de sistema de mensajería
   reemplaza clases en `infrastructure` sin tocar casos de uso ni entidades.
3. **Longevidad.** Los frameworks cambian; la lógica en código nativo puro sobrevive.

---

## 10. Trampas que invalidan la arquitectura

1. **Modelo de dominio anémico.** Entidades que solo tienen atributos y accesores, con la
   lógica en el caso de uso.
2. **Lenguaje técnico en el dominio.** Clases con jerga de herramienta en lugar de términos de
   negocio (`GitStatus` frente a un concepto propio).
3. **Fuga de frameworks hacia adentro.** `@Entity`, `@Transactional`, `@Autowired`,
   `#[derive(Serialize)]` o `tauri::` en `domain` o `application`.
4. **Casos de uso cajón.** Una clase con muchos métodos por tema en lugar de una por intención.
5. **I/O directo en `application`.** Llamar a `std::fs`, `java.nio.file` o un cliente HTTP desde
   un caso de uso: falta un contrato.
6. **Lógica de negocio en un punto de entrada.** Un controlador o suscriptor que decide cosas
   en lugar de delegar al caso de uso.
7. **Errores de la tecnología en el dominio.** Los contratos devuelven errores del dominio, no
   los de la base de datos o del cliente HTTP.
8. **Caso de uso que conoce el protocolo.** Recibir o devolver tipos HTTP, de Tauri o de
   cualquier framework de entrada.

---

## 11. Rendimiento

Las capas casi no cuestan en ejecución: los casos de uso con genéricos (Rust) se resuelven
en compilación, una llamada a través de `Arc<dyn Trait>` o de una interfaz (Java) cuesta
nanosegundos, y los `Command`/`Query` son estructuras que se mueven sin coste. Frente a una
lectura de disco o una llamada de red, eso es ruido.

El coste real aparece en dos sitios, y por eso son reglas:

1. **Los mappers consumen, no clonan.** Al cruzar una frontera (dominio a DTO de transporte,
   modelo de persistencia a entidad) se convierte moviendo el valor: en Rust
   `impl From<Note> for NoteDto` que toma el valor, nunca `clone()` de colecciones o textos
   grandes; en Java, sin copias defensivas innecesarias de listas grandes. Para datos
   compartidos se usan tipos baratos de copiar (`Arc<str>`, cadenas compactas).
2. **Contratos por lotes cuando el volumen es grande.** Si una operación se repite sobre
   miles de elementos, el contrato recibe el lote (`index_documents(&[Document])`), no un
   elemento por llamada. Así el bucle caliente vive entero dentro de la implementación y no
   cruza una frontera por cada elemento.

Además:

- Los bucles críticos (escaneo, indexado, búsqueda) viven completos dentro de su `Impl`; las
  capas quedan alrededor, no dentro.
- En Rust, un método `async` en un trait usado como `dyn` asigna en el heap en cada llamada:
  aceptable en I/O, a evitar en caminos muy calientes.
- Los genéricos aumentan el tiempo de compilación y el tamaño del binario con cada
  combinación de tipos; si crece demasiado, se pasan a `dyn` los menos críticos.
- Toda migración de arquitectura se acompaña de mediciones de las operaciones críticas
  antes y después de cada etapa.

---

## 12. Guardián automático

Las reglas se verifican en la suite de pruebas.

### Java (ArchUnit)

```java
@AnalyzeClasses(packages = "com.empresa.proyecto", importOptions = ImportOption.DoNotIncludeTests.class)
class ArchitectureTest {

    @ArchTest
    static final ArchRule respect_onion_layers = layeredArchitecture()
        .consideringOnlyDependenciesInLayers()
        .layer("Domain").definedBy("..domain..")
        .layer("Application").definedBy("..application..")
        .layer("Infrastructure").definedBy("..infrastructure..")
        .whereLayer("Domain").mayNotAccessAnyLayer()
        .whereLayer("Application").mayOnlyAccessLayers("Domain")
        .whereLayer("Infrastructure").mayOnlyAccessLayers("Application", "Domain");

    @ArchTest
    static final ArchRule inner_layers_are_framework_free = noClasses()
        .that().resideInAnyPackage("..domain..", "..application..")
        .should().dependOnClassesThat().resideInAnyPackage(
            "org.springframework..", "jakarta.persistence..", "javax.persistence..",
            "com.fasterxml.jackson..", "org.apache.kafka..");

    @ArchTest
    static final ArchRule implementations_end_with_impl = classes()
        .that().resideInAnyPackage("..infrastructure.repository..", "..infrastructure.publisher..",
                                   "..infrastructure.integration..")
        .and().areNotInterfaces()
        .and().haveSimpleNameNotContaining("Mapper")
        .and().haveSimpleNameNotContaining("Entity")
        .should().haveSimpleNameEndingWith("Impl");
}
```

### Rust (`tests/architecture.rs`)

Rust no tiene una herramienta estándar equivalente, así que el guardián revisa las
importaciones de cada capa:

```rust
fn offenders(layer: &str, forbidden: &[&str], allowed: &[&str]) -> Vec<String> { /* ... */ }

#[test]
fn domain_knows_nothing_outside() {
    let bad = offenders(
        "domain",
        &["crate::application", "crate::infrastructure", "tauri", "serde",
          "std::fs", "std::process", "tantivy", "reqwest"],
        &[],
    );
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn application_knows_only_domain() {
    let bad = offenders(
        "application",
        &["crate::infrastructure", "tauri", "serde", "std::fs", "std::process", "reqwest"],
        &[], // cada excepción lleva un comentario con el motivo y cuándo se elimina
    );
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
```
