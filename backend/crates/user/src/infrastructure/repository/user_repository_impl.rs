use crate::domain::user::user_repository::{ProfileUpdate, UserRepository};
use crate::domain::user::{ProfessionalProfile, Sex, User};
use app_core::domain::access::Role;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use async_trait::async_trait;
use toasty::Db;
use toasty::stmt::Type;
use tracing::{debug, error};
use uuid::Uuid;

#[derive(Debug, Clone, toasty::Model)]
#[table = "user_account"]
pub struct UserAccount {
    #[key]
    pub id: uuid::Uuid,
    pub network_id: uuid::Uuid,
    pub active: bool,
    pub is_owner: bool,
    pub provider_info: String,
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub document_type: Option<String>,
    pub document_value: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub birth_date: Option<jiff::civil::Date>,
    pub address: Option<String>,
    pub preferred_role: String,
    pub sex: Option<String>,
    pub professional_body: Option<String>,
    pub license_number: Option<String>,
    pub specialty: Option<String>,
    pub rne: Option<String>,
    pub document_verified: bool,
    pub license_verified: bool,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

pub(crate) struct UserRepositoryImpl {
    pub(crate) db: Db,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn exist_user_by_document(
        &self,
        network_id: &Uuid,
        identifier: Identifier,
    ) -> Result<bool, ClickCareError> {
        debug!("exist_user_by_document network_id: {network_id} identifier: {identifier}");

        let dni = identifier.value().to_string();
        let exist = toasty::sql::query(
            "select 1 from identity.user_account where network_id = $1 and document_type = $2 and document_value = $3 limit 1",
        )
        .bind(*network_id)
        .bind(identifier.type_code())
        .bind(dni.clone())
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| {
            error!(
                "Error desconocido al tratar de buscar el user_account con dni:{} en network_id:{} ({})",
                dni, network_id, e
            );
            ClickCareError::generic(e.to_string())
        })?;
        Ok(!exist.is_empty())
    }

