//! Casos de uso de membresías y de configuración y activación de la clínica.
//!
//! Cada operación arma el [`AccessTarget`] con sus propios repositorios y consulta la
//! regla de [`authorize`] antes de cambiar nada. Quien llama aporta el
//! [`RequestContext`] (identidad, datos presentes y membresías del usuario).

use crate::application::state::AdministrationState;
use crate::domain::membership::Membership;
use crate::domain::organization::Organization;
use app_core::domain::access::{
    AccessDenied, AccessTarget, ClinicStatus, MembershipRole, MembershipStatus, ProtectedAction,
    RequestContext, authorize, is_valid_ruc,
};
use app_core::domain::error::ClickCareError;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

/// Errores de los casos de uso de membresías.
#[derive(Debug, Error)]
pub enum MembershipError {
    #[error("Acceso denegado: {0:?}")]
    Denied(AccessDenied),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    AlreadyExists(String),
    #[error("{0}")]
    InvalidInput(String),
    #[error("No se puede suspender al único dueño activo de la clínica")]
    LastOwner,
    #[error(transparent)]
    Unknown(#[from] ClickCareError),
}

impl From<AccessDenied> for MembershipError {
    fn from(denied: AccessDenied) -> Self {
        MembershipError::Denied(denied)
    }
}

/// Datos legales de la clínica que se guardan mientras está pendiente de activación.
#[derive(Debug, Clone, Default)]
pub struct ClinicSetup {
    pub tax_id: Option<String>,
    pub legal_name: Option<String>,
    pub trade_name: Option<String>,
    pub ipress_code: Option<String>,
}

/// Membresía junto con la clínica a la que pertenece.
#[derive(Debug, Clone)]
pub struct MembershipWithClinic {
    pub membership: Membership,
    pub organization: Organization,
}

/// Casos de uso de membresías y de configuración de la clínica.
pub struct MembershipUseCases {
    pub(crate) state: AdministrationState,
}

impl MembershipUseCases {
    pub fn new(state: AdministrationState) -> Self {
        Self { state }
    }

    /// Membresías del usuario con su clínica, para la interfaz y para armar el contexto.
    pub async fn list_for_user(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<MembershipWithClinic>, MembershipError> {
        let memberships = self
            .state
            .membership_repository
            .find_by_user(user_id)
            .await?;
        let mut result = Vec::with_capacity(memberships.len());
        for membership in memberships {
            if let Some(organization) = self
                .state
                .organization_repository
                .find_by_id(&membership.organization_id)
                .await?
            {
                result.push(MembershipWithClinic {
                    membership,
                    organization,
                });
            }
        }
        Ok(result)
    }

    /// Estado de una clínica, si existe.
    pub async fn clinic(&self, organization_id: &Uuid) -> Result<Organization, MembershipError> {
        self.state
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| MembershipError::NotFound("La clínica no existe".to_string()))
    }

