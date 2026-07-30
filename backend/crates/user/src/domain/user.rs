use app_core::domain::error::ClickCareError;
use log::{debug, error};
use std::str::FromStr;
use strum_macros::Display;
use uuid::{Uuid, Version};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub document: Option<Document>,
    pub is_owner: bool,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Display)]
pub enum Document {
    #[strum(to_string = "DNI: {0}")]
    DNI(String),
}

impl User {
    pub fn new(
        id: String,
        name: String,
        document: Option<Document>,
        is_owner: bool,
        email: String,
        first_name: String,
        last_name: String,
    ) -> Result<Self, ClickCareError> {
        debug!("user.id: {id}");

        match Uuid::from_str(id.as_str()) {
            Ok(id) if id.get_version() == Some(Version::SortRand) => Ok(Self {
                id,
                name,
                document,
                is_owner,
                email,
                first_name,
                last_name,
            }),
            Ok(id) => Err(ClickCareError::generic(format!(
                "El id no es un UUID V7, id: {}",
                id
            ))),
            Err(e) => {
                error!("Error desconocido al parsear el id, error: {e}");
                Err(ClickCareError::generic(format!(
                    "Error desconocido al parsear el id, error: {}",
                    e
                )))
            }
        }
    }
}
