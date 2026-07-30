use std::sync::Arc;
use crate::application::create_user_usecase::command::CrueateUserError::{UnknownError, UserAlreadyExists};
use crate::application::create_user_usecase::command::{CreateUserCommand, CreateUserResponse, CrueateUserError};
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::Identifier::DNI;
use crate::domain::user::{Identifier, User};
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use log::error;

pub mod command {
    use app_core::domain::error::ClickCareError;
    use crate::application::command::CrueateUserError::UnknownError;
    use crate::domain::user::Identifier;

    #[derive(Debug)]
    pub struct CreateUserCommand {
        pub id_token: String,
        pub user_id: String,
        pub provider_id: String,
        pub provider_name: String,
        pub provider_avatar_url: Option<String>,
        pub email: String,
        pub identifier: Option<Identifier>,
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

pub trait CreateUserUseCase: UseCase<
    Command = CreateUserCommand,
    Response = CreateUserResponse,
    Error = CrueateUserError,
> {}

pub(crate) struct CreateUserUseCaseImpl {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) clinic_repository: Arc<dyn ClinicRepository>,
}

impl CreateUserUseCase for CreateUserUseCaseImpl {}

#[async_trait]
impl UseCase for CreateUserUseCaseImpl {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
    type Error = CrueateUserError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {

        let mut identifier: Option<Identifier> = None;
        let exist_user = match &command.identifier {
            Some(DNI(value)) => {
                identifier = Some(DNI(value.clone()));
                self.user_repository
                    .exist_user_by_document("DNI", value)
                    .await
                    .map_err(|e| UnknownError(ClickCareError::generic(format!("User with document ID {}", value))))?
            },
            _ => false
        };

        if exist_user {
            error!("User with document ID {:?} already exists", command.identifier);
            let msg = format!("User with document ID {:?} already exists", command.identifier);
            return Err(UserAlreadyExists(ClickCareError::generic(msg)))
        }

        let user = User::new(
            command.user_id,
            vec![command.username],
            command.first_name,
            command.last_name,
            identifier,
            command.create_clinic,
            command.email,
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


