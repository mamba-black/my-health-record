use crate::application::create_user_usecase::model::CreateUserError::{
    UnknownError, UserAlreadyExists,
};
use crate::application::create_user_usecase::model::*;
use crate::domain::repository::user_repository::UserRepository;
use crate::domain::user::Identifier::DNI;
use crate::domain::user::{SignUpIntent, User};
use app_core::application::UseCase;
use app_core::domain::error::ClickCareError;
use app_core::domain::event::{EventPublisher, FounderRegistered, UserCreatedEvent};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

/// Caso de uso para registrar y autenticar un usuario en el sistema.
///
/// Valida la no existencia previa del documento de identidad y del correo electrónico
/// en el ámbito de la red médica (`network_id`), persiste la cuenta y emite el evento `UserCreatedEvent`.
pub type CreateUserUseCase = dyn UseCase<Command = CreateUserCommand, Response = CreateUserResponse, Error = CreateUserError>;

pub(crate) struct CreateUserUseCaseImpl {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) event_publisher: Arc<dyn EventPublisher>,
}

#[async_trait]
impl UseCase for CreateUserUseCaseImpl {
    type Command = CreateUserCommand;
    type Response = CreateUserResponse;
    type Error = CreateUserError;

    async fn execute(&self, command: Self::Command) -> Result<Self::Response, Self::Error> {
        let user_exists_by_document = match &command.identifier {
            Some(DNI(value)) => self
                .user_repository
                .exist_user_by_document(&command.network_id, DNI(value.clone()))
                .await
                .map_err(|_e| {
                    UnknownError(ClickCareError::generic(format!(
                        "User with document ID {}",
                        value
                    )))
                })?,
            _ => false,
        };

        if user_exists_by_document {
            error!(
                "User with document ID {:?} already exists in network {}",
                command.identifier, command.network_id
            );
            let msg = format!(
                "User with document ID {:?} already exists in network {}",
                command.identifier, command.network_id
            );
            return Err(UserAlreadyExists(ClickCareError::generic(msg)));
        }

        let email_already_registered = self
            .user_repository
            .exist_user_by_email(&command.network_id, &command.email)
            .await
            .map_err(|_e| {
                UnknownError(ClickCareError::generic(format!(
                    "Error verifying email {}",
                    command.email
                )))
            })?;

        if email_already_registered {
            error!(
                "User with email {} already exists in network {}",
                command.email, command.network_id
            );
            let msg = format!(
                "User with email {} already exists in network {}",
                command.email, command.network_id
            );
            return Err(UserAlreadyExists(ClickCareError::generic(msg)));
        }

        let is_clinic_owner = command.intent == SignUpIntent::ClinicOwner
            || (command.intent == SignUpIntent::Unspecified && command.create_clinic);

        let user: Result<User, ClickCareError> = command.into();
        let user = user?;
        let user_id = user.id;

        self.user_repository.save_user(&user).await?;

        // El usuario ya está persistido: una caída de la cola no debe convertirse en
        // un error de registro para el cliente. Se reporta y se sigue adelante.
        let user_created_event: UserCreatedEvent = user.clone().into();
        if let Err(error) = self
            .event_publisher
            .publish_user_created(user_created_event)
            .await
        {
            // La clínica del propietario no se crea aquí: es `crates/administration` quien
            // la materializa al consumir este evento, dentro de su propio contexto acotado.
            error!(
                "No se pudo publicar UserCreatedEvent para user_id={}: {}",
                user_id, error
            );
        }

        // Emisión condicional de FounderRegistered si la intención es dueño de clínica
        let organization_id = if is_clinic_owner {
            let org_id = Uuid::now_v7();
            let founder_event = FounderRegistered {
                user_id: user.id,
                person_id: user.person.id,
                organization_id: org_id,
            };
            if let Err(error) = self
                .event_publisher
                .publish_founder_registered(founder_event)
                .await
            {
                error!(
                    "No se pudo publicar FounderRegistered para user_id={}, org_id={}: {}",
                    user_id, org_id, error
                );
            }
            Some(org_id)
        } else {
            None
        };

        Ok(CreateUserResponse {
            user_id: user_id.to_string(),
            organization_id: organization_id.map(|id| id.to_string()),
        })
    }
}

