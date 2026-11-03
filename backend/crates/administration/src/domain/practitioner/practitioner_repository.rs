use crate::domain::practitioner::Practitioner;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia de la ficha `Practitioner`.
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait PractitionerRepository: Send + Sync {
    /// Devuelve el identificador de la ficha que el usuario ya tiene **en esa clínica**.
    ///
    /// La consulta se acota siempre por `organization_id`: un mismo médico puede
    /// ejercer en varias clínicas, y su ficha en una no debe impedir la de otra.
    ///
    /// Devuelve el id en lugar de un booleano porque quien crea la clínica debe
    /// responder con la ficha del propietario, exista ya o se acabe de crear.
    /// Sirve además para la idempotencia: la entrega de la cola es *at-least-once*
    /// y el mismo evento puede llegar más de una vez sin duplicar fichas.
    async fn find_id_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError>;

    /// Persiste la ficha del profesional junto a su recurso FHIR `Person`.
    async fn save(&self, practitioner: &Practitioner) -> Result<(), ClickCareError>;
}
