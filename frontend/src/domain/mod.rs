pub(crate) mod error;
pub mod patient;

pub trait GetField {
    fn get(&self, field_name: &str) -> String;
}
