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
