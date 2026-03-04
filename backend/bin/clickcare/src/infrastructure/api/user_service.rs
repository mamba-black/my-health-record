use crate::infrastructure::api::user_service_server::UserService;
use crate::infrastructure::api::SignUpRequest;
use crate::infrastructure::api::*;
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use tonic::*;
use user::application::dto::{CreateUserCommand, CrueateUserError};
use user::application::CreateUserUseCase;

impl UserServiceImpl {
    pub async fn new() -> Result<UserServiceImpl, ClickCareError> {
        let create_user_use_case = CreateUserUseCase::new().await?;
        Ok(Self { create_user_use_case })
    }
}

pub struct UserServiceImpl {
    create_user_use_case: CreateUserUseCase,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        let create_user_request = request.into_inner();
        let command = CreateUserCommand {
            username: create_user_request.email.clone(),
            email: create_user_request.email,
            password: "123".to_string(),
            document_id: create_user_request.document_id,
            document_type: create_user_request.document_type,
            create_clinic: create_user_request.create_clinic,
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
    use dotenvy::dotenv;
    use crate::infrastructure::api::user_service::UserServiceImpl;
    use crate::infrastructure::api::user_service_server::UserService;
    use crate::infrastructure::api::{SignUpRequest, SignUpResponse};
    use app_core::domain::error::ClickCareError;
    use tonic::Request;
    use user::domain::user::DocumentType;

    #[tokio::test]
    async fn user_service_server_tests() -> Result<(), ClickCareError> {
        dotenv().ok();

        let user_service_server = UserServiceImpl::new().await?;

        let request = Request::new(SignUpRequest {
            user_id: "xxxx".to_string(),
            email: "miuler@gmail.com".to_string(),
            document_id: "40404040".to_string(),
            document_type: DocumentType::DNI.to_string(),
            create_clinic: true,
        });

        let result = user_service_server.sign_up(request).await;
        assert!(result.is_ok());

        let sign_up_response = *(result.unwrap()).get_ref();
        assert_eq!(sign_up_response, SignUpResponse {});

        Ok(())
    }
}
