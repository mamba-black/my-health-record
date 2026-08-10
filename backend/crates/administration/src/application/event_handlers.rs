use crate::domain::organization::Organization;
use crate::domain::patient::Patient;
use crate::domain::practitioner::Practitioner;
use app_core::domain::error::ClickCareError;
use app_core::domain::event::UserCreatedEvent;
use tracing::info;
use uuid::Uuid;

/// Handler del evento `UserCreatedEvent` ejecutado por Apalis Worker en segundo plano.
///
/// Aplica las reglas del Bounded Context `administration`:
/// 1. Si `create_clinic == true`, inicializa la `Organization` (clínica) y el `Practitioner` (médico).
/// 2. Inicializa el expediente de `Patient` asociando la entidad FHIR `Person`.
pub async fn handle_user_created_event(event: UserCreatedEvent) -> Result<(), ClickCareError> {
    info!(
        "Apalis Worker [administration]: Procesando UserCreatedEvent para user_id={}",
        event.user_id
    );

    if event.create_clinic {
        let clinic_id = Uuid::now_v7();
        let clinic_name = format!("Clínica de {}", event.person.name().text());
        let organization = Organization::new(clinic_id, clinic_name, event.user_id);
        info!(
            "Apalis Worker [administration]: Organización de clínica creada: {:?}",
            organization
        );

        let practitioner_id = Uuid::now_v7();
        let practitioner = Practitioner::new(
            practitioner_id,
            event.user_id,
            "CMP-PENDIENTE".to_string(), // Colegiatura inicial o por verificar
            event.person.clone(),
        );
        info!(
            "Apalis Worker [administration]: Perfil Practitioner creado: {:?}",
            practitioner
        );
    }

    let patient_id = Uuid::now_v7();
    let patient = Patient::new(patient_id, event.user_id, event.person);

    info!(
        "Apalis Worker [administration]: Expediente Patient creado con la entidad Person: {:?}",
        patient
    );

    Ok(())
}
