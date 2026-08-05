use clickcare::infrastructure::grpc::SignUpRequest;
use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
use log::{debug, info};
use rstest::*;
use rstest_bdd_macros::{given, then, when};
use tonic::Status;

use crate::test_env;

#[derive(Debug, Default, Clone)]
pub struct SignUpContext {
    pub user_id: Option<uuid::Uuid>,
    pub request: Option<SignUpRequest>,
    pub response_ok: Option<bool>,
    pub response_status: Option<Status>,
}

#[fixture]
pub fn sign_up_context() -> SignUpContext {
    SignUpContext::default()
}

// ---- Given Steps -----------------------------------------------------------

#[given("un entorno activo con servicio gRPC de usuario y base de datos")]
pub async fn given_running_service_and_db() {
    let _ = test_env().await;
}

// ---- When Steps ------------------------------------------------------------

#[when(
    "se envía una solicitud de registro con ID \"{id}\", email \"{email}\", nombre \"{nombre}\", primer apellido \"{primer_apellido}\", segundo apellido \"{segundo_apellido}\", DNI \"{dni}\", teléfono \"{telefono}\", fecha nacimiento \"{fecha_nacimiento}\" y crear clínica \"{crear_clinica}\""
)]
pub async fn when_sign_up_request_sent(
    sign_up_context: &mut SignUpContext,
    id: String,
    email: String,
    nombre: String,
    primer_apellido: String,
    segundo_apellido: String,
    dni: String,
    telefono: String,
    fecha_nacimiento: String,
    crear_clinica: String,
) {
    let env = test_env().await;
    let mut client = UserApiClient::connect(env.grpc_addr.clone())
        .await
        .expect("Fallo al conectar con el servidor gRPC");

    let user_id = if id == "auto" || id.is_empty() {
        uuid::Uuid::now_v7()
    } else {
        uuid::Uuid::parse_str(&id).unwrap_or_else(|_| uuid::Uuid::now_v7())
    };

    let nonce_short = &uuid::Uuid::now_v7().to_string()[..8];
    let unique_email = email.replace('@', &format!("-{}@", nonce_short));

    let identifier = if !dni.is_empty() && dni != "-" {
        static DNI_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let offset = DNI_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let base = 80000000 + (jiff::Timestamp::now().as_nanosecond() % 10000000) as u32;
        let unique_dni = format!("{:08}", base + offset);
        Some(clickcare::infrastructure::grpc::Identifier {
            identifier_type: Some(
                clickcare::infrastructure::grpc::identifier::IdentifierType::Dni(unique_dni),
            ),
        })
    } else {
        None
    };

    let family_name = if primer_apellido.is_empty() || primer_apellido == "-" {
        None
    } else {
        Some(primer_apellido)
    };
    let second_family_name = if segundo_apellido.is_empty() || segundo_apellido == "-" {
        None
    } else {
        Some(segundo_apellido)
    };
    let phone = if telefono == "-" { String::new() } else { telefono };
    let birth_date = if fecha_nacimiento == "-" { String::new() } else { fecha_nacimiento };
    let create_clinic = crear_clinica.parse::<bool>().unwrap_or(false);

    let sign_up_request = SignUpRequest {
        id_token: "test-token".into(),
        user_id: user_id.to_string(),
        email: unique_email,
        identifier,
        given_name: nombre,
        family_name,
        second_family_name,
        phone,
        birth_date,
        create_clinic,
        ..Default::default()
    };
    let request = tonic::Request::new(sign_up_request.clone());

    let response = client.sign_up(request).await;

    info!("Response sign_up table row: {:?}", response);
    sign_up_context.user_id = Some(user_id);
    sign_up_context.request = Some(sign_up_request);
    sign_up_context.response_ok = Some(response.is_ok());
    if let Err(ref status) = response {
        sign_up_context.response_status = Some(status.clone());
    }
}

