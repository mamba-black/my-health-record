use display_json::DisplayAsJson;
use leptos::{ReadSignal, WriteSignal};
use serde::Serialize;

use crate::domain::patient::Patient;

#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    // pub patient: Option<Patient>,
    // pub patient: ReadSignal<Option<Patient>>,
    // pub patient_set: WriteSignal<Option<Patient>>,
    // pub patient: String,
}
