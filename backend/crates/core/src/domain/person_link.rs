//! Puerto de los vínculos de la persona (FHIR R4 `Person.link`).
//!
//! La identidad (`Person`) es del contexto `user`, pero otros contextos crean los
//! recursos a los que se vincula: `administration` crea la Organization raíz del
//! dueño y sus clínicas. Este puerto les permite registrar el vínculo sin depender
//! del crate `user`; `bin/clickcare` conecta la implementación.

use crate::domain::error::ClickCareError;
use crate::domain::fhir::PersonLinkTarget;
use async_trait::async_trait;
use uuid::Uuid;

/// Registro de los `Person.link` de una cuenta.
#[async_trait]
pub trait PersonLinkRegistry: Send + Sync {
    /// Vincula la persona de la cuenta con un recurso. Es idempotente: vincular dos
    /// veces el mismo recurso deja un solo vínculo.
    async fn link(&self, user_id: &Uuid, target: PersonLinkTarget) -> Result<(), ClickCareError>;

    /// Vínculos de la persona de la cuenta.
    async fn links_of(&self, user_id: &Uuid) -> Result<Vec<PersonLinkTarget>, ClickCareError>;
}
