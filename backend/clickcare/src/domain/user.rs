use strum_macros::Display;

#[derive(Debug, Clone)]
pub(crate) struct User{
    pub(crate) name: String,
    pub(crate) document_type: DocumentType,
    pub(crate) document_number: String,
    pub(crate) create_clinic: bool,
}

#[derive(Debug, Clone, Display)]
pub(crate) enum DocumentType {
    DNI,
}
