use crate::application::dto::complete_profile::{CompleteProfileCommand, CompleteProfileError};
use crate::application::integration::service::token_service::TokenService;
use crate::domain::user::User;
use crate::domain::user::user_repository::{ProfileUpdate, UserRepository};
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use std::sync::Arc;

// Tamaños de las columnas de `identity.user_account`.
const GIVEN_NAME_MAX_CHARS: usize = 100;
const FAMILY_NAME_MAX_CHARS: usize = 50;
const PHONE_MAX_CHARS: usize = 20;
const ADDRESS_MAX_CHARS: usize = 255;
const DNI_DIGITS: usize = 8;

/// Caso de uso para completar los datos de una cuenta existente.
///
/// La cuenta se obtiene del correo del token verificado, nunca de un dato enviado por el
/// cliente: así nadie puede modificar la cuenta de otra persona.
pub struct CompleteProfileUseCase {
    pub(crate) user_repository: Arc<dyn UserRepository>,
    pub(crate) token_service: Arc<dyn TokenService>,
}

impl CompleteProfileUseCase {
    pub async fn execute(
        &self,
        command: CompleteProfileCommand,
    ) -> Result<User, CompleteProfileError> {
        let profile = validated_profile(&command)?;

        let email = self
            .token_service
            .validate_and_extract_email(&command.id_token)
            .await?;
        let user = self.find_user(&command, &email).await?;

        if let Some(identifier) = &profile.identifier
            && user.person.identifier() != &Some(identifier.clone())
            && self
                .user_repository
                .exist_user_by_document(&command.network_id, identifier.clone())
                .await?
        {
            return Err(CompleteProfileError::DocumentAlreadyRegistered(
                ClickCareError::generic("El DNI ya está registrado en otra cuenta".to_string()),
            ));
        }

        self.user_repository
            .update_profile(&user.id, &profile)
            .await?;

        self.find_user(&command, &email).await
    }

    async fn find_user(
        &self,
        command: &CompleteProfileCommand,
        email: &str,
    ) -> Result<User, CompleteProfileError> {
        self.user_repository
            .find_user_by_email(&command.network_id, email)
            .await?
            .ok_or_else(|| {
                CompleteProfileError::UserNotFound(ClickCareError::generic(format!(
                    "No existe cuenta registrada para el correo {email}"
                )))
            })
    }
}

fn validated_profile(
    command: &CompleteProfileCommand,
) -> Result<ProfileUpdate, CompleteProfileError> {
    let birth_date = required_text(&command.birth_date, "fecha de nacimiento", usize::MAX)?;
    let parsed_birth_date: jiff::civil::Date = birth_date.parse().map_err(|_| {
        CompleteProfileError::InvalidInput(
            "La fecha de nacimiento debe tener el formato AAAA-MM-DD".to_string(),
        )
    })?;
    if parsed_birth_date > jiff::Zoned::now().date() {
        return Err(CompleteProfileError::InvalidInput(
            "La fecha de nacimiento no puede ser futura".to_string(),
        ));
    }

    Ok(ProfileUpdate {
        given_name: required_text(&command.given_name, "nombres", GIVEN_NAME_MAX_CHARS)?
            .to_string(),
        family_name: optional_text(
            command.family_name.as_deref(),
            "primer apellido",
            FAMILY_NAME_MAX_CHARS,
        )?,
        second_family_name: optional_text(
            command.second_family_name.as_deref(),
            "segundo apellido",
            FAMILY_NAME_MAX_CHARS,
        )?,
        identifier: validated_identifier(command.identifier.as_ref())?,
        phone: required_text(&command.phone, "teléfono", PHONE_MAX_CHARS)?.to_string(),
        birth_date: birth_date.to_string(),
        address: optional_text(command.address.as_deref(), "dirección", ADDRESS_MAX_CHARS)?,
    })
}

