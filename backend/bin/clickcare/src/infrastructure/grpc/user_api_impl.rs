use crate::infrastructure::grpc::user_api_server::UserApi;
use crate::infrastructure::grpc::SignUpRequest;
use crate::infrastructure::grpc::*;
use app_core::domain::error::ClickCareError;
use tonic::*;
use user::application::dto::{CreateUserCommand, CrueateUserError};
use user::application::CreateUserUseCase;
use user::infrastructure::di;
use user::infrastructure::di::DBType;

pub struct UserApiImpl {
    create_user_use_case: Box<dyn CreateUserUseCase>,
}

impl UserApiImpl {
    pub async fn new() -> Result<UserApiImpl, ClickCareError> {
        let create_user_use_case = di::new(DBType::Postgres).await?;
        Ok(Self { create_user_use_case })
    }
}

#[async_trait]
impl UserApi for UserApiImpl {
    async fn sign_up(
        &self,
        sign_up_request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        let sign_up_request = sign_up_request.into_inner();

        let command = CreateUserCommand {
            id_token: sign_up_request.id_token,
            user_id: sign_up_request.user_id,
            provider_id: sign_up_request.provider_id,
            provider_name: sign_up_request.provider_name,
            provider_avatar_url: sign_up_request.provider_avatar_url,
            email: sign_up_request.email.clone(),
            document_type: sign_up_request.document_type,
            document_id: sign_up_request.document_id,
            first_name: sign_up_request.first_name,
            last_name: sign_up_request.last_name,
            second_last_name: sign_up_request.second_last_name,
            phone: sign_up_request.phone,
            address: sign_up_request.address,
            birthdate: sign_up_request.birthdate,
            display_name: sign_up_request.display_name,
            create_clinic: sign_up_request.create_clinic,
            username: sign_up_request.email,
            password: "123".to_string(),
        };

        self.create_user_use_case
            .execute(command)
            .await
            .map(|_a| Response::new(SignUpResponse {}))
            .map_err(|err| match err {
                CrueateUserError::UserAlreadyExists(e) => Status::already_exists(e.to_string()),
                CrueateUserError::UnknownError(e) => Status::unknown(e.to_string()),
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
    static INIT: Once = Once::new();

    use crate::infrastructure::grpc::user_api_impl::UserApiImpl;
    use crate::infrastructure::grpc::user_api_server::UserApi;
    use crate::infrastructure::grpc::{SignUpRequest, SignUpResponse};
    use crate::infrastructure::log::init_logger;
    use app_core::domain::error::ClickCareError;
    use dotenvy::dotenv;
    use log::info;
    use rstest::{fixture, rstest};
    use std::sync::Once;
    use tonic::Request;
    use user::domain::user::DocumentType;
    use uuid::Uuid;
    use user::infrastructure::di;
    use user::infrastructure::di::DBType;

    type TestResult = Result<(), ClickCareError>;

    #[fixture]
    async fn user_api_impl() -> UserApiImpl {
        INIT.call_once(|| {
            dotenv().ok();
            init_logger();
        });
        let create_user_use_case = di::new(DBType::Mock)
            .await
            .expect("Error al crear el UserApiImpl");
        UserApiImpl { create_user_use_case }
    }

    #[rstest]
    #[tokio::test]
    async fn test_sign_up_uuid_error(#[future(awt)] user_api_impl: UserApiImpl) -> TestResult {

        let request = Request::new(SignUpRequest{
            id_token: "".to_string(),
            user_id: "".to_string(),
            provider_id: "".to_string(),
            provider_name: "".to_string(),
            provider_avatar_url: None,
            email: "".to_string(),
            document_type: "".to_string(),
            document_id: "".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            second_last_name: None,
            phone: "".to_string(),
            address: "".to_string(),
            birthdate: "".to_string(),
            display_name: None,
            create_clinic: false,
        });
        let result = user_api_impl.sign_up(request).await;
        assert!(result.is_err());
        info!("error: {}", result.err().unwrap());

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_sign_up_uuid_v4_error(#[future(awt)] user_api_impl: UserApiImpl) -> TestResult {

        let request = Request::new(SignUpRequest{
            id_token: "".to_string(),
            user_id: Uuid::new_v4().to_string(),
            provider_id: "".to_string(),
            provider_name: "".to_string(),
            provider_avatar_url: None,
            email: "".to_string(),
            document_type: "".to_string(),
            document_id: "".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            second_last_name: None,
            phone: "".to_string(),
            address: "".to_string(),
            birthdate: "".to_string(),
            display_name: None,
            create_clinic: false,
        });
        let result = user_api_impl.sign_up(request).await;
        assert!(result.is_err());
        info!("error: {}", result.err().unwrap());

        Ok(())
    }

    #[ignore]
    #[rstest]
    #[tokio::test]
    async fn user_service_server_tests(#[future(awt)] user_api_impl: UserApiImpl) -> TestResult {

        let request = Request::new(SignUpRequest {
            id_token: "".to_string(),
            user_id: "xxxx".to_string(),
            provider_id: "".to_string(),
            provider_name: "".to_string(),
            provider_avatar_url: None,
            email: "miuler@gmail.com".to_string(),
            document_id: "40404040".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            second_last_name: None,
            phone: "".to_string(),
            address: "".to_string(),
            birthdate: "".to_string(),
            document_type: DocumentType::DNI.to_string(),
            create_clinic: true,
            display_name: None,
        });

        let result = user_api_impl.sign_up(request).await;
        assert!(result.is_ok());

        let sign_up_response = *(result.unwrap()).get_ref();
        assert_eq!(sign_up_response, SignUpResponse {});

        Ok(())
    }
}
