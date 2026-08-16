use crate::application::create_user_usecase::command::CreateUserError::{
    UnknownError, UserAlreadyExists,
};
use crate::application::create_user_usecase::command::*;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::Identifier::DNI;
use crate::domain::user::User;
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{EventPublisher, UserCreatedEvent};
use async_trait::async_trait;
use log::error;
use std::sync::Arc;

pub trait CreateUserUseCase:
    UseCase<Command = CreateUserCommand, Response = CreateUserResponse, Error = CreateUserError>
{
}

pub(crate) struct CreateUserUseCaseImpl {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) event_publisher: Arc<dyn EventPublisher>,
}

impl CreateUserUseCase for CreateUserUseCaseImpl {}

#[async_trait]
impl UseCase for CreateUserUseCaseImpl {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
    type Error = CreateUserError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {
        let exist_user = match &command.identifier {
            Some(DNI(value)) => self
                .user_repository
                .exist_user_by_document("DNI", value)
                .await
                .map_err(|_e| {
                    UnknownError(ClickCareError::generic(format!(
                        "User with document ID {}",
                        value
                    )))
                })?,
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

        let user: Result<User, ClickCareError> = command.into();
        let user = user?;

        self.user_repository.save_user(&user).await?;

        // La clínica del propietario no se crea aquí: es `crates/administration` quien
        // la materializa al consumir este evento, dentro de su propio contexto acotado.
        let event = UserCreatedEvent {
            user_id: user.id,
            person: user.person.clone(),
            create_clinic: user.is_owner,
        };
        // El usuario ya está persistido: una caída de la cola no debe convertirse en
        // un error de registro para el cliente. Se reporta y se sigue adelante.
        if let Err(error) = self.event_publisher.publish_user_created(event).await {
            error!(
                "No se pudo publicar UserCreatedEvent para user_id={}: {}",
                user.id, error
            );
        }

        Ok(CreateUserResponse {
            user_id: user.id.to_string(),
        })
    }
}

pub mod command {
    use crate::application::create_user_usecase::command::CreateUserError::UnknownError;
    use crate::domain::user::{Identifier, User};
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

    impl From<CreateUserCommand> for Result<User, ClickCareError> {
        fn from(command: CreateUserCommand) -> Self {
            User::new(
                command.user_id,
                vec![command.first_name],
                command.last_name,
                command.second_last_name,
                command.identifier,
                command.create_clinic,
                command.email,
                Some(command.phone),
                Some(command.birthdate),
            )
        }
    }

    pub struct CreateUserResponse {
        pub user_id: String,
    }

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
