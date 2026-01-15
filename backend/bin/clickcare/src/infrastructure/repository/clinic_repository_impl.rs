use crate::domain::clinic_repository::ClinicRepository;
use crate::domain::user::User;
use crate::infrastructure::repository::emitter::Emitter;

pub(crate) struct ClinicRepositoryImpl {
    pub(crate) user_emitter: Box<dyn Emitter<User> + Send + Sync>,
}

impl ClinicRepository for ClinicRepositoryImpl {
    fn create_clinic_for_user(&self, user: &User,) -> Result<(), String> {
        self.user_emitter.emit(user);
        Ok(())
    }
}
