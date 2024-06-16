use derive_builder::Builder;
use display_json::DisplayAsJson;
use serde::Serialize;

#[derive(Serialize, Default, DisplayAsJson, Builder, Debug, PartialEq, Clone)]
#[builder(field(public), default)]
pub struct Patient {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub second_name: Option<String>,
    pub birthdate: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub other: String,
    pub online: bool,
    pub avatar: Option<String>,
    pub address: Option<Address>,
    pub allergies: Vec<String>,
    pub hepatitis: bool,
    pub diabetes: bool,
    pub hemorrhage: bool,
    pub high_pressure: bool,
    pub low_pressure: bool,
    pub cholesterol: bool,
    pub asthma: bool,
    pub tbc: bool,
}

impl Patient {
    pub fn full_name(&self) -> String {
        if let Some(second_name) = &self.second_name {
            format!("{} {} {}", self.first_name, second_name, self.last_name)
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }
}

impl PatientBuilder {
    pub fn from_patient(patient: Patient) -> PatientBuilder {
        PatientBuilder::default()
            .id(patient.id)
            .first_name(patient.first_name)
            .last_name(patient.last_name)
            .second_name(patient.second_name)
            .birthdate(patient.birthdate)
            .phone_number(patient.phone_number)
            .email(patient.email)
            .other(patient.other)
            .online(patient.online)
            .avatar(patient.avatar)
            .address(patient.address)
            .allergies(patient.allergies)
            .clone()
    }
}

#[derive(Serialize, DisplayAsJson, Builder, Default, Debug, PartialEq, Clone)]
#[builder(default)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub district: Option<String>,
    pub apartment_number: Option<String>,
}

impl AddressBuilder {
    pub fn from_address(address: Address) -> Self {
        AddressBuilder::default()
            .street(address.street)
            .city(address.city)
            .state_province(address.state_province)
            .postal_code(address.postal_code)
            .country(address.country)
            .district(address.district)
            .apartment_number(address.apartment_number)
            .clone()
    }
}

pub enum Allergy {
    Hepatitis(bool),
    Diabetes(bool),
    Hemorrhage(bool),
    HighPressure(bool),
    LowPressure(bool),
    Cholesterol(bool),
    Asthma(bool),
    TBC(bool),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_address() {
        let address = AddressBuilder::default().build().unwrap();
    }

    #[test]
    fn test_patient_new() {
        let patient = PatientBuilder::default()
            .id("123".to_string())
            .first_name("Miuler".to_string())
            .last_name("Miuler".to_string())
            .second_name(Some("Miuler".to_string()))
            .email(Some("email".to_string()))
            .online(true)
            .avatar(Some("avatar".to_string()))
            .build()
            .unwrap();

        println!("patient: {}", patient);
    }
}
