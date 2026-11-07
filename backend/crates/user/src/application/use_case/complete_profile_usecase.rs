use crate::application::dto::complete_profile::{CompleteProfileCommand, CompleteProfileError};
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
const PROFESSIONAL_BODY_MAX_CHARS: usize = 20;
const LICENSE_MAX_CHARS: usize = 30;
const SPECIALTY_MAX_CHARS: usize = 100;
const DNI_DIGITS: usize = 8;
/// Carné de extranjería y pasaporte: alfanuméricos, entre 6 y 12 caracteres.
const FOREIGN_DOCUMENT_CHARS: std::ops::RangeInclusive<usize> = 6..=12;

/// Mensaje para un dato verificado que el usuario intenta cambiar.
pub const READ_ONLY_MESSAGE: &str = "Para cambiarlo, acércate a recepción";

/// Caso de uso para guardar datos del perfil de la cuenta de la sesión.
///
/// Ningún dato es obligatorio al guardar: se exigen solo al intentar una acción que los
/// requiere (`openspec/specs/identity/progressive-profiling`). Sí se valida el formato de
/// todo lo que se ingresa, aunque no sea obligatorio en ese momento.
pub struct CompleteProfileUseCase {
    pub(crate) user_repository: Arc<dyn UserRepository>,
}

impl CompleteProfileUseCase {
    pub async fn execute(
        &self,
        command: CompleteProfileCommand,
    ) -> Result<User, CompleteProfileError> {
        let profile = validated_profile(&command)?;
        let user = self.find_user(&command).await?;

        reject_changes_to_verified_data(&user, &profile)?;
        require_license_with_body(&user, &profile)?;

        if let Some(identifier) = &profile.identifier
            && user.person.identifier() != &Some(identifier.clone())
            && self
                .user_repository
                .exist_user_by_document(&command.network_id, identifier.clone())
                .await?
        {
            return Err(CompleteProfileError::DocumentAlreadyRegistered(
                ClickCareError::generic(
                    "El documento ya está registrado en otra cuenta".to_string(),
                ),
            ));
        }

        self.user_repository
            .update_profile(&user.id, &profile)
            .await?;

        self.find_user(&command).await
    }

    async fn find_user(
        &self,
        command: &CompleteProfileCommand,
    ) -> Result<User, CompleteProfileError> {
        self.user_repository
            .find_user_by_id(&command.user_id)
            .await?
            .filter(|user| user.network_id == command.network_id)
            .ok_or_else(|| {
                CompleteProfileError::UserNotFound(ClickCareError::generic(format!(
                    "No existe la cuenta {} en la red de la sesión",
                    command.user_id
                )))
            })
    }
}

/// Un documento o colegiatura verificados no se pueden cambiar desde el perfil.
fn reject_changes_to_verified_data(
    user: &User,
    profile: &ProfileUpdate,
) -> Result<(), CompleteProfileError> {
    if user.document_verified
        && let Some(identifier) = &profile.identifier
        && user.person.identifier() != &Some(identifier.clone())
    {
        return Err(CompleteProfileError::ReadOnly(format!(
            "Tu documento está verificado. {READ_ONLY_MESSAGE}"
        )));
    }
    let changes = |new: &Option<String>, current: &Option<String>| {
        new.as_ref()
            .is_some_and(|value| Some(value) != current.as_ref())
    };
    if user.license_verified
        && (changes(
            &profile.professional_body,
            &user.professional.professional_body,
        ) || changes(&profile.license_number, &user.professional.license_number))
    {
        return Err(CompleteProfileError::ReadOnly(format!(
            "Tu colegiatura está confirmada. {READ_ONLY_MESSAGE}"
        )));
    }
    Ok(())
}

/// El número de colegiatura va siempre acompañado de su colegio profesional, y viceversa.
fn require_license_with_body(
    user: &User,
    profile: &ProfileUpdate,
) -> Result<(), CompleteProfileError> {
    let body = profile
        .professional_body
        .as_ref()
        .or(user.professional.professional_body.as_ref());
    let number = profile
        .license_number
        .as_ref()
        .or(user.professional.license_number.as_ref());
    match (body, number) {
        (Some(_), None) => Err(CompleteProfileError::InvalidInput(
            "Indica también tu número de colegiatura".to_string(),
        )),
        (None, Some(_)) => Err(CompleteProfileError::InvalidInput(
            "Indica también tu colegio profesional".to_string(),
        )),
        _ => Ok(()),
    }
}

