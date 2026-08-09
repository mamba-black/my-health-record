use crate::infrastructure::grpc::SignUpRequest;
use crate::infrastructure::grpc::identifier::IdentifierType;
use crate::infrastructure::grpc::user_api_server::UserApi;
use crate::infrastructure::grpc::*;
use app_core::domain::error::ClickCareError;
use std::sync::Arc;
use tonic::*;
use tracing::debug;
use user::application::CreateUserUseCase;
use user::application::command::{CreateUserCommand, CreateUserError};
use user::domain::user::Identifier::DNI;
use user::infrastructure::di;
use user::infrastructure::di::DBType;

pub struct UserApiImpl {
    create_user_use_case: Arc<dyn CreateUserUseCase>,
}

impl UserApiImpl {
    pub async fn new(url: Option<String>) -> Result<UserApiImpl, ClickCareError> {
        let di = di::new(DBType::Postgres(url)).await?;
        Ok(Self {
            create_user_use_case: di.create_user_use_case,
        })
    }
}

#[async_trait]
impl UserApi for UserApiImpl {
    #[tracing::instrument(skip(self, sign_up_request))]
    async fn sign_up(
        &self,
        sign_up_request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        let sign_up_request = sign_up_request.into_inner();

        let command = CreateUserCommand {
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
            second_last_name: sign_up_request.second_family_name,
            phone: sign_up_request.phone,
            address: sign_up_request.address,
            birthdate: sign_up_request.birth_date,
            display_name: sign_up_request.display_name,
            create_clinic: sign_up_request.create_clinic,
            username: sign_up_request.email,
            password: "123".to_string(),
        };
        debug!("command: {:?}", command);

        let user_id = sign_up_request.user_id;
        self.create_user_use_case
            .execute(command)
            .await
            .map(|_a| {
                Response::new(SignUpResponse {
                    status: SignUpStatus::Success as i32,
                    message: "Usuario registrado exitosamente.".to_string(),
                    user_id: Some(user_id),
                    link: None,
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

#[cfg(test)]
mod test {
    use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
    use crate::infrastructure::grpc::user_api_server::UserApi;
    use crate::infrastructure::grpc::{SignUpRequest, SignUpResponse};
    use crate::infrastructure::log::init_logger;
    use app_core::domain::error::ClickCareError;
    use dotenvy::dotenv;
    use log::info;
    use rstest::{fixture, rstest};
    use std::sync::{Arc, LazyLock, Once};
    use tokio::sync::{Mutex, OnceCell};
    use tonic::{Request, Response, Status};
    use user::domain::repository::user_repository::UserRepository;
    use user::domain::user::Identifier;
    use user::infrastructure::di;
    use user::infrastructure::di::{DBType, DI, DIOverrides, MockUserRepositoryImpl};
    use uuid::Uuid;

    static INIT: Once = Once::new();
    static USER_API_INSTANCE: OnceCell<UserApiImpl> = OnceCell::const_new();
    static USER_DI: OnceCell<DI> = OnceCell::const_new();
    static USER_REPOSITORY_MOCK: OnceCell<Arc<MockUserRepositoryImpl>> = OnceCell::const_new();
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
    });

    type TestResult = Result<(), ClickCareError>;

    #[fixture]
    async fn user_api_impl() -> &'static UserApiImpl {
        INIT.call_once(|| {
            dotenv().ok();
            init_logger();
        });
        let user_repository = USER_REPOSITORY_MOCK
            .get_or_init(|| async {
                Arc::new(MockUserRepositoryImpl {
                    saved_users: Mutex::new(Vec::new()),
                })
            })
            .await;

        let user_di = USER_DI
            .get_or_init(|| async {
                let user_di = DIOverrides {
                    user_repository: Some(user_repository.clone()),
                    ..DIOverrides::default()
                };
                di::new_with_overrides(DBType::Mock, user_di)
                    .await
                    .expect("")
            })
            .await;

        USER_API_INSTANCE
            .get_or_init(|| async {
                UserApiImpl {
                    create_user_use_case: user_di.create_user_use_case.clone(),
                }
            })
            .await
    }

    #[rstest]
    #[case::empty_user_id("", "El ID de usuario está vacío")]
    #[case::uuid_v4(Uuid::new_v4().to_string(), "Se está usando un UUID v4 inválido")]
    #[tokio::test]
    async fn sign_up_fails_with_invalid_user_id(
        #[future(awt)] user_api_impl: &UserApiImpl,
        #[case] user_id: String,
        #[case] case_message: &str,
    ) -> TestResult {
        info!("Probando caso: {}", case_message);
        let request = Request::new(SignUpRequest {
            user_id,
            ..SIGN_UP_REQUEST.clone()
        });
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
    async fn user_service_server_tests(#[future(awt)] user_api_impl: &UserApiImpl) -> TestResult {
        let user_id = Uuid::now_v7().to_string();
        let request = Request::new(SignUpRequest {
            user_id: user_id.clone(),
            email: "miuler@gmail.com".to_string(),
            identifier: None,
            ..SIGN_UP_REQUEST.clone()
        });
        let result = user_api_impl.sign_up(request).await;

        match result {
            Ok(response) => {
                let sign_up_response = response.get_ref();
                assert_eq!(sign_up_response.user_id, Some(user_id.clone()));
            }
            Err(e) => {
                info!("error: {}", e);
                assert!(false, "Error inesperado: {}", e);
            }
        }

        let mock_user_repository = USER_REPOSITORY_MOCK.get().unwrap();
        let users = mock_user_repository.saved_users.lock().await;
        assert_eq!(users.len(), 1);
        let user = users.get(0).unwrap();
        assert_eq!(user.id.to_string(), user_id);

        Ok(())
    }
}
