use crate::infrastructure::api::user_service_server::UserService;
use crate::infrastructure::api::*;
use tonic::*;

#[derive(Default)]
pub struct UserServiceImpl {}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(&self, request: Request<CreateUserRequest>) -> std::result::Result<Response<CreateUserResponse>, Status> {
        todo!()
    }
}
