use crate::application::state::AdministrationState;
use crate::domain::organization::Organization;
use crate::domain::practitioner::Practitioner;
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::{ContactPoint, HumanName, Person};
use async_trait::async_trait;
use thiserror::Error;
use tracing::info;
use uuid::{Uuid, Version};

/// Colegiatura provisional cuando el propietario aún no declara la suya.
///
/// El número real se registra más adelante, al activar su perfil profesional.
const PENDING_MEDICAL_LICENSE: &str = "CMP-PENDIENTE";

/// Crea una clínica y deja a quien la solicita como su propietario.
pub trait CreateClinicUseCase:
    UseCase<Command = CreateClinicCommand, Response = CreateClinicResponse, Error = CreateClinicError>
{
}

/// Datos de entrada, planos, tal como llegan desde la API.
///
/// La demografía del propietario viaja aquí porque este contexto acotado **no
/// consulta** al de identidad: construye su propia réplica local del profesional.
#[derive(Debug, Clone)]
pub struct CreateClinicCommand {
    pub owner_user_id: String,
    pub name: String,
    pub tax_id: Option<String>,
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub medical_license_number: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateClinicResponse {
    pub organization_id: Uuid,
    pub practitioner_id: Uuid,
    /// La clínica ya existía y se devolvió sin recrearla.
    pub already_existed: bool,
}

#[derive(Debug, Error)]
pub enum CreateClinicError {
    #[error("El identificador del propietario debe ser un UUID v7: {0}")]
    InvalidOwnerUserId(String),

    #[error("El nombre de la clínica no puede estar vacío")]
    EmptyName,

    #[error("La ficha de profesional del propietario no pudo resolverse")]
    MissingPractitioner,

    #[error(transparent)]
    Unknown(#[from] ClickCareError),
}

pub(crate) struct CreateClinicUseCaseImpl {
    pub(crate) state: AdministrationState,
}

impl CreateClinicUseCase for CreateClinicUseCaseImpl {}

#[async_trait]
impl UseCase for CreateClinicUseCaseImpl {
    type Command = CreateClinicCommand;
    type Response = CreateClinicResponse;
    type Error = CreateClinicError;