pub mod model {
    use crate::application::create_user_usecase::model::CreateUserError::UnknownError;
    use crate::domain::user::{Identifier, SignUpIntent, User};
    use app_core::domain::error::ClickCareError;
    use uuid::Uuid;

    /// Parámetros de entrada para el caso de uso de registro de usuario (`CreateUserUseCase`).
    #[derive(Debug, Clone)]
    pub struct CreateUserCommand {
        /// Red médica en la que se registra el usuario (UUID v7).
        pub network_id: Uuid,
        /// Token de identidad emitido por el proveedor OAuth/OIDC.
        pub id_token: String,
        /// Identificador único del usuario (UUID v7).
        pub user_id: String,
        /// Identificador del usuario provisto por el proveedor OAuth (e.g. Google sub).
        pub provider_id: String,
        /// Nombre del proveedor de identidad (e.g. "Google").
        pub provider_name: String,
        /// URL del avatar provisto por el proveedor de identidad.
        pub provider_avatar_url: Option<String>,
        /// Correo electrónico principal del usuario.
        pub email: String,
        /// Documento de identidad oficial (ej. DNI, Pasaporte).
        pub identifier: Option<Identifier>,
        /// Primer nombre de la persona.
        pub first_name: String,
        /// Primer apellido o apellido paterno.
        pub last_name: Option<String>,
        /// Segundo apellido o apellido materno.
        pub second_family_name: Option<String>,
        /// Número telefónico de contacto.
        pub phone: String,
        /// Dirección de residencia física.
        pub address: String,
        /// Fecha de nacimiento en formato ISO 8601 (YYYY-MM-DD).
        pub birthdate: String,
        /// Nombre para mostrar o alias preferido.
        pub display_name: Option<String>,
        /// Indica si el usuario solicita crear una clínica como propietario.
        pub create_clinic: bool,
        /// Intención declarada de perfil durante el registro preliminar.
        pub intent: SignUpIntent,
        /// Nombre de usuario del sistema.
        pub username: String,
        /// Contraseña asociada a la cuenta (cuando aplique).
        pub password: String,
    }

    /// Resultado exitoso tras ejecutar `CreateUserUseCase`.
    #[derive(Debug, Clone)]
    pub struct CreateUserResponse {
        /// Identificador del usuario registrado (UUID v7).
        pub user_id: String,
        /// Identificador de la clínica creada si correspondía a un propietario (opcional).
        pub organization_id: Option<String>,
    }

    /// Posibles errores que pueden ocurrir durante la ejecución de `CreateUserUseCase`.
    #[derive(Debug)]
    pub enum CreateUserError {
        /// El usuario ya existe en la red médica indicada (colisión de correo o documento).
        UserAlreadyExists(ClickCareError),
        /// Error inesperado o fallo de infraestructura.
        UnknownError(ClickCareError),
    }

    impl From<CreateUserCommand> for Result<User, ClickCareError> {
        fn from(command: CreateUserCommand) -> Self {
            let is_owner = command.intent == SignUpIntent::ClinicOwner
                || (command.intent == SignUpIntent::Unspecified && command.create_clinic);
            User::new(
                command.user_id,
                command.network_id,
                vec![command.first_name],
                command.last_name,
                command.second_family_name,
                command.identifier,
                is_owner,
                command.email,
                Some(command.phone),
                Some(command.birthdate),
            )
        }
    }

