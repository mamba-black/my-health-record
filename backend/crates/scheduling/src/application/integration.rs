//! Puertos que la agenda usa para hablar con otros contextos y con el exterior. Los
//! implementa `bin/clickcare` con adaptadores sobre `administration` y `user`, así
//! este crate no depende de ellos.

use crate::domain::calendar::{Instant, Zone};
use crate::domain::error::SchedulingError;
use crate::domain::reminder::ReminderMessage;
use app_core::domain::access::ClinicStatus;
use async_trait::async_trait;
use uuid::Uuid;

/// Clínica tal como la ve la agenda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClinicInfo {
    pub id: Uuid,
    /// Organization raíz de la clínica (la red).
    pub network_id: Uuid,
    pub subdomain: String,
    pub name: String,
    pub status: ClinicStatus,
    pub zone: Zone,
    pub address: Option<String>,
}

/// Médico activo de una clínica, con lo único que se muestra al público.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PractitionerInfo {
    pub user_id: Uuid,
    pub name: String,
    pub specialty: Option<String>,
}

#[async_trait]
pub trait ClinicDirectory: Send + Sync {
    async fn by_subdomain(&self, subdomain: &str) -> Result<Option<ClinicInfo>, SchedulingError>;

    async fn by_id(&self, organization_id: &Uuid) -> Result<Option<ClinicInfo>, SchedulingError>;

    /// Médicos con membresía activa en la clínica.
    async fn active_practitioners(
        &self,
        organization_id: &Uuid,
    ) -> Result<Vec<PractitionerInfo>, SchedulingError>;

    /// Nombre para mostrar de una cuenta.
    async fn person_name(&self, user_id: &Uuid) -> Result<Option<String>, SchedulingError>;
}

/// Registro de un paciente en una clínica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatientRef {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Cuenta propia del registro; `None` si recepción lo registró sin cuenta.
    pub user_id: Option<Uuid>,
    pub name: String,
}

/// Datos de un paciente nuevo que escribe recepción.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewPatientData {
    pub given_name: String,
    pub family_name: String,
    pub second_family_name: Option<String>,
    /// `DNI`, `CE` o `PASSPORT`.
    pub document_type: String,
    pub document_value: String,
    pub phone: String,
    pub birth_date: Option<String>,
}

/// Cuenta sugerida para vincular a un registro sin cuenta (mismo documento, misma red).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSuggestion {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait PatientRegistry: Send + Sync {
    /// Registro de la cuenta en la clínica; lo crea con los datos de la cuenta si es
    /// su primera cita allí.
    async fn ensure_for_account(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<PatientRef, SchedulingError>;

    /// Registra un paciente sin cuenta. Responde
    /// [`SchedulingError::MissingPatientData`] con el dato que falta.
    async fn register(
        &self,
        organization_id: &Uuid,
        data: NewPatientData,
    ) -> Result<PatientRef, SchedulingError>;

    /// Registro de la clínica, o `None` si no es de ella.
    async fn find(
        &self,
        organization_id: &Uuid,
        patient_id: &Uuid,
    ) -> Result<Option<PatientRef>, SchedulingError>;

    /// Registros de la cuenta en la red: los suyos y los vinculados activos.
    async fn records_of_account(
        &self,
        network_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<Uuid>, SchedulingError>;

    /// Cuenta del registro: la propia o la de su vínculo activo.
    async fn account_of(&self, patient_id: &Uuid) -> Result<Option<Uuid>, SchedulingError>;

    /// Registra en el paciente de la clínica que recepción verificó su documento físico.
    async fn mark_document_verified(
        &self,
        patient_id: &Uuid,
        verified_by: &Uuid,
    ) -> Result<(), SchedulingError>;

    /// Cuentas de la misma red con el mismo documento que un registro sin cuenta.
    async fn suggest_links(
        &self,
        network_id: &Uuid,
        organization_id: &Uuid,
        patient_id: &Uuid,
    ) -> Result<Vec<LinkSuggestion>, SchedulingError>;

    /// Vincula el registro sin cuenta con la cuenta. Rechaza documentos distintos.
    async fn link(
        &self,
        network_id: &Uuid,
        organization_id: &Uuid,
        patient_id: &Uuid,
        user_id: &Uuid,
        linked_by: &Uuid,
    ) -> Result<Uuid, SchedulingError>;

    /// Cierra un vínculo de la clínica, con su motivo. Devuelve la cuenta desvinculada.
    async fn unlink(
        &self,
        organization_id: &Uuid,
        link_id: &Uuid,
        unlinked_by: &Uuid,
        reason: &str,
    ) -> Result<Uuid, SchedulingError>;
}

#[async_trait]
pub trait IdentityVerification: Send + Sync {
    /// Marca verificado el documento de la cuenta (ya no lo edita el usuario).
    async fn verify_document(&self, user_id: &Uuid) -> Result<(), SchedulingError>;

    /// Correo de la cuenta, para el recordatorio.
    async fn email_of(&self, user_id: &Uuid) -> Result<Option<String>, SchedulingError>;
}

#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Envía el recordatorio. El error es un texto para el registro, sin datos personales.
    async fn send(&self, to: &str, message: &ReminderMessage) -> Result<(), String>;
}

#[async_trait]
pub trait ReminderScheduler: Send + Sync {
    /// Despierta al worker en `due_at` para ese recordatorio.
    async fn schedule(&self, reminder_id: &Uuid, due_at: Instant) -> Result<(), SchedulingError>;
}

/// Reloj, para fijar el tiempo en las pruebas.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

/// Reloj del sistema.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}
