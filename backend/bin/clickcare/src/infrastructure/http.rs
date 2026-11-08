//! Borde HTTP del servidor: dirección de escucha, CORS, salud y tablero de colas.
//!
//! Variables de entorno:
//!
//! | Variable | Por defecto | Uso |
//! |---|---|---|
//! | `CLICKCARE_HOST` | `127.0.0.1` | Interfaz de escucha. En un contenedor, `0.0.0.0`. |
//! | `CLICKCARE_PORT` | `50051` | Puerto de escucha. |
//! | `CORS_ALLOWED_ORIGINS` | `http://localhost:5173` | Orígenes del navegador permitidos, separados por comas. Admite un comodín de subdominio: `https://*.clickcare.pe`. |
//! | `QUEUE_BOARD_USER` / `QUEUE_BOARD_PASSWORD` | — | Credenciales del tablero de colas (apalis-board). Sin ambas, el tablero no se publica. |

use apalis_board::axum::ui::ServeUI;
use app_core::domain::error::ClickCareError;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 50051;
const DEFAULT_ORIGIN: &str = "http://localhost:5173";
/// Prefijo de la API del tablero de colas.
pub const QUEUE_BOARD_API: &str = "/api/v1";

/// Configuración del borde HTTP, leída del entorno.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub addr: SocketAddr,
    pub cors: CorsPolicy,
    pub queue_board: Option<BoardCredentials>,
}

impl HttpConfig {
    /// Lee la configuración del entorno; falla si la dirección o un origen son inválidos.
    pub fn from_env() -> Result<Self, ClickCareError> {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        Self::from_values(
            var("CLICKCARE_HOST").as_deref(),
            var("CLICKCARE_PORT").as_deref(),
            var("CORS_ALLOWED_ORIGINS").as_deref(),
            var("QUEUE_BOARD_USER").as_deref(),
            var("QUEUE_BOARD_PASSWORD").as_deref(),
        )
    }

    fn from_values(
        host: Option<&str>,
        port: Option<&str>,
        origins: Option<&str>,
        board_user: Option<&str>,
        board_password: Option<&str>,
    ) -> Result<Self, ClickCareError> {
        let host: IpAddr = host.unwrap_or(DEFAULT_HOST).trim().parse().map_err(|e| {
            ClickCareError::generic(format!("CLICKCARE_HOST no es una IP válida: {e}"))
        })?;
        let port: u16 = match port {
            Some(p) => p.trim().parse().map_err(|e| {
                ClickCareError::generic(format!("CLICKCARE_PORT no es un puerto válido: {e}"))
            })?,
            None => DEFAULT_PORT,
        };
        let cors = CorsPolicy::parse(origins.unwrap_or(DEFAULT_ORIGIN))?;
        let queue_board = match (board_user, board_password) {
            (Some(user), Some(password)) => Some(BoardCredentials::new(user, password)),
            (None, None) => None,
            _ => {
                return Err(ClickCareError::generic(
                    "QUEUE_BOARD_USER y QUEUE_BOARD_PASSWORD van juntas: falta una de las dos",
                ));
            }
        };
        Ok(Self {
            addr: SocketAddr::new(host, port),
            cors,
            queue_board,
        })
    }
}

/// Origen permitido: exacto o con comodín de subdominio (`https://*.dominio`).
#[derive(Debug, Clone, PartialEq, Eq)]
enum AllowedOrigin {
    Exact(String),
    /// Esquema (`https://`) y sufijo (`.dominio[:puerto]`); exige al menos una etiqueta delante.
    Subdomains {
        scheme: String,
        suffix: String,
    },
}

/// Lista de orígenes del navegador que pueden llamar a la API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsPolicy {
    origins: Vec<AllowedOrigin>,
}

