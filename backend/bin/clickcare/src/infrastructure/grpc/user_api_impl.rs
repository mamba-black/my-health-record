use crate::infrastructure::grpc::SignUpRequest;
use crate::infrastructure::grpc::access::{
    AccessContextBuilder, Authenticator, action_to_proto, field_to_proto,
};
use crate::infrastructure::grpc::subdomain_resolver::SubdomainResolver;
use crate::infrastructure::grpc::user_api_server::UserApi;
use crate::infrastructure::grpc::*;
use administration::application::membership_usecase::{MembershipUseCases, MembershipWithClinic};
use app_core::domain::access::{
    self as access, ClinicStatus, FieldSubject, MembershipRole, MembershipStatus,
    ProfileField as Field, Role as DomainRole,
};
use std::sync::Arc;
use tonic::*;
use tracing::{debug, error};
use user::application::dto::complete_profile::{CompleteProfileCommand, CompleteProfileError};
use user::application::dto::create_user::{CreateUserCommand, CreateUserError};
use user::application::dto::sign_in::{SignInCommand, SignInError};
use user::application::integration::service::session_service::{
    SessionService, SessionTokens as DomainSessionTokens,
};
use user::application::use_case::complete_profile_usecase::CompleteProfileUseCase;
use user::application::use_case::create_user_usecase::CreateUserUseCase;
use user::application::use_case::refresh_session_usecase::{
    RefreshSessionError, RefreshSessionUseCase,
};
use user::application::use_case::sign_in_usecase::SignInUseCase;
use user::domain::user::{Sex as DomainSex, User};
use user::infrastructure::config::DI as UserDI;

/// Implementación del servicio gRPC `UserApi` para gestión e identidad de usuarios.
///
/// `SignUp`, `SignIn` y `RefreshSession` son públicas; el resto exige sesión.
pub struct UserApiImpl {
    create_user_use_case: Arc<CreateUserUseCase>,
    sign_in_use_case: Arc<SignInUseCase>,
    complete_profile_use_case: Arc<CompleteProfileUseCase>,
    refresh_session_use_case: Arc<RefreshSessionUseCase>,
    session_service: Arc<dyn SessionService>,
    authenticator: Authenticator,
    access: Arc<AccessContextBuilder>,
    membership_use_cases: Arc<MembershipUseCases>,
    /// Resolvedor de subdominio utilizado para determinar `network_id` de cada solicitud.
    pub subdomain_resolver: Arc<SubdomainResolver>,
}

impl UserApiImpl {
    pub fn new(
        user_di: &UserDI,
        authenticator: Authenticator,
        access: Arc<AccessContextBuilder>,
        membership_use_cases: Arc<MembershipUseCases>,
        subdomain_resolver: Arc<SubdomainResolver>,
    ) -> Self {
        Self {
            create_user_use_case: Arc::clone(&user_di.create_user_use_case),
            sign_in_use_case: Arc::clone(&user_di.sign_in_use_case),
            complete_profile_use_case: Arc::clone(&user_di.complete_profile_use_case),
            refresh_session_use_case: Arc::clone(&user_di.refresh_session_use_case),
            session_service: Arc::clone(&user_di.session_service),
            authenticator,
            access,
            membership_use_cases,
            subdomain_resolver,
        }
    }

    fn open_session(
        &self,
        user_id: uuid::Uuid,
        network_id: uuid::Uuid,
    ) -> Result<SessionTokens, Status> {
        self.session_service
            .issue(user_id, network_id)
            .map(session_to_proto)
            .map_err(|error| Status::internal(error.to_string()))
    }
}

#[async_trait]
impl UserApi for UserApiImpl {
    /// Registra un nuevo usuario en la red clínica resuelta a partir del subdominio de la solicitud.
    #[tracing::instrument(skip(self, sign_up_request))]
    async fn sign_up(
        &self,
        sign_up_request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        let (network_id, _organization_id) =
            self.subdomain_resolver.resolve(&sign_up_request).await?;

        let sign_up_request = sign_up_request.into_inner();

        let mut create_user_command: CreateUserCommand = sign_up_request.into();
        create_user_command.network_id = network_id;
        debug!("command: {:?}", create_user_command);

        let created = self
            .create_user_use_case
            .execute(create_user_command)
            .await
            .map_err(|err| match err {
                CreateUserError::UserAlreadyExists(error) => {
                    Status::already_exists(error.to_string())
                }
                CreateUserError::UnknownError(error) => Status::unknown(error.to_string()),
            })?;
        let user_id = uuid::Uuid::parse_str(&created.user_id)
            .map_err(|error| Status::internal(error.to_string()))?;

        Ok(Response::new(SignUpResponse {
            status: SignUpStatus::Success as i32,
            message: "Usuario registrado exitosamente.".to_string(),
            user_id: Some(created.user_id),
            link: None,
            organization_id: created.organization_id,
            session: Some(self.open_session(user_id, network_id)?),
        }))
    }

