mod scenarios;
mod scheduling;
mod steps;

use clickcare::infrastructure::grpc::clinic_api_server::ClinicApiServer;
use clickcare::infrastructure::grpc::patient_api_server::PatientApiServer;
use clickcare::infrastructure::grpc::user_api_server::UserApiServer;
use clickcare::infrastructure::log::init_observability;
use dotenvy::dotenv;
use rstest::*;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::OnceCell;
use tonic::transport::Server;
use tracing::debug;
use tracing::info;
use user::infrastructure::config::JwtSessionServiceImpl;

pub struct TestEnv {
    pub grpc_addr: String,
    pub pg_connection_string: String,
}

static TEST_ENV: OnceCell<TestEnv> = OnceCell::const_new();

/// Clave de sesión de las pruebas: permite fabricar tokens vencidos o ajenos.
pub const TEST_SESSION_SECRET: &[u8] = b"clave-de-sesion-de-pruebas";

/// Servicio de sesión con la clave de las pruebas (acceso de 5 minutos).
pub fn test_session_service() -> JwtSessionServiceImpl {
    JwtSessionServiceImpl::new(TEST_SESSION_SECRET.to_vec(), 300, 3600)
}

/// Nombre fijo del contenedor de Postgres de las pruebas de integración.
const TEST_CONTAINER_NAME: &str = "clickcare-test";

/// Borra el contenedor de pruebas anterior, si existe. Prueba con podman y, si no está
/// instalado, con docker; que no exista el contenedor no es un error.
fn remove_test_container() {
    for engine in ["podman", "docker"] {
        let removed = std::process::Command::new(engine)
            .args(["rm", "-f", TEST_CONTAINER_NAME])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if removed.is_ok() {
            return;
        }
    }
}