    /// Flujo: crear la `Organization`, dejar al solicitante como propietario y
    /// materializar su `Practitioner` en esa misma clínica.
    ///
    /// **Idempotente**: si el usuario ya posee una clínica se devuelve la existente
    /// en lugar de crear una segunda. Que un usuario pueda tener varias clínicas es
    /// una decisión aparte; hoy la base lo impide con `UNIQUE (owner_user_id)`.
    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {
        let owner_user_id = parse_owner_user_id(&command.owner_user_id)?;

        let clinic_name = command.name.trim().to_string();
        if clinic_name.is_empty() {
            return Err(CreateClinicError::EmptyName);
        }

        if let Some(organization_id) = self
            .state
            .organization_repository
            .find_id_by_owner_user_id(&owner_user_id)
            .await?
        {
            info!(
                "El usuario user_id={owner_user_id} ya posee la clínica {organization_id}; \
                 se devuelve sin recrearla"
            );
            let practitioner_id = self
                .ensure_practitioner(&organization_id, &owner_user_id, &command)
                .await?;

            return Ok(CreateClinicResponse {
                organization_id,
                practitioner_id,
                already_existed: true,
            });
        }

        let organization = Organization::new(
            Uuid::now_v7(),
            clinic_name,
            command.tax_id.clone(),
            owner_user_id,
        );
        self.state
            .organization_repository
            .save(&organization)
            .await?;
        info!(
            "Clínica creada: id={} para el propietario user_id={owner_user_id}",
            organization.id
        );

        let practitioner_id = self
            .ensure_practitioner(&organization.id, &owner_user_id, &command)
            .await?;

        Ok(CreateClinicResponse {
            organization_id: organization.id,
            practitioner_id,
            already_existed: false,
        })
    }
}

impl CreateClinicUseCaseImpl {
    /// Materializa la ficha del propietario en esa clínica, salvo que ya exista.
    async fn ensure_practitioner(
        &self,
        organization_id: &Uuid,
        owner_user_id: &Uuid,
        command: &CreateClinicCommand,
    ) -> Result<Uuid, CreateClinicError> {
        if let Some(practitioner_id) = self
            .state
            .practitioner_repository
            .find_id_by_user_id(organization_id, owner_user_id)
            .await?
        {
            return Ok(practitioner_id);
        }

        let practitioner = Practitioner::new(
            Uuid::now_v7(),
            *organization_id,
            *owner_user_id,
            command
                .medical_license_number
                .clone()
                .unwrap_or_else(|| PENDING_MEDICAL_LICENSE.to_string()),
            build_owner_person(owner_user_id, command),
        );

        self.state
            .practitioner_repository
            .save(&practitioner)
            .await?;
        info!(
            "Ficha de profesional creada: id={} en la clínica {organization_id}",
            practitioner.id
        );

        Ok(practitioner.id)
    }
}

/// Valida que el identificador del propietario sea un UUID v7.
fn parse_owner_user_id(raw: &str) -> Result<Uuid, CreateClinicError> {
    match Uuid::parse_str(raw) {
        Ok(uuid) if uuid.get_version() == Some(Version::SortRand) => Ok(uuid),
        _ => Err(CreateClinicError::InvalidOwnerUserId(raw.to_string())),
    }
}

/// Mapea los campos planos del comando al recurso FHIR `Person` del dominio.
///
/// Nunca se filtra la estructura plana del DTO dentro de la entidad: entra al
/// dominio ya convertida en Value Objects.
fn build_owner_person(owner_user_id: &Uuid, command: &CreateClinicCommand) -> Person {
    let name = HumanName::new(
        vec![command.given_name.clone()],
        command.family_name.clone(),
        command.second_family_name.clone(),
    );

    let mut telecom = Vec::new();
    if let Some(email) = &command.email {
        telecom.push(ContactPoint::email(email.clone()));
    }
    if let Some(phone) = &command.phone {
        telecom.push(ContactPoint::phone(phone.clone(), None));
    }

    Person::new(*owner_user_id, name, telecom, None, None)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::patient::Patient;
    use crate::domain::repository::organization_repository::OrganizationRepository;
    use crate::domain::repository::patient_repository::PatientRepository;
    use crate::domain::repository::practitioner_repository::PractitionerRepository;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Repositorio en memoria que registra lo guardado y puede simular preexistencia.
    #[derive(Default)]
    struct SpyRepository {
        existing_id: Option<Uuid>,
        saved_organizations: Mutex<Vec<Organization>>,
        saved_practitioners: Mutex<Vec<Practitioner>>,
    }

    impl SpyRepository {
        fn empty() -> Arc<Self> {
            Arc::new(Self::default())
        }

        fn holding(existing_id: Uuid) -> Arc<Self> {
            Arc::new(Self {
                existing_id: Some(existing_id),
                ..Default::default()
            })
        }
    }

    #[async_trait]
    impl OrganizationRepository for SpyRepository {
        async fn find_id_by_owner_user_id(
            &self,
            _owner_user_id: &Uuid,
        ) -> Result<Option<Uuid>, ClickCareError> {
            Ok(self.existing_id)
        }

        async fn save(&self, organization: &Organization) -> Result<(), ClickCareError> {
            self.saved_organizations
                .lock()
                .expect("El mutex del espía no debió envenenarse")
                .push(organization.clone());
            Ok(())
        }
    }

    #[async_trait]
    impl PractitionerRepository for SpyRepository {
        async fn find_id_by_user_id(
            &self,
            _organization_id: &Uuid,
            _user_id: &Uuid,
        ) -> Result<Option<Uuid>, ClickCareError> {
            Ok(self.existing_id)
        }

        async fn save(&self, practitioner: &Practitioner) -> Result<(), ClickCareError> {
            self.saved_practitioners
                .lock()
                .expect("El mutex del espía no debió envenenarse")
                .push(practitioner.clone());
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
            Ok(false)
        }

        async fn save(&self, _patient: &Patient) -> Result<(), ClickCareError> {
            Ok(())
        }
    }

    fn use_case_with(
        organization: Arc<SpyRepository>,
        practitioner: Arc<SpyRepository>,
    ) -> CreateClinicUseCaseImpl {
        CreateClinicUseCaseImpl {
            state: AdministrationState {
                organization_repository: organization,
                patient_repository: SpyRepository::empty(),
                practitioner_repository: practitioner,
            },
        }
    }

    fn command_for(owner_user_id: &str) -> CreateClinicCommand {
        CreateClinicCommand {
            owner_user_id: owner_user_id.to_string(),
            name: "Clínica San Borja".to_string(),
            tax_id: Some("20512345678".to_string()),
            given_name: "Ana".to_string(),
            family_name: Some("Ramírez".to_string()),
            second_family_name: Some("Salazar".to_string()),
            email: Some("ana@example.com".to_string()),
            phone: Some("+51999888777".to_string()),
            medical_license_number: None,
        }
    }

    #[tokio::test]
    async fn creates_the_clinic_and_the_owner_practitioner() {
        let organization = SpyRepository::empty();
        let practitioner = SpyRepository::empty();
        let owner_user_id = Uuid::now_v7();

        let response = use_case_with(Arc::clone(&organization), Arc::clone(&practitioner))
            .execute(command_for(&owner_user_id.to_string()))
            .await
            .expect("La creación de la clínica no debió fallar");

        assert!(!response.already_existed);

        let clinics = organization.saved_organizations.lock().unwrap();
        assert_eq!(clinics.len(), 1, "Debió crear una clínica");
        assert_eq!(clinics[0].owner_user_id, owner_user_id);
        assert_eq!(clinics[0].tax_id.as_deref(), Some("20512345678"));

        let fichas = practitioner.saved_practitioners.lock().unwrap();
        assert_eq!(fichas.len(), 1, "Debió crear la ficha del propietario");
        assert_eq!(
            fichas[0].organization_id, response.organization_id,
            "La ficha debe colgar de la clínica recién creada"
        );
        assert_eq!(
            fichas[0].medical_license_number, PENDING_MEDICAL_LICENSE,
            "Sin colegiatura declarada debe quedar la provisional"
        );
    }

    /// Reintentar no debe producir una segunda clínica: se devuelve la existente.
    #[tokio::test]
    async fn returns_the_existing_clinic_without_creating_another() {
        let existing_id = Uuid::now_v7();
        let organization = SpyRepository::holding(existing_id);
        let practitioner = SpyRepository::holding(existing_id);

        let response = use_case_with(Arc::clone(&organization), Arc::clone(&practitioner))
            .execute(command_for(&Uuid::now_v7().to_string()))
            .await
            .expect("La creación de la clínica no debió fallar");

        assert!(response.already_existed);
        assert_eq!(response.organization_id, existing_id);
        assert!(organization.saved_organizations.lock().unwrap().is_empty());
        assert!(practitioner.saved_practitioners.lock().unwrap().is_empty());
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::not_a_uuid("no-soy-un-uuid")]
    #[case::uuid_v4("f47ac10b-58cc-4372-a567-0e02b2c3d479")]
    #[tokio::test]
    async fn rejects_an_owner_user_id_that_is_not_uuid_v7(#[case] owner_user_id: &str) {
        let result = use_case_with(SpyRepository::empty(), SpyRepository::empty())
            .execute(command_for(owner_user_id))
            .await;

        assert!(
            matches!(result, Err(CreateClinicError::InvalidOwnerUserId(_))),
            "Debió rechazar el identificador {owner_user_id}"
        );
    }

    #[tokio::test]
    async fn rejects_a_clinic_name_that_is_only_whitespace() {
        let mut command = command_for(&Uuid::now_v7().to_string());
        command.name = "   ".to_string();

        let result = use_case_with(SpyRepository::empty(), SpyRepository::empty())
            .execute(command)
            .await;

        assert!(matches!(result, Err(CreateClinicError::EmptyName)));
    }
}
