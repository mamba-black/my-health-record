use crate::domain::patient::Patient;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de repositorio de dominio para la persistencia del expediente `Patient`.
///
/// Define el contrato que la capa de infraestructura debe implementar.
#[async_trait]
pub trait PatientRepository: Send + Sync {
    /// Indica si el usuario indicado ya tiene un expediente **en esa clínica**.
    ///
    /// La consulta se acota siempre por `organization_id`: un expediente en otra
    /// clínica no debe impedir que se cree el de ésta. Sin ese filtro, la primera
    /// clínica en atender a una persona bloquearía a todas las demás.
    ///
    /// Existe para que el consumo del evento sea idempotente: la entrega es
    /// *at-least-once*, de modo que el mismo `UserCreatedEvent` puede llegar más
    /// de una vez y no debe producir expedientes duplicados.
    async fn exists_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, ClickCareError>;

    /// Persiste el expediente del paciente junto a su recurso FHIR `Person`.
    async fn save(&self, patient: &Patient) -> Result<(), ClickCareError>;

    /// Expedientes activos de **una** clínica cuyo nombre contiene el texto (vacío: todos).
    async fn search(
        &self,
        organization_id: &Uuid,
        name: &str,
    ) -> Result<Vec<Patient>, ClickCareError>;

    /// Expediente por su id, solo si pertenece a esa clínica.
    async fn find_by_id(
        &self,
        organization_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<Patient>, ClickCareError>;
}
