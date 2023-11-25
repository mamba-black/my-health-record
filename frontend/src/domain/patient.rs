use derive_builder::Builder;
use display_json::DisplayAsJson;
use serde::Serialize;

#[derive(Serialize, DisplayAsJson, Builder, Debug, PartialEq, Clone)]
pub struct Patient {
    pub id: String,
    pub full_name: String,
    pub first_name: String,
    pub last_name: String,
    pub second_name: String,
    pub email: String,
    pub other: String,
    pub online: bool,
    pub avatar: String,
}

impl Patient {
    pub fn new(
        id: String,
        full_name: String,
        first_name: String,
        last_name: String,
        second_name: String,
        email: String,
        other: String,
        online: bool,
        avatar: String,
    ) -> Patient {
        Patient {
            id,
            full_name,
            first_name,
            last_name,
            second_name,
            email,
            other,
            online,
            avatar,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_patient_new() {
        let patient = Patient::new(
            "123".to_string(),
            "Miuler".to_string(),
            "Miuler".to_string(),
            "Miuler".to_string(),
            "Miuler".to_string(),
            "email".to_string(),
            "other".to_string(),
            true,
            "avatar".to_string(),
        );

        println!("patient: {}", patient);
    }
}
