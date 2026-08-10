use bon::bon;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Tipo de dato HL7 FHIR R4: `HumanName`.
///
/// Ref: <https://hl7.org/fhir/R4/datatypes.html#HumanName>
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize, Deserialize)]
pub struct HumanName {
    /// Nombres de pila (`given`), ej. `["Juan", "Carlos"]`.
    given: Vec<String>,
    /// Primer apellido (`family`), opcional según FHIR R4.
    family: Option<String>,
    /// Segundo apellido (extensión hispana de `family`), ej. `"Gómez"`.
    second_family: Option<String>,
    /// Nombre completo formateado y representable en texto (`text`).
    text: String,
}

#[bon]
impl HumanName {
    /// Smart Constructor principal: Garantiza que `text` siempre sea pre-calculado e inmutable.
    pub fn new(given: Vec<String>, family: Option<String>, second_family: Option<String>) -> Self {
        let text = match (&family, &second_family) {
            (Some(family_name), Some(second_family_name)) => {
                format!("{} {} {}", given.join(" "), family_name, second_family_name)
            }
            (Some(family_name), None) => format!("{} {}", given.join(" "), family_name),
            (None, _) => given.join(" "),
        };
        Self {
            given,
            family,
            second_family,
            text,
        }
    }

    /// Builder fluente provisto por `bon` que reutiliza la lógica de `new()`.
    #[builder]
    pub fn builder(
        given: Vec<String>,
        family: Option<String>,
        second_family: Option<String>,
    ) -> Self {
        Self::new(given, family, second_family)
    }
}