impl CorsPolicy {
    /// Interpreta una lista separada por comas. `*` no se acepta: la API usa credenciales.
    pub fn parse(list: &str) -> Result<Self, ClickCareError> {
        let mut origins = Vec::new();
        for raw in list.split(',').map(str::trim).filter(|o| !o.is_empty()) {
            let origin = raw.trim_end_matches('/').to_ascii_lowercase();
            let Some((scheme, host)) = origin.split_once("://") else {
                return Err(ClickCareError::generic(format!(
                    "Origen CORS inválido «{raw}»: falta el esquema (http:// o https://)"
                )));
            };
            if scheme != "http" && scheme != "https" {
                return Err(ClickCareError::generic(format!(
                    "Origen CORS inválido «{raw}»: el esquema debe ser http o https"
                )));
            }
            if host.is_empty() || host.contains('/') {
                return Err(ClickCareError::generic(format!(
                    "Origen CORS inválido «{raw}»: debe ser esquema://host[:puerto], sin ruta"
                )));
            }
            if let Some(rest) = host.strip_prefix("*.") {
                if rest.is_empty() || rest.contains('*') {
                    return Err(ClickCareError::generic(format!(
                        "Origen CORS inválido «{raw}»"
                    )));
                }
                origins.push(AllowedOrigin::Subdomains {
                    scheme: format!("{scheme}://"),
                    suffix: format!(".{rest}"),
                });
            } else if host.contains('*') {
                return Err(ClickCareError::generic(format!(
                    "Origen CORS inválido «{raw}»: el comodín solo vale como «*.dominio»"
                )));
            } else {
                origins.push(AllowedOrigin::Exact(origin));
            }
        }
        if origins.is_empty() {
            return Err(ClickCareError::generic(
                "CORS_ALLOWED_ORIGINS no tiene ningún origen",
            ));
        }
        Ok(Self { origins })
    }

    /// `true` si el navegador puede llamar desde `origin`.
    pub fn allows(&self, origin: &str) -> bool {
        let origin = origin.to_ascii_lowercase();
        self.origins.iter().any(|allowed| match allowed {
            AllowedOrigin::Exact(o) => *o == origin,
            AllowedOrigin::Subdomains { scheme, suffix } => origin
                .strip_prefix(scheme.as_str())
                .and_then(|host| host.strip_suffix(suffix.as_str()))
                .is_some_and(|label| !label.is_empty() && label.split('.').all(is_dns_label)),
        })
    }
}

fn is_dns_label(part: &str) -> bool {
    !part.is_empty()
        && part.len() <= 63
        && !part.starts_with('-')
        && !part.ends_with('-')
        && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Credenciales del tablero de colas (HTTP Basic).
#[derive(Clone)]
pub struct BoardCredentials {
    /// Cabecera `Authorization` esperada, ya codificada.
    expected: String,
}

impl std::fmt::Debug for BoardCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BoardCredentials(***)")
    }
}

impl BoardCredentials {
    pub fn new(user: &str, password: &str) -> Self {
        Self {
            expected: format!("Basic {}", STANDARD.encode(format!("{user}:{password}"))),
        }
    }

    fn accepts(&self, headers: &HeaderMap) -> bool {
        headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|given| constant_time_eq(given.as_bytes(), self.expected.as_bytes()))
    }
}

/// Compara sin cortar en el primer byte distinto, para no filtrar la contraseña por tiempos.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Arma la app HTTP: gRPC/gRPC-Web, `/health`, el tablero (si tiene credenciales) y el CORS.
///
/// `queue_board_api` es el router de apalis-board **sin** prefijo; se monta en [`QUEUE_BOARD_API`].
pub fn build_app(
    grpc: axum::Router,
    queue_board_api: axum::Router,
    config: &HttpConfig,
) -> axum::Router {
    let mut app = grpc.route("/health", axum::routing::get(health));

    match &config.queue_board {
        Some(credentials) => {
            let board = axum::Router::new()
                .nest(QUEUE_BOARD_API, queue_board_api)
                .fallback_service(ServeUI::new())
                .layer(axum::middleware::from_fn_with_state(
                    Arc::new(credentials.clone()),
                    require_board_credentials,
                ));
            // Como fallback y no con `merge`: así el router de tonic conserva sus rutas y
            // cualquier otra ruta pasa por las credenciales antes de llegar al tablero.
            app = app.fallback_service(board);
        }
        None => {
            tracing::info!(
                "Tablero de colas deshabilitado: define QUEUE_BOARD_USER y QUEUE_BOARD_PASSWORD para publicarlo"
            );
        }
    }

    app.layer(axum::middleware::from_fn_with_state(
        Arc::new(config.cors.clone()),
        cors_middleware,
    ))
}

async fn health() -> &'static str {
    "ok"
}

async fn require_board_credentials(
    State(credentials): State<Arc<BoardCredentials>>,
    request: Request,
    next: Next,
) -> Response {
    if credentials.accepts(request.headers()) {
        return next.run(request).await;
    }
    (
        StatusCode::UNAUTHORIZED,
        [(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"clickcare-queues\", charset=\"UTF-8\""),
        )],
    )
        .into_response()
}