    /// Autentica con el `id_token` de Google y abre una sesión del backend.
    #[tracing::instrument(skip(self, sign_in_request))]
    async fn sign_in(
        &self,
        sign_in_request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        let (network_id, _organization_id) =
            self.subdomain_resolver.resolve(&sign_in_request).await?;

        let sign_in_request = sign_in_request.into_inner();
        let mut sign_in_command: SignInCommand = sign_in_request.into();
        sign_in_command.network_id = network_id;

        let user = self
            .sign_in_use_case
            .execute(sign_in_command)
            .await
            .map_err(|sign_in_error| {
                error!("sign_in_error: {:?}", sign_in_error);
                match sign_in_error {
                    SignInError::InvalidToken(error) => Status::unauthenticated(error.to_string()),
                    SignInError::UserNotFound(error) => Status::not_found(error.to_string()),
                    SignInError::UnknownError(error) => Status::internal(error.to_string()),
                }
            })?
            .user;
        let session = self.open_session(user.id, user.network_id)?;
        let mut response = to_sign_in_response(user);
        response.session = Some(session);
        Ok(Response::new(response))
    }

    /// Guarda datos del perfil de la cuenta de la sesión. Nada es obligatorio aquí.
    #[tracing::instrument(skip(self, request))]
    async fn complete_profile(
        &self,
        request: Request<CompleteProfileRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        let claims = self.authenticator.authenticate(request.metadata())?;
        let request = request.into_inner();

        let user = self
            .complete_profile_use_case
            .execute(CompleteProfileCommand {
                network_id: claims.network_id,
                user_id: claims.user_id,
                given_name: Some(request.given_name),
                family_name: request.family_name,
                second_family_name: request.second_family_name,
                identifier: request.identifier.and_then(identifier_from_proto),
                phone: Some(request.phone),
                birth_date: Some(request.birth_date),
                address: request.address,
                sex: sex_from_proto(request.sex),
                professional_body: request.professional_body,
                license_number: request.license_number,
                specialty: request.specialty,
                rne: request.rne,
                preferred_role: role_from_proto(request.preferred_role),
            })
            .await
            .map_err(|complete_profile_error| {
                error!("complete_profile_error: {:?}", complete_profile_error);
                match complete_profile_error {
                    CompleteProfileError::InvalidInput(message) => {
                        Status::invalid_argument(message)
                    }
                    CompleteProfileError::ReadOnly(message) => Status::failed_precondition(message),
                    CompleteProfileError::DocumentAlreadyRegistered(error) => {
                        Status::already_exists(error.to_string())
                    }
                    CompleteProfileError::UserNotFound(error) => {
                        Status::unauthenticated(error.to_string())
                    }
                    CompleteProfileError::UnknownError(error) => {
                        Status::internal(error.to_string())
                    }
                }
            })?;
        Ok(Response::new(to_sign_in_response(user)))
    }

    /// Renueva la sesión con el token de renovación.
    async fn refresh_session(
        &self,
        request: Request<RefreshSessionRequest>,
    ) -> Result<Response<SessionTokens>, Status> {
        self.refresh_session_use_case
            .execute(&request.into_inner().refresh_token)
            .await
            .map(|tokens| Response::new(session_to_proto(tokens)))
            .map_err(|error| match error {
                RefreshSessionError::Unauthenticated(message) => Status::unauthenticated(message),
                RefreshSessionError::UnknownError(error) => Status::internal(error.to_string()),
            })
    }

