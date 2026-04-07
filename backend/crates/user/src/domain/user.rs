use std::str::FromStr;
use strum_macros::Display;
use app_core::domain::error::ClickCareError;
use uuid::{Uuid, Version};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub document_type: DocumentType,
    pub document_number: String,
    pub is_owner: bool,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Display)]
pub enum DocumentType {
    DNI,
}

impl User {
    pub fn new(
        id: String,
        name: String,
        document_type: DocumentType,
        document_number: String,
        is_owner: bool,
        email: String,
        first_name: String,
        last_name: String,
    ) -> Result<Self, ClickCareError> {

        match Uuid::from_str(id.as_str()) {
            Ok(id) if id.get_version() == Some(Version::SortRand) => {
                Ok(Self {
                    id,
                    name,
                    document_type,
                    document_number,
                    is_owner,
                    email,
                    first_name,
                    last_name,
                })
            }
            Ok(id) => {
                Err(ClickCareError::generic(format!("El id no es un UUID V, id: {}", id)))
            }
            Err(e) => {
                Err(ClickCareError::generic(format!("El id no es un UUID, error: {}", e)))
            }
        }

    }
}
