use strum_macros::Display;

#[derive(Debug)]
pub(crate) struct User{
    pub(crate) name: String,
    pub(crate) document_type: DocumentType,
    pub(crate) document_number: String,
}

#[derive(Debug, Clone, Display)]
pub(crate) enum DocumentType {
    DNI,
}
