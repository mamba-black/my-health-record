use std::str::FromStr;
use strum_macros::Display;
use app_core::domain::error::ClickCareError;
use uuid::{Uuid, Version};

#[derive(Debug, Clone)]
pub struct User {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) document_type: DocumentType,
    pub(crate) document_number: String,
    pub(crate) is_owner: bool,
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
    ) -> Result<Self, ClickCareError> {

        match Uuid::from_str(id.as_str()) {
            Ok(id) if id.get_version() == Some(Version::SortRand) => {
                Ok(Self {
                    id,
                    name,
                    document_type,
                    document_number,
                    is_owner,
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
