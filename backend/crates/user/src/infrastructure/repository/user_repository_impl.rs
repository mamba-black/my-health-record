use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::User;
use app_core::domain::error::ClickCareError;

#[derive(Default)]
pub(crate) struct UserRepositoryImpl {
}

impl UserRepository for UserRepositoryImpl {
    fn find_user_by_id(&self, _user_id: &str) -> Option<User> {
        // Aquí iría la lógica para buscar un usuario en la base de datos
        None
    }

    fn save_user(&self, _user: &User) -> Result<(), ClickCareError> {
        // Aquí iría la lógica para guardar un usuario en la base de datos
        Ok(())
    }
}
