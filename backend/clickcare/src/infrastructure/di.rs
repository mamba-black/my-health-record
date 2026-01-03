use crate::application::create_user_usecase::CreateUserUseCase;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;

impl Default for CreateUserUseCase {
    fn default() -> Self {
        Self {
            user_repository: Box::new(UserRepositoryImpl::default()),
            clinic_repository: Box::new(ClinicRepositoryImpl::default()),
        }
    }
}
