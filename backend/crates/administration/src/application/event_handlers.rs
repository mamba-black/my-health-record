use crate::application::state::AdministrationState;
use crate::domain::organization::Organization;
use crate::domain::patient::Patient;
use crate::domain::practitioner::Practitioner;
use app_core::domain::error::ClickCareError;
use app_core::domain::event::UserCreatedEvent;
use tracing::info;
use uuid::Uuid;

/// Colegiatura provisional asignada al crear la ficha del profesional.
///
/// El número real se registra más adelante, cuando el médico activa su perfil
/// y verifica su identidad (ver el registro progresivo en `crates/user`).
const PENDING_MEDICAL_LICENSE: &str = "CMP-PENDIENTE";

/// Handler del evento `UserCreatedEvent`, ejecutado en segundo plano.
///
/// Aplica las reglas del Bounded Context `administration`:
/// 1. Si `create_clinic == false`, **no materializa nada**. Toda entidad local de
///    este contexto pertenece a una clínica, y un usuario que no crea ninguna no
///    tiene todavía a cuál pertenecer.
/// 2. Si `create_clinic == true`, inicializa la `Organization`, la ficha
///    `Practitioner` de su dueño y su expediente `Patient`, los tres colgados de
///    esa clínica.
///
/// Es una función de aplicación pura: no conoce Apalis ni ningún tipo de
/// infraestructura. Quien la registra en el worker es `infrastructure/di.rs`.
///
/// **Idempotente por diseño**: la entrega de la cola es *at-least-once*, así que
/// el mismo evento puede llegar más de una vez. Cada entidad se crea solo si el
/// par `(clínica, usuario)` no la tiene ya.
pub async fn handle_user_created_event(
    event: UserCreatedEvent,
    state: &AdministrationState,
) -> Result<(), ClickCareError> {
    info!("Procesando UserCreatedEvent para user_id={}", event.user_id);

    if !event.create_clinic {
        info!(
            "El usuario user_id={} no crea clínica; no hay entidades locales que materializar",
            event.user_id
        );
        return Ok(());
    }

    let organization_id = ensure_organization(&event, state).await?;
    create_practitioner_if_absent(&event, state, organization_id).await?;
    create_patient_if_absent(&event, state, organization_id).await?;

    Ok(())
}

/// Devuelve la clínica del usuario propietario, creándola si aún no existe.
///
/// Devuelve el identificador en ambos casos porque las entidades locales que se
/// materializan a continuación necesitan colgar de esa clínica.
async fn ensure_organization(
    event: &UserCreatedEvent,
    state: &AdministrationState,
) -> Result<Uuid, ClickCareError> {
    if let Some(organization_id) = state
        .organization_repository
        .find_id_by_owner_user_id(&event.user_id)
        .await?
    {
        info!(
            "El usuario user_id={} ya tiene la organización {organization_id}; se omite su creación",
            event.user_id
        );
        return Ok(organization_id);
    }

    let clinic_name = format!("Clínica de {}", event.person.name().text());
    let organization = Organization::new(Uuid::now_v7(), clinic_name, event.user_id);

    state.organization_repository.save(&organization).await?;
    info!(
        "Organización creada: id={} para user_id={}",
        organization.id, event.user_id
    );

    Ok(organization.id)
}

/// Crea la ficha del profesional en esa clínica, salvo que ya exista.
async fn create_practitioner_if_absent(
    event: &UserCreatedEvent,
    state: &AdministrationState,
    organization_id: Uuid,
) -> Result<(), ClickCareError> {
    if state
        .practitioner_repository
        .exists_by_user_id(&organization_id, &event.user_id)
        .await?
    {
        info!(
            "El usuario user_id={} ya tiene ficha de profesional en la clínica {organization_id}; se omite su creación",
            event.user_id
        );
        return Ok(());
    }

    let practitioner = Practitioner::new(
        Uuid::now_v7(),
        organization_id,
        event.user_id,
        PENDING_MEDICAL_LICENSE.to_string(),
        event.person.clone(),
    );

    state.practitioner_repository.save(&practitioner).await?;
    info!(
        "Ficha de profesional creada: id={} en la clínica {organization_id} para user_id={}",
        practitioner.id, event.user_id
    );

    Ok(())
}

