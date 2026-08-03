use crate::domain::user::User;
use async_trait::async_trait;

#[async_trait]
pub trait ClinicRepository: Send + Sync {
    async fn create_clinic_for_user(&self, user: &User) -> Result<(), String>;
}
