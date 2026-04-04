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
    use crate::application::dto::CrueateUserError::UnknownError;

    pub struct CreateUserCommand {
        pub id_token: String,
        pub user_id: String,
        pub provider_id: String,
        pub provider_name: String,
        pub provider_avatar_url: Option<String>,
        pub email: String,
        pub document_type: String,
        pub document_id: String,
        pub first_name: String,
        pub last_name: String,
        pub second_last_name: Option<String>,
        pub phone: String,
        pub address: String,
        pub birthdate: String,
        pub display_name: Option<String>,
        pub create_clinic: bool,
        pub username: String,
        pub password: String,
    }

    pub struct CreateUserResponse {}

    pub enum CrueateUserError {
        UserAlreadyExists(ClickCareError),
        UnknownError(ClickCareError),
    }

    impl From<ClickCareError> for CrueateUserError {
        fn from(value: ClickCareError) -> Self {
            UnknownError(value)
        }
    }
}

pub(crate) struct CreateUserUseCaseImpl {
    pub(crate) user_repository: Box<dyn UserRepository + Send + Sync>,
    pub(crate) clinic_repository: Box<dyn ClinicRepository + Send + Sync>,
}

pub trait CreateUserUseCase: UseCase<
    Command = CreateUserCommand,
    Response = CreateUserResponse,
    Error = CrueateUserError,
> {}

impl CreateUserUseCase for CreateUserUseCaseImpl {}

#[async_trait]
impl UseCase for CreateUserUseCaseImpl {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
    type Error = CrueateUserError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {

        let exist_user = self.user_repository
            .exist_user(command.document_id.as_str())
            .await
            .map_err(|_| UnknownError(ClickCareError::generic(format!("User with document ID {}", command.document_id))))?;
        if  exist_user {
            error!("User with document ID {} already exists", command.document_id);
            return Err(UserAlreadyExists(ClickCareError::generic(format!("User with document ID {} already exists", command.document_id))))
        }

        let user = User::new(
            command.user_id,
            command.username,
            DNI,
            command.document_id,
            command.create_clinic,
            command.email,
            command.first_name,
            command.last_name,
        )?;

        self.user_repository
            .save_user(&user)
            .await?;

        if user.is_owner {
            self.clinic_repository
                .create_clinic_for_user(&user)
                .await
                .map_err(|e| UnknownError(ClickCareError::generic(format!("Error en creoar la clinica para el usuario ({})", e))))?;
        }

        Ok(CreateUserResponse {})
    }
}


