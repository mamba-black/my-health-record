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
    pub exams: Option<Vec<String>>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum State {
    InProgress,
    Booked,
    Completed,
    Canceled,
}

#[cfg(test)]
mod test {
    use crate::domain::appointment::AppointmentBuilder;
    use crate::domain::appointment::State::InProgress;
    use chrono::Local;
    use chrono_tz::America::Lima;

    #[test]
    fn test_appointment() {
        let appointment = AppointmentBuilder::default()
            .patient_id("123".to_string())
            .date(Local::now().with_timezone(&Lima))
            .state(InProgress)
            .build();

        println!("appointment: {:?}", appointment);
        assert!(appointment.is_ok());
    }
}
