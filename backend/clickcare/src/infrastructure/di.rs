use crate::application::create_user_usecase::CreateUserUseCase;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;

impl Default for CreateUserUseCase {
    fn default() -> Self {
        Self {
            user_repository: Box::new(UserRepositoryImpl::default()),
        }
    }
}
