use display_json::DisplayAsJson;
use serde::Serialize;

#[derive(Serialize, DisplayAsJson, Debug, PartialEq, Clone)]
pub struct Patient {
    pub id: String,
    pub name: String,
    pub email: String,
    pub other: String,
    pub online: bool,
    pub avatar: String,
}

impl Patient {
    pub fn new(id: String, name: String, email: String, other: String, online: bool, avatar: String) -> Patient {
        Patient {
            id,
            name,
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
        let patient = Patient::new("123", "Miuler".to_string(), "email".to_string(), "other".to_string(), true, "avatar".to_string());

        println!("patient: {}", patient);
    }
}