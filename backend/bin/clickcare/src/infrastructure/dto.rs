use fake::Fake;
use fake::faker::chrono::raw::Date;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::raw::{FirstName, LastName};
use fake::faker::phone_number::raw::PhoneNumber;
use fake::locales::EN;
use fake::rand::rngs::ThreadRng;
use ulid::Ulid;
use crate::infrastructure::api::{ContactPoint, HumanName, Identifier, Patient};

impl Patient {
    fn default() -> Self {
        let min_value: u32 = 00_000_000;
        let max_value: u32 = 99_999_999;
        let mut rng = ThreadRng::default();

        Patient {
            id: Ulid::new().to_string(),
            identifier: vec![Identifier {
                r#use: "Oficial".to_string(),
                system: "(ej. <https://fhir.org/id/dni>)".to_string(),
                value: (min_value ..= max_value).fake_with_rng::<u32, ThreadRng>(&mut rng).to_string(),
            }],
            name: vec![HumanName {
                given: vec![FirstName(EN).fake(), FirstName(EN).fake()],
                family: vec![LastName(EN).fake(), LastName(EN).fake()],
                r#use: "".to_string()
            }],
            active: true,
            telecom: vec![ContactPoint {
                system: 0,
                value: FreeEmail(EN).fake(),
                r#use: 0,
            }, ContactPoint {
                system: 1,
                value: PhoneNumber(EN).fake(),
                r#use: 0,
            }],
            birth_date: Date(EN).fake(),
            gender: 1,
            // icon: Some("https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80".to_string()),
            photo: vec![],
            deceased_status: None,

            managing_organization_reference: "".to_string(),
        }
    }
}

// note: Some("Co-Founder / CEO".to_string()),
