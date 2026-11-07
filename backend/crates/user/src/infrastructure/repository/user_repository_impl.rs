use crate::domain::user::User;
use crate::domain::user::user_repository::{ProfileUpdate, UserRepository};
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

        let Identifier::DNI(dni) = identifier;
        let exist = toasty::sql::query(
            "select 1 from identity.user_account where network_id = $1 and document_type = $2 and document_value = $3 limit 1",
        )
        .bind(*network_id)
        .bind("DNI")
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
        error!("Resultado: {:?}", exist);
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

        let rows = toasty::sql::query(
            "select id, active, is_owner, given_name, \
             coalesce(family_name, ''), coalesce(second_family_name, ''), \
             coalesce(document_type, ''), coalesce(document_value, ''), \
             coalesce(email, ''), coalesce(phone, ''), \
             coalesce(birth_date::text, ''), coalesce(address, '') \
             from identity.user_account where network_id = $1 and email = $2 limit 1",
        )
        .bind(*network_id)
        .bind(email.to_string())
        .column_types([
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
        ])
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

        map_row_to_user(row, *network_id).map(Some)
    }

    async fn find_user_by_id(&self, _user_id: &str) -> Result<User, ClickCareError> {
        // Aquí iría la lógica para buscar un usuario en la base de datos
        // let a = query_as!(User, "SELECT * FROM user WHERE id = $1", user_id)
        //     .fetch_one(self.pool)
        //     .await?;
        // Ok(a)
        todo!();
    }

    async fn update_profile(
        &self,
        user_id: &Uuid,
        profile: &ProfileUpdate,
    ) -> Result<(), ClickCareError> {
        let dni = profile.identifier.as_ref().map(|identifier| {
            let Identifier::DNI(dni) = identifier;
            dni.clone()
        });
        let updated = toasty::sql::query(
            "update identity.user_account set given_name = $1, \
             family_name = coalesce(nullif($2::text, ''), family_name), \
             second_family_name = coalesce(nullif($3::text, ''), second_family_name), \
             document_type = case when $4::text <> '' then 'DNI' else document_type end, \
             document_value = coalesce(nullif($4::text, ''), document_value), \
             phone = $5, birth_date = ($6::text)::date, \
             address = coalesce(nullif($7::text, ''), address), \
             updated_at = current_timestamp \
             where id = $8 returning id",
        )
        .bind(profile.given_name.clone())
        .bind(profile.family_name.clone().unwrap_or_default())
        .bind(profile.second_family_name.clone().unwrap_or_default())
        .bind(dni.unwrap_or_default())
        .bind(profile.phone.clone())
        .bind(profile.birth_date.clone())
        .bind(profile.address.clone().unwrap_or_default())
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
            Some(crate::domain::user::Identifier::DNI(val)) => {
                (Some("DNI".to_string()), Some(val.clone()))
            }
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

fn map_row_to_user(row: toasty::stmt::Value, network_id: Uuid) -> Result<User, ClickCareError> {
    let mut fields = row.into_record().fields.into_iter();

    let user_id = next_uuid_column(&mut fields, "id")?;
    let active = next_bool_column(&mut fields, "active")?;
    let is_owner = next_bool_column(&mut fields, "is_owner")?;
    let given_name = next_string_column(&mut fields, "given_name")?;
    let family_name_raw = next_string_column(&mut fields, "family_name")?;
    let second_family_name_raw = next_string_column(&mut fields, "second_family_name")?;
    let document_type_raw = next_string_column(&mut fields, "document_type")?;
    let document_value_raw = next_string_column(&mut fields, "document_value")?;
    let email_string = next_string_column(&mut fields, "email")?;
    let phone_raw = next_string_column(&mut fields, "phone")?;
    let birth_date_raw = next_string_column(&mut fields, "birth_date")?;
    let address_raw = next_string_column(&mut fields, "address")?;

    let family_name = (!family_name_raw.is_empty()).then_some(family_name_raw);
    let second_family_name = (!second_family_name_raw.is_empty()).then_some(second_family_name_raw);
    let identifier = if document_type_raw == "DNI" && !document_value_raw.is_empty() {
        Some(Identifier::DNI(document_value_raw))
    } else {
        None
    };
    let phone = (!phone_raw.is_empty()).then_some(phone_raw);

    let mut user = User::new(
        user_id.to_string(),
        network_id,
        vec![given_name],
        family_name,
        second_family_name,
        identifier,
        is_owner,
        email_string,
        phone,
        (!birth_date_raw.is_empty()).then_some(birth_date_raw),
    )?;
    user.active = active;
    user.address = (!address_raw.is_empty()).then_some(address_raw);

    Ok(user)
}