fn validated_identifier(
    identifier: Option<&Identifier>,
) -> Result<Option<Identifier>, CompleteProfileError> {
    let Some(Identifier::DNI(dni)) = identifier else {
        return Ok(None);
    };
    let dni = dni.trim();
    if dni.is_empty() {
        return Ok(None);
    }
    if dni.len() != DNI_DIGITS || !dni.chars().all(|digit| digit.is_ascii_digit()) {
        return Err(CompleteProfileError::InvalidInput(format!(
            "El DNI debe tener {DNI_DIGITS} dígitos"
        )));
    }
    Ok(Some(Identifier::DNI(dni.to_string())))
}

fn required_text<'a>(
    value: &'a str,
    label: &str,
    max_chars: usize,
) -> Result<&'a str, CompleteProfileError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CompleteProfileError::InvalidInput(format!(
            "El campo {label} es obligatorio"
        )));
    }
    if value.chars().count() > max_chars {
        return Err(CompleteProfileError::InvalidInput(format!(
            "El campo {label} no puede tener más de {max_chars} caracteres"
        )));
    }
    Ok(value)
}

fn optional_text(
    value: Option<&str>,
    label: &str,
    max_chars: usize,
) -> Result<Option<String>, CompleteProfileError> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => required_text(value, label, max_chars).map(|v| Some(v.to_string())),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::sign_in::SignInError;
    use crate::domain::user::RequiredProfileField;
    use crate::testing::MockUserRepositoryImpl;
    use async_trait::async_trait;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    const EMAIL: &str = "paciente@example.com";

    /// Identifica siempre a `EMAIL`, salvo que el token sea `"invalido"`.
    struct FixedTokenService;

    #[async_trait]
    impl TokenService for FixedTokenService {
        async fn validate_and_extract_email(&self, id_token: &str) -> Result<String, SignInError> {
            if id_token == "invalido" {
                return Err(SignInError::InvalidToken(ClickCareError::generic(
                    "token inválido".to_string(),
                )));
            }
            Ok(EMAIL.to_string())
        }
    }

    fn incomplete_user(network_id: Uuid, family_name: Option<&str>) -> User {
        User::new(
            Uuid::now_v7().to_string(),
            network_id,
            vec![String::new()],
            family_name.map(str::to_string),
            None,
            None,
            false,
            EMAIL.to_string(),
            None,
            None,
        )
        .expect("el id es un UUID v7")
    }

    fn use_case(users: Vec<User>) -> CompleteProfileUseCase {
        CompleteProfileUseCase {
            user_repository: Arc::new(MockUserRepositoryImpl {
                saved_users: Mutex::new(users),
            }),
            token_service: Arc::new(FixedTokenService),
        }
    }

    fn command(network_id: Uuid) -> CompleteProfileCommand {
        CompleteProfileCommand {
            network_id,
            id_token: "valido".to_string(),
            given_name: "  Elena ".to_string(),
            family_name: None,
            second_family_name: None,
            identifier: None,
            phone: " 999888777 ".to_string(),
            birth_date: "1990-05-10".to_string(),
            address: None,
        }
    }

    #[test]
    fn incomplete_account_reports_name_and_phone_as_missing() {
        let user = incomplete_user(Uuid::now_v7(), None);

        assert_eq!(
            user.missing_required_fields(),
            vec![
                RequiredProfileField::GivenName,
                RequiredProfileField::Phone,
                RequiredProfileField::BirthDate,
            ]
        );
    }

    #[tokio::test]
    async fn completes_account_of_token_owner() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, None)]);

        let user = use_case
            .execute(command(network_id))
            .await
            .expect("debe completar la cuenta");

        assert_eq!(user.person.name().given().join(" "), "Elena");
        assert_eq!(user.phone().as_deref(), Some("999888777"));
        assert!(user.missing_required_fields().is_empty());
    }

    #[tokio::test]
    async fn saves_every_field() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, None)]);

        let user = use_case
            .execute(CompleteProfileCommand {
                family_name: Some("García".to_string()),
                second_family_name: Some("Pérez".to_string()),
                identifier: Some(Identifier::DNI("12345678".to_string())),
                address: Some("Av. Principal 123".to_string()),
                ..command(network_id)
            })
            .await
            .expect("debe guardar la cuenta");

        assert_eq!(user.person.name().family().as_deref(), Some("García"));
        assert_eq!(user.person.name().second_family().as_deref(), Some("Pérez"));
        assert_eq!(
            user.person.identifier(),
            &Some(Identifier::DNI("12345678".to_string()))
        );
        assert_eq!(user.birth_date(), "1990-05-10");
        assert_eq!(user.address.as_deref(), Some("Av. Principal 123"));
    }

    #[tokio::test]
    async fn rejects_invalid_birth_date_and_dni() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, None)]);

        for invalid in [
            CompleteProfileCommand {
                birth_date: "   ".to_string(),
                ..command(network_id)
            },
            CompleteProfileCommand {
                birth_date: "10/05/1990".to_string(),
                ..command(network_id)
            },
            CompleteProfileCommand {
                birth_date: "2999-01-01".to_string(),
                ..command(network_id)
            },
            CompleteProfileCommand {
                identifier: Some(Identifier::DNI("1234".to_string())),
                ..command(network_id)
            },
        ] {
            assert!(matches!(
                use_case.execute(invalid).await,
                Err(CompleteProfileError::InvalidInput(_))
            ));
        }
    }

    #[tokio::test]
    async fn rejects_dni_of_another_account_but_accepts_own() {
        let network_id = Uuid::now_v7();
        let mut other = incomplete_user(network_id, None);
        other.person.identifier = Some(Identifier::DNI("11111111".to_string()));
        other.person.telecom = vec![];
        let use_case = use_case(vec![incomplete_user(network_id, None), other]);

        let taken = CompleteProfileCommand {
            identifier: Some(Identifier::DNI("11111111".to_string())),
            ..command(network_id)
        };
        assert!(matches!(
            use_case.execute(taken).await,
            Err(CompleteProfileError::DocumentAlreadyRegistered(_))
        ));

        let own = CompleteProfileCommand {
            identifier: Some(Identifier::DNI("22222222".to_string())),
            ..command(network_id)
        };
        use_case.execute(own.clone()).await.expect("DNI libre");
        use_case.execute(own).await.expect("su propio DNI");
    }

    #[tokio::test]
    async fn keeps_family_name_when_omitted() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, Some("García"))]);

        let user = use_case
            .execute(command(network_id))
            .await
            .expect("debe completar la cuenta");

        assert_eq!(user.person.name().family().as_deref(), Some("García"));
    }

    #[tokio::test]
    async fn rejects_blank_required_fields() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, None)]);

        let blank_name = CompleteProfileCommand {
            given_name: "   ".to_string(),
            ..command(network_id)
        };
        let blank_phone = CompleteProfileCommand {
            phone: "   ".to_string(),
            ..command(network_id)
        };

        assert!(matches!(
            use_case.execute(blank_name).await,
            Err(CompleteProfileError::InvalidInput(_))
        ));
        assert!(matches!(
            use_case.execute(blank_phone).await,
            Err(CompleteProfileError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn rejects_phone_longer_than_column() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, None)]);
        let long_phone = CompleteProfileCommand {
            phone: "1".repeat(PHONE_MAX_CHARS + 1),
            ..command(network_id)
        };

        assert!(matches!(
            use_case.execute(long_phone).await,
            Err(CompleteProfileError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn rejects_invalid_token() {
        let network_id = Uuid::now_v7();
        let use_case = use_case(vec![incomplete_user(network_id, None)]);
        let invalid_token = CompleteProfileCommand {
            id_token: "invalido".to_string(),
            ..command(network_id)
        };

        assert!(matches!(
            use_case.execute(invalid_token).await,
            Err(CompleteProfileError::InvalidToken(_))
        ));
    }

    #[tokio::test]
    async fn rejects_token_owner_without_account_in_network() {
        let use_case = use_case(vec![incomplete_user(Uuid::now_v7(), None)]);

        assert!(matches!(
            use_case.execute(command(Uuid::now_v7())).await,
            Err(CompleteProfileError::UserNotFound(_))
        ));
    }
}
