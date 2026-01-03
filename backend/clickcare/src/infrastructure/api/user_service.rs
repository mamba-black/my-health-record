use crate::application::UseCase;
use crate::application::create_user_usecase::{
    CreateUserCommand, CreateUserUseCase, CrueateUserError,
};
use crate::infrastructure::api::SignUpRequest;
use crate::infrastructure::api::user_service_server::UserService;
use crate::infrastructure::api::*;
use tonic::*;

#[derive(Default)]
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
            .map(|a| Response::new(SignUpResponse {}))
            .map_err(|err| match err {
                CrueateUserError::UserAlreadyExists(e) => Status::already_exists(e.message),
                CrueateUserError::UnknownError(e) => Status::unknown(e.message),
            })
    }

    async fn sign_in(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::domain::user::DocumentType;
    use crate::infrastructure::api::user_service::UserServiceImpl;
    use crate::infrastructure::api::user_service_server::UserService;
    use crate::infrastructure::api::{SignUpRequest, SignUpResponse};
    use tonic::Request;

    #[tokio::test]
    async fn user_service_server_tests() {
        let user_service_server = UserServiceImpl::default();

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
    }
}
