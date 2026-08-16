mod scenarios;
mod steps;

use clickcare::infrastructure::grpc::user_api_impl::UserApiImpl;
use clickcare::infrastructure::grpc::user_api_server::UserApiServer;
use clickcare::infrastructure::log::init_logger;
use dotenvy::dotenv;
use log::info;
use rstest::*;
use std::path::PathBuf;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::OnceCell;
use tonic::transport::Server;
use tracing::debug;

pub struct TestEnv {
    pub grpc_addr: String,
    pub pg_connection_string: String,
}

static TEST_ENV: OnceCell<TestEnv> = OnceCell::const_new();

#[fixture]
pub async fn test_env() -> &'static TestEnv {
    debug!("llamando a test_env()");
    TEST_ENV
        .get_or_init(|| async {
            init_logger();
            dotenv().ok();

            let pg_cache_path = std::env::temp_dir().join("clickcare_pg_env.txt");
            let pg_lock_path = std::env::temp_dir().join("clickcare_pg.lock");

            async fn check_pg_active(pg_conn: &str) -> bool {
                let clean = pg_conn.trim();
                if let Some(pos) = clean.rfind(':')
                    && let Some(slash_pos) = clean[pos + 1..].find('/')
                {
                    let port_str = &clean[pos + 1..pos + 1 + slash_pos];
                    if let Ok(port) = port_str.parse::<u16>() {
                        return TcpStream::connect(format!("127.0.0.1:{port}"))
                            .await
                            .is_ok();
                    }
                }
                false
            }

            // 1. Compartir una única instancia de Postgres entre todos los procesos
            let pg_connection_string: String = 'init_db: {
                if let Ok(cached_pg) = std::fs::read_to_string(&pg_cache_path) {
                    let pg_conn = cached_pg.trim().to_string();
                    if check_pg_active(&pg_conn).await {
                        info!("Reutilizando contenedor de Postgres activo en {pg_conn}");
                        break 'init_db pg_conn;
                    }
                }

                let lock_file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&pg_lock_path)
                    .expect("Fallo al abrir lock de Postgres");

                #[cfg(unix)]
                unsafe {
                    use std::os::unix::io::AsRawFd;
                    libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX);
                }

                if let Ok(cached_pg) = std::fs::read_to_string(&pg_cache_path) {
                    let pg_conn = cached_pg.trim().to_string();
                    if check_pg_active(&pg_conn).await {
                        #[cfg(unix)]
                        unsafe {
                            use std::os::unix::io::AsRawFd;
                            libc::flock(lock_file.as_raw_fd(), libc::LOCK_UN);
                        }
                        break 'init_db pg_conn;
                    }
                }

                log::debug!("== INICIANDO CONTENEDOR POSTGRES COMPARTIDO ==");
                let user = "admin";
                let password = "admin123";
                let schema_path =
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ddl/table.sql");
                info!("schema_path: {:?}", schema_path);

                let timestamp = jiff::Timestamp::now()
                    .strftime("%Y%m%d%H%M%S%f")
                    .to_string();
                let container_name = format!("clickcare-test-{}", timestamp);

                let pg_container = postgres::Postgres::default()
                    .with_user(user)
                    .with_password(password)
                    .with_tag("18")
                    .with_container_name(container_name.clone())
                    .with_copy_to("/docker-entrypoint-initdb.d/001_schema.sql", schema_path)
                    .start()
                    .await
                    .unwrap();
                info!("Container INICIADO");

                let host_port = pg_container.get_host_port_ipv4(5432).await.unwrap();
                let pg_conn = format!(
                    "postgres://{}:{}@127.0.0.1:{host_port}/postgres?options=-c%20search_path%3Didentity%2Cpublic",
                    user, password
                );

                let _ = std::fs::write(&pg_cache_path, &pg_conn);

                #[cfg(unix)]
                unsafe {
                    use std::os::unix::io::AsRawFd;
                    libc::flock(lock_file.as_raw_fd(), libc::LOCK_UN);
                }

                // Programar la autodestrucción del contenedor y archivos en /tmp tras 10 segundos
                let cleanup_cmd = format!(
                    "#sleep 10 && podman rm -f {} && rm -f {} {}",
                    container_name,
                    pg_cache_path.display(),
                    pg_lock_path.display()
                );
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("({}) >/dev/null 2>&1 &", cleanup_cmd))
                    .spawn();

                std::mem::forget(pg_container);

                pg_conn
            };

            // 2. Cada proceso de test inicia su propio servidor gRPC local (<1ms) conectado a la misma DB
            let server_rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            let conn_str = pg_connection_string.clone();
            let grpc_addr = server_rt
                .spawn(async move {
                    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                    let addr = listener.local_addr().unwrap();
                    let grpc_addr = format!("http://{}", addr);

                    let service = UserApiImpl::new(Some(conn_str)).await.unwrap();

                    tokio::spawn(async move {
                        Server::builder()
                            .add_service(UserApiServer::new(service))
                            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                                listener,
                            ))
                            .await
                            .unwrap();
                    });

                    grpc_addr
                })
                .await
                .unwrap();
            Box::leak(Box::new(server_rt));

            log::debug!("== SERVIDOR gRPC LISTO EN {} ==", grpc_addr);

            TestEnv {
                grpc_addr,
                pg_connection_string,
            }
        })
        .await
}

mod sign_up {
    use crate::{TestEnv, test_env};
    use rstest::*;

