use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use uuid::Uuid;

/// Vínculo de un registro sin cuenta con la cuenta que el paciente creó después.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatientLink {
    pub id: Uuid,
    pub network_id: Uuid,
    pub organization_id: Uuid,
    pub patient_id: Uuid,
    pub user_id: Uuid,
    pub linked_by: Uuid,
}

/// Registros de paciente de una cuenta y sus vínculos (nunca cruza redes).
#[async_trait]
pub trait PatientLinkRepository: Send + Sync {
    /// Registro propio de la cuenta en la clínica.
    async fn record_of_account(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError>;

    /// Registros de la cuenta en la red: los suyos y los vinculados activos.
    async fn records_of_account(
        &self,
        network_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<Uuid>, ClickCareError>;

    /// Cuenta del registro: la propia o la de su vínculo activo.
    async fn account_of(&self, patient_id: &Uuid) -> Result<Option<Uuid>, ClickCareError>;

    /// Vínculo activo del registro.
    async fn active_link(&self, patient_id: &Uuid) -> Result<Option<PatientLink>, ClickCareError>;

    async fn insert(&self, link: &PatientLink) -> Result<(), ClickCareError>;

    /// Cierra el vínculo activo de la clínica. Devuelve la cuenta, o `None` si no existe.
    async fn close(
        &self,
        organization_id: &Uuid,
        link_id: &Uuid,
        unlinked_by: &Uuid,
        reason: &str,
    ) -> Result<Option<Uuid>, ClickCareError>;

    /// Registra en el paciente que su documento se verificó en persona.
    async fn mark_document_verified(
        &self,
        patient_id: &Uuid,
        verified_by: &Uuid,
    ) -> Result<(), ClickCareError>;
}