/// CORS para gRPC-Web: refleja el `Origin` solo si está en la lista.
///
/// Un origen ajeno recibe la respuesta sin cabeceras CORS, así que el navegador la bloquea;
/// los clientes que no son navegador (sin `Origin`) no se ven afectados.
async fn cors_middleware(
    State(policy): State<Arc<CorsPolicy>>,
    request: Request,
    next: Next,
) -> Response {
    let allowed_origin = request
        .headers()
        .get(header::ORIGIN)
        .filter(|o| o.to_str().is_ok_and(|o| policy.allows(o)))
        .cloned();

    let mut response = if request.method() == Method::OPTIONS {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(request).await
    };

    let headers = response.headers_mut();
    headers.append(header::VARY, HeaderValue::from_static("origin"));
    if let Some(origin) = allowed_origin {
        insert_grpc_web_cors_headers(headers, origin);
    }
    response
}

fn insert_grpc_web_cors_headers(headers: &mut HeaderMap, origin: HeaderValue) {
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, GET, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static(
            "content-type, x-grpc-web, x-user-agent, x-subdomain, authorization",
        ),
    );
    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static(
            "grpc-status, grpc-message, grpc-status-details-bin, grpc-encoding, grpc-accept-encoding",
        ),
    );
    headers.insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("600"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;

    fn config(origins: &str, board: Option<(&str, &str)>) -> HttpConfig {
        HttpConfig::from_values(
            None,
            None,
            Some(origins),
            board.map(|b| b.0),
            board.map(|b| b.1),
        )
        .expect("configuración válida")
    }

    fn app(config: &HttpConfig) -> axum::Router {
        let board_api = axum::Router::new().route("/queues", axum::routing::get(|| async { "[]" }));
        build_app(axum::Router::new(), board_api, config)
    }

    async fn send(app: axum::Router, request: axum::http::Request<Body>) -> Response {
        app.oneshot(request).await.expect("respuesta")
    }

    fn get(uri: &str) -> axum::http::request::Builder {
        axum::http::Request::builder().uri(uri)
    }

    #[test]
    fn valores_por_defecto() {
        let c = HttpConfig::from_values(None, None, None, None, None).expect("config");
        assert_eq!(c.addr, "127.0.0.1:50051".parse().expect("addr"));
        assert!(c.cors.allows("http://localhost:5173"));
        assert!(!c.cors.allows("http://localhost:5174"));
        assert!(c.queue_board.is_none());
    }

    #[test]
    fn host_y_puerto_configurables() {
        let c = HttpConfig::from_values(Some("0.0.0.0"), Some("8080"), None, None, None)
            .expect("config");
        assert_eq!(c.addr, "0.0.0.0:8080".parse().expect("addr"));
        assert!(HttpConfig::from_values(Some("no-ip"), None, None, None, None).is_err());
        assert!(HttpConfig::from_values(None, Some("x"), None, None, None).is_err());
    }

    #[test]
    fn credenciales_del_tablero_van_juntas() {
        assert!(HttpConfig::from_values(None, None, None, Some("u"), None).is_err());
        assert!(HttpConfig::from_values(None, None, None, None, Some("p")).is_err());
    }

    #[test]
    fn origenes_exactos_y_con_comodin() {
        let p = CorsPolicy::parse(
            "https://app.clickcare.pe/, https://*.clickcare.pe, http://localhost:5174",
        )
        .expect("política");
        assert!(p.allows("https://app.clickcare.pe"));
        assert!(p.allows("https://clinica-sol.clickcare.pe"));
        assert!(p.allows("HTTPS://Clinica.ClickCare.pe"));
        assert!(p.allows("http://localhost:5174"));
        // El comodín exige un subdominio, con el mismo esquema y sin trucos de sufijo.
        assert!(!p.allows("https://clickcare.pe"));
        assert!(!p.allows("http://clinica.clickcare.pe"));
        assert!(!p.allows("https://evilclickcare.pe"));
        assert!(!p.allows("https://clickcare.pe.evil.com"));
        assert!(!p.allows("https://a b.clickcare.pe"));
        assert!(!p.allows("null"));
    }

    #[test]
    fn origenes_invalidos() {
        for bad in [
            "*",
            "localhost:5173",
            "ftp://x.pe",
            "https://x.pe/ruta",
            "https://a*.x.pe",
            " , ",
        ] {
            assert!(CorsPolicy::parse(bad).is_err(), "debió rechazar «{bad}»");
        }
    }

    #[test]
    fn comparacion_de_credenciales() {
        let c = BoardCredentials::new("ops", "secreto");
        let mut h = HeaderMap::new();
        assert!(!c.accepts(&h));
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic b3BzOnNlY3JldG8="),
        );
        assert!(c.accepts(&h));
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic b3BzOm90cmE="),
        );
        assert!(!c.accepts(&h));
        assert!(!format!("{c:?}").contains("b3Bz"));
    }

    #[tokio::test]
    async fn sin_credenciales_el_tablero_no_existe() {
        let app = app(&config("http://localhost:5173", None));
        let r = send(
            app.clone(),
            get("/api/v1/queues").body(Body::empty()).expect("req"),
        )
        .await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let r = send(app, get("/").body(Body::empty()).expect("req")).await;
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn el_tablero_exige_credenciales() {
        let app = app(&config("http://localhost:5173", Some(("ops", "secreto"))));

        for uri in ["/api/v1/queues", "/", "/queues/scheduling-reminders"] {
            let r = send(app.clone(), get(uri).body(Body::empty()).expect("req")).await;
            assert_eq!(r.status(), StatusCode::UNAUTHORIZED, "{uri}");
            assert!(r.headers().contains_key(header::WWW_AUTHENTICATE));
        }

        let r = send(
            app.clone(),
            get("/api/v1/queues")
                .header(header::AUTHORIZATION, "Basic b3BzOm90cmE=")
                .body(Body::empty())
                .expect("req"),
        )
        .await;
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

        let r = send(
            app,
            get("/api/v1/queues")
                .header(header::AUTHORIZATION, "Basic b3BzOnNlY3JldG8=")
                .body(Body::empty())
                .expect("req"),
        )
        .await;
        assert_eq!(r.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn se_combina_con_el_router_de_tonic() {
        let grpc: axum::Router = tonic::service::Routes::default()
            .add_service(
                tonic_reflection::server::Builder::configure()
                    .register_encoded_file_descriptor_set(
                        crate::infrastructure::grpc::FILE_DESCRIPTOR_SET,
                    )
                    .build_v1()
                    .expect("reflexión"),
            )
            .into_axum_router()
            .layer(tonic_web::GrpcWebLayer::new());
        let config = config("http://localhost:5173", Some(("ops", "secreto")));
        let board_api = axum::Router::new().route("/queues", axum::routing::get(|| async { "[]" }));
        let app = build_app(grpc, board_api, &config);
        let r = send(
            app.clone(),
            get("/health").body(Body::empty()).expect("req"),
        )
        .await;
        assert_eq!(r.status(), StatusCode::OK);
        let r = send(app, get("/api/v1/queues").body(Body::empty()).expect("req")).await;
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn health_sin_credenciales() {
        let app = app(&config("http://localhost:5173", Some(("ops", "secreto"))));
        let r = send(app, get("/health").body(Body::empty()).expect("req")).await;
        assert_eq!(r.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn cors_solo_para_origenes_permitidos() {
        let app = app(&config("http://localhost:5173", None));
        let preflight = |origin: &'static str| {
            axum::http::Request::builder()
                .method(Method::OPTIONS)
                .uri("/user.UserApi/SignIn")
                .header(header::ORIGIN, origin)
                .body(Body::empty())
                .expect("req")
        };

        let r = send(app.clone(), preflight("http://localhost:5173")).await;
        assert_eq!(r.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            r.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static("http://localhost:5173"))
        );
        assert!(
            r.headers()
                .contains_key(header::ACCESS_CONTROL_EXPOSE_HEADERS)
        );

        let r = send(app.clone(), preflight("https://evil.example")).await;
        assert!(
            !r.headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
        assert_eq!(
            r.headers().get(header::VARY),
            Some(&HeaderValue::from_static("origin"))
        );

        let r = send(
            app,
            get("/health")
                .header(header::ORIGIN, "https://evil.example")
                .body(Body::empty())
                .expect("req"),
        )
        .await;
        assert_eq!(r.status(), StatusCode::OK);
        assert!(
            !r.headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
    }
}