fn validated_profile(
    command: &CompleteProfileCommand,
) -> Result<ProfileUpdate, CompleteProfileError> {
    let birth_date = optional_text(command.birth_date.as_deref(), "fecha de nacimiento", 10)?;
    if let Some(birth_date) = &birth_date {
        let parsed: jiff::civil::Date = birth_date.parse().map_err(|_| {
            CompleteProfileError::InvalidInput(
                "La fecha de nacimiento debe tener el formato AAAA-MM-DD".to_string(),
            )
        })?;
        if parsed > jiff::Zoned::now().date() {
            return Err(CompleteProfileError::InvalidInput(
                "La fecha de nacimiento no puede ser futura".to_string(),
            ));
        }
    }

    Ok(ProfileUpdate {
        given_name: optional_text(
            command.given_name.as_deref(),
            "nombres",
            GIVEN_NAME_MAX_CHARS,
        )?,
        family_name: optional_text(
            command.family_name.as_deref(),
            "apellido paterno",
            FAMILY_NAME_MAX_CHARS,
        )?,
        second_family_name: optional_text(
            command.second_family_name.as_deref(),
            "apellido materno",
            FAMILY_NAME_MAX_CHARS,
        )?,
        identifier: validated_identifier(command.identifier.as_ref())?,
        phone: optional_text(command.phone.as_deref(), "teléfono", PHONE_MAX_CHARS)?,
        birth_date,
        address: optional_text(command.address.as_deref(), "dirección", ADDRESS_MAX_CHARS)?,
        sex: command.sex,
        professional_body: optional_text(
            command.professional_body.as_deref(),
            "colegio profesional",
            PROFESSIONAL_BODY_MAX_CHARS,
        )?
        .map(|body| body.to_uppercase()),
        license_number: optional_text(
            command.license_number.as_deref(),
            "número de colegiatura",
            LICENSE_MAX_CHARS,
        )?,
        specialty: optional_text(
            command.specialty.as_deref(),
            "especialidad",
            SPECIALTY_MAX_CHARS,
        )?,
        rne: optional_text(command.rne.as_deref(), "RNE", LICENSE_MAX_CHARS)?,
        preferred_role: command.preferred_role,
    })
}

/// Valida el formato del documento: DNI de 8 dígitos; CE y pasaporte alfanuméricos.
fn validated_identifier(
    identifier: Option<&Identifier>,
) -> Result<Option<Identifier>, CompleteProfileError> {
    let Some(identifier) = identifier else {
        return Ok(None);
    };
    let value = identifier.value().trim().to_uppercase();
    if value.is_empty() {
        return Ok(None);
    }
    match identifier {
        Identifier::DNI(_) => {
            if value.len() != DNI_DIGITS || !value.chars().all(|c| c.is_ascii_digit()) {
                return Err(CompleteProfileError::InvalidInput(format!(
                    "El DNI debe tener {DNI_DIGITS} dígitos"
                )));
            }
            Ok(Some(Identifier::DNI(value)))
        }
        Identifier::CE(_) | Identifier::Passport(_) => {
            if !FOREIGN_DOCUMENT_CHARS.contains(&value.len())
                || !value.chars().all(|c| c.is_ascii_alphanumeric())
            {
                return Err(CompleteProfileError::InvalidInput(format!(
                    "El documento debe tener entre {} y {} letras o dígitos",
                    FOREIGN_DOCUMENT_CHARS.start(),
                    FOREIGN_DOCUMENT_CHARS.end()
                )));
            }
            Ok(Identifier::from_parts(identifier.type_code(), value))
        }
    }
}

