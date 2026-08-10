use crate::application::create_user_usecase::command::CreateUserError::{
    UnknownError, UserAlreadyExists,
};
use crate::application::create_user_usecase::command::{
    CreateUserCommand, CreateUserError, CreateUserResponse,
};
use crate::domain::repository::clinic_repository::ClinicRepository;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::Identifier::DNI;
use crate::domain::user::{Identifier, User};
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use log::error;
use std::sync::Arc;

pub mod command {
    use crate::application::create_user_usecase::command::CreateUserError::UnknownError;
    use crate::domain::user::Identifier;
    use app_core::domain::error::ClickCareError;

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
        pub last_name: Option<String>,
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

    pub enum CreateUserError {
        UserAlreadyExists(ClickCareError),
        UnknownError(ClickCareError),
    }

    impl From<ClickCareError> for CreateUserError {
        fn from(value: ClickCareError) -> Self {
            UnknownError(value)
        }
    }
}

pub trait CreateUserUseCase:
    UseCase<Command = CreateUserCommand, Response = CreateUserResponse, Error = CreateUserError>
{
}

use app_core::domain::event::{EventPublisher, UserCreatedEvent};

pub(crate) struct CreateUserUseCaseImpl {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) clinic_repository: Arc<dyn ClinicRepository>,
    pub(crate) event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl CreateUserUseCase for CreateUserUseCaseImpl {}

#[async_trait]
impl UseCase for CreateUserUseCaseImpl {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
    type Error = CreateUserError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {
        let mut identifier: Option<Identifier> = None;
        let exist_user = match &command.identifier {
            Some(DNI(value)) => {
                identifier = Some(DNI(value.clone()));
                self.user_repository
                    .exist_user_by_document("DNI", value)
                    .await
                    .map_err(|_e| {
                        UnknownError(ClickCareError::generic(format!(
                            "User with document ID {}",
                            value
                        )))
                    })?
            }
            _ => false,
        };

        if exist_user {
            error!(
                "User with document ID {:?} already exists",
                command.identifier
            );
            let msg = format!(
                "User with document ID {:?} already exists",
                command.identifier
            );
            return Err(UserAlreadyExists(ClickCareError::generic(msg)));
        }

        let user = User::new(
            command.user_id,
            vec![command.first_name.clone()],
            command.last_name.clone(),
            command.second_last_name.clone(),
            identifier,
            command.create_clinic,
            command.email.clone(),
        )?;

        self.user_repository.save_user(&user).await?;

        if user.is_owner {
            self.clinic_repository
                .create_clinic_for_user(&user)
                .await
                .map_err(|e| {
                    UnknownError(ClickCareError::generic(format!(
                        "Error en creoar la clinica para el usuario ({})",
                        e
                    )))
                })?;
        }

        if let Some(publisher) = &self.event_publisher {
            let identifier_dni = match &command.identifier {
                Some(DNI(val)) => Some(val.clone()),
                _ => None,
            };
            let event = UserCreatedEvent {
                user_id: user.id,
                person_id: user.person.id,
                email: command.email,
                given_name: command.first_name,
                family_name: command.last_name,
                second_family_name: command.second_last_name,
                identifier_dni,
                phone: command.phone,
                birth_date: command.birthdate,
                create_clinic: command.create_clinic,
            };
            publisher.publish_user_created(event).await?;
        }

        Ok(CreateUserResponse {})
    }
}
