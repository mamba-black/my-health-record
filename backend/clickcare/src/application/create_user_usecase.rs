use crate::application::create_user_usecase::CrueateUserError::UserAlreadyExists;
use crate::application::UseCase;
use crate::domain::error::ClickCareError;
use crate::domain::user::DocumentType::DNI;
use crate::domain::user::User;
use crate::domain::user_repository::UserRepository;

pub struct CreateUserUseCase {
    pub(crate) user_repository: Box<dyn UserRepository + Send + Sync>,
}

impl UseCase for CreateUserUseCase {
    type Command = CreateUserCommand;
    type Response = ();
    type Error = CrueateUserError;

    fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {
        let user = self.user_repository.find_user_by_id(command.document_id.as_str());

        if user.is_some() {
            return Err(UserAlreadyExists(ClickCareError{
                message: format!("User with document ID {} already exists", command.document_id),
            }));
        }

        let user = User {
            name: command.username,
            document_type: DNI,
            document_number: command.document_id
        };
        self.user_repository.save_user(user);

        Ok(())
    }
}


pub(crate) struct CreateUserCommand {
    pub username: String,
    pub email: String,
    pub password: String,
    pub document_id: String,
    pub document_type: String,
}

pub(crate) enum CrueateUserError {
    UserAlreadyExists(ClickCareError),
}

