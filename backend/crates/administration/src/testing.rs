//! Dobles de prueba en memoria del contexto de administración (solo pruebas).

use crate::domain::membership::Membership;
use crate::domain::membership::membership_repository::MembershipRepository;
use crate::domain::organization::Organization;
use crate::domain::organization::organization_repository::OrganizationRepository;
use app_core::domain::access::{MembershipRole, MembershipStatus};
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use std::sync::Mutex;
use uuid::Uuid;

/// Membresías en memoria.
#[derive(Default)]
pub struct InMemoryMembershipRepository {
    pub memberships: Mutex<Vec<Membership>>,
}

#[async_trait]
impl MembershipRepository for InMemoryMembershipRepository {
    async fn find_by_user(&self, user_id: &Uuid) -> Result<Vec<Membership>, ClickCareError> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.user_id == *user_id)
            .cloned()
            .collect())
    }

    async fn find_by_organization(
        &self,
        organization_id: &Uuid,
    ) -> Result<Vec<Membership>, ClickCareError> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.organization_id == *organization_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Membership>, ClickCareError> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.id == *id)
            .cloned())
    }

    async fn find(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        role: MembershipRole,
    ) -> Result<Option<Membership>, ClickCareError> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .find(|m| {
                m.organization_id == *organization_id && m.user_id == *user_id && m.role == role
            })
            .cloned())
    }

    async fn count_active_owners(&self, organization_id: &Uuid) -> Result<usize, ClickCareError> {
        Ok(self
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|m| {
                m.organization_id == *organization_id
                    && m.role == MembershipRole::Owner
                    && m.status == MembershipStatus::Active
            })
            .count())
    }

    async fn insert(&self, membership: &Membership) -> Result<(), ClickCareError> {
        self.memberships.lock().unwrap().push(membership.clone());
        Ok(())
    }

    async fn update(
        &self,
        membership: &Membership,
        _confirmed_by: Option<Uuid>,
    ) -> Result<(), ClickCareError> {
        let mut memberships = self.memberships.lock().unwrap();
        if let Some(stored) = memberships.iter_mut().find(|m| m.id == membership.id) {
            *stored = membership.clone();
        }
        Ok(())
    }
}

/// Organizaciones en memoria.
#[derive(Default)]
pub struct InMemoryOrganizationRepository {
    pub organizations: Mutex<Vec<Organization>>,
}

#[async_trait]
impl OrganizationRepository for InMemoryOrganizationRepository {
    async fn find_id_by_owner_user_id(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError> {
        Ok(self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.owner_user_id == *owner_user_id)
            .map(|o| o.id))
    }

    async fn find_organization_and_network_by_subdomain(
        &self,
        subdomain: &str,
    ) -> Result<Option<(Uuid, Uuid)>, ClickCareError> {
        Ok(self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.subdomain == subdomain)
            .map(|o| (o.id, o.network_id)))
    }

    async fn save(&self, organization: &Organization) -> Result<(), ClickCareError> {
        self.organizations
            .lock()
            .unwrap()
            .push(organization.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Organization>, ClickCareError> {
        Ok(self
            .organizations
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.id == *id)
            .cloned())
    }

    async fn update_setup(&self, organization: &Organization) -> Result<(), ClickCareError> {
        let mut organizations = self.organizations.lock().unwrap();
        if let Some(stored) = organizations.iter_mut().find(|o| o.id == organization.id) {
            *stored = organization.clone();
        }
        Ok(())
    }
}

/// Repositorio que no hace nada, para los puertos que una prueba no usa.
pub struct NoopRepository;

