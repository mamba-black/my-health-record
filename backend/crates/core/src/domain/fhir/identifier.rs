use strum_macros::Display;

/// Identificador oficial del usuario o paciente (FHIR Identifier).
///
/// Ref: <https://hl7.org/fhir/R4/datatypes.html#Identifier>
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Identifier {
    /// Documento Nacional de Identidad (DNI), 8 dígitos.
    #[strum(to_string = "DNI: {0}")]
    DNI(String),
    /// Carné de extranjería.
    #[strum(to_string = "CE: {0}")]
    CE(String),
    /// Pasaporte.
    #[strum(to_string = "PASAPORTE: {0}")]
    Passport(String),
}

impl Identifier {
    /// Código del tipo de documento tal como se guarda en la base (`document_type`).
    pub fn type_code(&self) -> &'static str {
        match self {
            Identifier::DNI(_) => "DNI",
            Identifier::CE(_) => "CE",
            Identifier::Passport(_) => "PASSPORT",
        }
    }

    /// Número del documento.
    pub fn value(&self) -> &str {
        match self {
            Identifier::DNI(value) | Identifier::CE(value) | Identifier::Passport(value) => value,
        }
    }

    /// Reconstruye el documento a partir de su tipo y número; `None` si el tipo es desconocido.
    pub fn from_parts(type_code: &str, value: String) -> Option<Self> {
        match type_code {
            "DNI" => Some(Identifier::DNI(value)),
            "CE" => Some(Identifier::CE(value)),
            "PASSPORT" => Some(Identifier::Passport(value)),
            _ => None,
        }
    }
}
