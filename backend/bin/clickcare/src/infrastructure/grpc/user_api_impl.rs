use crate::infrastructure::grpc::SignUpRequest;
use crate::infrastructure::grpc::subdomain_resolver::SubdomainResolver;
use crate::infrastructure::grpc::user_api_server::UserApi;
use crate::infrastructure::grpc::*;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;
use tonic::*;
use tracing::debug;
use user::application::CreateUserUseCase;
use user::application::command::{CreateUserCommand, CreateUserError};
use user::infrastructure::di;
use user::infrastructure::di::DBType;

use user::domain::repository::user_repository::UserRepository;

/// Implementación del servicio gRPC `UserApi` para gestión e identidad de usuarios.
pub struct UserApiImpl {
    create_user_use_case: Arc<CreateUserUseCase>,
    #[allow(dead_code)]
    pub user_repository: Arc<dyn UserRepository>,
    /// Resolvedor de subdominio utilizado para determinar `network_id` de cada solicitud.
    pub subdomain_resolver: Arc<SubdomainResolver>,
}

impl UserApiImpl {
    /// Inicializa la implementación conectándola a PostgreSQL mediante una URL opcional.
    pub async fn new(
        database_url: Option<String>,
        subdomain_resolver: Arc<SubdomainResolver>,
    ) -> Result<UserApiImpl, ClickCareError> {
        let database_type = match database_url {
            Some(url) => DBType::Postgres(Some(url)),
            None => DBType::Postgres(None),
        };
        Self::new_with_database_type(database_type, subdomain_resolver).await
    }

    /// Inicializa una versión mock en memoria para pruebas unitarias.
    #[allow(dead_code)]
    pub async fn new_mock() -> Result<UserApiImpl, ClickCareError> {
        Self::new_with_database_type(DBType::Mock, Arc::new(SubdomainResolver::new_mock())).await
    }

    /// Inicializa la implementación con un tipo de base de datos específico (`DBType`).
    pub async fn new_with_database_type(
        database_type: DBType,
        subdomain_resolver: Arc<SubdomainResolver>,
    ) -> Result<UserApiImpl, ClickCareError> {
        let di = di::new(database_type).await?;
        Ok(Self {
            create_user_use_case: di.create_user_use_case,
            user_repository: di.user_repository,
            subdomain_resolver,
        })
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

        self.create_user_use_case
            .execute(create_user_command)
            .await
            .map(|create_user_response| {
                Response::new(SignUpResponse {
                    status: SignUpStatus::Success as i32,
                    message: "Usuario registrado exitosamente.".to_string(),
                    user_id: Some(create_user_response.user_id),
                    link: None,
                    organization_id: create_user_response.organization_id,
                })
            })
            .map_err(|err| match err {
                CreateUserError::UserAlreadyExists(e) => Status::already_exists(e.to_string()),
                CreateUserError::UnknownError(e) => Status::unknown(e.to_string()),
            })
    }

    async fn sign_in(
        &self,
        _request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        todo!()
    }
}

mod mapper {
    use crate::infrastructure::grpc::identifier::IdentifierType;
    use crate::infrastructure::grpc::{SignUpIntent, SignUpRequest};
    use app_core::domain::fhir::Identifier::DNI;
    use user::application::command::CreateUserCommand;
    use uuid::Uuid;

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
                identifier: sign_up_request.identifier.and_then(|identifier| {
                    identifier
                        .identifier_type
                        .map(|IdentifierType::Dni(dni)| DNI(dni))
                }),
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

#[cfg(test)]
mod test {
    use crate::infrastructure::grpc::SignUpRequest;
    use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
    use crate::infrastructure::grpc::user_api_server::UserApi;
    use crate::infrastructure::log::init_observability;
    use app_core::domain::error::ClickCareError;
    use dotenvy::dotenv;
    use rstest::{fixture, rstest};
    use std::sync::{LazyLock, Once};
    use tonic::Request;
    use tracing::info;
    use uuid::Uuid;

    static INIT: Once = Once::new();
    // `create_clinic` está deprecado en el contrato; sigue en uso mientras el worker
    // lo consuma desde `UserCreatedEvent`.
    #[allow(deprecated)]
    static SIGN_UP_REQUEST: LazyLock<SignUpRequest> = LazyLock::new(|| SignUpRequest {
        id_token: "".to_string(),
        user_id: "".to_string(),
        provider_id: "".to_string(),
        provider_name: "".to_string(),
        provider_avatar_url: None,
        email: "".to_string(),
        identifier: None,
        given_name: "".to_string(),
        family_name: None,
        second_family_name: None,
        phone: "".to_string(),
        address: "".to_string(),
        birth_date: "".to_string(),
        display_name: None,
        create_clinic: false,
        confirm_pending_presencial_link: None,
        intent: 0,
    });

    type TestResult = Result<(), ClickCareError>;

    #[fixture]
    async fn user_api_impl() -> UserApiImpl {
        INIT.call_once(|| {
            dotenv().ok();
            init_observability();
        });
        UserApiImpl::new_mock()
            .await
            .expect("Failed to initialize Mock UserApiImpl")
    }

    #[rstest]
    #[case::empty_user_id("", "El ID de usuario está vacío")]
    #[case::uuid_v4(Uuid::new_v4().to_string(), "Se está usando un UUID v4 inválido")]
    #[tokio::test]
    async fn sign_up_fails_with_invalid_user_id(
        #[case] user_id: String,
        #[case] case_message: &str,
    ) -> TestResult {
        info!("Probando caso: {}", case_message);
        let request = Request::new(SignUpRequest {
            user_id,
            ..SIGN_UP_REQUEST.clone()
        });
        let user_api_impl = user_api_impl().await;
        let result = user_api_impl.sign_up(request).await;
        assert!(
            result.is_err(),
            "Se esperaba un error para: {}",
            case_message
        );
        info!(
            "Error detectado para '{}': {}",
            case_message,
            result.err().unwrap()
        );

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn user_service_server_tests() -> TestResult {
        let user_id = Uuid::now_v7().to_string();
        let request = Request::new(SignUpRequest {
            user_id: user_id.clone(),
            email: "miuler@gmail.com".to_string(),
            identifier: None,
            ..SIGN_UP_REQUEST.clone()
        });
        let user_api_impl = user_api_impl().await;
        let result = user_api_impl.sign_up(request).await;
        let response = result.expect("Fallo inesperado al ejecutar sign_up");
        let sign_up_response = response.get_ref();
        assert_eq!(sign_up_response.user_id, Some(user_id.clone()));

        let user = user_api_impl
            .user_repository
            .find_user_by_id(&user_id)
            .await?;
        assert_eq!(user.id.to_string(), user_id);

        Ok(())
    }
}