#[when(
    "se envía una solicitud de registro con un UUID v4 inválido \"{user_id_str}\" y email \"{email}\""
)]
pub async fn when_sign_up_request_invalid_uuid_v4_sent(
    sign_up_context: &mut SignUpContext,
    user_id_str: String,
    email: String,
) {
    let env = test_env().await;
    let mut client = UserApiClient::connect(env.grpc_addr.clone())
        .await
        .expect("Fallo al conectar con el servidor gRPC");

    let nonce_short = &uuid::Uuid::new_v4().to_string()[..8];
    let unique_email = email.replace('@', &format!("-{}@", nonce_short));

    let request = tonic::Request::new(SignUpRequest {
        id_token: "test-token".into(),
        user_id: user_id_str,
        email: unique_email,
        ..Default::default()
    });

    let response = client.sign_up(request).await;

    info!("Response sign_up invalid UUID v4: {:?}", response);
    sign_up_context.response_ok = Some(response.is_ok());
    if let Err(status) = response {
        sign_up_context.response_status = Some(status);
    }
}

// ---- Then Steps ------------------------------------------------------------

#[then(
    "la respuesta de registro es exitosa y el usuario con nombre \"{expected_given_name}\" se persiste en la base de datos"
)]
pub async fn then_sign_up_successful_and_persisted(
    sign_up_context: &SignUpContext,
    expected_given_name: String,
) {
    debug!("sign_up_context: {:?}", sign_up_context);

    let env = test_env().await;
    assert!(
        sign_up_context.response_ok.unwrap_or(false),
        "Se esperaba que el registro fuera exitoso, pero fallo con status: {:?}",
        sign_up_context.response_status
    );

    let user_id = sign_up_context
        .user_id
        .expect("Falta el User ID en el contexto");
    let req = sign_up_context
        .request
        .as_ref()
        .expect("Falta la Request en el contexto");
    let req_email = req.email.clone();
    let req_family_name = req.family_name.clone();
    let req_second_family_name = req.second_family_name.clone();

    let pg_conn = env.pg_connection_string.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_client = ::postgres::Client::connect(&pg_conn, ::postgres::NoTls)
            .expect("Error al conectar a la base de datos PostgreSQL");

        let row = db_client
            .query_one(
                "SELECT id, email, active, given_name, family_name, second_family_name FROM user_account WHERE id = $1",
                &[&user_id],
            )
            .expect("No se encontró el usuario registrado en la base de datos");

        let db_id: uuid::Uuid = row.get("id");
        let db_email: String = row.get("email");
        let db_active: bool = row.get("active");
        let given_name: String = row.get("given_name");
        let family_name: Option<String> = row.get("family_name");
        let second_family_name: Option<String> = row.get("second_family_name");

        assert_eq!(db_id, user_id);
        assert_eq!(db_email, req_email);
        assert_eq!(given_name, expected_given_name);
        assert_eq!(family_name, req_family_name);
        assert_eq!(second_family_name, req_second_family_name);
        assert!(db_active);
    })
    .await
    .expect("Fallo la tarea asíncrona de consulta a PostgreSQL");
}

#[then("la respuesta de registro devuelve un error indicando \"{expected_error_substr}\"")]
pub async fn then_sign_up_fails_with_invalid_uuid_v7_error(
    sign_up_context: &SignUpContext,
    expected_error_substr: String,
) {
    debug!("sign_up_context: {:?}", sign_up_context);
    assert!(
        !sign_up_context.response_ok.unwrap_or(true),
        "Se esperaba un error en el registro pero la respuesta fue Ok"
    );

    let status = sign_up_context
        .response_status
        .as_ref()
        .expect("Se esperaba un Status de error gRPC en el contexto");

    assert!(
        status.message().contains(&expected_error_substr),
        "Mensaje de error inesperado: {}, se esperaba contenga: {}",
        status.message(),
        expected_error_substr
    );
}