    /// Un dueño o administrador invita a un usuario ya registrado.
    pub async fn invite(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        invitee_user_id: Uuid,
        role: MembershipRole,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let organization = self.clinic(&organization_id).await?;
        authorize(
            ProtectedAction::InviteStaff,
            ctx,
            &clinic_target(&organization),
        )?;
        if role == MembershipRole::Owner {
            return Err(MembershipError::InvalidInput(
                "Solo se puede invitar como médico, administrador o recepcionista".to_string(),
            ));
        }
        if self
            .state
            .membership_repository
            .find(&organization_id, &invitee_user_id, role)
            .await?
            .is_some()
        {
            return Err(MembershipError::AlreadyExists(
                "Esa persona ya tiene ese rol en la clínica".to_string(),
            ));
        }
        let membership = Membership::new(
            organization_id,
            invitee_user_id,
            role,
            MembershipStatus::Invited,
        );
        self.state.membership_repository.insert(&membership).await?;
        info!(
            "Invitación creada: membership={} org={organization_id} rol={}",
            membership.id,
            role.code()
        );
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// El invitado acepta la invitación.
    pub async fn join(
        &self,
        ctx: &RequestContext,
        membership_id: Uuid,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let (mut membership, organization) = self.own_membership(ctx, membership_id).await?;
        authorize(
            ProtectedAction::JoinClinic,
            ctx,
            &membership_target(&membership, &organization),
        )?;
        membership.status = status_after_admission(membership.role);
        self.state
            .membership_repository
            .update(&membership, None)
            .await?;
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// El médico activa su perfil: con la colegiatura ya confirmada queda activo; si no,
    /// queda pendiente hasta que la clínica la confirme.
    pub async fn activate_practitioner(
        &self,
        ctx: &RequestContext,
        membership_id: Uuid,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let (mut membership, organization) = self.own_membership(ctx, membership_id).await?;
        authorize(
            ProtectedAction::ActivatePractitioner,
            ctx,
            &membership_target(&membership, &organization),
        )?;
        membership.activation_requested = true;
        membership.status = if membership.license_confirmed {
            MembershipStatus::Active
        } else {
            MembershipStatus::Pending
        };
        self.state
            .membership_repository
            .update(&membership, None)
            .await?;
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// Un dueño o administrador confirma la colegiatura de un médico de su clínica.
    pub async fn confirm_license(
        &self,
        ctx: &RequestContext,
        membership_id: Uuid,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let (mut membership, organization) = self.any_membership(membership_id).await?;
        authorize(
            ProtectedAction::ConfirmLicense,
            ctx,
            &clinic_target(&organization),
        )?;
        if membership.role != MembershipRole::Practitioner {
            return Err(MembershipError::InvalidInput(
                "Solo se confirma la colegiatura de un médico".to_string(),
            ));
        }
        membership.license_confirmed = true;
        if membership.activation_requested && membership.status == MembershipStatus::Pending {
            membership.status = MembershipStatus::Active;
        }
        self.state
            .membership_repository
            .update(&membership, Some(ctx.user_id))
            .await?;
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// Un dueño o administrador suspende una membresía de su clínica.
    pub async fn suspend(
        &self,
        ctx: &RequestContext,
        membership_id: Uuid,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let (mut membership, organization) = self.any_membership(membership_id).await?;
        authorize(
            ProtectedAction::SuspendMember,
            ctx,
            &clinic_target(&organization),
        )?;
        if membership.role == MembershipRole::Owner
            && membership.status == MembershipStatus::Active
            && self
                .state
                .membership_repository
                .count_active_owners(&organization.id)
                .await?
                <= 1
        {
            return Err(MembershipError::LastOwner);
        }
        membership.status = MembershipStatus::Suspended;
        self.state
            .membership_repository
            .update(&membership, None)
            .await?;
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// Una persona que no pertenece a la clínica pide entrar como médico o recepcionista.
    ///
    /// La clínica se identifica por su subdominio y debe ser de la red de quien pide:
    /// una clínica de otra red se trata como inexistente. La solicitud no concede nada
    /// hasta que el dueño o un administrador la aprueba ([`Self::approve`]).
    pub async fn request_membership(
        &self,
        ctx: &RequestContext,
        subdomain: &str,
        role: MembershipRole,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let not_found =
            || MembershipError::NotFound("No existe una clínica con ese enlace".to_string());
        let subdomain = subdomain.trim().to_lowercase();
        if subdomain.is_empty() {
            return Err(MembershipError::InvalidInput(
                "Indica el enlace de la clínica".to_string(),
            ));
        }
        let (organization_id, network_id) = self
            .state
            .organization_repository
            .find_organization_and_network_by_subdomain(&subdomain)
            .await?
            .ok_or_else(not_found)?;
        if network_id != ctx.network_id {
            return Err(not_found());
        }
        let organization = self.clinic(&organization_id).await?;
        authorize(
            ProtectedAction::RequestMembership,
            ctx,
            &clinic_target(&organization),
        )?;
        if !matches!(
            role,
            MembershipRole::Practitioner | MembershipRole::Receptionist
        ) {
            return Err(MembershipError::InvalidInput(
                "Solo se puede pedir entrar como médico o recepcionista".to_string(),
            ));
        }
        if self
            .state
            .membership_repository
            .find(&organization_id, &ctx.user_id, role)
            .await?
            .is_some()
        {
            return Err(MembershipError::AlreadyExists(
                "Ya tienes ese rol en la clínica o una solicitud en curso".to_string(),
            ));
        }
        let membership = Membership::new(
            organization_id,
            ctx.user_id,
            role,
            MembershipStatus::Requested,
        );
        self.state.membership_repository.insert(&membership).await?;
        info!(
            "Solicitud de ingreso: membership={} org={organization_id} rol={}",
            membership.id,
            role.code()
        );
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// El dueño o un administrador aprueba una solicitud de ingreso.
    ///
    /// El recepcionista queda activo. El médico queda pendiente: todavía debe activar
    /// su perfil y la clínica confirmar su colegiatura, igual que si lo hubieran invitado.
    pub async fn approve(
        &self,
        ctx: &RequestContext,
        membership_id: Uuid,
    ) -> Result<MembershipWithClinic, MembershipError> {
        let (mut membership, organization) = self.any_membership(membership_id).await?;
        authorize(
            ProtectedAction::ApproveMember,
            ctx,
            &clinic_target(&organization),
        )?;
        if membership.status != MembershipStatus::Requested {
            return Err(MembershipError::InvalidInput(
                "Esa membresía no es una solicitud pendiente".to_string(),
            ));
        }
        membership.status = status_after_admission(membership.role);
        self.state
            .membership_repository
            .update(&membership, None)
            .await?;
        info!(
            "Solicitud aprobada: membership={} por user={}",
            membership.id, ctx.user_id
        );
        Ok(MembershipWithClinic {
            membership,
            organization,
        })
    }

    /// Personal de la clínica (miembros, invitaciones y solicitudes), para el dueño o
    /// un administrador.
    pub async fn list_members(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
    ) -> Result<Vec<Membership>, MembershipError> {
        let organization = self.clinic(&organization_id).await?;
        authorize(
            ProtectedAction::ApproveMember,
            ctx,
            &clinic_target(&organization),
        )?;
        Ok(self
            .state
            .membership_repository
            .find_by_organization(&organization_id)
            .await?)
    }

    /// Guarda los datos legales de la clínica mientras está pendiente de activación.
    pub async fn update_setup(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
        setup: ClinicSetup,
    ) -> Result<Organization, MembershipError> {
        let mut organization = self.clinic(&organization_id).await?;
        authorize(
            ProtectedAction::ConfigureClinic,
            ctx,
            &clinic_target(&organization),
        )?;
        let clean = |value: Option<String>| {
            value
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let tax_id = clean(setup.tax_id);
        let legal_name = clean(setup.legal_name);
        if organization.status != ClinicStatus::PendingSetup
            && (tax_id.is_some() || legal_name.is_some())
        {
            return Err(MembershipError::InvalidInput(
                "El RUC y la razón social no se cambian en una clínica ya activada".to_string(),
            ));
        }
        if let Some(ruc) = &tax_id
            && !is_valid_ruc(ruc)
        {
            return Err(MembershipError::InvalidInput(
                "El RUC no es válido: debe tener 11 dígitos y un dígito verificador correcto"
                    .to_string(),
            ));
        }
        organization.tax_id = tax_id.or(organization.tax_id);
        organization.legal_name = legal_name.or(organization.legal_name);
        organization.trade_name = clean(setup.trade_name).or(organization.trade_name);
        organization.ipress_code = clean(setup.ipress_code).or(organization.ipress_code);
        self.state
            .organization_repository
            .update_setup(&organization)
            .await?;
        Ok(organization)
    }

    /// El dueño activa la clínica cuando él y la clínica tienen los datos exigidos.
    pub async fn activate_clinic(
        &self,
        ctx: &RequestContext,
        organization_id: Uuid,
    ) -> Result<Organization, MembershipError> {
        let mut organization = self.clinic(&organization_id).await?;
        authorize(
            ProtectedAction::ActivateClinic,
            ctx,
            &clinic_target(&organization),
        )?;
        organization.status = ClinicStatus::Active;
        self.state
            .organization_repository
            .update_setup(&organization)
            .await?;
        info!("Clínica activada: {organization_id}");
        Ok(organization)
    }

    /// Membresía del propio usuario. La de otra persona se trata como inexistente.
    async fn own_membership(
        &self,
        ctx: &RequestContext,
        membership_id: Uuid,
    ) -> Result<(Membership, Organization), MembershipError> {
        let (membership, organization) = self.any_membership(membership_id).await?;
        if membership.user_id != ctx.user_id {
            return Err(MembershipError::NotFound(
                "La membresía no existe".to_string(),
            ));
        }
        Ok((membership, organization))
    }

    async fn any_membership(
        &self,
        membership_id: Uuid,
    ) -> Result<(Membership, Organization), MembershipError> {
        let membership = self
            .state
            .membership_repository
            .find_by_id(&membership_id)
            .await?
            .ok_or_else(|| MembershipError::NotFound("La membresía no existe".to_string()))?;
        let organization = self.clinic(&membership.organization_id).await?;
        Ok((membership, organization))
    }
}

/// Estado de quien entra a la clínica, por invitación aceptada o por solicitud aprobada:
/// el médico aún debe activar su perfil; los demás quedan activos.
fn status_after_admission(role: MembershipRole) -> MembershipStatus {
    match role {
        MembershipRole::Practitioner => MembershipStatus::Pending,
        _ => MembershipStatus::Active,
    }
}

/// Objetivo de una acción sobre una clínica.
pub fn clinic_target(organization: &Organization) -> AccessTarget {
    AccessTarget {
        organization_id: Some(organization.id),
        clinic_status: Some(organization.status),
        clinic_fields: organization.present_fields(),
        membership_id: None,
    }
}

/// Objetivo de una acción sobre una membresía propia.
fn membership_target(membership: &Membership, organization: &Organization) -> AccessTarget {
    AccessTarget {
        membership_id: Some(membership.id),
        ..clinic_target(organization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        InMemoryMembershipRepository, InMemoryOrganizationRepository, NoopRepository,
    };
    use app_core::domain::access::{ProfileField, Role};
    use std::collections::HashSet;
    use std::sync::Arc;

    struct Fixture {
        use_cases: MembershipUseCases,
        memberships: Arc<InMemoryMembershipRepository>,
        organization: Organization,
        owner: Uuid,
    }

    /// Clínica con su dueño activo. Los demás repositorios no se usan aquí.
    fn fixture(status: ClinicStatus) -> Fixture {
        let owner = Uuid::now_v7();
        let mut organization = Organization::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            "san-borja".to_string(),
            "Clínica San Borja".to_string(),
            None,
            owner,
        );
        organization.status = status;
        let organizations = Arc::new(InMemoryOrganizationRepository::default());
        organizations
            .organizations
            .lock()
            .unwrap()
            .push(organization.clone());
        let memberships = Arc::new(InMemoryMembershipRepository::default());
        memberships
            .memberships
            .lock()
            .unwrap()
            .push(Membership::new(
                organization.id,
                owner,
                MembershipRole::Owner,
                MembershipStatus::Active,
            ));
        let unused = Arc::new(NoopRepository);
        let state = AdministrationState {
            clinical_network_repository: unused.clone(),
            organization_repository: organizations,
            patient_repository: unused.clone(),
            practitioner_repository: unused,
            membership_repository: memberships.clone(),
            person_links: Arc::new(crate::testing::InMemoryPersonLinks::default()),
        };
        Fixture {
            use_cases: MembershipUseCases::new(state),
            memberships,
            organization,
            owner,
        }
    }

    impl Fixture {
        /// Contexto de un usuario con sus membresías actuales y los datos indicados.
        fn ctx(&self, user_id: Uuid, present: &[ProfileField]) -> RequestContext {
            RequestContext {
                user_id,
                network_id: self.organization.network_id,
                preferred_role: Role::Patient,
                present_fields: present.iter().copied().collect::<HashSet<_>>(),
                memberships: self
                    .memberships
                    .memberships
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|m| m.user_id == user_id)
                    .map(Membership::view)
                    .collect(),
            }
        }
    }

    const DOCTOR_DATA: &[ProfileField] = &[
        ProfileField::GivenName,
        ProfileField::FamilyName,
        ProfileField::Phone,
        ProfileField::Document,
        ProfileField::ProfessionalLicense,
    ];

    #[tokio::test]
    async fn activacion_completa_del_medico() {
        let f = fixture(ClinicStatus::Active);
        let doctor = Uuid::now_v7();
        let invited = f
            .use_cases
            .invite(
                &f.ctx(f.owner, &[]),
                f.organization.id,
                doctor,
                MembershipRole::Practitioner,
            )
            .await
            .expect("el dueño invita");
        let id = invited.membership.id;

        f.use_cases
            .join(&f.ctx(doctor, &[ProfileField::GivenName]), id)
            .await
            .expect("el médico se une");
        let pending = f
            .use_cases
            .activate_practitioner(&f.ctx(doctor, DOCTOR_DATA), id)
            .await
            .expect("con todos sus datos");
        assert_eq!(pending.membership.status, MembershipStatus::Pending);

        let active = f
            .use_cases
            .confirm_license(&f.ctx(f.owner, &[]), id)
            .await
            .expect("el dueño confirma");
        assert_eq!(active.membership.status, MembershipStatus::Active);
        assert!(active.membership.license_confirmed);
    }

    /// Escenario «Dueño que también atiende»: dos membresías en la misma clínica.
    #[tokio::test]
    async fn dueno_que_tambien_atiende() {
        let f = fixture(ClinicStatus::Active);
        f.use_cases
            .invite(
                &f.ctx(f.owner, &[]),
                f.organization.id,
                f.owner,
                MembershipRole::Practitioner,
            )
            .await
            .expect("el dueño se suma como médico");
        let roles: Vec<_> = f
            .use_cases
            .list_for_user(&f.owner)
            .await
            .expect("membresías")
            .into_iter()
            .map(|entry| (entry.membership.role, entry.membership.status))
            .collect();
        assert_eq!(
            roles,
            vec![
                (MembershipRole::Owner, MembershipStatus::Active),
                (MembershipRole::Practitioner, MembershipStatus::Invited),
            ]
        );
    }

    #[tokio::test]
    async fn activacion_sin_colegiatura() {
        let f = fixture(ClinicStatus::Active);
        let doctor = Uuid::now_v7();
        let id = f
            .use_cases
            .invite(
                &f.ctx(f.owner, &[]),
                f.organization.id,
                doctor,
                MembershipRole::Practitioner,
            )
            .await
            .expect("invitación")
            .membership
            .id;
        let result = f
            .use_cases
            .activate_practitioner(
                &f.ctx(
                    doctor,
                    &[
                        ProfileField::GivenName,
                        ProfileField::FamilyName,
                        ProfileField::Phone,
                        ProfileField::Document,
                    ],
                ),
                id,
            )
            .await;
        assert!(matches!(
            result,
            Err(MembershipError::Denied(AccessDenied::MissingData { fields, .. }))
                if fields == vec![ProfileField::ProfessionalLicense]
        ));
    }

    #[tokio::test]
    async fn un_medico_no_invita() {
        let f = fixture(ClinicStatus::Active);
        let doctor = Uuid::now_v7();
        f.memberships
            .memberships
            .lock()
            .unwrap()
            .push(Membership::new(
                f.organization.id,
                doctor,
                MembershipRole::Practitioner,
                MembershipStatus::Active,
            ));
        let result = f
            .use_cases
            .invite(
                &f.ctx(doctor, &[]),
                f.organization.id,
                Uuid::now_v7(),
                MembershipRole::Practitioner,
            )
            .await;
        assert!(matches!(
            result,
            Err(MembershipError::Denied(AccessDenied::Forbidden))
        ));
    }

    #[tokio::test]
    async fn no_se_suspende_al_ultimo_dueno() {
        let f = fixture(ClinicStatus::Active);
        let owner_membership = f.memberships.memberships.lock().unwrap()[0].id;
        assert!(matches!(
            f.use_cases
                .suspend(&f.ctx(f.owner, &[]), owner_membership)
                .await,
            Err(MembershipError::LastOwner)
        ));
    }

    #[tokio::test]
    async fn medico_suspendido_pierde_la_membresia_activa() {
        let f = fixture(ClinicStatus::Active);
        let doctor = Uuid::now_v7();
        let membership = Membership::new(
            f.organization.id,
            doctor,
            MembershipRole::Practitioner,
            MembershipStatus::Active,
        );
        f.memberships
            .memberships
            .lock()
            .unwrap()
            .push(membership.clone());
        f.use_cases
            .suspend(&f.ctx(f.owner, &[]), membership.id)
            .await
            .expect("el dueño suspende");
        let target = clinic_target(&f.organization);
        assert_eq!(
            authorize(
                ProtectedAction::SearchPatients,
                &f.ctx(doctor, &[]),
                &target
            ),
            Err(AccessDenied::Forbidden)
        );
    }

    #[tokio::test]
    async fn configurar_y_activar_clinica() {
        let f = fixture(ClinicStatus::PendingSetup);
        let owner_data = &[
            ProfileField::GivenName,
            ProfileField::FamilyName,
            ProfileField::Phone,
            ProfileField::Document,
        ];
        let without_ruc = f
            .use_cases
            .activate_clinic(&f.ctx(f.owner, owner_data), f.organization.id)
            .await;
        assert!(matches!(
            without_ruc,
            Err(MembershipError::Denied(AccessDenied::MissingData { fields, .. }))
                if fields == vec![ProfileField::ClinicTaxId, ProfileField::ClinicLegalName]
        ));

        let invalid_ruc = f
            .use_cases
            .update_setup(
                &f.ctx(f.owner, &[]),
                f.organization.id,
                ClinicSetup {
                    tax_id: Some("20100070971".to_string()),
                    ..Default::default()
                },
            )
            .await;
        assert!(matches!(invalid_ruc, Err(MembershipError::InvalidInput(_))));

        f.use_cases
            .update_setup(
                &f.ctx(f.owner, &[]),
                f.organization.id,
                ClinicSetup {
                    tax_id: Some("20100070970".to_string()),
                    legal_name: Some("Clínica San Borja S.A.C.".to_string()),
                    ..Default::default()
                },
            )
            .await
            .expect("datos legales válidos");
        let active = f
            .use_cases
            .activate_clinic(&f.ctx(f.owner, owner_data), f.organization.id)
            .await
            .expect("activación");
        assert_eq!(active.status, ClinicStatus::Active);
    }

    const DATOS_DE_INGRESO: &[ProfileField] = &[
        ProfileField::GivenName,
        ProfileField::FamilyName,
        ProfileField::Phone,
        ProfileField::Document,
    ];

    #[tokio::test]
    async fn recepcionista_pide_entrar_y_el_dueno_aprueba() {
        let f = fixture(ClinicStatus::Active);
        let person = Uuid::now_v7();
        let request = f
            .use_cases
            .request_membership(
                &f.ctx(person, DATOS_DE_INGRESO),
                "San-Borja ",
                MembershipRole::Receptionist,
            )
            .await
            .expect("pide entrar");
        assert_eq!(request.membership.status, MembershipStatus::Requested);

        let members = f
            .use_cases
            .list_members(&f.ctx(f.owner, &[]), f.organization.id)
            .await
            .expect("el dueño ve al personal");
        assert_eq!(members.len(), 2);

        let approved = f
            .use_cases
            .approve(&f.ctx(f.owner, &[]), request.membership.id)
            .await
            .expect("el dueño aprueba");
        assert_eq!(approved.membership.status, MembershipStatus::Active);
    }

    #[tokio::test]
    async fn medico_aprobado_aun_debe_activar_su_perfil() {
        let f = fixture(ClinicStatus::Active);
        let doctor = Uuid::now_v7();
        let request = f
            .use_cases
            .request_membership(
                &f.ctx(doctor, DATOS_DE_INGRESO),
                "san-borja",
                MembershipRole::Practitioner,
            )
            .await
            .expect("pide entrar");
        let approved = f
            .use_cases
            .approve(&f.ctx(f.owner, &[]), request.membership.id)
            .await
            .expect("el dueño aprueba");
        assert_eq!(approved.membership.status, MembershipStatus::Pending);
    }

    #[tokio::test]
    async fn no_se_pide_entrar_como_dueno_ni_dos_veces() {
        let f = fixture(ClinicStatus::Active);
        let person = Uuid::now_v7();
        let ctx = f.ctx(person, DATOS_DE_INGRESO);
        assert!(matches!(
            f.use_cases
                .request_membership(&ctx, "san-borja", MembershipRole::Owner)
                .await,
            Err(MembershipError::InvalidInput(_))
        ));
        f.use_cases
            .request_membership(&ctx, "san-borja", MembershipRole::Receptionist)
            .await
            .expect("primera solicitud");
        assert!(matches!(
            f.use_cases
                .request_membership(&ctx, "san-borja", MembershipRole::Receptionist)
                .await,
            Err(MembershipError::AlreadyExists(_))
        ));
    }

    #[tokio::test]
    async fn clinica_de_otra_red_no_existe() {
        let f = fixture(ClinicStatus::Active);
        let mut ctx = f.ctx(Uuid::now_v7(), DATOS_DE_INGRESO);
        ctx.network_id = Uuid::now_v7();
        assert!(matches!(
            f.use_cases
                .request_membership(&ctx, "san-borja", MembershipRole::Receptionist)
                .await,
            Err(MembershipError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn un_recepcionista_no_aprueba_solicitudes() {
        let f = fixture(ClinicStatus::Active);
        let receptionist = Uuid::now_v7();
        f.memberships
            .memberships
            .lock()
            .unwrap()
            .push(Membership::new(
                f.organization.id,
                receptionist,
                MembershipRole::Receptionist,
                MembershipStatus::Active,
            ));
        let request = f
            .use_cases
            .request_membership(
                &f.ctx(Uuid::now_v7(), DATOS_DE_INGRESO),
                "san-borja",
                MembershipRole::Receptionist,
            )
            .await
            .expect("pide entrar");
        assert!(matches!(
            f.use_cases
                .approve(&f.ctx(receptionist, &[]), request.membership.id)
                .await,
            Err(MembershipError::Denied(AccessDenied::Forbidden))
        ));
    }
}