    // ---- Happy path (BDD Feature Scenarios) ---------------------------------

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

/// Verifica la tubería de eventos completa: `sign_up` encola un `UserCreatedEvent`
/// en Postgres y el worker de `crates/administration` lo consume.
mod administration_worker {
    use crate::{TestEnv, test_env};
    use administration::infrastructure::di as administration_di;
    use clickcare::infrastructure::grpc::SignUpRequest;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use rstest::*;
    use std::time::Duration;

    /// Consulta el estado del job encolado para `user_id`, o `None` si aún no existe.
    async fn job_status(pg_conn: &str, user_id: uuid::Uuid) -> Option<String> {
        let conn_str = pg_conn.to_string();
        tokio::task::spawn_blocking(move || {
            let mut db_client = ::postgres::Client::connect(&conn_str, ::postgres::NoTls).ok()?;
            let rows = db_client
                .query(
                    "SELECT status FROM apalis.jobs \
                     WHERE job_type = $1 AND convert_from(job, 'UTF8') LIKE $2",
                    &[
                        &app_core::domain::event::UserCreatedEvent::QUEUE,
                        &format!("%{user_id}%"),
                    ],
                )
                .ok()?;
            rows.first().map(|row| row.get::<_, String>("status"))
        })
        .await
        .ok()
        .flatten()
    }

    /// Cuenta las filas de una tabla de `administration` asociadas a `user_id`.
    ///
    /// `owner_column` existe porque la organización referencia al usuario como
    /// `owner_user_id` y las otras dos como `user_id`.
    async fn count_rows(
        pg_conn: &str,
        table: &str,
        owner_column: &str,
        user_id: uuid::Uuid,
    ) -> i64 {
        let conn_str = pg_conn.to_string();
        let statement =
            format!("SELECT count(*) FROM administration.{table} WHERE {owner_column} = $1");
        tokio::task::spawn_blocking(move || {
            let mut db_client = ::postgres::Client::connect(&conn_str, ::postgres::NoTls)
                .expect("Fallo al conectar con Postgres");
            let row = db_client
                .query_one(statement.as_str(), &[&user_id])
                .expect("Fallo al contar las filas de administration");
            row.get::<_, i64>(0)
        })
        .await
        .expect("La consulta bloqueante falló")
    }

    /// Corre el worker hasta que el job de `user_id` quede en `Done`, o venza el plazo.
    async fn run_worker_until_done(pg_conn: &str, user_id: uuid::Uuid) -> Option<String> {
        let di = administration_di::new(administration_di::DBType::Postgres(Some(
            pg_conn.to_string(),
        )))
        .await
        .expect("Fallo al construir el DI de administration");

        let worker = tokio::spawn(di.run_worker());

        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        let mut last_status = None;
        while std::time::Instant::now() < deadline {
            last_status = job_status(pg_conn, user_id).await;
            if last_status.as_deref() == Some("Done") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        worker.abort();

        last_status
    }

    /// Registra un usuario por gRPC y devuelve su `user_id`.
    async fn sign_up(test_env: &TestEnv, create_clinic: bool) -> uuid::Uuid {
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");

        let user_id = uuid::Uuid::now_v7();
        let nonce = &uuid::Uuid::now_v7().to_string()[..8];
        let response = client
            .sign_up(tonic::Request::new(SignUpRequest {
                id_token: "test-token".into(),
                user_id: user_id.to_string(),
                email: format!("worker-{nonce}@example.com"),
                given_name: "Worker".into(),
                create_clinic,
                ..Default::default()
            }))
            .await;
        assert!(response.is_ok(), "El sign_up falló: {response:?}");

        user_id
    }

    #[rstest]
    #[tokio::test]
    async fn sign_up_enqueues_event_and_worker_persists_the_patient_record(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        // ── 1. Un sign_up debe encolar el evento ─────────────────────────────
        let user_id = sign_up(test_env, false).await;

        let status = job_status(&test_env.pg_connection_string, user_id).await;
        assert!(
            status.is_some(),
            "El sign_up no encoló ningún UserCreatedEvent para user_id={user_id}"
        );

        // ── 2. El worker debe consumirlo hasta dejarlo en 'Done' ─────────────
        let last_status = run_worker_until_done(&test_env.pg_connection_string, user_id).await;
        assert_eq!(
            last_status.as_deref(),
            Some("Done"),
            "El worker de administration no procesó el evento de user_id={user_id} \
             (último estado observado: {last_status:?})"
        );

        // ── 3. Y debe haber dejado el expediente persistido ──────────────────
        assert_eq!(
            count_rows(
                &test_env.pg_connection_string,
                "patient",
                "user_id",
                user_id
            )
            .await,
            1,
            "El worker no persistió el expediente de user_id={user_id}"
        );
        assert_eq!(
            count_rows(
                &test_env.pg_connection_string,
                "organization",
                "owner_user_id",
                user_id
            )
            .await,
            0,
            "No debió crear una organización: el usuario no es propietario de clínica"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn worker_persists_organization_and_practitioner_for_a_clinic_owner(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let user_id = sign_up(test_env, true).await;

        let last_status = run_worker_until_done(&test_env.pg_connection_string, user_id).await;
        assert_eq!(
            last_status.as_deref(),
            Some("Done"),
            "El worker no procesó el evento de user_id={user_id} \
             (último estado observado: {last_status:?})"
        );

        for (table, column) in [
            ("organization", "owner_user_id"),
            ("practitioner", "user_id"),
            ("patient", "user_id"),
        ] {
            assert_eq!(
                count_rows(&test_env.pg_connection_string, table, column, user_id).await,
                1,
                "El worker no persistió administration.{table} para user_id={user_id}"
            );
        }
    }
}

mod sign_in {
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
