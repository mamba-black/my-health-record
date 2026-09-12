use crate::steps::user_progressive_profiling::{SignUpContext, sign_up_context};
use rstest_bdd_macros::scenario;

#[scenario(
    path = "tests/features/patient_appointment.feature",
    name = "Intento fallido de reserva de cita por falta de DNI y teléfono requeridos por Ley N° 30024"
)]
#[tokio::test]
#[ignore = "TODO: Requiere integración con servicio de citas (crates/clinic)"]
async fn appointment_booking_fails_without_dni_and_phone(sign_up_context: SignUpContext) {
    let _ = sign_up_context;
}

#[scenario(
    path = "tests/features/patient_appointment.feature",
    name = "Reserva exitosa de cita médica al completar progresivamente el DNI y teléfono"
)]
#[tokio::test]
#[ignore = "TODO: Requiere integración con servicio de citas (crates/clinic)"]
async fn appointment_booking_succeeds_after_providing_dni_and_phone(
    sign_up_context: SignUpContext,
) {
    let _ = sign_up_context;
}
