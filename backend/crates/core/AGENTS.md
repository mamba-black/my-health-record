# Núcleo Compartido (`crates/core`, alias `app_core`)

## Especificación

* **Responsabilidad**: Contratos y Value Objects transversales e inmutables que todos los contextos acotados comparten. **No es un Bounded Context**: no tiene entidades propias ni persistencia.
* **Capa**: Core — la capa más interna de la Arquitectura Cebolla. **No depende de ninguna otra crate del workspace.**
* **Módulos (`src/`)**:

| Ruta | Contenido |
|---|---|
| `application/mod.rs` | Trait base `UseCase` (`Command`, `Response`, `Error`) que implementan todos los casos de uso. |
| `domain/error.rs` | `ClickCareError`, el error transversal del sistema. |
| `domain/event.rs` | `DomainEvent`, el puerto `EventPublisher`, `UserCreatedEvent` y `LoggingEventPublisher`. |
| `domain/fhir/` | Value Objects FHIR compartidos: `person.rs`, `human_name.rs`, `contact_point.rs`, `identifier.rs`. |
| `domain/repository/emitter.rs` | Puerto `Emitter` para difusión in-process. |

---

## Reglas de Dominio

* **Dependencia cero**: `crates/core` no importa de ningún otro crate del workspace, ni de librerías de infraestructura (bases de datos, colas, gRPC). Si algo aquí necesita `sqlx`, `apalis` o `tonic`, está en la capa equivocada.
* **Los Value Objects FHIR viven aquí, no en los contextos**: `HumanName`, `ContactPoint`, `Identifier`, `Address`, `Attachment` y `Person` se definen una sola vez en `app_core::domain::fhir` para que todos los contextos compartan la misma representación. Está prohibido que un contexto acotado redefina o duplique uno de estos tipos.
* **Los puertos se definen aquí, los adaptadores no**: `EventPublisher` es un trait de este crate; `ApalisEventPublisher` es su adaptador y vive en la infraestructura del crate productor (`crates/user`). El núcleo declara el contrato, nunca la implementación de infraestructura.
* **El nombre de la cola pertenece al evento**: cada evento de dominio ancla su cola como constante asociada (p. ej. `UserCreatedEvent::QUEUE`). Productor y consumidor referencian esa constante, nunca un literal suelto, de modo que el compilador garantice que ambos hablan de la misma cola.
* **Smart Constructors obligatorios**: los Value Objects mantienen los campos privados y calculan sus derivados en `new()` (p. ej. `HumanName.text`). Ver la regla del Patrón Builder Seguro en [AGENTS.md](../../AGENTS.md) §3.8.
