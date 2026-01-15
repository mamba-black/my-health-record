use crate::domain::user::User;

pub(crate) trait ClinicRepository {
    fn create_clinic_for_user(&self, user: &User,) -> Result<(), String>;
}