/// Crea el expediente del paciente en esa clínica, salvo que ya exista.
async fn create_patient_if_absent(
    event: &UserCreatedEvent,
    state: &AdministrationState,
    organization_id: Uuid,
) -> Result<(), ClickCareError> {
    if state
        .patient_repository
        .exists_by_user_id(&organization_id, &event.user_id)
        .await?
    {
        info!(
            "El usuario user_id={} ya tiene expediente en la clínica {organization_id}; se omite su creación",
            event.user_id
        );
        return Ok(());
    }

    let patient = Patient::new(
        Uuid::now_v7(),
        organization_id,
        event.user_id,
        event.person.clone(),
    );

    state.patient_repository.save(&patient).await?;
    info!(
        "Expediente de paciente creado: id={} en la clínica {organization_id} para user_id={}",
        patient.id, event.user_id
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::repository::organization_repository::OrganizationRepository;
    use crate::domain::repository::patient_repository::PatientRepository;
    use crate::domain::repository::practitioner_repository::PractitionerRepository;
    use app_core::domain::fhir::{HumanName, Person};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Repositorio en memoria que cuenta las escrituras y simula si la entidad ya existe.
    ///
    /// Sirve para las tres entidades: al handler solo le importa el par
    /// «¿existe?» / «guardar», no el tipo concreto que persiste.
    #[derive(Default)]
    struct SpyRepository {
        already_exists: bool,
        saved: AtomicUsize,
    }

    impl SpyRepository {
        fn existing() -> Arc<Self> {
            Arc::new(Self {
                already_exists: true,
                saved: AtomicUsize::new(0),
            })
        }

        fn empty() -> Arc<Self> {
            Arc::new(Self::default())
        }

        fn saved_count(&self) -> usize {
            self.saved.load(Ordering::SeqCst)
        }

        fn record_save(&self) {
            self.saved.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Clínica que el espía reporta cuando simula que el usuario ya tiene una.
    const EXISTING_ORGANIZATION_ID: Uuid = Uuid::nil();

    #[async_trait]
    impl OrganizationRepository for SpyRepository {
        async fn find_id_by_owner_user_id(
            &self,
            _owner_user_id: &Uuid,
        ) -> Result<Option<Uuid>, ClickCareError> {
            Ok(self.already_exists.then_some(EXISTING_ORGANIZATION_ID))
        }

        async fn save(&self, _organization: &Organization) -> Result<(), ClickCareError> {
            self.record_save();
            Ok(())
        }
    }

    #[async_trait]
    impl PatientRepository for SpyRepository {
        async fn exists_by_user_id(
            &self,
            _organization_id: &Uuid,
            _user_id: &Uuid,
        ) -> Result<bool, ClickCareError> {
            Ok(self.already_exists)
        }

        async fn save(&self, _patient: &Patient) -> Result<(), ClickCareError> {
            self.record_save();
            Ok(())
        }
    }

    #[async_trait]
    impl PractitionerRepository for SpyRepository {
        async fn exists_by_user_id(
            &self,
            _organization_id: &Uuid,
            _user_id: &Uuid,
        ) -> Result<bool, ClickCareError> {
            Ok(self.already_exists)
        }

        async fn save(&self, _practitioner: &Practitioner) -> Result<(), ClickCareError> {
            self.record_save();
            Ok(())
        }
    }

    /// Arma el estado con los tres espías y lo devuelve junto a ellos para poder afirmarlos.
    fn state_with(
        organization: Arc<SpyRepository>,
        patient: Arc<SpyRepository>,
        practitioner: Arc<SpyRepository>,
    ) -> AdministrationState {
        AdministrationState {
            organization_repository: organization,
            patient_repository: patient,
            practitioner_repository: practitioner,
        }
    }

    fn event_for(create_clinic: bool) -> UserCreatedEvent {
        let name = HumanName::new(
            vec!["Ana".to_string()],
            Some("Ramírez".to_string()),
            Some("Salazar".to_string()),
        );
        let person = Person::new(Uuid::now_v7(), name, vec![], None, None);

        UserCreatedEvent {
            user_id: Uuid::now_v7(),
            person,
            create_clinic,
        }
    }

    /// Toda entidad de este contexto pertenece a una clínica. Si el usuario no crea
    /// ninguna, no hay clínica a la que colgar su expediente y no se materializa nada:
    /// su expediente aparecerá por demanda cuando una clínica lo registre.
    #[tokio::test]
    async fn creates_nothing_when_the_user_does_not_own_a_clinic() {
        let organization = SpyRepository::empty();
        let patient = SpyRepository::empty();
        let practitioner = SpyRepository::empty();

        handle_user_created_event(
            event_for(false),
            &state_with(
                Arc::clone(&organization),
                Arc::clone(&patient),
                Arc::clone(&practitioner),
            ),
        )
        .await
        .expect("El handler no debió fallar");

        assert_eq!(
            patient.saved_count(),
            0,
            "No debió crear un expediente sin clínica"
        );
        assert_eq!(
            organization.saved_count(),
            0,
            "No debió crear ninguna organización"
        );
        assert_eq!(
            practitioner.saved_count(),
            0,
            "No debió crear ninguna ficha de profesional"
        );
    }

    #[tokio::test]
    async fn creates_the_three_records_when_the_user_owns_a_clinic() {
        let organization = SpyRepository::empty();
        let patient = SpyRepository::empty();
        let practitioner = SpyRepository::empty();

        handle_user_created_event(
            event_for(true),
            &state_with(
                Arc::clone(&organization),
                Arc::clone(&patient),
                Arc::clone(&practitioner),
            ),
        )
        .await
        .expect("El handler no debió fallar");

        assert_eq!(organization.saved_count(), 1, "Debió crear la organización");
        assert_eq!(
            practitioner.saved_count(),
            1,
            "Debió crear la ficha de profesional"
        );
        assert_eq!(patient.saved_count(), 1, "Debió crear el expediente");
    }

    /// La entrega de la cola es at-least-once: reprocesar el mismo evento no debe duplicar nada.
    #[tokio::test]
    async fn does_not_write_anything_when_the_records_already_exist() {
        let organization = SpyRepository::existing();
        let patient = SpyRepository::existing();
        let practitioner = SpyRepository::existing();

        handle_user_created_event(
            event_for(true),
            &state_with(
                Arc::clone(&organization),
                Arc::clone(&patient),
                Arc::clone(&practitioner),
            ),
        )
        .await
        .expect("El handler no debió fallar");

        assert_eq!(organization.saved_count(), 0);
        assert_eq!(practitioner.saved_count(), 0);
        assert_eq!(patient.saved_count(), 0);
    }
}