#[async_trait::async_trait]
impl crate::domain::clinical_network::clinical_network_repository::ClinicalNetworkRepository
    for NoopRepository
{
    async fn find_by_id(
        &self,
        _id: &Uuid,
    ) -> Result<Option<crate::domain::clinical_network::ClinicalNetwork>, ClickCareError> {
        Ok(None)
    }
    async fn find_by_owner(
        &self,
        _owner_user_id: &Uuid,
    ) -> Result<Option<crate::domain::clinical_network::ClinicalNetwork>, ClickCareError> {
        Ok(None)
    }
    async fn save(
        &self,
        _network: &crate::domain::clinical_network::ClinicalNetwork,
    ) -> Result<(), ClickCareError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::domain::patient::patient_repository::PatientRepository for NoopRepository {
    async fn exists_by_user_id(&self, _o: &Uuid, _u: &Uuid) -> Result<bool, ClickCareError> {
        Ok(false)
    }
    async fn save(&self, _patient: &crate::domain::patient::Patient) -> Result<(), ClickCareError> {
        Ok(())
    }
    async fn search(
        &self,
        _o: &Uuid,
        _n: &str,
    ) -> Result<Vec<crate::domain::patient::Patient>, ClickCareError> {
        Ok(vec![])
    }
    async fn find_by_id(
        &self,
        _o: &Uuid,
        _i: &Uuid,
    ) -> Result<Option<crate::domain::patient::Patient>, ClickCareError> {
        Ok(None)
    }
    async fn find_by_document(
        &self,
        _o: &Uuid,
        _i: &app_core::domain::fhir::Identifier,
    ) -> Result<Option<crate::domain::patient::Patient>, ClickCareError> {
        Ok(None)
    }
}

#[async_trait::async_trait]
impl crate::domain::practitioner::practitioner_repository::PractitionerRepository
    for NoopRepository
{
    async fn find_id_by_user_id(
        &self,
        _o: &Uuid,
        _u: &Uuid,
    ) -> Result<Option<Uuid>, ClickCareError> {
        Ok(None)
    }
    async fn save(
        &self,
        _p: &crate::domain::practitioner::Practitioner,
    ) -> Result<(), ClickCareError> {
        Ok(())
    }
}

/// Expedientes de pacientes en memoria.
#[derive(Default)]
pub struct InMemoryPatientRepository {
    pub patients: Mutex<Vec<crate::domain::patient::Patient>>,
}

#[async_trait]
impl crate::domain::patient::patient_repository::PatientRepository for InMemoryPatientRepository {
    async fn exists_by_user_id(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, ClickCareError> {
        Ok(self
            .patients
            .lock()
            .unwrap()
            .iter()
            .any(|p| p.organization_id == *organization_id && p.user_id == Some(*user_id)))
    }

    async fn save(&self, patient: &crate::domain::patient::Patient) -> Result<(), ClickCareError> {
        self.patients.lock().unwrap().push(patient.clone());
        Ok(())
    }

    async fn search(
        &self,
        organization_id: &Uuid,
        _name: &str,
    ) -> Result<Vec<crate::domain::patient::Patient>, ClickCareError> {
        Ok(self
            .patients
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.organization_id == *organization_id)
            .cloned()
            .collect())
    }

    async fn find_by_document(
        &self,
        organization_id: &Uuid,
        identifier: &app_core::domain::fhir::Identifier,
    ) -> Result<Option<crate::domain::patient::Patient>, ClickCareError> {
        Ok(self
            .patients
            .lock()
            .unwrap()
            .iter()
            .find(|p| {
                p.organization_id == *organization_id
                    && p.person.identifier().as_ref() == Some(identifier)
            })
            .cloned())
    }

    async fn find_by_id(
        &self,
        organization_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<crate::domain::patient::Patient>, ClickCareError> {
        Ok(self
            .patients
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.organization_id == *organization_id && p.id == *id)
            .cloned())
    }
}

/// Organizations raíz en memoria, con el índice único por dueño.
#[derive(Default)]
pub struct InMemoryNetworkRepository {
    pub networks: Mutex<Vec<crate::domain::clinical_network::ClinicalNetwork>>,
}

#[async_trait]
impl crate::domain::clinical_network::clinical_network_repository::ClinicalNetworkRepository
    for InMemoryNetworkRepository
{
    async fn find_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<crate::domain::clinical_network::ClinicalNetwork>, ClickCareError> {
        Ok(self
            .networks
            .lock()
            .unwrap()
            .iter()
            .find(|n| n.id == *id)
            .cloned())
    }

    async fn find_by_owner(
        &self,
        owner_user_id: &Uuid,
    ) -> Result<Option<crate::domain::clinical_network::ClinicalNetwork>, ClickCareError> {
        Ok(self
            .networks
            .lock()
            .unwrap()
            .iter()
            .find(|n| n.owner_user_id == Some(*owner_user_id))
            .cloned())
    }

    async fn save(
        &self,
        network: &crate::domain::clinical_network::ClinicalNetwork,
    ) -> Result<(), ClickCareError> {
        let mut networks = self.networks.lock().unwrap();
        if network.owner_user_id.is_some()
            && networks
                .iter()
                .any(|n| n.owner_user_id == network.owner_user_id)
        {
            return Err(ClickCareError::generic("El dueño ya tiene una raíz"));
        }
        networks.push(network.clone());
        Ok(())
    }
}

/// Vínculos de la persona en memoria.
#[derive(Default)]
pub struct InMemoryPersonLinks {
    pub links: Mutex<Vec<(Uuid, app_core::domain::fhir::PersonLinkTarget)>>,
}

#[async_trait]
impl app_core::domain::person_link::PersonLinkRegistry for InMemoryPersonLinks {
    async fn link(
        &self,
        user_id: &Uuid,
        target: app_core::domain::fhir::PersonLinkTarget,
    ) -> Result<(), ClickCareError> {
        let mut links = self.links.lock().unwrap();
        if !links.iter().any(|(u, t)| u == user_id && *t == target) {
            links.push((*user_id, target));
        }
        Ok(())
    }

    async fn links_of(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<app_core::domain::fhir::PersonLinkTarget>, ClickCareError> {
        Ok(self
            .links
            .lock()
            .unwrap()
            .iter()
            .filter(|(u, _)| u == user_id)
            .map(|(_, t)| t.clone())
            .collect())
    }
}
