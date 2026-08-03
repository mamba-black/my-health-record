use clickcare::infrastructure::grpc::user_api_impl::UserApiImpl;
use clickcare::infrastructure::grpc::user_api_server::UserApiServer;
use clickcare::infrastructure::log::init_logger;
use dotenvy::dotenv;
use dtor::dtor;
use log::info;
use rstest::*;
use std::path::PathBuf;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tonic::transport::Server;

struct TestEnv {
    grpc_addr: String,
    pg_connection_string: String,
    // Mantiene el contenedor vivo durante toda la suite.
    _container: ContainerAsync<postgres::Postgres>,
}

static TEST_ENV: OnceCell<TestEnv> = OnceCell::const_new();

#[fixture]
async fn test_env() -> &'static TestEnv {
    TEST_ENV
        .get_or_init(|| async {
            init_logger();
            dotenv().ok();

            let user = "admin";
            let password = "admin123";
            let timestamp = jiff::Timestamp::now().strftime("%Y%m%d%H%M%S").to_string();
            let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ddl/table.sql");
            info!("schema_path: {:?}", schema_path);

            let container = postgres::Postgres::default()
                .with_user(user)
                .with_password(password)
                .with_tag("18")
                .with_container_name(format!("clickcare-test-{}", timestamp))
                .with_copy_to("/docker-entrypoint-initdb.d/001_schema.sql", schema_path)
                .start()
                .await
                .unwrap();
            info!("Container INICIADO");

            let host_port = container.get_host_port_ipv4(5432).await.unwrap();
            let connection_string = format!(
                "postgres://{}:{}@127.0.0.1:{host_port}/postgres",
                user, password
            );

            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let service = UserApiImpl::new(Some(connection_string.clone()))
                .await
                .unwrap();

            tokio::spawn(async move {
                Server::builder()
                    .add_service(UserApiServer::new(service))
                    .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                    .await
                    .unwrap();
            });

            TestEnv {
                grpc_addr: format!("http://{}", addr),
                pg_connection_string: connection_string,
                _container: container,
            }
        })
        .await
}

#[dtor(unsafe)]
fn shutdown() {
    if let Some(env) = TEST_ENV.get() {
        info!(
            "Shutting down test environment with gRPC server at {}",
            env.grpc_addr
        );
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Aquí podrías agregar lógica para limpiar la base de datos o realizar otras tareas de limpieza si es necesario.
            let _ = env._container.stop().await;
        });
    }
}

mod sign_up {
    use crate::{TestEnv, test_env};
    use clickcare::infrastructure::grpc::SignUpRequest;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use log::info;
    use rstest::rstest;

    // ---- Happy path -------------------------------------------------------

    #[rstest]
    #[tokio::test]
    async fn sign_up_succeeds_with_valid_uuid_v7(#[future(awt)] test_env: &'static TestEnv) {
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar");

        let user_id = uuid::Uuid::now_v7();

        let sign_up_request = SignUpRequest {
            id_token: "test-token".into(),
            user_id: user_id.to_string(),
            email: "test@example.com".into(),
            given_name: "Juan".into(),
            family_name: Some("Pérez".into()),
            ..Default::default()
        };
        let request = tonic::Request::new(sign_up_request.clone());

        let response = client.sign_up(request).await;

        info!("Response: {:?}", response);
        assert!(response.is_ok());
        let _inner = response.unwrap().into_inner();

        // Validar contenido del response
        let pg_conn = test_env.pg_connection_string.clone();
        tokio::task::spawn_blocking(move || {
            let mut db_client = ::postgres::Client::connect(&pg_conn, ::postgres::NoTls)
                .expect("Failed to connect to database");

            let row = db_client
                .query_one(
                    "SELECT id, email, active, given_name, family_name FROM user_account WHERE id = $1",
                    &[&user_id],
                )
                .expect("Failed to execute query or user not found");

            let db_id: uuid::Uuid = row.get("id");
            let db_email: String = row.get("email");
            let db_active: bool = row.get("active");
            let given_name: String = row.get("given_name");
            let family_name: Option<String> = row.get("family_name");

            assert_eq!(db_id, user_id);
            assert_eq!(db_email, sign_up_request.email);
            assert_eq!(given_name, sign_up_request.given_name);
            assert_eq!(family_name, sign_up_request.family_name);
            assert!(db_active);
        })
        .await
        .expect("Task failed");
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: requiere UserRepositoryImpl real (save_user) + lectura"]
    async fn sign_up_persists_user_in_database(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!(
            "tras un sign_up exitoso, leer el usuario por id y verificar que se guardó con los datos enviados"
        );
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: requiere lógica de creación de clínica (flag create_clinic)"]
    async fn sign_up_creates_clinic_when_create_clinic_is_true(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let _ = test_env;
        todo!(
            "sign_up con create_clinic=true -> verificar que se creó la clínica asociada al usuario"
        );
    }

    // ---- Validación de entrada -------------------------------------------

    #[rstest]
    #[tokio::test]
    async fn sign_up_fails_when_user_id_is_not_uuid_v7(#[future(awt)] test_env: &'static TestEnv) {
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar");

        let request = tonic::Request::new(SignUpRequest {
            id_token: "test-token".into(),
            user_id: uuid::Uuid::new_v4().to_string(),
            email: "test@example.com".into(),
            ..Default::default()
        });

        let response = client.sign_up(request).await;

        info!("Response: {:?}", response);
        let status = response.expect_err("se esperaba un error por UUID no-v7");
        assert!(
            status.message().contains("no es un UUID V7"),
            "mensaje inesperado: {}",
            status.message()
        );
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: requiere validación de email en el caso de uso"]
    async fn sign_up_fails_when_email_is_invalid(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!(
            "enviar email sin formato válido (ej. 'no-es-email') -> esperar status InvalidArgument"
        );
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: requiere validación de campos requeridos"]
    async fn sign_up_fails_when_required_fields_are_missing(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let _ = test_env;
        todo!(
            "enviar SignUpRequest con first_name/last_name/document_id vacíos -> esperar InvalidArgument"
        );
    }

    // ---- Autenticación ----------------------------------------------------

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: requiere verificación del id_token (auth)"]
    async fn sign_up_fails_when_id_token_is_invalid(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!("enviar id_token inválido/expirado -> esperar status Unauthenticated");
    }

    // ---- Reglas de negocio ------------------------------------------------

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: requiere UserRepositoryImpl real (exist_user)"]
    async fn sign_up_fails_when_user_already_exists(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!(
            "registrar un user_id v7, reintentar el mismo user_id -> esperar status AlreadyExists"
        );
    }
}

mod sign_in {
    // Imports que usarán los tests cuando se implementen (hoy son stubs).
    use crate::{TestEnv, test_env};
    #[allow(unused_imports)]
    use clickcare::infrastructure::grpc::SignInRequest;
    #[allow(unused_imports)]
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: SignIn no implementado en UserApiImpl"]
    async fn sign_in_succeeds_with_valid_credentials(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!(
            "registrar un usuario y luego sign_in con id_token+provider_id válidos -> esperar Ok"
        );
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: SignIn no implementado en UserApiImpl"]
    async fn sign_in_fails_when_user_not_found(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!("sign_in con un provider_id que no existe -> esperar status NotFound");
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "TODO: SignIn no implementado + verificación del id_token"]
    async fn sign_in_fails_when_id_token_is_invalid(#[future(awt)] test_env: &'static TestEnv) {
        let _ = test_env;
        todo!("sign_in con id_token inválido/expirado -> esperar status Unauthenticated");
    }
}
