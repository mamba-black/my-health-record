use crate::domain::user::User;
use app_core::domain::error::ClickCareError;

pub trait UserRepository {
    fn find_user_by_id(&self, user_id: &str) -> Option<User>;
    fn save_user(&self, user: &User) -> Result<(), ClickCareError>;
}
