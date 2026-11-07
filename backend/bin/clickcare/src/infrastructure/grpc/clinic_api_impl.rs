use crate::infrastructure::grpc;
use crate::infrastructure::grpc::access::{AccessContextBuilder, claims_of, denied_to_status};
use crate::infrastructure::grpc::clinic_api_impl::mapper::to_status;
use crate::infrastructure::grpc::clinic_api_server::ClinicApi;
use crate::infrastructure::grpc::user_api_impl::{clinic_status_to_proto, membership_to_proto};
use administration::application::create_clinic_usecase::CreateClinicUseCase;
use administration::application::dto::create_clinic::CreateClinicCommand;
use administration::application::membership_usecase::{
    ClinicSetup, MembershipError, MembershipUseCases, MembershipWithClinic,
};
use administration::domain::organization::Organization;
use app_core::domain::access::{AccessTarget, MembershipRole, ProtectedAction, authorize};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::debug;
use user::domain::user::user_repository::UserRepository;
use uuid::Uuid;

/// Adaptador del servicio gRPC `ClinicApi`. Todas sus operaciones exigen sesión: el
/// interceptor deja la identidad en la petición y cada operación consulta la regla.
pub struct ClinicApiImpl {
    create_clinic_use_case: Arc<CreateClinicUseCase>,
    membership_use_cases: Arc<MembershipUseCases>,
    access: Arc<AccessContextBuilder>,
    user_repository: Arc<dyn UserRepository>,
}

impl ClinicApiImpl {
    pub fn new(
        create_clinic_use_case: Arc<CreateClinicUseCase>,
        membership_use_cases: Arc<MembershipUseCases>,
        access: Arc<AccessContextBuilder>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            create_clinic_use_case,
            membership_use_cases,
            access,
            user_repository,
        }
    }

    async fn ctx<T>(
        &self,
        request: &Request<T>,
    ) -> Result<
        (
            user::domain::user::User,
            app_core::domain::access::RequestContext,
        ),
        Status,
    > {
        let claims = claims_of(request)?;
        self.access.build(&claims).await
    }
}

fn parse_id(raw: &str, label: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(raw.trim())
        .map_err(|_| Status::invalid_argument(format!("{label} no es válido")))
}

fn membership_error(error: MembershipError, organization_id: Option<Uuid>) -> Status {
    match error {
        MembershipError::Denied(denied) => denied_to_status(denied, organization_id),
        MembershipError::NotFound(message) => Status::not_found(message),
        MembershipError::AlreadyExists(message) => Status::already_exists(message),
        MembershipError::InvalidInput(message) => Status::invalid_argument(message),
        MembershipError::LastOwner => Status::failed_precondition(error.to_string()),
        MembershipError::Unknown(error) => Status::internal(error.to_string()),
    }
}

fn clinic_summary(organization: &Organization) -> grpc::ClinicSummary {
    grpc::ClinicSummary {
        organization_id: organization.id.to_string(),
        name: organization.name.clone(),
        status: clinic_status_to_proto(organization.status) as i32,
        tax_id: organization.tax_id.clone(),
        legal_name: organization.legal_name.clone(),
        trade_name: organization.trade_name.clone(),
        ipress_code: organization.ipress_code.clone(),
    }
}

#[tonic::async_trait]
impl ClinicApi for ClinicApiImpl {
    /// Crea una clínica pendiente de activación; quien la crea queda como su dueño.
    #[tracing::instrument(skip(self, request))]
    async fn create_clinic(
        &self,
        request: Request<grpc::CreateClinicRequest>,
    ) -> Result<Response<grpc::CreateClinicResponse>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        authorize(
            ProtectedAction::CreateClinic,
            &ctx,
            &AccessTarget::default(),
        )
        .map_err(|denied| denied_to_status(denied, None))?;

        let mut command: CreateClinicCommand = request.into_inner().into();
        // El dueño es siempre la cuenta de la sesión, nunca un dato del cliente.
        command.owner_user_id = ctx.user_id.to_string();
        debug!("command: {command:?}");

