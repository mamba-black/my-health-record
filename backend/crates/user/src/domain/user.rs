use strum_macros::Display;

#[derive(Debug, Clone)]
pub struct User{
    pub(crate) name: String,
    pub(crate) document_type: DocumentType,
    pub(crate) document_number: String,
    pub(crate) is_owner: bool,
}

#[derive(Debug, Clone, Display)]
pub enum DocumentType {
    DNI,
}
