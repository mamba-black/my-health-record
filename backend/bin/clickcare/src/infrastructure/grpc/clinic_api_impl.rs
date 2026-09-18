use crate::infrastructure::grpc::clinic_api_server::ClinicApi;
use crate::infrastructure::grpc::{CreateClinicRequest, CreateClinicResponse};
use administration::application::{CreateClinicCommand, CreateClinicError, CreateClinicUseCase};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::debug;

/// Adaptador del servicio gRPC `ClinicApi` para la creación y administración de clínicas.
pub struct ClinicApiImpl {
    create_clinic_use_case: Arc<CreateClinicUseCase>,
}

impl ClinicApiImpl {
    /// Crea una nueva instancia inyectando el caso de uso `CreateClinicUseCase`.
    pub fn new(create_clinic_use_case: Arc<CreateClinicUseCase>) -> Self {
        Self {
            create_clinic_use_case,
        }
    }
}

#[tonic::async_trait]
impl ClinicApi for ClinicApiImpl {
    /// Crea o recupera una clínica con su subdominio exclusivo y ficha de profesional propietario.
    #[tracing::instrument(skip(self, request))]
    async fn create_clinic(
        &self,
        request: Request<CreateClinicRequest>,
    ) -> Result<Response<CreateClinicResponse>, Status> {
        let command: CreateClinicCommand = request.into_inner().into();
        debug!("command: {command:?}");

        self.create_clinic_use_case
            .execute(command)
            .await
            .map(|response| {
                Response::new(CreateClinicResponse {
                    organization_id: response.organization_id.to_string(),
                    network_id: response.network_id.to_string(),
                    practitioner_id: response.practitioner_id.to_string(),
                    already_existed: response.already_existed,
                })
            })
            .map_err(|error| match error {
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
            })
    }
}

mod mapper {
    use crate::infrastructure::grpc::CreateClinicRequest;
    use administration::application::CreateClinicCommand;
    use uuid::Uuid;

    /// Traduce el DTO plano de la API al comando del caso de uso.
    ///
    /// El mapeo a Value Objects FHIR ocurre después, dentro del dominio: la
    /// estructura plana del DTO no cruza esa frontera.
    impl From<CreateClinicRequest> for CreateClinicCommand {
        fn from(request: CreateClinicRequest) -> Self {
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
}
