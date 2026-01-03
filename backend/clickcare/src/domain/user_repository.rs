use crate::domain::error::ClickCareError;
use crate::domain::user::User;

pub trait UserRepository {
    fn find_user_by_id(&self, user_id: &str) -> Option<User>;
    fn save_user(&self, user: User) -> Result<(), ClickCareError>;
}
