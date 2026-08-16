use crate::infrastructure::grpc::clinic_api_server::ClinicApi;
use crate::infrastructure::grpc::{CreateClinicRequest, CreateClinicResponse};
use administration::application::{CreateClinicCommand, CreateClinicError, CreateClinicUseCase};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::debug;

/// Adaptador gRPC del contexto acotado `administration`.
pub struct ClinicApiImpl {
    create_clinic_use_case: Arc<dyn CreateClinicUseCase>,
}

impl ClinicApiImpl {
    pub fn new(create_clinic_use_case: Arc<dyn CreateClinicUseCase>) -> Self {
        Self {
            create_clinic_use_case,
        }
    }
}

#[tonic::async_trait]
impl ClinicApi for ClinicApiImpl {
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
                    practitioner_id: response.practitioner_id.to_string(),
                    already_existed: response.already_existed,
                })
            })
            .map_err(|error| match error {
                CreateClinicError::InvalidOwnerUserId(_) | CreateClinicError::EmptyName => {
                    Status::invalid_argument(error.to_string())
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

    /// Traduce el DTO plano de la API al comando del caso de uso.
    ///
    /// El mapeo a Value Objects FHIR ocurre después, dentro del dominio: la
    /// estructura plana del DTO no cruza esa frontera.
    impl From<CreateClinicRequest> for CreateClinicCommand {
        fn from(request: CreateClinicRequest) -> Self {
            CreateClinicCommand {
                owner_user_id: request.owner_user_id,
                name: request.name,
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
