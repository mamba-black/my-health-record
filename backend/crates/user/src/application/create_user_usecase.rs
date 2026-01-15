use crate::application::create_user_usecase::CrueateUserError::{UnknownError, UserAlreadyExists};
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::DocumentType::DNI;
use crate::domain::user::User;
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;

pub struct CreateUserUseCase {
    pub(crate) user_repository: Box<dyn UserRepository + Send + Sync>,
    pub(crate) clinic_repository: Box<dyn ClinicRepository + Send + Sync>,
}

impl UseCase for CreateUserUseCase {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
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
            document_number: command.document_id,
            create_clinic: command.create_clinic,
        };

        self.user_repository
            .save_user(&user)
            .map_err(|e| UnknownError(e))?;

        if user.create_clinic {
            self.clinic_repository
                .create_clinic_for_user(&user)
                .map_err(|e| UnknownError(ClickCareError {message: e}))?;
        }

        Ok(CreateUserResponse {})
    }
}


pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
    pub password: String,
    pub document_id: String,
    pub document_type: String,
    pub create_clinic: bool,
}

pub struct CreateUserResponse {}

pub enum CrueateUserError {
    UserAlreadyExists(ClickCareError),
    UnknownError(ClickCareError),
}