        let response = self
            .create_clinic_use_case
            .execute(command)
            .await
            .map(|response| Response::new(response.into()))
            .map_err(to_status)?;
        self.access.invalidate(&ctx.user_id);
        Ok(response)
    }

    async fn update_clinic_setup(
        &self,
        request: Request<grpc::UpdateClinicSetupRequest>,
    ) -> Result<Response<grpc::ClinicSummary>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        self.membership_use_cases
            .update_setup(
                &ctx,
                organization_id,
                ClinicSetup {
                    tax_id: request.tax_id,
                    legal_name: request.legal_name,
                    trade_name: request.trade_name,
                    ipress_code: request.ipress_code,
                },
            )
            .await
            .map(|organization| Response::new(clinic_summary(&organization)))
            .map_err(|error| membership_error(error, Some(organization_id)))
    }

    async fn activate_clinic(
        &self,
        request: Request<grpc::ClinicIdRequest>,
    ) -> Result<Response<grpc::ClinicSummary>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let organization_id = parse_id(&request.get_ref().organization_id, "La clínica")?;
        self.membership_use_cases
            .activate_clinic(&ctx, organization_id)
            .await
            .map(|organization| Response::new(clinic_summary(&organization)))
            .map_err(|error| membership_error(error, Some(organization_id)))
    }

    async fn invite_member(
        &self,
        request: Request<grpc::InviteMemberRequest>,
    ) -> Result<Response<grpc::Membership>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let request = request.into_inner();
        let organization_id = parse_id(&request.organization_id, "La clínica")?;
        let role = match grpc::MembershipRole::try_from(request.role) {
            Ok(grpc::MembershipRole::Practitioner) => MembershipRole::Practitioner,
            Ok(grpc::MembershipRole::Admin) => MembershipRole::Admin,
            _ => {
                return Err(Status::invalid_argument(
                    "Solo se puede invitar como médico o administrador",
                ));
            }
        };
        // Primero se decide si puede invitar: así no se revela si un correo existe.
        let clinic = self
            .membership_use_cases
            .clinic(&organization_id)
            .await
            .map_err(|error| membership_error(error, Some(organization_id)))?;
        authorize(
            ProtectedAction::InviteStaff,
            &ctx,
            &administration::application::membership_usecase::clinic_target(&clinic),
        )
        .map_err(|denied| denied_to_status(denied, Some(organization_id)))?;
        let invitee = self
            .user_repository
            .find_user_by_email(&ctx.network_id, request.email.trim())
            .await
            .map_err(|error| Status::internal(error.to_string()))?
            .ok_or_else(|| {
                Status::not_found(
                    "No hay una cuenta con ese correo: la persona debe registrarse primero",
                )
            })?;
        let entry = self
            .membership_use_cases
            .invite(&ctx, organization_id, invitee.id, role)
            .await
            .map_err(|error| membership_error(error, Some(organization_id)))?;
        self.access.invalidate(&invitee.id);
        Ok(Response::new(membership_to_proto(&entry)))
    }

    async fn join_clinic(
        &self,
        request: Request<grpc::MembershipIdRequest>,
    ) -> Result<Response<grpc::Membership>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let membership_id = parse_id(&request.get_ref().membership_id, "La membresía")?;
        let entry = self
            .membership_use_cases
            .join(&ctx, membership_id)
            .await
            .map_err(|error| membership_error(error, None))?;
        self.done(entry)
    }

    async fn activate_practitioner(
        &self,
        request: Request<grpc::MembershipIdRequest>,
    ) -> Result<Response<grpc::Membership>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let membership_id = parse_id(&request.get_ref().membership_id, "La membresía")?;
        let entry = self
            .membership_use_cases
            .activate_practitioner(&ctx, membership_id)
            .await
            .map_err(|error| membership_error(error, None))?;
        self.done(entry)
    }

    async fn confirm_license(
        &self,
        request: Request<grpc::MembershipIdRequest>,
    ) -> Result<Response<grpc::Membership>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let membership_id = parse_id(&request.get_ref().membership_id, "La membresía")?;
        let entry = self
            .membership_use_cases
            .confirm_license(&ctx, membership_id)
            .await
            .map_err(|error| membership_error(error, None))?;
        // Confirmada por una clínica, la colegiatura deja de ser editable por el médico.
        self.user_repository
            .set_license_verified(&entry.membership.user_id)
            .await
            .map_err(|error| Status::internal(error.to_string()))?;
        self.done(entry)
    }

    async fn suspend_membership(
        &self,
        request: Request<grpc::MembershipIdRequest>,
    ) -> Result<Response<grpc::Membership>, Status> {
        let (_user, ctx) = self.ctx(&request).await?;
        let membership_id = parse_id(&request.get_ref().membership_id, "La membresía")?;
        let entry = self
            .membership_use_cases
            .suspend(&ctx, membership_id)
            .await
            .map_err(|error| membership_error(error, None))?;
        self.done(entry)
    }
}

impl ClinicApiImpl {
    /// Invalida las membresías en caché del afectado y responde.
    fn done(&self, entry: MembershipWithClinic) -> Result<Response<grpc::Membership>, Status> {
        self.access.invalidate(&entry.membership.user_id);
        Ok(Response::new(membership_to_proto(&entry)))
    }
}

mod mapper {
    use crate::infrastructure::grpc;
    use administration::application::dto::create_clinic::{
        CreateClinicCommand, CreateClinicError, CreateClinicResponse,
    };
    use tonic::Status;
    use uuid::Uuid;

    /// Traduce el DTO plano de la API al comando del caso de uso.
    ///
    /// El mapeo a Value Objects FHIR ocurre después, dentro del dominio: la
    /// estructura plana del DTO no cruza esa frontera.
    impl From<grpc::CreateClinicRequest> for CreateClinicCommand {
        fn from(request: grpc::CreateClinicRequest) -> Self {
            let network_id = request
                .network_id
                .as_deref()
                .and_then(|id| Uuid::parse_str(id).ok());

            CreateClinicCommand {
                owner_user_id: request.owner_user_id,
                name: request.name,
                subdomain: request.subdomain,
                network_id,
                tax_id: request.tax_id,
                given_name: request.given_name,
                family_name: request.family_name,
                second_family_name: request.second_family_name,
                email: request.email,
                phone: request.phone,
                medical_license_number: request.medical_license_number,
            }
        }
    }

    impl From<CreateClinicResponse> for grpc::CreateClinicResponse {
        fn from(response: CreateClinicResponse) -> Self {
            grpc::CreateClinicResponse {
                organization_id: response.organization_id.to_string(),
                network_id: response.network_id.to_string(),
                practitioner_id: response.practitioner_id.to_string(),
                already_existed: response.already_existed,
            }
        }
    }

    pub fn to_status(error: CreateClinicError) -> Status {
        match error {
            CreateClinicError::InvalidOwnerUserId(_)
            | CreateClinicError::EmptyName
            | CreateClinicError::EmptySubdomain
            | CreateClinicError::ReservedSubdomain(_)
            | CreateClinicError::InvalidSubdomainFormat(_) => {
                Status::invalid_argument(error.to_string())
            }
            CreateClinicError::SubdomainAlreadyExists(_) => {
                Status::already_exists(error.to_string())
            }
            CreateClinicError::MissingPractitioner | CreateClinicError::Unknown(_) => {
                Status::internal(error.to_string())
            }
        }
    }
}
