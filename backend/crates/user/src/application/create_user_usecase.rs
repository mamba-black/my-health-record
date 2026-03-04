use crate::application::create_user_usecase::dto::CrueateUserError::{UnknownError, UserAlreadyExists};
use crate::application::create_user_usecase::dto::{CreateUserCommand, CreateUserResponse, CrueateUserError};
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::DocumentType::DNI;
use crate::domain::user::User;
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use log::error;

pub mod dto {
    use app_core::domain::error::ClickCareError;

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
}

pub struct CreateUserUseCase {
    pub(crate) user_repository: Box<dyn UserRepository + Send + Sync>,
    pub(crate) clinic_repository: Box<dyn ClinicRepository + Send + Sync>,
}

#[async_trait]
impl UseCase for CreateUserUseCase {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
    type Error = CrueateUserError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {

        let exist_user = self.user_repository
            .exist_user(command.document_id.as_str())
            .await
            .map_err(|e| UnknownError(ClickCareError::generic(format!("User with document ID {}", command.document_id))))?;
        if  exist_user {
            error!("User with document ID {} already exists", command.document_id);
            return Err(UserAlreadyExists(ClickCareError::generic(format!("User with document ID {} already exists", command.document_id))))
        }

        let user = User {
            name: command.username,
            document_type: DNI,
            document_number: command.document_id,
            is_owner: command.create_clinic,
        };

        self.user_repository
            .save_user(&user)
            .await
            .map_err(UnknownError)?;

        if user.is_owner {
            self.clinic_repository
                .create_clinic_for_user(&user)
                .await
                .map_err(|e| UnknownError(ClickCareError::generic(format!("Error en creoar la clinica para el usuario ({})", e))))?;
        }

        Ok(CreateUserResponse {})
    }
}


