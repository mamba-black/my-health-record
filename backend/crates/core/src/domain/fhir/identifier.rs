use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Identificador oficial del usuario o paciente (FHIR Identifier).
///
/// Ref: <https://hl7.org/fhir/R4/datatypes.html#Identifier>
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Identifier {
    /// Documento Nacional de Identidad (DNI).
    #[strum(to_string = "DNI: {0}")]
    DNI(String),
}