    async fn exist_user_by_email(
        &self,
        network_id: &Uuid,
        email: &str,
    ) -> Result<bool, ClickCareError> {
        debug!("exist_user_by_email network_id: {network_id} email: {email}");

        let exist = toasty::sql::query(
            "select 1 from identity.user_account where network_id = $1 and email = $2 limit 1",
        )
        .bind(*network_id)
        .bind(email.to_string())
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| {
            error!(
                "Error al buscar user_account con email:{} en network_id:{} ({})",
                email, network_id, e
            );
            ClickCareError::generic(e.to_string())
        })?;
        Ok(!exist.is_empty())
    }

    async fn find_user_by_email(
        &self,
        network_id: &Uuid,
        email: &str,
    ) -> Result<Option<User>, ClickCareError> {
        debug!("find_user_by_email network_id: {network_id} email: {email}");

        let rows = toasty::sql::query(format!(
            "{USER_COLUMNS} from identity.user_account where network_id = $1 and email = $2 limit 1"
        ))
        .bind(*network_id)
        .bind(email.to_string())
        .column_types(user_column_types())
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!(
                "Error al consultar user_account con email:{} en network_id:{} ({})",
                email, network_id, error
            );
            ClickCareError::generic(error.to_string())
        })?;

        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };

        map_row_to_user(row).map(Some)
    }

    async fn find_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, ClickCareError> {
        let rows = toasty::sql::query(format!(
            "{USER_COLUMNS} from identity.user_account where id = $1 limit 1"
        ))
        .bind(*user_id)
        .column_types(user_column_types())
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al consultar user_account con id:{user_id} ({error})");
            ClickCareError::generic(error.to_string())
        })?;

        rows.into_iter().next().map(map_row_to_user).transpose()
    }

    async fn set_license_verified(&self, user_id: &Uuid) -> Result<(), ClickCareError> {
        toasty::sql::query(
            "update identity.user_account set license_verified = true, \
             updated_at = current_timestamp where id = $1",
        )
        .bind(*user_id)
        .exec(&mut self.db.clone())
        .await
        .map_err(|error| {
            error!("Error al confirmar la colegiatura de user_account {user_id}: {error}");
            ClickCareError::generic(error.to_string())
        })?;
        Ok(())
    }

    async fn update_profile(
        &self,
        user_id: &Uuid,
        profile: &ProfileUpdate,
    ) -> Result<(), ClickCareError> {
        let text = |value: &Option<String>| value.clone().unwrap_or_default();
        let (document_type, document_value) = profile
            .identifier
            .as_ref()
            .map(|identifier| {
                (
                    identifier.type_code().to_string(),
                    identifier.value().to_string(),
                )
            })
            .unwrap_or_default();
        // Cada dato vacío conserva el valor actual de la columna.
        let updated = toasty::sql::query(
            "update identity.user_account set \
             given_name = coalesce(nullif($1::text, ''), given_name), \
             family_name = coalesce(nullif($2::text, ''), family_name), \
             second_family_name = coalesce(nullif($3::text, ''), second_family_name), \
             document_type = coalesce(nullif($4::text, ''), document_type), \
             document_value = coalesce(nullif($5::text, ''), document_value), \
             phone = coalesce(nullif($6::text, ''), phone), \
             birth_date = coalesce((nullif($7::text, ''))::date, birth_date), \
             address = coalesce(nullif($8::text, ''), address), \
             sex = coalesce(nullif($9::text, ''), sex), \
             professional_body = coalesce(nullif($10::text, ''), professional_body), \
             license_number = coalesce(nullif($11::text, ''), license_number), \
             specialty = coalesce(nullif($12::text, ''), specialty), \
             rne = coalesce(nullif($13::text, ''), rne), \
             preferred_role = coalesce(nullif($14::text, ''), preferred_role), \
             updated_at = current_timestamp \
             where id = $15 returning id",
        )
        .bind(text(&profile.given_name))
        .bind(text(&profile.family_name))
        .bind(text(&profile.second_family_name))
        .bind(document_type)
        .bind(document_value)
        .bind(text(&profile.phone))
        .bind(text(&profile.birth_date))
        .bind(text(&profile.address))
        .bind(profile.sex.map(Sex::code).unwrap_or_default().to_string())
        .bind(text(&profile.professional_body))
        .bind(text(&profile.license_number))
        .bind(text(&profile.specialty))
        .bind(text(&profile.rne))
        .bind(
            profile
                .preferred_role
                .map(Role::code)
                .unwrap_or_default()
                .to_string(),
        )
        .bind(*user_id)
        .column_types([Type::Uuid])
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| {
            error!("Error al actualizar user_account con id {user_id}: {e}");
            ClickCareError::generic(e.to_string())
        })?;

        if updated.is_empty() {
            return Err(ClickCareError::generic(format!(
                "No existe user_account con id {user_id}"
            )));
        }
        Ok(())
    }

    async fn save_user(&self, user: &User) -> Result<(), ClickCareError> {
        let given_name = user.person.name().given().join(" ");
        let family_name = user.person.name().family().clone();
        let second_family_name = user.person.name().second_family().clone();

        let (document_type, document_value) = match user.person.identifier() {
            Some(identifier) => (
                Some(identifier.type_code().to_string()),
                Some(identifier.value().to_string()),
            ),
            None => (None, None),
        };

        let email = user
            .person
            .telecom()
            .iter()
            .find(|c| matches!(c.system, crate::domain::user::ContactPointSystem::Email))
            .map(|c| c.value.clone());

        let phone = user
            .person
            .telecom()
            .iter()
            .find(|c| matches!(c.system, crate::domain::user::ContactPointSystem::Phone))
            .map(|c| c.value.clone());

        let birth_date = user
            .person
            .birth_date()
            .as_deref()
            .filter(|birth_date| !birth_date.trim().is_empty())
            .map(str::parse::<jiff::civil::Date>)
            .transpose()
            .map_err(|e| ClickCareError::generic(format!("Fecha de nacimiento inválida: {e}")))?;

        let _user_account = toasty::create!(UserAccount {
            id: user.id,
            network_id: user.network_id,
            active: user.active,
            is_owner: user.is_owner,
            provider_info: "Google".to_string(),
            given_name,
            family_name,
            second_family_name,
            document_type,
            document_value,
            email,
            phone,
            birth_date,
            address: user.address.clone(),
            preferred_role: user.preferred_role.code().to_string(),
            sex: user.sex.map(|sex| sex.code().to_string()),
            professional_body: user.professional.professional_body.clone(),
            license_number: user.professional.license_number.clone(),
            specialty: user.professional.specialty.clone(),
            rne: user.professional.rne.clone(),
            document_verified: user.document_verified,
            license_verified: user.license_verified,
        })
        .exec(&mut self.db.clone())
        .await
        .map_err(|e| {
            let e = ClickCareError::generic(format!(
                "Error al guardar usuario en la base de datos: {e}"
            ));
            error!("Error al guardar user_account con id {}", user.id);
            error!("Error: {e}");
            e
        })?;

        Ok(())
    }
}

