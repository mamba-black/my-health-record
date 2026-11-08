//! Registro de pacientes desde la clínica (`identity/patient-account-linking`,
//! requisito «Paciente sin cuenta»).
//!
//! Recepción (dueño, administrador o recepcionista) registra a una persona que aún no
//! tiene cuenta. Su expediente queda sin `user_id` hasta que se vincule, en persona,
//! con la cuenta que cree después.

use crate::application::membership_usecase::{MembershipError, clinic_target};
use crate::application::state::AdministrationState;
use crate::domain::patient::Patient;
use app_core::domain::access::{ProtectedAction, RequestContext, authorize};
use app_core::domain::fhir::{ContactPoint, ContactPointUse, HumanName, Identifier, Person};
use tracing::info;
use uuid::Uuid;

/// Datos del paciente que recepción registra.
#[derive(Debug, Clone, Default)]
pub struct NewPatient {
    pub given_name: String,
    pub family_name: String,
    pub second_family_name: Option<String>,
    pub identifier: Option<Identifier>,
    pub phone: String,
    pub birth_date: Option<String>,
}

/// Casos de uso de pacientes de la clínica.
pub struct PatientUseCases {
    state: AdministrationState,
}

impl PatientUseCases {
    pub fn new(state: AdministrationState) -> Self {
        Self { state }
    }

    /// Registra en la clínica a un paciente sin cuenta.
    ///
    /// Exige nombres, apellido paterno, documento y teléfono. Rechaza el registro si
    /// la clínica ya tiene un paciente con el mismo documento, tenga o no cuenta.
    pub async fn register(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        data: NewPatient,
    ) -> Result<Patient, MembershipError> {
        let organization = self
            .state
            .organization_repository
            .find_by_id(&organization_id)
            .await?
            .ok_or_else(|| MembershipError::NotFound("La clínica no existe".to_string()))?;
        authorize(
            ProtectedAction::RegisterPatient,
            ctx,
            &clinic_target(&organization),
        )?;
        let patient = self.register_authorized(organization_id, data).await?;
        info!(
            "Paciente sin cuenta registrado: patient={} org={organization_id} por user={}",
            patient.id, ctx.user_id
        );
        Ok(patient)
    }

