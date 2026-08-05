use crate::steps::user_progressive_profiling::{SignUpContext, sign_up_context};
use rstest_bdd_macros::scenario;

#[scenario(
    path = "tests/features/user_progressive_profiling.feature",
    name = "Registro exitoso con un UUID v7 válido"
)]
#[tokio::test]
async fn sign_up_succeeds_with_valid_uuid_v7(sign_up_context: SignUpContext) {
    let _ = sign_up_context;
}

#[scenario(
    path = "tests/features/user_progressive_profiling.feature",
    name = "Registro fallido cuando el ID de usuario no es UUID v7"
)]
#[tokio::test]
async fn sign_up_fails_when_user_id_is_not_uuid_v7(sign_up_context: SignUpContext) {
    let _ = sign_up_context;
}
