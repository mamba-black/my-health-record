use crate::steps::user_progressive_profiling::{SignUpContext, sign_up_context};
use rstest_bdd_macros::scenario;

#[scenario(
    path = "tests/features/user_progressive_profiling.feature",
    name = "Registro exitoso de usuario con diferentes grados de completitud de perfil"
)]
#[tokio::test]
async fn sign_up_succeeds_with_various_profiles(
    sign_up_context: SignUpContext,
    id: String,
    email: String,
    nombre: String,
    primer_apellido: String,
    segundo_apellido: String,
    dni: String,
    telefono: String,
    fecha_nacimiento: String,
    crear_clinica: String,
    caso_de_uso: String,
) {
    let _ = (
        sign_up_context,
        id,
        email,
        nombre,
        primer_apellido,
        segundo_apellido,
        dni,
        telefono,
        fecha_nacimiento,
        crear_clinica,
        caso_de_uso,
    );
}

#[scenario(
    path = "tests/features/user_progressive_profiling.feature",
    name = "Registro fallido cuando el ID de usuario no es UUID v7"
)]
#[tokio::test]
async fn sign_up_fails_when_user_id_is_not_uuid_v7(sign_up_context: SignUpContext) {
    let _ = sign_up_context;
}