    /// Registra el paciente sin volver a autorizar: lo usa la agenda al reservar en su
    /// nombre, que ya autorizó `BookForPatient`.
    pub async fn register_authorized(
        &self,
        organization_id: Uuid,
        data: NewPatient,
    ) -> Result<Patient, MembershipError> {
        let given_name = data.given_name.trim().to_string();
        let family_name = data.family_name.trim().to_string();
        let phone = data.phone.trim().to_string();
        let identifier = data.identifier.filter(|id| !id.value().trim().is_empty());
        let mut missing = Vec::new();
        if given_name.is_empty() {
            missing.push("nombres");
        }
        if family_name.is_empty() {
            missing.push("apellido paterno");
        }
        if identifier.is_none() {
            missing.push("documento");
        }
        if phone.is_empty() {
            missing.push("teléfono");
        }
        let Some(identifier) = identifier.filter(|_| missing.is_empty()) else {
            return Err(MembershipError::InvalidInput(format!(
                "Faltan datos del paciente: {}",
                missing.join(", ")
            )));
        };
        if let Identifier::DNI(dni) = &identifier
            && (dni.len() != 8 || !dni.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(MembershipError::InvalidInput(
                "El DNI debe tener 8 dígitos".to_string(),
            ));
        }
        if self
            .state
            .patient_repository
            .find_by_document(&organization_id, &identifier)
            .await?
            .is_some()
        {
            return Err(MembershipError::AlreadyExists(
                "La clínica ya tiene un paciente con ese documento".to_string(),
            ));
        }

        let second_family_name = data
            .second_family_name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let birth_date = data
            .birth_date
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let person = Person::new(
            Uuid::now_v7(),
            HumanName::new(vec![given_name], Some(family_name), second_family_name),
            vec![ContactPoint::phone(phone, Some(ContactPointUse::Mobile))],
            Some(identifier),
            birth_date,
        );
        let patient = Patient::without_account(Uuid::now_v7(), organization_id, person);
        self.state.patient_repository.save(&patient).await?;
        Ok(patient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::membership::Membership;
    use crate::domain::organization::Organization;
    use crate::testing::{
        InMemoryMembershipRepository, InMemoryOrganizationRepository, InMemoryPatientRepository,
    };
    use app_core::domain::access::{ClinicStatus, MembershipRole, MembershipStatus};
    use std::sync::Arc;

    struct Fixture {
        use_cases: PatientUseCases,
        patients: Arc<InMemoryPatientRepository>,
        organization: Organization,
    }

    fn fixture() -> Fixture {
        let mut organization = Organization::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            "san-borja".to_string(),
            "Clínica San Borja".to_string(),
            None,
            Uuid::now_v7(),
        );
        organization.status = ClinicStatus::Active;
        let organizations = Arc::new(InMemoryOrganizationRepository::default());
        organizations
            .organizations
            .lock()
            .unwrap()
            .push(organization.clone());
        let memberships = Arc::new(InMemoryMembershipRepository::default());
        let patients = Arc::new(InMemoryPatientRepository::default());
        let state = AdministrationState {
            clinical_network_repository: Arc::new(crate::testing::NoopRepository),
            organization_repository: organizations,
            patient_repository: patients.clone(),
            practitioner_repository: Arc::new(crate::testing::NoopRepository),
            membership_repository: memberships,
            person_links: Arc::new(crate::testing::InMemoryPersonLinks::default()),
        };
        Fixture {
            use_cases: PatientUseCases::new(state),
            patients,
            organization,
        }
    }

    fn ctx(
        organization_id: Uuid,
        role: MembershipRole,
        status: MembershipStatus,
    ) -> RequestContext {
        RequestContext {
            user_id: Uuid::now_v7(),
            memberships: vec![
                Membership::new(organization_id, Uuid::now_v7(), role, status).view(),
            ],
            ..RequestContext::default()
        }
    }

    fn data() -> NewPatient {
        NewPatient {
            given_name: "Rosa".to_string(),
            family_name: "Quispe".to_string(),
            identifier: Some(Identifier::DNI("12345678".to_string())),
            phone: "987654321".to_string(),
            ..NewPatient::default()
        }
    }

    #[tokio::test]
    async fn recepcion_registra_paciente_que_llamo_por_telefono() {
        let f = fixture();
        let patient = f
            .use_cases
            .register(
                &ctx(
                    f.organization.id,
                    MembershipRole::Receptionist,
                    MembershipStatus::Active,
                ),
                f.organization.id,
                data(),
            )
            .await
            .expect("recepción registra al paciente");
        assert_eq!(patient.user_id, None);
        assert_eq!(patient.organization_id, f.organization.id);
        assert_eq!(f.patients.patients.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn sin_telefono_no_se_registra() {
        let f = fixture();
        let result = f
            .use_cases
            .register(
                &ctx(
                    f.organization.id,
                    MembershipRole::Admin,
                    MembershipStatus::Active,
                ),
                f.organization.id,
                NewPatient {
                    phone: " ".to_string(),
                    ..data()
                },
            )
            .await;
        assert!(matches!(result, Err(MembershipError::InvalidInput(m)) if m.contains("teléfono")));
    }

    #[tokio::test]
    async fn documento_repetido_en_la_clinica() {
        let f = fixture();
        let reception = ctx(
            f.organization.id,
            MembershipRole::Owner,
            MembershipStatus::Active,
        );
        f.use_cases
            .register(&reception, f.organization.id, data())
            .await
            .expect("primer registro");
        let result = f
            .use_cases
            .register(&reception, f.organization.id, data())
            .await;
        assert!(matches!(result, Err(MembershipError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn recepcionista_con_solicitud_pendiente_no_registra() {
        let f = fixture();
        let result = f
            .use_cases
            .register(
                &ctx(
                    f.organization.id,
                    MembershipRole::Receptionist,
                    MembershipStatus::Requested,
                ),
                f.organization.id,
                data(),
            )
            .await;
        assert!(matches!(result, Err(MembershipError::Denied(_))));
        assert!(f.patients.patients.lock().unwrap().is_empty());
    }
}