fn optional_text(
    value: Option<&str>,
    label: &str,
    max_chars: usize,
) -> Result<Option<String>, CompleteProfileError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.chars().count() > max_chars {
        return Err(CompleteProfileError::InvalidInput(format!(
            "El campo {label} no puede tener más de {max_chars} caracteres"
        )));
    }
    Ok(Some(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::Sex;
    use crate::testing::MockUserRepositoryImpl;
    use app_core::domain::access::{ProfileField, ProtectedAction};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    const EMAIL: &str = "paciente@example.com";

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
        }
    }

    fn command(user: &User) -> CompleteProfileCommand {
        CompleteProfileCommand {
            network_id: user.network_id,
            user_id: user.id,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn guarda_sin_exigir_datos_si_no_hay_accion_en_curso() {
        let user = incomplete_user(Uuid::now_v7(), None);
        let saved = use_case(vec![user.clone()])
            .execute(CompleteProfileCommand {
                address: Some("Av. Principal 123".to_string()),
                ..command(&user)
            })
            .await
            .expect("guardar sin DNI ni teléfono");

        assert_eq!(saved.address.as_deref(), Some("Av. Principal 123"));
        assert_eq!(
            saved.missing_for(ProtectedAction::ConfirmAppointment),
            vec![
                ProfileField::GivenName,
                ProfileField::FamilyName,
                ProfileField::Phone,
                ProfileField::Document,
            ]
        );
    }

    #[tokio::test]
    async fn guarda_todos_los_datos() {
        let user = incomplete_user(Uuid::now_v7(), None);
        let saved = use_case(vec![user.clone()])
            .execute(CompleteProfileCommand {
                given_name: Some("  Elena ".to_string()),
                family_name: Some("García".to_string()),
                second_family_name: Some("Pérez".to_string()),
                identifier: Some(Identifier::DNI("12345678".to_string())),
                phone: Some(" 999888777 ".to_string()),
                birth_date: Some("1990-05-10".to_string()),
                address: Some("Av. Principal 123".to_string()),
                sex: Some(Sex::Female),
                professional_body: Some("cmp".to_string()),
                license_number: Some("012345".to_string()),
                specialty: Some("Pediatría".to_string()),
                rne: Some("RNE-123".to_string()),
                ..command(&user)
            })
            .await
            .expect("debe guardar la cuenta");

        assert_eq!(saved.person.name().given().join(" "), "Elena");
        assert_eq!(saved.phone().as_deref(), Some("999888777"));
        assert_eq!(saved.birth_date(), "1990-05-10");
        assert_eq!(saved.sex, Some(Sex::Female));
        assert_eq!(saved.professional.professional_body.as_deref(), Some("CMP"));
        assert!(
            saved
                .missing_for(ProtectedAction::ActivatePractitioner)
                .is_empty()
        );
    }

    #[tokio::test]
    async fn conserva_lo_que_no_se_envia() {
        let user = incomplete_user(Uuid::now_v7(), Some("García"));
        let saved = use_case(vec![user.clone()])
            .execute(command(&user))
            .await
            .expect("guardar vacío");
        assert_eq!(saved.person.name().family().as_deref(), Some("García"));
    }

    /// Escenarios «Formato y validación de los datos».
    #[rstest::rstest]
    #[case::dni_de_7_digitos(CompleteProfileCommand { identifier: Some(Identifier::DNI("1234567".into())), ..Default::default() })]
    #[case::dni_con_letras(CompleteProfileCommand { identifier: Some(Identifier::DNI("1234567A".into())), ..Default::default() })]
    #[case::ce_corto(CompleteProfileCommand { identifier: Some(Identifier::CE("123".into())), ..Default::default() })]
    #[case::fecha_mal_formada(CompleteProfileCommand { birth_date: Some("10/05/1990".into()), ..Default::default() })]
    #[case::fecha_futura(CompleteProfileCommand { birth_date: Some("2999-01-01".into()), ..Default::default() })]
    #[case::telefono_largo(CompleteProfileCommand { phone: Some("1".repeat(PHONE_MAX_CHARS + 1)), ..Default::default() })]
    #[case::colegiatura_sin_colegio(CompleteProfileCommand { license_number: Some("012345".into()), ..Default::default() })]
    #[case::colegio_sin_numero(CompleteProfileCommand { professional_body: Some("CMP".into()), ..Default::default() })]
    #[tokio::test]
    async fn rechaza_datos_mal_formados(#[case] invalid: CompleteProfileCommand) {
        let user = incomplete_user(Uuid::now_v7(), None);
        let result = use_case(vec![user.clone()])
            .execute(CompleteProfileCommand {
                network_id: user.network_id,
                user_id: user.id,
                ..invalid
            })
            .await;
        assert!(
            matches!(result, Err(CompleteProfileError::InvalidInput(_))),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn acepta_carne_de_extranjeria() {
        let user = incomplete_user(Uuid::now_v7(), None);
        let saved = use_case(vec![user.clone()])
            .execute(CompleteProfileCommand {
                identifier: Some(Identifier::CE("x00123456".to_string())),
                ..command(&user)
            })
            .await
            .expect("CE válido");
        assert_eq!(
            saved.person.identifier(),
            &Some(Identifier::CE("X00123456".to_string()))
        );
        assert!(saved.present_fields().contains(&ProfileField::Document));
    }

    #[tokio::test]
    async fn rechaza_documento_de_otra_cuenta_pero_acepta_el_propio() {
        let user = incomplete_user(Uuid::now_v7(), None);
        let mut other = incomplete_user(user.network_id, None);
        other.person.identifier = Some(Identifier::DNI("11111111".to_string()));
        other.person.telecom = vec![];
        let use_case = use_case(vec![user.clone(), other]);

        let taken = CompleteProfileCommand {
            identifier: Some(Identifier::DNI("11111111".to_string())),
            ..command(&user)
        };
        assert!(matches!(
            use_case.execute(taken).await,
            Err(CompleteProfileError::DocumentAlreadyRegistered(_))
        ));

        let own = CompleteProfileCommand {
            identifier: Some(Identifier::DNI("22222222".to_string())),
            ..command(&user)
        };
        use_case.execute(own.clone()).await.expect("DNI libre");
        use_case.execute(own).await.expect("su propio DNI");
    }

    /// Escenarios «DNI verificado» y «DNI no verificado».
    #[tokio::test]
    async fn un_dni_verificado_no_se_cambia_y_uno_sin_verificar_si() {
        let mut verified = incomplete_user(Uuid::now_v7(), None);
        verified.person.identifier = Some(Identifier::DNI("11111111".to_string()));
        verified.document_verified = true;
        let result = use_case(vec![verified.clone()])
            .execute(CompleteProfileCommand {
                identifier: Some(Identifier::DNI("22222222".to_string())),
                ..command(&verified)
            })
            .await;
        assert!(
            matches!(&result, Err(CompleteProfileError::ReadOnly(message)) if message.contains(READ_ONLY_MESSAGE))
        );

        let mut unverified = verified.clone();
        unverified.document_verified = false;
        let saved = use_case(vec![unverified.clone()])
            .execute(CompleteProfileCommand {
                identifier: Some(Identifier::DNI("22222222".to_string())),
                ..command(&unverified)
            })
            .await
            .expect("DNI sin verificar es editable");
        assert_eq!(
            saved.person.identifier(),
            &Some(Identifier::DNI("22222222".to_string()))
        );
    }

    #[tokio::test]
    async fn una_colegiatura_confirmada_no_se_cambia() {
        let mut user = incomplete_user(Uuid::now_v7(), None);
        user.professional.professional_body = Some("CMP".to_string());
        user.professional.license_number = Some("012345".to_string());
        user.license_verified = true;
        let use_case = use_case(vec![user.clone()]);

        assert!(matches!(
            use_case
                .execute(CompleteProfileCommand {
                    license_number: Some("999999".to_string()),
                    ..command(&user)
                })
                .await,
            Err(CompleteProfileError::ReadOnly(_))
        ));
        use_case
            .execute(CompleteProfileCommand {
                license_number: Some("012345".to_string()),
                specialty: Some("Cardiología".to_string()),
                ..command(&user)
            })
            .await
            .expect("repetir el mismo valor no es un cambio");
    }

    #[tokio::test]
    async fn la_cuenta_sale_de_la_sesion_y_de_su_red() {
        let user = incomplete_user(Uuid::now_v7(), None);
        let result = use_case(vec![user.clone()])
            .execute(CompleteProfileCommand {
                network_id: Uuid::now_v7(),
                ..command(&user)
            })
            .await;
        assert!(matches!(result, Err(CompleteProfileError::UserNotFound(_))));
    }
}