    impl From<ClickCareError> for CreateUserError {
        fn from(value: ClickCareError) -> Self {
            UnknownError(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::di::MockUserRepositoryImpl;
    use app_core::domain::event::DomainEvent;
    use std::sync::Mutex as StdMutex;
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct RecordingEventPublisher {
        user_created_events: StdMutex<Vec<UserCreatedEvent>>,
        founder_registered_events: StdMutex<Vec<FounderRegistered>>,
    }

    #[async_trait]
    impl EventPublisher for RecordingEventPublisher {
        async fn publish_user_created(
            &self,
            event: UserCreatedEvent,
        ) -> Result<(), ClickCareError> {
            self.user_created_events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_founder_registered(
            &self,
            event: FounderRegistered,
        ) -> Result<(), ClickCareError> {
            self.founder_registered_events.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn sample_command(intent: SignUpIntent, create_clinic: bool) -> CreateUserCommand {
        sample_command_with_network(Uuid::now_v7(), intent, create_clinic)
    }

    fn sample_command_with_network(
        network_id: Uuid,
        intent: SignUpIntent,
        create_clinic: bool,
    ) -> CreateUserCommand {
        CreateUserCommand {
            network_id,
            id_token: "mock-token".to_string(),
            user_id: Uuid::now_v7().to_string(),
            provider_id: "google.com".to_string(),
            provider_name: "Google".to_string(),
            provider_avatar_url: None,
            email: format!("user-{}@example.com", Uuid::now_v7()),
            identifier: None,
            first_name: "Juan".to_string(),
            last_name: Some("Pérez".to_string()),
            second_family_name: None,
            phone: "987654321".to_string(),
            address: "Av. Principal 123".to_string(),
            birthdate: "1990-01-01".to_string(),
            display_name: None,
            create_clinic,
            intent,
            username: "juanperez".to_string(),
            password: "password123".to_string(),
        }
    }

    #[tokio::test]
    async fn test_sign_up_clinic_owner_emits_founder_registered_and_generates_organization_id() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let cmd = sample_command(SignUpIntent::ClinicOwner, false);
        let user_id = cmd.user_id.clone();
        let res = use_case.execute(cmd).await.expect("execute failed");

        assert_eq!(res.user_id, user_id);
        assert!(
            res.organization_id.is_some(),
            "organization_id must be generated for clinic owner"
        );
        let org_id_str = res.organization_id.unwrap();
        let org_uuid = Uuid::parse_str(&org_id_str).expect("organization_id must be valid UUID");
        assert_eq!(
            org_uuid.get_version(),
            Some(uuid::Version::SortRand),
            "organization_id must be UUIDv7"
        );

        let founder_events = publisher.founder_registered_events.lock().unwrap();
        assert_eq!(founder_events.len(), 1);
        assert_eq!(founder_events[0].user_id.to_string(), user_id);
        assert_eq!(founder_events[0].organization_id, org_uuid);
        assert_eq!(founder_events[0].event_name(), FounderRegistered::QUEUE);

        let user_events = publisher.user_created_events.lock().unwrap();
        assert_eq!(user_events.len(), 1);
        assert_eq!(user_events[0].user_id.to_string(), user_id);
    }

    #[tokio::test]
    async fn test_sign_up_practitioner_does_not_emit_founder_registered() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let cmd = sample_command(SignUpIntent::Practitioner, false);
        let user_id = cmd.user_id.clone();
        let res = use_case.execute(cmd).await.expect("execute failed");

        assert_eq!(res.user_id, user_id);
        assert!(
            res.organization_id.is_none(),
            "organization_id must be None for practitioner"
        );

        let founder_events = publisher.founder_registered_events.lock().unwrap();
        assert!(
            founder_events.is_empty(),
            "FounderRegistered must not be emitted for practitioner"
        );

        let user_events = publisher.user_created_events.lock().unwrap();
        assert_eq!(user_events.len(), 1);
    }

    #[tokio::test]
    async fn test_sign_up_patient_does_not_emit_founder_registered() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let cmd = sample_command(SignUpIntent::Patient, false);
        let user_id = cmd.user_id.clone();
        let res = use_case.execute(cmd).await.expect("execute failed");

        assert_eq!(res.user_id, user_id);
        assert!(
            res.organization_id.is_none(),
            "organization_id must be None for patient"
        );

        let founder_events = publisher.founder_registered_events.lock().unwrap();
        assert!(
            founder_events.is_empty(),
            "FounderRegistered must not be emitted for patient"
        );

        let user_events = publisher.user_created_events.lock().unwrap();
        assert_eq!(user_events.len(), 1);
    }

    #[tokio::test]
    async fn test_sign_up_legacy_create_clinic_emits_founder_registered() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let cmd = sample_command(SignUpIntent::Unspecified, true);
        let user_id = cmd.user_id.clone();
        let res = use_case.execute(cmd).await.expect("execute failed");

        assert_eq!(res.user_id, user_id);
        assert!(
            res.organization_id.is_some(),
            "organization_id must be generated for legacy create_clinic = true"
        );

        let founder_events = publisher.founder_registered_events.lock().unwrap();
        assert_eq!(founder_events.len(), 1);
        assert_eq!(founder_events[0].user_id.to_string(), user_id);
    }

    #[tokio::test]
    async fn test_sign_up_same_email_different_networks_succeeds() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let email = "doctor@example.com".to_string();
        let network_a = Uuid::now_v7();
        let network_b = Uuid::now_v7();

        let mut cmd_a = sample_command_with_network(network_a, SignUpIntent::Patient, false);
        cmd_a.email = email.clone();
        let res_a = use_case.execute(cmd_a).await;
        assert!(
            res_a.is_ok(),
            "User in network A must be created successfully"
        );

        let mut cmd_b = sample_command_with_network(network_b, SignUpIntent::Patient, false);
        cmd_b.email = email.clone();
        let res_b = use_case.execute(cmd_b).await;
        assert!(
            res_b.is_ok(),
            "Same email in independent network B must be created successfully"
        );
    }

