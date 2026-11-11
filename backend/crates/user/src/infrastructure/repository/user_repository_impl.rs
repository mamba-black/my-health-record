use crate::domain::user::user_repository::{ProfileUpdate, UserRepository};
use crate::domain::user::{ProfessionalProfile, Sex, User};
use app_core::domain::access::Role;
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use async_trait::async_trait;
use toasty::Db;
use tracing::{debug, error};
use uuid::Uuid;

/// Fila de `identity.user_account`.
///
/// Los atributos `#[index]`/`#[unique]` solo generan los buscadores (`filter_by_*`):
/// los índices los crean las migraciones de `backend/migrations/` y deben coincidir.
#[derive(Debug, Clone, toasty::Model)]
#[table = "user_account"]
#[unique(network_id, email)]
pub struct UserAccount {
    #[key]
    pub id: uuid::Uuid,
    #[index]
    pub network_id: uuid::Uuid,
    pub active: bool,
    pub is_owner: bool,
    pub provider_info: String,
    pub given_name: String,
    pub family_name: Option<String>,
    pub second_family_name: Option<String>,
    pub document_type: Option<String>,
    pub document_value: Option<String>,
    pub email: String,
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
    #[update(jiff::Timestamp::now())]
    pub updated_at: jiff::Timestamp,
}

impl TryFrom<UserAccount> for User {
    type Error = ClickCareError;

    fn try_from(account: UserAccount) -> Result<Self, Self::Error> {
        let identifier = match (&account.document_type, account.document_value) {
            (Some(document_type), Some(value)) if !value.is_empty() => {
                Identifier::from_parts(document_type, value)
            }
            _ => None,
        };

        let mut user = User::new(
            account.id.to_string(),
            account.network_id,
            vec![account.given_name],
            account.family_name,
            account.second_family_name,
            identifier,
            account.is_owner,
            account.email,
            account.phone,
            account.birth_date.map(|date| date.to_string()),
        )?;
        user.active = account.active;
        user.address = account.address;
        user.preferred_role = Role::from_code(&account.preferred_role);
        user.sex = account.sex.as_deref().and_then(Sex::from_code);
        user.professional = ProfessionalProfile {
            professional_body: account.professional_body,
            license_number: account.license_number,
            specialty: account.specialty,
            rne: account.rne,
        };
        user.document_verified = account.document_verified;
        user.license_verified = account.license_verified;
        Ok(user)
    }
}

pub(crate) struct UserRepositoryImpl {
    pub(crate) db: Db,
}

impl UserRepositoryImpl {
    fn db(&self) -> Db {
        self.db.clone()
    }
}

/// Convierte un error de toasty en `ClickCareError`, dejando el detalle en el log.
fn db_error(context: impl std::fmt::Display) -> impl FnOnce(toasty::Error) -> ClickCareError {
    move |error| {
        error!("{context}: {error}");
        ClickCareError::generic(format!("{context}: {error}"))
    }
}

