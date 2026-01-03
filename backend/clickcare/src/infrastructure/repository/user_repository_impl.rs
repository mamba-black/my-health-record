use crate::domain::error::ClickCareError;
use crate::domain::user::User;
use crate::domain::user_repository::UserRepository;

#[derive(Default)]
pub(crate) struct UserRepositoryImpl {
}

impl UserRepository for UserRepositoryImpl {
    fn find_user_by_id(&self, user_id: &str) -> Option<User> {
        // Aquí iría la lógica para buscar un usuario en la base de datos
        None
    }

    fn save_user(&self, user: &User) -> Result<(), ClickCareError> {
        // Aquí iría la lógica para guardar un usuario en la base de datos
        Ok(())
    }
}
