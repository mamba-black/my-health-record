use crate::application::CreateUserUseCase;
use crate::domain::user::User;
use crate::infrastructure::repository::clinic_repository_impl::ClinicRepositoryImpl;
use crate::infrastructure::repository::emitter_impl::EmitterImpl;
use crate::infrastructure::repository::user_repository_impl::UserRepositoryImpl;
use tokio::sync::broadcast::channel;

impl Default for CreateUserUseCase {
    fn default() -> Self {
        Self {
            user_repository: Box::new(UserRepositoryImpl::default()),
            clinic_repository: Box::new(ClinicRepositoryImpl::default()),
        }
    }
}

impl Default for ClinicRepositoryImpl {
    fn default() -> Self {

        let (sender, _receiver) = channel::<User>(100);
        Self {
            user_emitter: Box::new(EmitterImpl{sender: sender}),
        }
    }
}
