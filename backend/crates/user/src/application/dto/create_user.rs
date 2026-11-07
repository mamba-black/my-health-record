use crate::application::dto::create_user::CreateUserError::UnknownError;
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
        .map(|mut user| {
            user.address = Some(command.address).filter(|address| !address.trim().is_empty());
            user.preferred_role = command.intent.preferred_role(command.create_clinic);
            user
        })
    }
}

impl From<ClickCareError> for CreateUserError {
    fn from(value: ClickCareError) -> Self {
        UnknownError(value)
    }
}
