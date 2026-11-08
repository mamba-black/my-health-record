//! Agenda y citas (FHIR R4 `Schedule`, `Slot`, `Appointment`).
//!
//! No depende de `administration` ni de `user`: habla con ellos por los puertos de
//! [`application::integration`], que conecta `bin/clickcare`.

pub mod application;
pub mod domain;
pub mod infrastructure;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;