    #[tokio::test]
    async fn test_sign_up_same_email_same_network_fails() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let email = "doctor@example.com".to_string();
        let network_id = Uuid::now_v7();

        let mut cmd1 = sample_command_with_network(network_id, SignUpIntent::Patient, false);
        cmd1.email = email.clone();
        let res1 = use_case.execute(cmd1).await;
        assert!(res1.is_ok());

        let mut cmd2 = sample_command_with_network(network_id, SignUpIntent::Patient, false);
        cmd2.email = email.clone();
        let res2 = use_case.execute(cmd2).await;
        assert!(
            matches!(res2, Err(CreateUserError::UserAlreadyExists(_))),
            "Duplicate email in same network must be rejected"
        );
    }

    #[tokio::test]
    async fn test_sign_up_same_document_same_network_fails() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let network_id = Uuid::now_v7();
        let mut cmd1 = sample_command_with_network(network_id, SignUpIntent::Patient, false);
        cmd1.identifier = Some(DNI("12345678".to_string()));
        let res1 = use_case.execute(cmd1).await;
        assert!(res1.is_ok());

        let mut cmd2 = sample_command_with_network(network_id, SignUpIntent::Patient, false);
        cmd2.identifier = Some(DNI("12345678".to_string()));
        let res2 = use_case.execute(cmd2).await;
        assert!(
            matches!(res2, Err(CreateUserError::UserAlreadyExists(_))),
            "Duplicate document in same network must be rejected"
        );
    }

    #[tokio::test]
    async fn test_sign_up_same_document_different_networks_succeeds() {
        let repo = Arc::new(MockUserRepositoryImpl {
            saved_users: Mutex::new(Vec::new()),
        });
        let publisher = Arc::new(RecordingEventPublisher::default());
        let use_case = CreateUserUseCaseImpl {
            user_repository: repo.clone(),
            event_publisher: publisher.clone(),
        };

        let network_a = Uuid::now_v7();
        let network_b = Uuid::now_v7();

        let mut cmd1 = sample_command_with_network(network_a, SignUpIntent::Patient, false);
        cmd1.identifier = Some(DNI("12345678".to_string()));
        let res1 = use_case.execute(cmd1).await;
        assert!(res1.is_ok());

        let mut cmd2 = sample_command_with_network(network_b, SignUpIntent::Patient, false);
        cmd2.identifier = Some(DNI("12345678".to_string()));
        let res2 = use_case.execute(cmd2).await;
        assert!(
            res2.is_ok(),
            "Same document in different networks must succeed"
        );
    }
}
