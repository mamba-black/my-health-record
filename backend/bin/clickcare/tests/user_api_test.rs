mod scenarios;
mod steps;

use administration::infrastructure::di as administration_di;
use clickcare::infrastructure::grpc::clinic_api_impl::ClinicApiImpl;
use clickcare::infrastructure::grpc::clinic_api_server::ClinicApiServer;
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

                    let service = UserApiImpl::new(Some(conn_str.clone())).await.unwrap();

                    // `administration` se cablea con su propio DI: el harness no
                    // construye tipos ajenos, solo pide el caso de uso ya resuelto.
                    let administration = administration_di::new(
                        administration_di::DBType::Postgres(Some(conn_str)),
                    )
                    .await
                    .unwrap();
                    let clinic_service = ClinicApiImpl::new(std::sync::Arc::clone(
                        &administration.create_clinic_use_case,
                    ));

                    tokio::spawn(async move {
                        Server::builder()
                            .add_service(UserApiServer::new(service))
                            .add_service(ClinicApiServer::new(clinic_service))
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
    use std::sync::LazyLock;
    use std::time::Duration;

    /// Serializa los tests que levantan un worker.
    ///
    /// Los dos comparten la misma cola de Apalis. Corriendo en paralelo, el worker de
    /// un test puede tomar el job del otro y, al abortarse su tarea al terminar, dejarlo
    /// bloqueado sin llegar nunca a `Done`: el otro test agota su plazo esperando un job
    /// que ya nadie procesa. Con el lock, cada worker vive lo suficiente para terminar
    /// lo que tomó.
    static WORKER_LOCK: LazyLock<tokio::sync::Mutex<()>> =
        LazyLock::new(|| tokio::sync::Mutex::new(()));

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

    /// Devuelve el `organization_id` de la única fila de `table` asociada a `user_id`.
    ///
    /// Existe para verificar el discriminador de inquilino: que las entidades locales
    /// no solo se persistan, sino que cuelguen de la clínica correcta.
    async fn organization_id_of(pg_conn: &str, table: &str, user_id: uuid::Uuid) -> uuid::Uuid {
        let conn_str = pg_conn.to_string();
        let statement =
            format!("SELECT organization_id FROM administration.{table} WHERE user_id = $1");
        tokio::task::spawn_blocking(move || {
            let mut db_client = ::postgres::Client::connect(&conn_str, ::postgres::NoTls)
                .expect("Fallo al conectar con Postgres");
            let row = db_client
                .query_one(statement.as_str(), &[&user_id])
                .expect("Fallo al leer el organization_id de administration");
            row.get::<_, uuid::Uuid>(0)
        })
        .await
        .expect("La consulta bloqueante falló")
    }

    /// Devuelve el `id` de la organización cuyo propietario es `user_id`.
    async fn organization_id_owned_by(pg_conn: &str, user_id: uuid::Uuid) -> uuid::Uuid {
        let conn_str = pg_conn.to_string();
        tokio::task::spawn_blocking(move || {
            let mut db_client = ::postgres::Client::connect(&conn_str, ::postgres::NoTls)
                .expect("Fallo al conectar con Postgres");
            let row = db_client
                .query_one(
                    "SELECT id FROM administration.organization WHERE owner_user_id = $1",
                    &[&user_id],
                )
                .expect("Fallo al leer la organización del propietario");
            row.get::<_, uuid::Uuid>(0)
        })
        .await
        .expect("La consulta bloqueante falló")
    }

    /// Corre el worker hasta que el job de `user_id` quede en `Done`, o venza el plazo.
    async fn run_worker_until_done(pg_conn: &str, user_id: uuid::Uuid) -> Option<String> {
        let _worker_guard = WORKER_LOCK.lock().await;

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
    // `create_clinic` está deprecado en el contrato; sigue en uso mientras el worker
    // lo consuma desde `UserCreatedEvent`.
    #[allow(deprecated)]
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

    /// Toda entidad de `administration` pertenece a una clínica. Un usuario que se
    /// registra sin crear ninguna no tiene a cuál pertenecer, así que el evento se
    /// encola y se procesa, pero no materializa nada: su expediente aparecerá por
    /// demanda cuando una clínica lo registre o confirme su primera cita.
    #[rstest]
    #[tokio::test]
    async fn worker_materializes_nothing_for_a_user_without_a_clinic(
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

        // ── 3. Y no debe haber materializado ninguna entidad local ───────────
        for (table, column) in [
            ("patient", "user_id"),
            ("practitioner", "user_id"),
            ("organization", "owner_user_id"),
        ] {
            assert_eq!(
                count_rows(&test_env.pg_connection_string, table, column, user_id).await,
                0,
                "El usuario user_id={user_id} no creó clínica: \
                 administration.{table} no debió recibir ninguna fila"
            );
        }
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

        // El discriminador de inquilino: no basta con que las filas existan, tienen
        // que colgar de la clínica que se acaba de crear.
        let organization_id =
            organization_id_owned_by(&test_env.pg_connection_string, user_id).await;

        for table in ["practitioner", "patient"] {
            assert_eq!(
                organization_id_of(&test_env.pg_connection_string, table, user_id).await,
                organization_id,
                "administration.{table} de user_id={user_id} no apunta a su clínica"
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

/// Verifica `ClinicApi.CreateClinic`: la creación de clínica como operación propia,
/// independiente del alta, disponible para un usuario que ya existe.
mod clinic_api {
    use crate::{TestEnv, test_env};
    use clickcare::infrastructure::grpc::CreateClinicRequest;
    use clickcare::infrastructure::grpc::clinic_api_client::ClinicApiClient;
    use rstest::*;

    async fn create_clinic(
        test_env: &TestEnv,
        owner_user_id: &str,
        name: &str,
    ) -> Result<tonic::Response<clickcare::infrastructure::grpc::CreateClinicResponse>, tonic::Status>
    {
        let mut client = ClinicApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");

        client
            .create_clinic(tonic::Request::new(CreateClinicRequest {
                owner_user_id: owner_user_id.to_string(),
                name: name.to_string(),
                tax_id: Some("20512345678".to_string()),
                given_name: "Ana".into(),
                family_name: Some("Ramírez".into()),
                email: Some(format!("ana-{owner_user_id}@example.com")),
                ..Default::default()
            }))
            .await
    }

    /// Cuenta filas de una tabla de `administration` por columna y valor UUID.
    async fn count_by(pg_conn: &str, table: &str, column: &str, value: uuid::Uuid) -> i64 {
        let conn_str = pg_conn.to_string();
        let statement = format!("SELECT count(*) FROM administration.{table} WHERE {column} = $1");
        tokio::task::spawn_blocking(move || {
            let mut db_client = ::postgres::Client::connect(&conn_str, ::postgres::NoTls)
                .expect("Fallo al conectar con Postgres");
            db_client
                .query_one(statement.as_str(), &[&value])
                .expect("Fallo al contar filas")
                .get::<_, i64>(0)
        })
        .await
        .expect("La consulta bloqueante falló")
    }

    #[rstest]
    #[tokio::test]
    async fn creates_the_clinic_and_the_owner_practitioner(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let owner_user_id = uuid::Uuid::now_v7();

        let response = create_clinic(test_env, &owner_user_id.to_string(), "Clínica San Borja")
            .await
            .expect("La creación de la clínica falló")
            .into_inner();

        assert!(
            !response.already_existed,
            "Es la primera clínica de este usuario"
        );

        let organization_id: uuid::Uuid = response
            .organization_id
            .parse()
            .expect("El organization_id devuelto no es un UUID");

        assert_eq!(
            count_by(
                &test_env.pg_connection_string,
                "organization",
                "owner_user_id",
                owner_user_id
            )
            .await,
            1,
            "No se persistió la clínica"
        );
        assert_eq!(
            count_by(
                &test_env.pg_connection_string,
                "practitioner",
                "organization_id",
                organization_id
            )
            .await,
            1,
            "No se materializó la ficha del propietario en su clínica"
        );
    }

    /// Reintentar no crea una segunda clínica: devuelve la misma.
    #[rstest]
    #[tokio::test]
    async fn returns_the_same_clinic_when_called_twice(#[future(awt)] test_env: &'static TestEnv) {
        let owner_user_id = uuid::Uuid::now_v7().to_string();

        let first = create_clinic(test_env, &owner_user_id, "Clínica Lince")
            .await
            .expect("La primera creación falló")
            .into_inner();
        let second = create_clinic(test_env, &owner_user_id, "Clínica Lince")
            .await
            .expect("La segunda creación falló")
            .into_inner();

        assert!(!first.already_existed);
        assert!(
            second.already_existed,
            "La segunda debió reportar que ya existía"
        );
        assert_eq!(
            first.organization_id, second.organization_id,
            "Debió devolver la misma clínica, no crear otra"
        );
        assert_eq!(first.practitioner_id, second.practitioner_id);
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_an_owner_user_id_that_is_not_uuid_v7(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let status = create_clinic(
            test_env,
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "Clínica X",
        )
        .await
        .expect_err("Debió rechazar un UUID v4");

        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }
}
