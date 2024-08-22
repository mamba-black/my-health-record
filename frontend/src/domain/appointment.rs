use chrono::DateTime;
use chrono_tz::Tz;
use derive_builder::Builder;
use display_json::DisplayAsJson;
use serde::Serialize;

#[derive(Serialize, DisplayAsJson, Builder, Debug, PartialEq, Clone)]
pub struct Appointment {
    pub patient_id: String,
    pub date: DateTime<Tz>,
    pub state: State,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum State {
    InProgress,
    Booked,
    Completed,
    Canceled,
}