/// Programa el borrado del contenedor para cuando termine el último proceso de
/// prueba que lo usa.
///
/// Cada proceso de prueba toma un lock compartido sobre `users_lock` mientras vive.
/// El vigilante, un proceso aparte, pide ese mismo lock en exclusiva: lo obtiene
/// solo cuando ya no queda ningún proceso de prueba, y entonces borra el contenedor
/// y la caché. Se puede conservar el contenedor para revisarlo a mano con
/// `CLICKCARE_KEEP_TEST_DB=1`.
fn schedule_test_container_cleanup(users_lock: &std::path::Path, pg_cache: &std::path::Path) {
    if std::env::var_os("CLICKCARE_KEEP_TEST_DB").is_some() {
        info!("CLICKCARE_KEEP_TEST_DB definido: el contenedor {TEST_CONTAINER_NAME} se conserva");
        return;
    }
    let cleanup = format!(
        "flock -x '{users}' sh -c '(podman rm -f {name} || docker rm -f {name}); rm -f \"{cache}\"'",
        users = users_lock.display(),
        name = TEST_CONTAINER_NAME,
        cache = pg_cache.display(),
    );
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("({cleanup}) >/dev/null 2>&1 &"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

#[fixture]
pub async fn test_env() -> &'static TestEnv {
    debug!("llamando a test_env()");
    TEST_ENV
        .get_or_init(|| async {
            init_observability();
            dotenv().ok();

            let pg_cache_path = std::env::temp_dir().join("clickcare_pg_env.txt");
            let pg_lock_path = std::env::temp_dir().join("clickcare_pg.lock");
            let users_lock_path = std::env::temp_dir().join("clickcare_pg_users.lock");

            // Este proceso usa el contenedor hasta que termina: el lock compartido se
            // suelta solo al salir, porque el archivo no se cierra nunca. Se toma antes
            // de mirar la caché, así un borrado en curso termina antes de reutilizarla.
            let users_lock = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&users_lock_path)
                .expect("Fallo al abrir el lock de usuarios de Postgres");
            #[cfg(unix)]
            unsafe {
                use std::os::unix::io::AsRawFd;
                libc::flock(users_lock.as_raw_fd(), libc::LOCK_SH);
            }
            std::mem::forget(users_lock);

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

            // 1. Compartir una única instancia de Postgres entre los procesos de prueba.
            //    El contenedor tiene nombre fijo y testcontainers no lo borra al salir
            //    (`mem::forget`), porque otro proceso puede estar usándolo. Lo borra el
            //    vigilante de `schedule_test_container_cleanup` cuando termina el último
            //    proceso. Si queda uno anterior que no responde, se reemplaza.
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

            let cached = std::fs::read_to_string(&pg_cache_path)
                .ok()
                .map(|cached| cached.trim().to_string());
            let pg_connection_string = match cached {
                Some(pg_conn) if check_pg_active(&pg_conn).await => {
                    info!("Reutilizando el contenedor de Postgres {TEST_CONTAINER_NAME}");
                    pg_conn
                }
                _ => {
                    remove_test_container();

                    tracing::debug!("== INICIANDO CONTENEDOR POSTGRES COMPARTIDO ==");
                    let user = "admin";
                    let password = "admin123";
                    let pg_container = postgres::Postgres::default()
                        .with_user(user)
                        .with_password(password)
                        .with_tag("18")
                        .with_container_name(TEST_CONTAINER_NAME)
                        .start()
                        .await
                        .unwrap();
                    info!("Container INICIADO");

                    let host_port = pg_container.get_host_port_ipv4(5432).await.unwrap();
                    std::mem::forget(pg_container);
                    format!(
                        "postgres://{}:{}@127.0.0.1:{host_port}/postgres?options=-c%20search_path%3Didentity%2Cpublic",
                        user, password
                    )
                }
            };

            // El esquema sale de las mismas migraciones que aplica el servidor al
            // arrancar. Se migra siempre, también al reutilizar el contenedor, para que
            // una migración nueva llegue a la base de pruebas. Va dentro del lock y antes
            // de publicar la URL, así ningún proceso ve una base a medio migrar.
            clickcare::infrastructure::migration::run(&pg_connection_string)
                .await
                .expect("Fallo al aplicar las migraciones en el contenedor de pruebas");
            let _ = std::fs::write(&pg_cache_path, &pg_connection_string);

            #[cfg(unix)]
            unsafe {
                use std::os::unix::io::AsRawFd;
                libc::flock(lock_file.as_raw_fd(), libc::LOCK_UN);
            }
            schedule_test_container_cleanup(&users_lock_path, &pg_cache_path);

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

                    // Mismo cableado que el servidor real, con el verificador de Google
                    // simulado y una clave de sesión conocida por las pruebas.
                    let services = clickcare::infrastructure::build_services(
                        Some(conn_str),
                        user::infrastructure::config::DIOverrides {
                            token_service: Some(std::sync::Arc::new(
                                user::testing::MockTokenServiceImpl,
                            )),
                            session_service: Some(std::sync::Arc::new(test_session_service())),
                            ..Default::default()
                        },
                    )
                    .await
                    .unwrap();
                    let authenticator = services.authenticator.clone();

                    tokio::spawn(async move {
                        Server::builder()
                            .add_service(UserApiServer::new(services.user_api))
                            .add_service(ClinicApiServer::with_interceptor(
                                services.clinic_api,
                                authenticator.interceptor(),
                            ))
                            .add_service(PatientApiServer::with_interceptor(
                                services.patient_api,
                                authenticator.interceptor(),
                            ))
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

            tracing::debug!("== SERVIDOR gRPC LISTO EN {} ==", grpc_addr);

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
    use administration::infrastructure::config as administration_di;
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

        let user_di = user::infrastructure::config::new(
            user::infrastructure::config::DBType::Postgres(Some(pg_conn.to_string())),
        )
        .await
        .expect("Fallo al construir el DI de user");
        let di = administration_di::new(
            administration_di::DBType::Postgres(Some(pg_conn.to_string())),
            std::sync::Arc::clone(&user_di.person_links),
        )
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
        let nonce = user_id.simple();
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
    #[ignore = "intermitente: a veces el worker no termina en 15 s (my-health-record-bo3)"]
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

        for table in ["practitioner", "patient", "membership"] {
            assert_eq!(
                organization_id_of(&test_env.pg_connection_string, table, user_id).await,
                organization_id,
                "administration.{table} de user_id={user_id} no apunta a su clínica"
            );
        }

        // Los permisos salen de la membresía: el dueño debe quedar activo en su clínica.
        assert_eq!(
            count_rows(
                &test_env.pg_connection_string,
                "membership",
                "role = 'OWNER' AND status = 'ACTIVE' AND user_id",
                user_id
            )
            .await,
            1,
            "El worker no dio al usuario user_id={user_id} la membresía de dueño activa"
        );
    }
}

mod sign_in {
    use crate::{TestEnv, test_env};
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::{SignInRequest, SignUpIntent, SignUpRequest};
    use rstest::rstest;

    fn encode_base64url(input_bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut encoded = String::new();
        let mut index = 0;

        while index < input_bytes.len() {
            let byte0 = input_bytes[index];
            let byte1 = if index + 1 < input_bytes.len() {
                input_bytes[index + 1]
            } else {
                0
            };
            let byte2 = if index + 2 < input_bytes.len() {
                input_bytes[index + 2]
            } else {
                0
            };

            encoded.push(ALPHABET[(byte0 >> 2) as usize] as char);
            encoded.push(ALPHABET[(((byte0 & 0x03) << 4) | (byte1 >> 4)) as usize] as char);
            if index + 1 < input_bytes.len() {
                encoded.push(ALPHABET[(((byte1 & 0x0F) << 2) | (byte2 >> 6)) as usize] as char);
            }
            if index + 2 < input_bytes.len() {
                encoded.push(ALPHABET[(byte2 & 0x3F) as usize] as char);
            }

            index += 3;
        }

        encoded
    }

    fn build_jwt_token(email: &str, expiration_epoch_seconds: Option<i64>) -> String {
        let header = encode_base64url(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload_json = match expiration_epoch_seconds {
            Some(exp) => format!(r#"{{"sub":"google-sub-123","email":"{email}","exp":{exp}}}"#),
            None => format!(r#"{{"sub":"google-sub-123","email":"{email}"}}"#),
        };
        let payload = encode_base64url(payload_json.as_bytes());
        format!("{header}.{payload}.signature")
    }

    #[rstest]
    #[tokio::test]
    async fn sign_in_succeeds_with_valid_credentials(#[future(awt)] test_env: &'static TestEnv) {
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");

        let user_id = uuid::Uuid::now_v7();
        let email = format!("signin-ok-{}@example.com", user_id.simple());
        let future_expiration = jiff::Timestamp::now().as_second() + 3600;
        let id_token = build_jwt_token(&email, Some(future_expiration));

        client
            .sign_up(tonic::Request::new(SignUpRequest {
                id_token: id_token.clone(),
                user_id: user_id.to_string(),
                provider_id: "google.com".into(),
                provider_name: "Google".into(),
                email: email.clone(),
                given_name: "Lucía".into(),
                family_name: Some("Mendoza".into()),
                intent: SignUpIntent::IntentClinicOwner as i32,
                ..Default::default()
            }))
            .await
            .expect("El registro previo al sign_in debió tener éxito");

        let sign_in_response = client
            .sign_in(tonic::Request::new(SignInRequest {
                id_token,
                provider_id: "google.com".into(),
            }))
            .await
            .expect("El sign_in con credenciales válidas debió tener éxito")
            .into_inner();

        assert_eq!(sign_in_response.user_id, user_id.to_string());
        assert_eq!(sign_in_response.email, email);
        assert_eq!(sign_in_response.given_name, "Lucía");
        assert_eq!(sign_in_response.family_name.as_deref(), Some("Mendoza"));
        assert!(sign_in_response.is_owner);
    }

    #[rstest]
    #[tokio::test]
    async fn sign_in_fails_when_user_not_found(#[future(awt)] test_env: &'static TestEnv) {
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");

        let unregistered_email =
            format!("unregistered-{}@example.com", uuid::Uuid::now_v7().simple());
        let id_token = build_jwt_token(&unregistered_email, None);

        let status = client
            .sign_in(tonic::Request::new(SignInRequest {
                id_token,
                provider_id: "google.com".into(),
            }))
            .await
            .expect_err("El sign_in de un usuario no registrado debió fallar con NotFound");

        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[rstest]
    #[tokio::test]
    async fn sign_in_fails_when_id_token_is_invalid(#[future(awt)] test_env: &'static TestEnv) {
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");

        let malformed_status = client
            .sign_in(tonic::Request::new(SignInRequest {
                id_token: "token-malformado".into(),
                provider_id: "google.com".into(),
            }))
            .await
            .expect_err("Un id_token malformado debió ser rechazado");
        assert_eq!(malformed_status.code(), tonic::Code::Unauthenticated);

        let expired_epoch = jiff::Timestamp::now().as_second() - 120;
        let expired_token = build_jwt_token("expired@example.com", Some(expired_epoch));
        let expired_status = client
            .sign_in(tonic::Request::new(SignInRequest {
                id_token: expired_token,
                provider_id: "google.com".into(),
            }))
            .await
            .expect_err("Un id_token expirado debió ser rechazado");
        assert_eq!(expired_status.code(), tonic::Code::Unauthenticated);
    }
}

/// Utilidades compartidas: cuentas con sesión y peticiones autenticadas.
pub mod support {
    use crate::TestEnv;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::{SignUpIntent, SignUpRequest};
    use uuid::Uuid;

    fn encode_base64url(input_bytes: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut encoded = String::new();
        for chunk in input_bytes.chunks(3) {
            let b0 = chunk[0];
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);
            encoded.push(ALPHABET[(b0 >> 2) as usize] as char);
            encoded.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            if chunk.len() > 1 {
                encoded.push(ALPHABET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
            }
            if chunk.len() > 2 {
                encoded.push(ALPHABET[(b2 & 0x3F) as usize] as char);
            }
        }
        encoded
    }

    /// `id_token` que acepta el verificador simulado de las pruebas.
    pub fn id_token(email: &str) -> String {
        let header = encode_base64url(br#"{"alg":"RS256","typ":"JWT"}"#);
        let exp = jiff::Timestamp::now().as_second() + 3600;
        let payload = encode_base64url(
            format!(r#"{{"sub":"google-sub","email":"{email}","exp":{exp}}}"#).as_bytes(),
        );
        format!("{header}.{payload}.signature")
    }

    /// Cuenta registrada con su sesión abierta.
    pub struct Account {
        pub user_id: Uuid,
        pub email: String,
        pub access_token: String,
        pub refresh_token: String,
    }

    /// Registra una cuenta (en la red por defecto) y devuelve su sesión.
    pub async fn sign_up(test_env: &TestEnv, given_name: &str, intent: SignUpIntent) -> Account {
        sign_up_in(test_env, given_name, intent, None).await
    }

    /// Registra una cuenta entrando por el enlace de una clínica: queda en la red de esa
    /// clínica. Sin subdominio, en la red por defecto.
    pub async fn sign_up_in(
        test_env: &TestEnv,
        given_name: &str,
        intent: SignUpIntent,
        subdomain: Option<&str>,
    ) -> Account {
        let user_id = Uuid::now_v7();
        let email = format!(
            "{}-{}@example.com",
            given_name.to_lowercase(),
            user_id.simple()
        );
        let mut client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");
        let mut request = tonic::Request::new(SignUpRequest {
            id_token: id_token(&email),
            user_id: user_id.to_string(),
            provider_id: "google.com".into(),
            provider_name: "Google".into(),
            email: email.clone(),
            given_name: given_name.into(),
            intent: intent as i32,
            ..Default::default()
        });
        if let Some(subdomain) = subdomain {
            request
                .metadata_mut()
                .insert("x-subdomain", subdomain.parse().expect("subdominio válido"));
        }
        let session = client
            .sign_up(request)
            .await
            .expect("El registro debió tener éxito")
            .into_inner()
            .session
            .expect("El registro debió abrir una sesión");
        Account {
            user_id,
            email,
            access_token: session.access_token,
            refresh_token: session.refresh_token,
        }
    }

    /// Petición con `authorization: Bearer <token>`.
    pub fn authed<T>(message: T, token: &str) -> tonic::Request<T> {
        let mut request = tonic::Request::new(message);
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {token}")
                .parse()
                .expect("encabezado válido"),
        );
        request
    }
}

/// Verifica `ClinicApi.CreateClinic`: la creación de clínica como operación propia,
/// independiente del alta, disponible para un usuario que ya existe.
mod clinic_api {
    use crate::support::{Account, authed, sign_up};
    use crate::{TestEnv, test_env};
    use clickcare::infrastructure::grpc::clinic_api_client::ClinicApiClient;
    use clickcare::infrastructure::grpc::{CreateClinicRequest, SignUpIntent};
    use rstest::*;

    async fn create_clinic(
        test_env: &TestEnv,
        owner: &Account,
        name: &str,
    ) -> Result<tonic::Response<clickcare::infrastructure::grpc::CreateClinicResponse>, tonic::Status>
    {
        let mut client = ClinicApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");
        client
            .create_clinic(authed(
                CreateClinicRequest {
                    // El dueño sale de la sesión: este dato se ignora.
                    owner_user_id: uuid::Uuid::new_v4().to_string(),
                    name: name.to_string(),
                    subdomain: format!("clinica-{}", owner.user_id.simple()),
                    given_name: "Ana".into(),
                    family_name: Some("Ramírez".into()),
                    email: Some(owner.email.clone()),
                    ..Default::default()
                },
                &owner.access_token,
            ))
            .await
    }

    /// Cuenta filas de una tabla de `administration` por columna y valor UUID.
    pub(crate) async fn count_by(
        pg_conn: &str,
        table: &str,
        column: &str,
        value: uuid::Uuid,
    ) -> i64 {
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
        let owner = sign_up(test_env, "Ana", SignUpIntent::IntentClinicOwner).await;
        let response = create_clinic(test_env, &owner, "Clínica San Borja")
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

        let pg = &test_env.pg_connection_string;
        assert_eq!(
            count_by(pg, "organization", "owner_user_id", owner.user_id).await,
            1,
            "No se persistió la clínica a nombre del dueño de la sesión"
        );
        assert_eq!(
            count_by(pg, "practitioner", "organization_id", organization_id).await,
            1,
            "No se materializó la ficha del propietario en su clínica"
        );
        assert_eq!(
            count_by(pg, "membership", "organization_id", organization_id).await,
            1,
            "Creación de clínica: debió quedar la membresía de dueño"
        );
    }

    /// Reintentar no crea una segunda clínica: devuelve la misma.
    #[rstest]
    #[tokio::test]
    async fn returns_the_same_clinic_when_called_twice(#[future(awt)] test_env: &'static TestEnv) {
        let owner = sign_up(test_env, "Lince", SignUpIntent::IntentClinicOwner).await;
        let first = create_clinic(test_env, &owner, "Clínica Lince")
            .await
            .expect("La primera creación falló")
            .into_inner();
        let second = create_clinic(test_env, &owner, "Clínica Lince")
            .await
            .expect("La segunda creación falló")
            .into_inner();

        assert!(!first.already_existed);
        assert!(
            second.already_existed,
            "La segunda debió reportar que ya existía"
        );
        assert_eq!(first.organization_id, second.organization_id);
        assert_eq!(first.practitioner_id, second.practitioner_id);
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_creating_a_clinic_without_session(#[future(awt)] test_env: &'static TestEnv) {
        let mut client = ClinicApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con el servidor gRPC");
        let status = client
            .create_clinic(tonic::Request::new(CreateClinicRequest {
                name: "Clínica X".into(),
                subdomain: "clinica-sin-sesion".into(),
                given_name: "X".into(),
                ..Default::default()
            }))
            .await
            .expect_err("Sin sesión debió rechazarse");
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }
}

/// Pruebas de integración de aislamiento multi-tenant por subdominio y red clínica.
mod multi_tenancy {
    use crate::support::{authed, sign_up};
    use crate::{TestEnv, test_env};
    use clickcare::infrastructure::grpc::clinic_api_client::ClinicApiClient;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::{CreateClinicRequest, SignUpIntent, SignUpRequest};
    use rstest::*;

    #[rstest]
    #[tokio::test]
    async fn isolates_users_by_subdomain_and_network(#[future(awt)] test_env: &'static TestEnv) {
        let mut clinic_client = ClinicApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con ClinicApiClient");
        let mut user_client = UserApiClient::connect(test_env.grpc_addr.clone())
            .await
            .expect("Fallo al conectar con UserApiClient");

        let owner_a = sign_up(test_env, "PropietarioA", SignUpIntent::IntentClinicOwner).await;
        let owner_b = sign_up(test_env, "PropietarioB", SignUpIntent::IntentClinicOwner).await;

        let sub_a = format!("red-a-{}", owner_a.user_id.simple());
        let sub_b = format!("red-b-{}", owner_b.user_id.simple());

        // 1. Crear Clínica A y Clínica B con diferentes subdominios (y redes)
        clinic_client
            .create_clinic(authed(
                CreateClinicRequest {
                    name: "Clínica Red A".into(),
                    subdomain: sub_a.clone(),
                    given_name: "Propietario A".into(),
                    ..Default::default()
                },
                &owner_a.access_token,
            ))
            .await
            .expect("Fallo al crear Clínica Red A");

        clinic_client
            .create_clinic(authed(
                CreateClinicRequest {
                    name: "Clínica Red B".into(),
                    subdomain: sub_b.clone(),
                    given_name: "Propietario B".into(),
                    ..Default::default()
                },
                &owner_b.access_token,
            ))
            .await
            .expect("Fallo al crear Clínica Red B");

        let shared_email = format!(
            "medico-compartido-{}@example.com",
            uuid::Uuid::now_v7().simple()
        );

        // 2. Registrar usuario en Red A (usando x-subdomain)
        let user_id_a = uuid::Uuid::now_v7();
        let mut req_a = tonic::Request::new(SignUpRequest {
            id_token: "token-a".into(),
            user_id: user_id_a.to_string(),
            email: shared_email.clone(),
            given_name: "Médico".into(),
            family_name: Some("Red A".into()),
            ..Default::default()
        });
        req_a
            .metadata_mut()
            .insert("x-subdomain", sub_a.parse().unwrap());

        let res_a = user_client.sign_up(req_a).await;
        assert!(
            res_a.is_ok(),
            "El registro en Red A debió tener éxito: {res_a:?}"
        );

        // 3. Registrar al mismo correo en Red B (usando encabezado host)
        let user_id_b = uuid::Uuid::now_v7();
        let mut req_b = tonic::Request::new(SignUpRequest {
            id_token: "token-b".into(),
            user_id: user_id_b.to_string(),
            email: shared_email.clone(),
            given_name: "Médico".into(),
            family_name: Some("Red B".into()),
            ..Default::default()
        });
        req_b.metadata_mut().insert(
            "host",
            format!("{sub_b}.clickcare.com:50051").parse().unwrap(),
        );

        let res_b = user_client.sign_up(req_b).await;
        assert!(
            res_b.is_ok(),
            "El registro con el mismo email en Red B debió tener éxito por aislamiento de red: {res_b:?}"
        );

        // 4. Intentar registrar el mismo correo de nuevo en Red A -> Debe fallar por AlreadyExists
        let user_id_a_dup = uuid::Uuid::now_v7();
        let mut req_a_dup = tonic::Request::new(SignUpRequest {
            id_token: "token-a-dup".into(),
            user_id: user_id_a_dup.to_string(),
            email: shared_email.clone(),
            given_name: "Médico Duplicado".into(),
            ..Default::default()
        });
        req_a_dup
            .metadata_mut()
            .insert("x-subdomain", sub_a.parse().unwrap());

        let err_a_dup = user_client
            .sign_up(req_a_dup)
            .await
            .expect_err("Debió fallar por correo duplicado en Red A");
        assert_eq!(err_a_dup.code(), tonic::Code::AlreadyExists);

        // 5. Subdominio no existente -> NotFound
        let mut req_not_found = tonic::Request::new(SignUpRequest {
            id_token: "token".into(),
            user_id: uuid::Uuid::now_v7().to_string(),
            email: "otro@example.com".into(),
            given_name: "Usuario".into(),
            ..Default::default()
        });
        req_not_found
            .metadata_mut()
            .insert("x-subdomain", "no-existe-subdomain".parse().unwrap());
        let err_nf = user_client
            .sign_up(req_not_found)
            .await
            .expect_err("Subdominio no registrado debió retornar NotFound");
        assert_eq!(err_nf.code(), tonic::Code::NotFound);

        // 6. Subdominio reservado de la plataforma -> InvalidArgument
        let mut req_reserved = tonic::Request::new(SignUpRequest {
            id_token: "token".into(),
            user_id: uuid::Uuid::now_v7().to_string(),
            email: "admin@example.com".into(),
            given_name: "Admin".into(),
            ..Default::default()
        });
        req_reserved
            .metadata_mut()
            .insert("x-subdomain", "app".parse().unwrap());
        let err_reserved = user_client
            .sign_up(req_reserved)
            .await
            .expect_err("Subdominio reservado debió retornar InvalidArgument");
        assert_eq!(err_reserved.code(), tonic::Code::InvalidArgument);
    }
}

/// Control de acceso por rol y datos (`role-based-access-by-profile-data`).
mod access_control {
    use crate::support::{Account, authed, sign_up};
    use crate::{TEST_SESSION_SECRET, TestEnv, test_env, test_session_service};
    use clickcare::infrastructure::grpc::clinic_api_client::ClinicApiClient;
    use clickcare::infrastructure::grpc::patient_api_client::PatientApiClient;
    use clickcare::infrastructure::grpc::user_api_client::UserApiClient;
    use clickcare::infrastructure::grpc::*;
    use prost::Message;
    use rstest::*;
    use tonic::Code;
    use user::application::integration::service::session_service::SessionService;
    use user::infrastructure::config::JwtSessionServiceImpl;
    use uuid::Uuid;

    async fn users(env: &TestEnv) -> UserApiClient<tonic::transport::Channel> {
        UserApiClient::connect(env.grpc_addr.clone())
            .await
            .expect("conexión")
    }
    async fn clinics(env: &TestEnv) -> ClinicApiClient<tonic::transport::Channel> {
        ClinicApiClient::connect(env.grpc_addr.clone())
            .await
            .expect("conexión")
    }
    async fn patients(env: &TestEnv) -> PatientApiClient<tonic::transport::Channel> {
        PatientApiClient::connect(env.grpc_addr.clone())
            .await
            .expect("conexión")
    }

    /// Ejecuta SQL de preparación directamente en la base de pruebas.
    async fn sql(env: &TestEnv, statement: String) {
        let conn = env.pg_connection_string.clone();
        tokio::task::spawn_blocking(move || {
            let mut client =
                ::postgres::Client::connect(&conn, ::postgres::NoTls).expect("conexión");
            client
                .batch_execute(&statement)
                .expect("SQL de preparación");
        })
        .await
        .expect("tarea bloqueante");
    }

    /// Detalle `ActionRequirements` de un rechazo por datos faltantes.
    fn requirements(status: &tonic::Status) -> ActionRequirements {
        assert_eq!(status.code(), Code::FailedPrecondition, "{status:?}");
        ActionRequirements::decode(status.details()).expect("el rechazo debió traer el detalle")
    }

    fn missing(status: &tonic::Status) -> Vec<ProfileField> {
        requirements(status)
            .missing
            .into_iter()
            .map(|field| ProfileField::try_from(field).expect("dato conocido"))
            .collect()
    }

    async fn complete(env: &TestEnv, account: &Account, request: CompleteProfileRequest) {
        users(env)
            .await
            .complete_profile(authed(request, &account.access_token))
            .await
            .expect("guardar perfil");
    }

    fn full_person_profile(dni: &str) -> CompleteProfileRequest {
        CompleteProfileRequest {
            given_name: "Nombre".into(),
            family_name: Some("Apellido".into()),
            phone: "999888777".into(),
            identifier: Some(Identifier {
                identifier_type: Some(identifier::IdentifierType::Dni(dni.into())),
            }),
            ..Default::default()
        }
    }

    /// DNI único de 8 dígitos para no chocar entre pruebas.
    fn unique_dni() -> String {
        format!("{:08}", Uuid::now_v7().as_u128() % 100_000_000)
    }

    // ---- 1. Sesión -----------------------------------------------------------

    /// Escenario «La intención se conserva como rol preferido» y emisión de la sesión.
    #[rstest]
    #[tokio::test]
    async fn sign_in_emite_la_sesion_y_conserva_el_rol(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let doctor = sign_up(env, "Medica", SignUpIntent::IntentPractitioner).await;
        let response = users(env)
            .await
            .sign_in(tonic::Request::new(SignInRequest {
                id_token: crate::support::id_token(&doctor.email),
                provider_id: "google.com".into(),
            }))
            .await
            .expect("sign_in")
            .into_inner();

        let session = response
            .session
            .clone()
            .expect("SignIn debió abrir una sesión");
        let claims = test_session_service()
            .verify_access(&session.access_token)
            .expect("token válido");
        assert_eq!(claims.user_id, doctor.user_id);
        assert_eq!(response.preferred_role(), Role::Practitioner);

        let requirements = users(env)
            .await
            .get_profile_requirements(authed(
                GetProfileRequirementsRequest {},
                &session.access_token,
            ))
            .await
            .expect("requisitos")
            .into_inner();
        assert!(
            requirements
                .fields
                .iter()
                .any(|f| f.field() == ProfileField::ProfessionalLicense),
            "El perfil del médico debió mostrar la colegiatura"
        );

        let denied = patients(env)
            .await
            .search_patient(authed(
                SearchPatientRequest::default(),
                &session.access_token,
            ))
            .await
            .expect_err("sin membresía no ve pacientes");
        assert_eq!(denied.code(), Code::PermissionDenied);
    }

    /// Escenarios «Petición sin credencial» y «Credencial vencida».
    #[rstest]
    #[tokio::test]
    async fn sin_credencial_valida_no_autenticado(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let without = patients(env)
            .await
            .search_patient(tonic::Request::new(SearchPatientRequest::default()))
            .await
            .expect_err("sin token");
        assert_eq!(without.code(), Code::Unauthenticated);

        let account = sign_up(env, "Vencida", SignUpIntent::IntentPatient).await;
        let expired = JwtSessionServiceImpl::new(TEST_SESSION_SECRET.to_vec(), -60, 60)
            .issue(account.user_id, Uuid::nil())
            .expect("firma")
            .access_token;
        let forged = JwtSessionServiceImpl::new(b"otra-clave".to_vec(), 300, 300)
            .issue(account.user_id, Uuid::nil())
            .expect("firma")
            .access_token;
        for token in [expired, forged, account.refresh_token.clone()] {
            let status = users(env)
                .await
                .get_profile_requirements(authed(GetProfileRequirementsRequest {}, &token))
                .await
                .expect_err("token no válido");
            assert_eq!(status.code(), Code::Unauthenticated);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn renueva_la_sesion_con_el_token_de_renovacion(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let env = test_env;
        let account = sign_up(env, "Renueva", SignUpIntent::IntentPatient).await;
        let renewed = users(env)
            .await
            .refresh_session(tonic::Request::new(RefreshSessionRequest {
                refresh_token: account.refresh_token.clone(),
            }))
            .await
            .expect("renovación")
            .into_inner();
        users(env)
            .await
            .get_profile_requirements(authed(
                GetProfileRequirementsRequest {},
                &renewed.access_token,
            ))
            .await
            .expect("el token renovado sirve");

        let status = users(env)
            .await
            .refresh_session(tonic::Request::new(RefreshSessionRequest {
                refresh_token: account.access_token.clone(),
            }))
            .await
            .expect_err("un token de acceso no renueva");
        assert_eq!(status.code(), Code::Unauthenticated);
    }

    // ---- 2. Perfil -----------------------------------------------------------

    /// La cuenta sale de la sesión; un DNI verificado no se cambia (escenario «DNI verificado»).
    #[rstest]
    #[tokio::test]
    async fn completar_perfil_y_dni_verificado(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let account = sign_up(env, "Perfil", SignUpIntent::IntentPatient).await;
        let dni = unique_dni();
        let saved = users(env)
            .await
            .complete_profile(authed(full_person_profile(&dni), &account.access_token))
            .await
            .expect("guardar")
            .into_inner();
        assert_eq!(saved.user_id, account.user_id.to_string());
        assert!(
            saved.missing_fields.is_empty(),
            "{:?}",
            saved.missing_fields
        );

        sql(
            env,
            format!(
                "update identity.user_account set document_verified = true where id = '{}'",
                account.user_id
            ),
        )
        .await;
        let status = users(env)
            .await
            .complete_profile(authed(
                full_person_profile(&unique_dni()),
                &account.access_token,
            ))
            .await
            .expect_err("DNI verificado");
        assert_eq!(status.code(), Code::FailedPrecondition);
        assert!(status.message().contains("acércate a recepción"));
    }

    // ---- 3. Clínica, membresías y autorización ----------------------------------

    /// Flujo completo: activar la clínica, invitar y activar al médico, buscar pacientes
    /// y suspenderlo. Cubre «Error distinguible», «Activación sin colegiatura»,
    /// «Activación completa», «Activar clínica sin RUC» y «Médico suspendido».
    #[rstest]
    #[tokio::test]
    async fn flujo_de_clinica_y_medico(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let owner = sign_up(env, "Duena", SignUpIntent::IntentClinicOwner).await;
        let subdomain = format!("flujo-{}", owner.user_id.simple());

        let created = clinics(env)
            .await
            .create_clinic(authed(
                CreateClinicRequest {
                    name: "Clínica Flujo".into(),
                    subdomain: subdomain.clone(),
                    given_name: "Dueña".into(),
                    ..Default::default()
                },
                &owner.access_token,
            ))
            .await
            .expect("crear clínica")
            .into_inner();
        let organization_id = created.organization_id.clone();
        // El médico se registra por el enlace de la clínica: su cuenta queda en la
        // organización del dueño, que es donde la invitación lo busca.
        let doctor = crate::support::sign_up_in(
            env,
            "Doctor",
            SignUpIntent::IntentPractitioner,
            Some(&subdomain),
        )
        .await;

        // Activar sin RUC ni datos del dueño: precondición con la lista exacta.
        let status = clinics(env)
            .await
            .activate_clinic(authed(
                ClinicIdRequest {
                    organization_id: organization_id.clone(),
                },
                &owner.access_token,
            ))
            .await
            .expect_err("faltan datos");
        assert_eq!(
            missing(&status),
            vec![
                ProfileField::FamilyName,
                ProfileField::Phone,
                ProfileField::Document,
                ProfileField::ClinicTaxId,
                ProfileField::ClinicLegalName,
            ]
        );
        assert_eq!(
            requirements(&status).organization_id.as_deref(),
            Some(organization_id.as_str())
        );

        // Un médico sin membresía no puede activar la clínica ni ver datos faltantes.
        let forbidden = clinics(env)
            .await
            .activate_clinic(authed(
                ClinicIdRequest {
                    organization_id: organization_id.clone(),
                },
                &doctor.access_token,
            ))
            .await
            .expect_err("sin permiso");
        assert_eq!(forbidden.code(), Code::PermissionDenied);
        assert!(forbidden.details().is_empty(), "Sin rol no se listan datos");

        // Invitar en clínica pendiente está permitido («Dueño configura antes de activar»).
        let invited = clinics(env)
            .await
            .invite_member(authed(
                InviteMemberRequest {
                    organization_id: organization_id.clone(),
                    email: doctor.email.clone(),
                    role: MembershipRole::Practitioner as i32,
                },
                &owner.access_token,
            ))
            .await
            .expect("invitar")
            .into_inner();
        assert_eq!(invited.status(), MembershipStatus::Invited);

        // El dueño completa sus datos y los de la clínica, y la activa.
        complete(env, &owner, full_person_profile(&unique_dni())).await;
        let invalid_ruc = clinics(env)
            .await
            .update_clinic_setup(authed(
                UpdateClinicSetupRequest {
                    organization_id: organization_id.clone(),
                    tax_id: Some("20100070971".into()),
                    ..Default::default()
                },
                &owner.access_token,
            ))
            .await
            .expect_err("RUC con dígito verificador inválido");
        assert_eq!(invalid_ruc.code(), Code::InvalidArgument);
        clinics(env)
            .await
            .update_clinic_setup(authed(
                UpdateClinicSetupRequest {
                    organization_id: organization_id.clone(),
                    tax_id: Some("20100070970".into()),
                    legal_name: Some("Clínica Flujo S.A.C.".into()),
                    ..Default::default()
                },
                &owner.access_token,
            ))
            .await
            .expect("datos legales");
        let active = clinics(env)
            .await
            .activate_clinic(authed(
                ClinicIdRequest {
                    organization_id: organization_id.clone(),
                },
                &owner.access_token,
            ))
            .await
            .expect("activar clínica")
            .into_inner();
        assert_eq!(active.status(), ClinicStatus::Active);

        // El médico se une y activa sin colegiatura: solo falta la colegiatura.
        clinics(env)
            .await
            .join_clinic(authed(
                MembershipIdRequest {
                    membership_id: invited.id.clone(),
                },
                &doctor.access_token,
            ))
            .await
            .expect("unirse");
        complete(env, &doctor, full_person_profile(&unique_dni())).await;
        let status = clinics(env)
            .await
            .activate_practitioner(authed(
                MembershipIdRequest {
                    membership_id: invited.id.clone(),
                },
                &doctor.access_token,
            ))
            .await
            .expect_err("sin colegiatura");
        assert_eq!(missing(&status), vec![ProfileField::ProfessionalLicense]);
        assert_eq!(
            requirements(&status).action(),
            ProtectedAction::ActionActivatePractitioner
        );

        complete(
            env,
            &doctor,
            CompleteProfileRequest {
                professional_body: Some("CMP".into()),
                license_number: Some("012345".into()),
                ..Default::default()
            },
        )
        .await;
        let pending = clinics(env)
            .await
            .activate_practitioner(authed(
                MembershipIdRequest {
                    membership_id: invited.id.clone(),
                },
                &doctor.access_token,
            ))
            .await
            .expect("activar perfil")
            .into_inner();
        assert_eq!(pending.status(), MembershipStatus::Pending);

        let confirmed = clinics(env)
            .await
            .confirm_license(authed(
                MembershipIdRequest {
                    membership_id: invited.id.clone(),
                },
                &owner.access_token,
            ))
            .await
            .expect("confirmar colegiatura")
            .into_inner();
        assert_eq!(confirmed.status(), MembershipStatus::Active);

        // Ya activo, el médico busca pacientes de su clínica; el dueño (sin rol de médico) no.
        patients(env)
            .await
            .search_patient(authed(
                SearchPatientRequest::default(),
                &doctor.access_token,
            ))
            .await
            .expect("el médico activo busca");
        let owner_search = patients(env)
            .await
            .search_patient(authed(SearchPatientRequest::default(), &owner.access_token))
            .await
            .expect_err("el dueño no es médico");
        assert_eq!(owner_search.code(), Code::PermissionDenied);

        // La colegiatura confirmada ya no es editable por el médico.
        let read_only = users(env)
            .await
            .complete_profile(authed(
                CompleteProfileRequest {
                    license_number: Some("999999".into()),
                    ..Default::default()
                },
                &doctor.access_token,
            ))
            .await
            .expect_err("colegiatura confirmada");
        assert_eq!(read_only.code(), Code::FailedPrecondition);

        // Suspendido, su siguiente búsqueda se rechaza.
        clinics(env)
            .await
            .suspend_membership(authed(
                MembershipIdRequest {
                    membership_id: invited.id.clone(),
                },
                &owner.access_token,
            ))
            .await
            .expect("suspender");
        let suspended = patients(env)
            .await
            .search_patient(authed(
                SearchPatientRequest::default(),
                &doctor.access_token,
            ))
            .await
            .expect_err("suspendido");
        assert_eq!(suspended.code(), Code::PermissionDenied);
    }

    /// Organization raíz del dueño (`my-health-record-vrw`): cada dueño tiene la suya,
    /// su cuenta vive en ella, sus clínicas cuelgan de ella (`partOf`) y queda vinculado
    /// por `Person.link`. Sin red por defecto compartida entre dueños.
    #[rstest]
    #[tokio::test]
    #[allow(deprecated)] // envía `network_id` a propósito: debe ignorarse
    async fn cada_dueno_tiene_su_organizacion_raiz(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let owner_a = sign_up(env, "RaizA", SignUpIntent::IntentClinicOwner).await;
        let owner_b = sign_up(env, "RaizB", SignUpIntent::IntentClinicOwner).await;
        let create = |owner: &crate::support::Account, suffix: &str| CreateClinicRequest {
            name: format!("Clínica {suffix}"),
            subdomain: format!("raiz-{suffix}-{}", owner.user_id.simple()),
            given_name: "Dueño".into(),
            network_id: Some(Uuid::now_v7().to_string()), // se ignora
            ..Default::default()
        };
        let a1 = clinics(env)
            .await
            .create_clinic(authed(create(&owner_a, "a1"), &owner_a.access_token))
            .await
            .expect("clínica A1")
            .into_inner();
        let b1 = clinics(env)
            .await
            .create_clinic(authed(create(&owner_b, "b1"), &owner_b.access_token))
            .await
            .expect("clínica B1")
            .into_inner();
        assert_ne!(
            a1.network_id, b1.network_id,
            "Dueños distintos, raíces distintas"
        );

        let conn = env.pg_connection_string.clone();
        let (owner_a_id, a1_id) = (
            owner_a.user_id,
            Uuid::parse_str(&a1.organization_id).unwrap(),
        );
        let (account_network, root_owner, links): (Uuid, Uuid, i64) =
            tokio::task::spawn_blocking(move || {
                let mut db = ::postgres::Client::connect(&conn, ::postgres::NoTls).expect("conexión");
                let account_network: Uuid = db
                    .query_one(
                        "select network_id from identity.user_account where id = $1",
                        &[&owner_a_id],
                    )
                    .expect("cuenta")
                    .get(0);
                let root_owner: Uuid = db
                    .query_one(
                        "select n.owner_user_id from administration.organization o \
                         join administration.clinical_network n on n.id = o.network_id \
                         where o.id = $1",
                        &[&a1_id],
                    )
                    .expect("raíz de la clínica")
                    .get(0);
                let links: i64 = db
                    .query_one(
                        "select count(*) from identity.person_link \
                         where user_id = $1 and target_type = 'ORGANIZATION' \
                         and target_id in ($2, (select network_id from administration.organization where id = $2))",
                        &[&owner_a_id, &a1_id],
                    )
                    .expect("vínculos")
                    .get(0);
                (account_network, root_owner, links)
            })
            .await
            .expect("tarea bloqueante");
        assert_eq!(
            account_network.to_string(),
            a1.network_id,
            "La cuenta vive en su raíz"
        );
        assert_eq!(root_owner, owner_a.user_id, "La raíz es del dueño");
        assert_eq!(links, 2, "Person.link a la raíz y a la clínica");

        // Sin subdominio, el dueño inicia sesión en su organización.
        let signed_in = users(env)
            .await
            .sign_in(tonic::Request::new(SignInRequest {
                id_token: crate::support::id_token(&owner_a.email),
                provider_id: "google.com".into(),
            }))
            .await
            .expect("iniciar sesión sin subdominio")
            .into_inner();
        assert_eq!(signed_in.user_id, owner_a.user_id.to_string());

        // El mismo correo no abre una segunda organización.
        let again = users(env)
            .await
            .sign_up(tonic::Request::new(SignUpRequest {
                id_token: crate::support::id_token(&owner_a.email),
                user_id: Uuid::now_v7().to_string(),
                provider_id: "google.com".into(),
                provider_name: "Google".into(),
                email: owner_a.email.clone(),
                given_name: "RaizA".into(),
                intent: SignUpIntent::IntentClinicOwner as i32,
                ..Default::default()
            }))
            .await
            .expect_err("segunda cuenta de dueño");
        assert_eq!(again.code(), Code::AlreadyExists);
    }

    /// Rol recepcionista: pide entrar a una clínica existente, el dueño aprueba y registra
    /// pacientes sin cuenta (`identity/patient-account-linking`, «Paciente sin cuenta»).
    #[rstest]
    #[tokio::test]
    async fn recepcionista_pide_entrar_y_registra_pacientes_sin_cuenta(
        #[future(awt)] test_env: &'static TestEnv,
    ) {
        let env = test_env;
        let owner = sign_up(env, "DuenaRecepcion", SignUpIntent::IntentClinicOwner).await;
        let subdomain = format!("recep-{}", owner.user_id.simple());
        let created = clinics(env)
            .await
            .create_clinic(authed(
                CreateClinicRequest {
                    name: "Clínica Recepción".into(),
                    subdomain: subdomain.clone(),
                    given_name: "Dueña".into(),
                    ..Default::default()
                },
                &owner.access_token,
            ))
            .await
            .expect("crear clínica")
            .into_inner();
        let organization_id = created.organization_id.clone();
        sql(
            env,
            format!(
                "update administration.organization set status = 'ACTIVE' where id = '{organization_id}'"
            ),
        )
        .await;
        // El personal se registra entrando por el enlace de la clínica: queda en su red.
        let receptionist = crate::support::sign_up_in(
            env,
            "Recepcion",
            SignUpIntent::IntentReceptionist,
            Some(&subdomain),
        )
        .await;

        // La cuenta de recepción no funda clínicas.
        let no_clinic = clinics(env)
            .await
            .create_clinic(authed(
                CreateClinicRequest {
                    name: "Otra".into(),
                    subdomain: format!("recep-otra-{}", receptionist.user_id.simple()),
                    given_name: "Recepción".into(),
                    ..Default::default()
                },
                &receptionist.access_token,
            ))
            .await
            .expect_err("recepción no crea clínicas");
        assert_eq!(no_clinic.code(), Code::PermissionDenied);

        // Sin sus datos no puede pedir entrar; con ellos queda como solicitud.
        let request = || RequestMembershipRequest {
            subdomain: subdomain.clone(),
            role: MembershipRole::Receptionist as i32,
        };
        let status = clinics(env)
            .await
            .request_membership(authed(request(), &receptionist.access_token))
            .await
            .expect_err("faltan datos");
        assert_eq!(
            missing(&status),
            vec![
                ProfileField::FamilyName,
                ProfileField::Phone,
                ProfileField::Document
            ]
        );
        complete(env, &receptionist, full_person_profile(&unique_dni())).await;
        let requested = clinics(env)
            .await
            .request_membership(authed(request(), &receptionist.access_token))
            .await
            .expect("pedir entrar")
            .into_inner();
        assert_eq!(requested.status(), MembershipStatus::Requested);

        let new_patient = |dni: &str| RegisterPatientRequest {
            given_name: "Rosa".into(),
            family_name: "Quispe".into(),
            identifier: Some(Identifier {
                identifier_type: Some(identifier::IdentifierType::Dni(dni.into())),
            }),
            phone: "987654321".into(),
            ..Default::default()
        };
        let patient_dni = unique_dni();
        let not_yet = patients(env)
            .await
            .register_patient(authed(
                new_patient(&patient_dni),
                &receptionist.access_token,
            ))
            .await
            .expect_err("sin aprobar no registra");
        assert_eq!(not_yet.code(), Code::PermissionDenied);

        // El dueño ve la solicitud con el correo y la aprueba.
        let members = clinics(env)
            .await
            .list_clinic_members(authed(
                ClinicIdRequest {
                    organization_id: organization_id.clone(),
                },
                &owner.access_token,
            ))
            .await
            .expect("lista de personal")
            .into_inner()
            .members;
        let request_row = members
            .iter()
            .find(|member| member.email == receptionist.email)
            .expect("la solicitud aparece en el personal");
        let membership_id = request_row
            .membership
            .as_ref()
            .expect("membresía")
            .id
            .clone();
        let approved = clinics(env)
            .await
            .approve_membership(authed(
                MembershipIdRequest { membership_id },
                &owner.access_token,
            ))
            .await
            .expect("aprobar")
            .into_inner();
        assert_eq!(approved.status(), MembershipStatus::Active);

        // Ya aprobada, registra al paciente sin cuenta, una sola vez por documento.
        let patient = patients(env)
            .await
            .register_patient(authed(
                new_patient(&patient_dni),
                &receptionist.access_token,
            ))
            .await
            .expect("registrar paciente")
            .into_inner();
        let repeated = patients(env)
            .await
            .register_patient(authed(
                new_patient(&patient_dni),
                &receptionist.access_token,
            ))
            .await
            .expect_err("documento repetido");
        assert_eq!(repeated.code(), Code::AlreadyExists);
        let conn = env.pg_connection_string.clone();
        let patient_id = Uuid::parse_str(&patient.id).expect("uuid del paciente");
        let without_account: i64 = tokio::task::spawn_blocking(move || {
            ::postgres::Client::connect(&conn, ::postgres::NoTls)
                .expect("conexión")
                .query_one(
                    "select count(*) from administration.patient where id = $1 and user_id is null",
                    &[&patient_id],
                )
                .expect("contar")
                .get(0)
        })
        .await
        .expect("tarea bloqueante");
        assert_eq!(without_account, 1, "el expediente queda sin cuenta");

        // Recepción lo encuentra en la búsqueda de pacientes.
        let found = patients(env)
            .await
            .search_patient(authed(
                SearchPatientRequest {
                    name: Some("Quispe".into()),
                },
                &receptionist.access_token,
            ))
            .await
            .expect("buscar")
            .into_inner()
            .patients;
        assert!(found.iter().any(|p| p.id == patient.id));
    }

    /// Escenarios «Paciente de otra clínica», «Misma persona en dos clínicas» y
    /// «Médico de clínica pendiente».
    #[rstest]
    #[tokio::test]
    async fn aislamiento_por_clinica(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let owner_a = sign_up(env, "DuenoA", SignUpIntent::IntentClinicOwner).await;
        let owner_b = sign_up(env, "DuenoB", SignUpIntent::IntentClinicOwner).await;
        let doctor = sign_up(env, "MedicoA", SignUpIntent::IntentPractitioner).await;
        let mut ids = Vec::new();
        for owner in [&owner_a, &owner_b] {
            let created = clinics(env)
                .await
                .create_clinic(authed(
                    CreateClinicRequest {
                        name: format!("Clínica {}", owner.user_id.simple()),
                        subdomain: format!("aisla-{}", owner.user_id.simple()),
                        given_name: "Dueño".into(),
                        ..Default::default()
                    },
                    &owner.access_token,
                ))
                .await
                .expect("crear clínica")
                .into_inner();
            ids.push(Uuid::parse_str(&created.organization_id).expect("uuid"));
        }
        let (clinic_a, clinic_b) = (ids[0], ids[1]);
        let person = Uuid::now_v7();
        let (patient_a, patient_b) = (Uuid::now_v7(), Uuid::now_v7());
        let person_json = |name: &str| {
            format!(
                r#"{{"id":"{person}","name":{{"given":["{name}"],"family":"Compartida","second_family":null,"text":"{name} Compartida"}},"telecom":[],"identifier":null,"birth_date":null,"links":[]}}"#
            )
        };
        // Médico activo solo en la clínica A, que está pendiente de activación.
        sql(
            env,
            format!(
                "insert into administration.membership (id, organization_id, user_id, role, status) \
                 values ('{}', '{clinic_a}', '{}', 'PRACTITIONER', 'ACTIVE');
                 insert into administration.patient (id, organization_id, user_id, active, person) values
                   ('{patient_a}', '{clinic_a}', '{person}', true, '{}'),
                   ('{patient_b}', '{clinic_b}', '{person}', true, '{}');",
                Uuid::now_v7(),
                doctor.user_id,
                person_json("Ana"),
                person_json("Ana"),
            ),
        )
        .await;

        let pending = patients(env)
            .await
            .search_patient(authed(
                SearchPatientRequest::default(),
                &doctor.access_token,
            ))
            .await
            .expect_err("clínica pendiente");
        assert_eq!(pending.code(), Code::FailedPrecondition);
        assert!(pending.details().is_empty());

        sql(
            env,
            format!(
                "update administration.organization set status = 'ACTIVE' where id = '{clinic_a}'"
            ),
        )
        .await;

        let found = patients(env)
            .await
            .search_patient(authed(
                SearchPatientRequest {
                    name: Some("compartida".into()),
                },
                &doctor.access_token,
            ))
            .await
            .expect("buscar")
            .into_inner()
            .patients;
        assert_eq!(
            found.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
            vec![patient_a.to_string()],
            "Solo el expediente de la clínica A"
        );

        patients(env)
            .await
            .get_patient_by_id(authed(
                PatientIdRequest {
                    id: Some(patient_a.to_string()),
                },
                &doctor.access_token,
            ))
            .await
            .expect("expediente propio de la clínica");
        let other = patients(env)
            .await
            .get_patient_by_id(authed(
                PatientIdRequest {
                    id: Some(patient_b.to_string()),
                },
                &doctor.access_token,
            ))
            .await
            .expect_err("expediente de otra clínica");
        assert_eq!(other.code(), Code::NotFound);

        // «Expediente ajeno»: quien no es médico de la clínica no sabe si existe.
        let stranger = patients(env)
            .await
            .get_patient_by_id(authed(
                PatientIdRequest {
                    id: Some(patient_a.to_string()),
                },
                &owner_b.access_token,
            ))
            .await
            .expect_err("expediente ajeno");
        assert_eq!(stranger.code(), Code::NotFound);
    }

    /// Tarea 3.1: el relleno de V4 crea la membresía de cada dueño y de cada médico.
    #[rstest]
    #[tokio::test]
    async fn v4_rellena_membresias_de_duenos_y_medicos(#[future(awt)] test_env: &'static TestEnv) {
        let env = test_env;
        let database = format!("backfill_{}", Uuid::now_v7().simple());
        sql(env, format!("create database {database}")).await;
        let url = env
            .pg_connection_string
            .replacen("/postgres?", &format!("/{database}?"), 1);

        clickcare::infrastructure::migration::run_until(&url, Some(3))
            .await
            .expect("migrar hasta V3");
        let (network, organization, owner, doctor) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        let backfill_env = TestEnv {
            grpc_addr: String::new(),
            pg_connection_string: url.clone(),
        };
        sql(
            &backfill_env,
            format!(
                "insert into administration.clinical_network (id, name) values ('{network}', 'Red');
                 insert into administration.organization (id, network_id, subdomain, name, owner_user_id)
                   values ('{organization}', '{network}', 'relleno', 'Clínica', '{owner}');
                 insert into administration.practitioner (id, organization_id, user_id, medical_license_number, person)
                   values ('{}', '{organization}', '{owner}', 'CMP-PENDIENTE', '{{}}'),
                          ('{}', '{organization}', '{doctor}', 'CMP-1', '{{}}');",
                Uuid::now_v7(),
                Uuid::now_v7()
            ),
        )
        .await;
        clickcare::infrastructure::migration::run(&url)
            .await
            .expect("migrar el resto");

        let rows = tokio::task::spawn_blocking(move || {
            let mut client =
                ::postgres::Client::connect(&url, ::postgres::NoTls).expect("conexión");
            client
                .query(
                    "select user_id, role, status from administration.membership order by role",
                    &[],
                )
                .expect("consulta")
                .into_iter()
                .map(|row| {
                    (
                        row.get::<_, Uuid>(0),
                        row.get::<_, String>(1),
                        row.get::<_, String>(2),
                    )
                })
                .collect::<Vec<_>>()
        })
        .await
        .expect("tarea");
        assert_eq!(
            rows,
            vec![
                (owner, "OWNER".to_string(), "ACTIVE".to_string()),
                (doctor, "PRACTITIONER".to_string(), "PENDING".to_string()),
            ]
        );
    }
}
