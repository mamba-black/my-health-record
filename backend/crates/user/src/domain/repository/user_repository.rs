use crate::domain::user::User;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_user_by_id(&self, user_id: &str) -> Result<User, ClickCareError>;
    async fn save_user(&self, user: &User) -> Result<(), ClickCareError>;
}
