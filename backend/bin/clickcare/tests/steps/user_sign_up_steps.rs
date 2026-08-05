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

#[when("se envía una solicitud de registro con un UUID v7 válido")]
pub async fn when_sign_up_request_sent(sign_up_context: &mut SignUpContext) {
    let env = test_env().await;
    let mut client = UserApiClient::connect(env.grpc_addr.clone())
        .await
        .expect("Fallo al conectar con el servidor gRPC");

    let user_id = uuid::Uuid::now_v7();
    let email = format!("test-{}@example.com", user_id);

    let sign_up_request = SignUpRequest {
        id_token: "test-token".into(),
        user_id: user_id.to_string(),
        email,
        given_name: "Juan".into(),
        family_name: Some("Pérez".into()),
        ..Default::default()
    };
    let request = tonic::Request::new(sign_up_request.clone());

    let response = client.sign_up(request).await;

    info!("Response sign_up valid UUID v7: {:?}", response);
    sign_up_context.user_id = Some(user_id);
    sign_up_context.request = Some(sign_up_request);
    sign_up_context.response_ok = Some(response.is_ok());
    if let Err(ref status) = response {
        sign_up_context.response_status = Some(status.clone());
    }
}

#[when("se envía una solicitud de registro con un UUID v4 inválido")]
pub async fn when_sign_up_request_invalid_uuid_v4_sent(sign_up_context: &mut SignUpContext) {
    let env = test_env().await;
    let mut client = UserApiClient::connect(env.grpc_addr.clone())
        .await
        .expect("Fallo al conectar con el servidor gRPC");

    let user_id_v4 = uuid::Uuid::new_v4();
    let email = format!("test-invalid-{}@example.com", user_id_v4);

    let request = tonic::Request::new(SignUpRequest {
        id_token: "test-token".into(),
        user_id: user_id_v4.to_string(),
        email,
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

#[then("la respuesta de registro es exitosa y el usuario se persiste en la base de datos")]
pub async fn then_sign_up_successful_and_persisted(sign_up_context: &SignUpContext) {
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
    let sign_up_request = sign_up_context
        .request
        .as_ref()
        .expect("Falta la Request en el contexto");

    let pg_conn = env.pg_connection_string.clone();
    let email = sign_up_request.email.clone();
    let given_name_req = sign_up_request.given_name.clone();
    let family_name_req = sign_up_request.family_name.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_client = ::postgres::Client::connect(&pg_conn, ::postgres::NoTls)
            .expect("Error al conectar a la base de datos PostgreSQL");

        let row = db_client
            .query_one(
                "SELECT id, email, active, given_name, family_name FROM user_account WHERE id = $1",
                &[&user_id],
            )
            .expect("No se encontró el usuario registrado en la base de datos");

        let db_id: uuid::Uuid = row.get("id");
        let db_email: String = row.get("email");
        let db_active: bool = row.get("active");
        let given_name: String = row.get("given_name");
        let family_name: Option<String> = row.get("family_name");

        assert_eq!(db_id, user_id);
        assert_eq!(db_email, email);
        assert_eq!(given_name, given_name_req);
        assert_eq!(family_name, family_name_req);
        assert!(db_active);
    })
    .await
    .expect("Fallo la tarea asíncrona de consulta a PostgreSQL");
}

#[then("la respuesta de registro devuelve un error indicando UUID v7 inválido")]
pub async fn then_sign_up_fails_with_invalid_uuid_v7_error(sign_up_context: &SignUpContext) {
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
        status.message().contains("no es un UUID V7"),
        "Mensaje de error inesperado: {}",
        status.message()
    );
}

// 2026-08-05T12:13:27.926469Z DEBUG user_api_test::steps::user_sign_up_steps: sign_up_context: SignUpContext { user_id: None, request: None, response_ok: Some(false), response_status: Some(Status { code: Unknown, message: "ClickCareError: El id no es un UUID V7, id: 113e464c-db18-4a0f-96b6-2400f63d0613 (en crates/user/src/domain/user.rs:146)", metadata: MetadataMap { headers: {"content-type": "application/grpc", "date": "Wed, 05 Aug 2026 12:13:27 GMT"} }, source: None }) }