fn next_uuid_column(
    fields: &mut std::vec::IntoIter<toasty::stmt::Value>,
    column_name: &str,
) -> Result<Uuid, ClickCareError> {
    let field_value = fields.next().ok_or_else(|| {
        ClickCareError::generic(format!("Falta columna {column_name} en user_account"))
    })?;
    Uuid::try_from(field_value).map_err(|error| {
        ClickCareError::generic(format!(
            "Columna {column_name} no es un UUID válido: {error}"
        ))
    })
}

fn next_bool_column(
    fields: &mut std::vec::IntoIter<toasty::stmt::Value>,
    column_name: &str,
) -> Result<bool, ClickCareError> {
    let field_value = fields.next().ok_or_else(|| {
        ClickCareError::generic(format!("Falta columna {column_name} en user_account"))
    })?;
    bool::try_from(field_value).map_err(|error| {
        ClickCareError::generic(format!("Columna {column_name} no es booleana: {error}"))
    })
}

fn next_string_column(
    fields: &mut std::vec::IntoIter<toasty::stmt::Value>,
    column_name: &str,
) -> Result<String, ClickCareError> {
    let field_value = fields.next().ok_or_else(|| {
        ClickCareError::generic(format!("Falta columna {column_name} en user_account"))
    })?;
    String::try_from(field_value).map_err(|error| {
        ClickCareError::generic(format!("Columna {column_name} no es texto: {error}"))
    })
}

/// Columnas de `user_account` en el orden que espera [`map_row_to_user`].
const USER_COLUMNS: &str = "select id, network_id, active, is_owner, given_name, \
     coalesce(family_name, ''), coalesce(second_family_name, ''), \
     coalesce(document_type, ''), coalesce(document_value, ''), \
     coalesce(email, ''), coalesce(phone, ''), \
     coalesce(birth_date::text, ''), coalesce(address, ''), \
     preferred_role, coalesce(sex, ''), coalesce(professional_body, ''), \
     coalesce(license_number, ''), coalesce(specialty, ''), coalesce(rne, ''), \
     document_verified, license_verified";

fn user_column_types() -> [Type; 21] {
    [
        Type::Uuid,
        Type::Uuid,
        Type::Bool,
        Type::Bool,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::String,
        Type::Bool,
        Type::Bool,
    ]
}

fn map_row_to_user(row: toasty::stmt::Value) -> Result<User, ClickCareError> {
    let mut fields = row.into_record().fields.into_iter();
    let non_empty = |value: String| (!value.is_empty()).then_some(value);

    let user_id = next_uuid_column(&mut fields, "id")?;
    let network_id = next_uuid_column(&mut fields, "network_id")?;
    let active = next_bool_column(&mut fields, "active")?;
    let is_owner = next_bool_column(&mut fields, "is_owner")?;
    let given_name = next_string_column(&mut fields, "given_name")?;
    let family_name = non_empty(next_string_column(&mut fields, "family_name")?);
    let second_family_name = non_empty(next_string_column(&mut fields, "second_family_name")?);
    let document_type = next_string_column(&mut fields, "document_type")?;
    let document_value = next_string_column(&mut fields, "document_value")?;
    let email = next_string_column(&mut fields, "email")?;
    let phone = non_empty(next_string_column(&mut fields, "phone")?);
    let birth_date = non_empty(next_string_column(&mut fields, "birth_date")?);
    let address = non_empty(next_string_column(&mut fields, "address")?);
    let preferred_role = Role::from_code(&next_string_column(&mut fields, "preferred_role")?);
    let sex = Sex::from_code(&next_string_column(&mut fields, "sex")?);
    let professional = ProfessionalProfile {
        professional_body: non_empty(next_string_column(&mut fields, "professional_body")?),
        license_number: non_empty(next_string_column(&mut fields, "license_number")?),
        specialty: non_empty(next_string_column(&mut fields, "specialty")?),
        rne: non_empty(next_string_column(&mut fields, "rne")?),
    };
    let document_verified = next_bool_column(&mut fields, "document_verified")?;
    let license_verified = next_bool_column(&mut fields, "license_verified")?;

    let identifier = if document_value.is_empty() {
        None
    } else {
        Identifier::from_parts(&document_type, document_value)
    };

    let mut user = User::new(
        user_id.to_string(),
        network_id,
        vec![given_name],
        family_name,
        second_family_name,
        identifier,
        is_owner,
        email,
        phone,
        birth_date,
    )?;
    user.active = active;
    user.address = address;
    user.preferred_role = preferred_role;
    user.sex = sex;
    user.professional = professional;
    user.document_verified = document_verified;
    user.license_verified = license_verified;

    Ok(user)
}