    /// Datos que aplican a los roles del usuario, para qué acción es obligatorio cada uno
    /// y cuáles faltan, junto con sus membresías.
    async fn get_profile_requirements(
        &self,
        request: Request<GetProfileRequirementsRequest>,
    ) -> Result<Response<ProfileRequirements>, Status> {
        let claims = self.authenticator.authenticate(request.metadata())?;
        self.access.invalidate(&claims.user_id);
        let (user, ctx) = self.access.build(&claims).await?;
        let memberships = self
            .membership_use_cases
            .list_for_user(&user.id)
            .await
            .map_err(|error| Status::internal(error.to_string()))?;

        // Datos de clínica: los de la primera clínica propia pendiente de activación.
        let pending_clinic = memberships.iter().find(|entry| {
            entry.membership.role == MembershipRole::Owner
                && entry.membership.status == MembershipStatus::Active
                && entry.organization.status == ClinicStatus::PendingSetup
        });
        let clinic_fields = pending_clinic
            .map(|entry| entry.organization.present_fields())
            .unwrap_or_default();
        let license_confirmed = memberships.iter().any(|entry| {
            entry.membership.role == MembershipRole::Practitioner
                && entry.membership.license_confirmed
        });

        let roles = ctx.roles();
        let fields = access::CATALOG
            .iter()
            .filter(|spec| spec.roles.iter().any(|role| roles.contains(role)))
            .map(|spec| {
                let present = match spec.subject {
                    FieldSubject::Person => ctx.present_fields.contains(&spec.field),
                    FieldSubject::Clinic => clinic_fields.contains(&spec.field),
                    FieldSubject::Membership => license_confirmed,
                };
                let read_only = !spec.user_editable
                    || (spec.field == Field::Document && user.document_verified)
                    || (spec.field == Field::ProfessionalLicense && user.license_verified);
                FieldRequirement {
                    field: field_to_proto(spec.field) as i32,
                    required_for: spec
                        .required_for
                        .iter()
                        .map(|action| action_to_proto(*action) as i32)
                        .collect(),
                    missing: !present,
                    read_only,
                    roles: spec
                        .roles
                        .iter()
                        .filter(|role| roles.contains(role))
                        .map(|role| role_to_proto(*role) as i32)
                        .collect(),
                }
            })
            .collect();

        Ok(Response::new(ProfileRequirements {
            fields,
            memberships: memberships.iter().map(membership_to_proto).collect(),
            preferred_role: role_to_proto(user.preferred_role) as i32,
            missing_for_habitual: habitual_missing(&user),
            account: Some(to_sign_in_response(user)),
        }))
    }
}

/// Datos que le faltan para la acción habitual de su rol preferido (aviso al iniciar sesión).
fn habitual_missing(user: &User) -> Vec<i32> {
    user.missing_for(access::habitual_action(user.preferred_role))
        .into_iter()
        .map(|field| field_to_proto(field) as i32)
        .collect()
}

pub(crate) fn to_sign_in_response(user: User) -> SignInResponse {
    SignInResponse {
        user_id: user.id.to_string(),
        email: user.email(),
        given_name: user.person.name().given().join(" "),
        family_name: user.person.name().family().clone(),
        is_owner: user.is_owner,
        phone: user.phone(),
        missing_fields: habitual_missing(&user),
        second_family_name: user.person.name().second_family().clone(),
        birth_date: Some(user.birth_date()).filter(|birth_date| !birth_date.is_empty()),
        identifier: user.person.identifier().as_ref().map(identifier_to_proto),
        address: user.address.clone(),
        session: None,
        preferred_role: role_to_proto(user.preferred_role) as i32,
        sex: match user.sex {
            Some(DomainSex::Female) => Sex::Female,
            Some(DomainSex::Male) => Sex::Male,
            Some(DomainSex::Other) => Sex::Other,
            None => Sex::Unspecified,
        } as i32,
        professional_body: user.professional.professional_body.clone(),
        license_number: user.professional.license_number.clone(),
        specialty: user.professional.specialty.clone(),
        rne: user.professional.rne.clone(),
        document_verified: user.document_verified,
        license_verified: user.license_verified,
    }
}

fn session_to_proto(tokens: DomainSessionTokens) -> SessionTokens {
    SessionTokens {
        access_token: tokens.access_token,
        access_expires_at: tokens.access_expires_at,
        refresh_token: tokens.refresh_token,
    }
}

pub(crate) fn identifier_to_proto(identifier: &app_core::domain::fhir::Identifier) -> Identifier {
    use app_core::domain::fhir::Identifier as Domain;
    Identifier {
        identifier_type: Some(match identifier {
            Domain::DNI(value) => identifier::IdentifierType::Dni(value.clone()),
            Domain::CE(value) => identifier::IdentifierType::Ce(value.clone()),
            Domain::Passport(value) => identifier::IdentifierType::Passport(value.clone()),
        }),
    }
}

pub(crate) fn identifier_from_proto(
    identifier: Identifier,
) -> Option<app_core::domain::fhir::Identifier> {
    use app_core::domain::fhir::Identifier as Domain;
    identifier.identifier_type.map(|kind| match kind {
        identifier::IdentifierType::Dni(value) => Domain::DNI(value),
        identifier::IdentifierType::Ce(value) => Domain::CE(value),
        identifier::IdentifierType::Passport(value) => Domain::Passport(value),
    })
}

fn sex_from_proto(sex: i32) -> Option<DomainSex> {
    match Sex::try_from(sex).unwrap_or(Sex::Unspecified) {
        Sex::Female => Some(DomainSex::Female),
        Sex::Male => Some(DomainSex::Male),
        Sex::Other => Some(DomainSex::Other),
        Sex::Unspecified => None,
    }
}

