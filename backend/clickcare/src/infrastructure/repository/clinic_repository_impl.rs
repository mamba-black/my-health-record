use crate::domain::clinic_repository::ClinicRepository;
use crate::domain::user::User;

#[derive(Default)]
pub(crate) struct ClinicRepositoryImpl {}

impl ClinicRepository for ClinicRepositoryImpl {
    fn create_clinic_for_user(&self, user: &User,) -> Result<(), String> {
        // Implementation goes here
        Ok(())
    }
}
