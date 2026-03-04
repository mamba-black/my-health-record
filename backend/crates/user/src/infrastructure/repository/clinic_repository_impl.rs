use async_trait::async_trait;
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::user::User;
use app_core::domain::repository::emitter::Emitter;

pub(crate) struct ClinicRepositoryImpl {
    pub(crate) user_emitter: Box<dyn Emitter<User> + Send + Sync>,
}

#[async_trait]
impl ClinicRepository for ClinicRepositoryImpl {
    async fn create_clinic_for_user(&self, user: &User,) -> Result<(), String> {
        let _ = self.user_emitter.emit(user).await?;
        Ok(())
    }
}