fn role_from_proto(role: i32) -> Option<DomainRole> {
    match Role::try_from(role).unwrap_or(Role::Unspecified) {
        Role::Patient => Some(DomainRole::Patient),
        Role::Practitioner => Some(DomainRole::Practitioner),
        Role::ClinicOwner => Some(DomainRole::ClinicOwner),
        Role::Unspecified => None,
    }
}

pub(crate) fn role_to_proto(role: DomainRole) -> Role {
    match role {
        DomainRole::Patient => Role::Patient,
        DomainRole::Practitioner => Role::Practitioner,
        DomainRole::ClinicOwner => Role::ClinicOwner,
    }
}

pub(crate) fn membership_to_proto(entry: &MembershipWithClinic) -> Membership {
    let membership = &entry.membership;
    Membership {
        id: membership.id.to_string(),
        organization_id: membership.organization_id.to_string(),
        organization_name: entry.organization.name.clone(),
        organization_status: clinic_status_to_proto(entry.organization.status) as i32,
        user_id: membership.user_id.to_string(),
        role: match membership.role {
            MembershipRole::Owner => grpc_gen::MembershipRole::Owner,
            MembershipRole::Admin => grpc_gen::MembershipRole::Admin,
            MembershipRole::Practitioner => grpc_gen::MembershipRole::Practitioner,
        } as i32,
        status: match membership.status {
            MembershipStatus::Invited => grpc_gen::MembershipStatus::Invited,
            MembershipStatus::Pending => grpc_gen::MembershipStatus::Pending,
            MembershipStatus::Active => grpc_gen::MembershipStatus::Active,
            MembershipStatus::Suspended => grpc_gen::MembershipStatus::Suspended,
        } as i32,
        license_confirmed: membership.license_confirmed,
    }
}

pub(crate) fn clinic_status_to_proto(status: ClinicStatus) -> grpc_gen::ClinicStatus {
    match status {
        ClinicStatus::PendingSetup => grpc_gen::ClinicStatus::PendingSetup,
        ClinicStatus::Active => grpc_gen::ClinicStatus::Active,
        ClinicStatus::Suspended => grpc_gen::ClinicStatus::Suspended,
    }
}

mod mapper {
    use crate::infrastructure::grpc::{SignInRequest, SignUpIntent, SignUpRequest};
    use user::application::dto::create_user::CreateUserCommand;
    use user::application::dto::sign_in::SignInCommand;
    use uuid::Uuid;

    impl From<SignInRequest> for SignInCommand {
        fn from(sign_in_request: SignInRequest) -> Self {
            SignInCommand {
                network_id: Uuid::nil(),
                id_token: sign_in_request.id_token,
                provider_id: sign_in_request.provider_id,
            }
        }
    }

    impl From<SignUpRequest> for CreateUserCommand {
        // `create_clinic` está deprecado en el contrato: la creación de clínica es
        // ahora `ClinicApi.CreateClinic`. Se sigue leyendo mientras el worker de
        // `administration` lo consuma desde `UserCreatedEvent`.
        #[allow(deprecated)]
        fn from(sign_up_request: SignUpRequest) -> Self {
            let intent = match sign_up_request.intent() {
                SignUpIntent::IntentClinicOwner => user::domain::user::SignUpIntent::ClinicOwner,
                SignUpIntent::IntentPractitioner => user::domain::user::SignUpIntent::Practitioner,
                SignUpIntent::IntentPatient => user::domain::user::SignUpIntent::Patient,
                SignUpIntent::IntentUnspecified => user::domain::user::SignUpIntent::Unspecified,
            };

            CreateUserCommand {
                network_id: Uuid::nil(),
                id_token: sign_up_request.id_token,
                user_id: sign_up_request.user_id.clone(),
                provider_id: sign_up_request.provider_id,
                provider_name: sign_up_request.provider_name,
                provider_avatar_url: sign_up_request.provider_avatar_url,
                email: sign_up_request.email.clone(),
                identifier: sign_up_request
                    .identifier
                    .and_then(super::identifier_from_proto),
                first_name: sign_up_request.given_name,
                last_name: sign_up_request.family_name,
                second_family_name: sign_up_request.second_family_name,
                phone: sign_up_request.phone,
                address: sign_up_request.address,
                birthdate: sign_up_request.birth_date,
                display_name: sign_up_request.display_name,
                create_clinic: sign_up_request.create_clinic,
                intent,
                username: sign_up_request.email,
                password: "123".to_string(),
            }
        }
    }
}