/// Filtro por documento (`document_type` + `document_value`) dentro de una red.
fn by_document(
    network_id: Uuid,
    identifier: &Identifier,
) -> <UserAccount as toasty::schema::Model>::Query<toasty::stmt::List<UserAccount>> {
    UserAccount::filter_by_network_id(network_id)
        .filter(
            UserAccount::fields()
                .document_type()
                .eq(identifier.type_code().to_string()),
        )
        .filter(
            UserAccount::fields()
                .document_value()
                .eq(identifier.value().to_string()),
        )
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn exist_user_by_document(
        &self,
        network_id: &Uuid,
        identifier: Identifier,
    ) -> Result<bool, ClickCareError> {
        debug!("exist_user_by_document network_id: {network_id} identifier: {identifier}");
        let found = by_document(*network_id, &identifier)
            .first()
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al buscar user_account por documento en network_id:{network_id}"
            )))?;
        Ok(found.is_some())
    }

    async fn exist_user_by_email(
        &self,
        network_id: &Uuid,
        email: &str,
    ) -> Result<bool, ClickCareError> {
        debug!("exist_user_by_email network_id: {network_id} email: {email}");
        Ok(self.find_user_by_email(network_id, email).await?.is_some())
    }

    async fn find_user_by_email(
        &self,
        network_id: &Uuid,
        email: &str,
    ) -> Result<Option<User>, ClickCareError> {
        debug!("find_user_by_email network_id: {network_id} email: {email}");
        UserAccount::filter_by_network_id_and_email(*network_id, email.to_string())
            .first()
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al consultar user_account con email:{email} en network_id:{network_id}"
            )))?
            .map(User::try_from)
            .transpose()
    }

    async fn find_users_by_email(&self, email: &str) -> Result<Vec<User>, ClickCareError> {
        UserAccount::all()
            .filter(UserAccount::fields().email().eq(email))
            .filter(UserAccount::fields().active().eq(true))
            .order_by(UserAccount::fields().created_at().asc())
            .limit(20)
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al buscar las cuentas del correo {email}"
            )))?
            .into_iter()
            .map(User::try_from)
            .collect()
    }

    async fn find_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, ClickCareError> {
        UserAccount::filter_by_id(*user_id)
            .first()
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al consultar user_account con id:{user_id}"
            )))?
            .map(User::try_from)
            .transpose()
    }

    async fn find_users_by_document(
        &self,
        network_id: &Uuid,
        identifier: &Identifier,
    ) -> Result<Vec<User>, ClickCareError> {
        by_document(*network_id, identifier)
            .filter(UserAccount::fields().active().eq(true))
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al buscar cuentas por documento en network_id:{network_id}"
            )))?
            .into_iter()
            .map(User::try_from)
            .collect()
    }

    async fn set_document_verified(&self, user_id: &Uuid) -> Result<(), ClickCareError> {
        UserAccount::filter_by_id(*user_id)
            .update()
            .document_verified(true)
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al verificar el documento de user_account {user_id}"
            )))?;
        Ok(())
    }

    async fn set_license_verified(&self, user_id: &Uuid) -> Result<(), ClickCareError> {
        UserAccount::filter_by_id(*user_id)
            .update()
            .license_verified(true)
            .exec(&mut self.db())
            .await
            .map_err(db_error(format!(
                "Error al confirmar la colegiatura de user_account {user_id}"
            )))?;
        Ok(())
    }

    async fn update_profile(
        &self,
        user_id: &Uuid,
        profile: &ProfileUpdate,
    ) -> Result<(), ClickCareError> {
        let mut db = self.db();
        let mut account = UserAccount::filter_by_id(*user_id)
            .first()
            .exec(&mut db)
            .await
            .map_err(db_error(format!(
                "Error al consultar user_account con id {user_id}"
            )))?
            .ok_or_else(|| {
                ClickCareError::generic(format!("No existe user_account con id {user_id}"))
            })?;

        // Cada dato vacío conserva el valor actual de la columna.
        let given = |value: &Option<String>| {
            value
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        };
        let birth_date = given(&profile.birth_date)
            .map(|date| date.parse::<jiff::civil::Date>())
            .transpose()
            .map_err(|e| ClickCareError::generic(format!("Fecha de nacimiento inválida: {e}")))?;

        let mut update = account.update();
        if let Some(value) = given(&profile.given_name) {
            update.set_given_name(value);
        }
        if let Some(value) = given(&profile.family_name) {
            update.set_family_name(Some(value));
        }
        if let Some(value) = given(&profile.second_family_name) {
            update.set_second_family_name(Some(value));
        }
        if let Some(identifier) = &profile.identifier {
            update.set_document_type(Some(identifier.type_code().to_string()));
            update.set_document_value(Some(identifier.value().to_string()));
        }
        if let Some(value) = given(&profile.phone) {
            update.set_phone(Some(value));
        }
        if let Some(value) = birth_date {
            update.set_birth_date(Some(value));
        }
        if let Some(value) = given(&profile.address) {
            update.set_address(Some(value));
        }
        if let Some(sex) = profile.sex {
            update.set_sex(Some(sex.code().to_string()));
        }
        if let Some(value) = given(&profile.professional_body) {
            update.set_professional_body(Some(value));
        }
        if let Some(value) = given(&profile.license_number) {
            update.set_license_number(Some(value));
        }
        if let Some(value) = given(&profile.specialty) {
            update.set_specialty(Some(value));
        }
        if let Some(value) = given(&profile.rne) {
            update.set_rne(Some(value));
        }
        if let Some(role) = profile.preferred_role {
            update.set_preferred_role(role.code().to_string());
        }
        update.exec(&mut db).await.map_err(db_error(format!(
            "Error al actualizar user_account con id {user_id}"
        )))?;
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
            .map(|c| c.value.clone())
            .ok_or_else(|| {
                ClickCareError::generic(format!("La cuenta {} no tiene correo", user.id))
            })?;

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
