use administration::domain::repository::organization_repository::OrganizationRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Status};
use tracing::{debug, info};
use uuid::Uuid;

/// Subdominios reservados de la plataforma que no corresponden a clínicas de clientes.
pub const RESERVED_SUBDOMAINS: &[&str] = &["app", "api", "admin", "www", "static"];

/// Identificador por defecto utilizado como fallback en entornos locales/tests sin subdominio.
pub const DEFAULT_DEV_NETWORK_ID: Uuid = uuid::uuid!("0191eb44-4860-7000-8000-000000000001");

/// Resolvedor de subdominio a contexto de red e inquilino `(network_id, organization_id)`
/// con caché en memoria `subdomain -> (network_id, organization_id)`.
#[derive(Clone)]
pub struct SubdomainResolver {
    cache: Arc<RwLock<HashMap<String, (Uuid, Uuid)>>>,
    organization_repository: Option<Arc<dyn OrganizationRepository>>,
}

impl SubdomainResolver {
    pub fn new(organization_repository: Arc<dyn OrganizationRepository>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            organization_repository: Some(organization_repository),
        }
    }

    pub fn new_mock() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            organization_repository: None,
        }
    }

    /// Registra manualmente una resolución en la caché (para tests o pre-calentamiento).
    #[allow(dead_code)]
    pub async fn register(&self, subdomain: String, network_id: Uuid, organization_id: Uuid) {
        let mut cache = self.cache.write().await;
        cache.insert(
            subdomain.trim().to_lowercase(),
            (network_id, organization_id),
        );
    }

    /// Extrae el subdominio desde los metadatos gRPC:
    /// 1. Prioridad: metadata `x-subdomain` (usado en entornos de desarrollo y pruebas locales).
    /// 2. Alternativa: encabezado `:authority`, `host` o `origin` HTTP/2.
    pub fn extract_subdomain<T>(&self, request: &Request<T>) -> Option<String> {
        // 1. Metadata 'x-subdomain'
        if let Some(val) = request
            .metadata()
            .get("x-subdomain")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
        {
            return Some(val);
        }

        // 2. Encabezado host / :authority / origin
        request
            .metadata()
            .get(":authority")
            .or_else(|| request.metadata().get("host"))
            .or_else(|| request.metadata().get("origin"))
            .and_then(|v| v.to_str().ok())
            .and_then(parse_subdomain_from_host)
    }

    /// Resuelve `(network_id, organization_id)` para la solicitud gRPC entrante.
    pub async fn resolve<T>(&self, request: &Request<T>) -> Result<(Uuid, Uuid), Status> {
        if let Some(subdomain) = self.extract_subdomain(request) {
            self.resolve_subdomain(&subdomain).await
        } else {
            // Si la solicitud no especifica subdominio en Host ni en metadatos x-subdomain
            // (por ejemplo en tests locales o cliente directo por IP), asignamos la red por defecto.
            Ok((DEFAULT_DEV_NETWORK_ID, DEFAULT_DEV_NETWORK_ID))
        }
    }

    /// Resuelve `(network_id, organization_id)` por nombre de subdominio.
    pub async fn resolve_subdomain(&self, subdomain: &str) -> Result<(Uuid, Uuid), Status> {
        let normalized = subdomain.trim().to_lowercase();

        if RESERVED_SUBDOMAINS.contains(&normalized.as_str()) {
            return Err(Status::invalid_argument(format!(
                "El subdominio '{normalized}' está reservado por la plataforma"
            )));
        }

        // 1. Verificar caché en memoria
        {
            let cache = self.cache.read().await;
            if let Some(&(network_id, org_id)) = cache.get(&normalized) {
                debug!(
                    "Subdominio '{normalized}' resuelto desde caché: network_id={network_id}, org_id={org_id}"
                );
                return Ok((network_id, org_id));
            }
        }

        // 2. Si no está en caché, consultar repositorio
        let Some(repo) = &self.organization_repository else {
            return Err(Status::not_found(format!(
                "Subdominio '{normalized}' no encontrado en la plataforma",
            )));
        };

        let result = repo
            .find_org_and_network_by_subdomain(&normalized)
            .await
            .map_err(|e| {
                Status::internal(format!("Error resolviendo subdominio '{normalized}': {e}"))
            })?;

        match result {
            Some((org_id, network_id)) => {
                info!(
                    "Subdominio '{normalized}' resuelto desde base de datos: network_id={network_id}, org_id={org_id}"
                );
                let mut cache = self.cache.write().await;
                cache.insert(normalized, (network_id, org_id));
                Ok((network_id, org_id))
            }
            None => Err(Status::not_found(format!(
                "Subdominio '{normalized}' no encontrado en la plataforma",
            ))),
        }
    }
}

/// Extrae el subdominio a partir del hostname u origen HTTP/2.
pub fn parse_subdomain_from_host(raw_host: &str) -> Option<String> {
    let host = raw_host
        .trim()
        .strip_prefix("https://")
        .or_else(|| raw_host.trim().strip_prefix("http://"))
        .unwrap_or(raw_host.trim());

    // Descartar ruta si la hay
    let authority = host.split('/').next().unwrap_or(host);

    // Descartar puerto si lo hay
    let host_no_port = authority.split(':').next().unwrap_or(authority);

    // Si es una dirección IP directa (e.g. 127.0.0.1 o ::1), no tiene subdominio
    if host_no_port.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let parts: Vec<&str> = host_no_port.split('.').collect();

    // Casos:
    // - "san-borja.clickcare.com" -> parts: ["san-borja", "clickcare", "com"] -> "san-borja"
    // - "san-borja.localhost" -> parts: ["san-borja", "localhost"] -> "san-borja"
    // - "clickcare.com" -> parts: ["clickcare", "com"] -> None
    // - "localhost" -> parts: ["localhost"] -> None
    // - "127.0.0.1" -> None
    if parts.len() >= 3 || (parts.len() == 2 && parts[1] == "localhost") {
        let first = parts[0].to_lowercase();
        if !first.is_empty() {
            return Some(first);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subdomains_correctly() {
        assert_eq!(
            parse_subdomain_from_host("san-borja.clickcare.com"),
            Some("san-borja".to_string())
        );
        assert_eq!(
            parse_subdomain_from_host("https://san-borja.clickcare.com:50051"),
            Some("san-borja".to_string())
        );
        assert_eq!(
            parse_subdomain_from_host("clinica-miraflores.localhost:3000"),
            Some("clinica-miraflores".to_string())
        );
        assert_eq!(parse_subdomain_from_host("clickcare.com"), None);
        assert_eq!(
            parse_subdomain_from_host("app.clickcare.com"),
            Some("app".to_string())
        );
        assert_eq!(parse_subdomain_from_host("localhost:50051"), None);
        assert_eq!(parse_subdomain_from_host("127.0.0.1:50051"), None);
    }

    #[tokio::test]
    async fn cache_resolves_registered_subdomain() {
        let resolver = SubdomainResolver::new_mock();
        let net_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();

        resolver
            .register("san-borja".to_string(), net_id, org_id)
            .await;

        let (res_net, res_org) = resolver
            .resolve_subdomain("san-borja")
            .await
            .expect("Debe resolver desde caché");

        assert_eq!(res_net, net_id);
        assert_eq!(res_org, org_id);
    }

    #[tokio::test]
    async fn rejects_reserved_subdomain() {
        let resolver = SubdomainResolver::new_mock();
        let err = resolver
            .resolve_subdomain("app")
            .await
            .expect_err("Debe rechazar subdominio reservado");

        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn returns_not_found_for_unregistered_subdomain() {
        let resolver = SubdomainResolver::new_mock();
        let err = resolver
            .resolve_subdomain("inexistente")
            .await
            .expect_err("Debe retornar NotFound");

        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn fallback_to_default_dev_network_when_no_subdomain_given() {
        let resolver = SubdomainResolver::new_mock();
        let request = Request::new(());

        let (net_id, org_id) = resolver
            .resolve(&request)
            .await
            .expect("Debe usar red por defecto");

        assert_eq!(net_id, DEFAULT_DEV_NETWORK_ID);
        assert_eq!(org_id, DEFAULT_DEV_NETWORK_ID);
    }

    #[tokio::test]
    async fn resolves_via_x_subdomain_metadata() {
        let resolver = SubdomainResolver::new_mock();
        let net_id = Uuid::now_v7();
        let org_id = Uuid::now_v7();
        resolver
            .register("sede-sur".to_string(), net_id, org_id)
            .await;

        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("x-subdomain", "sede-sur".parse().unwrap());

        let (res_net, res_org) = resolver
            .resolve(&request)
            .await
            .expect("Debe resolver vía x-subdomain");

        assert_eq!(res_net, net_id);
        assert_eq!(res_org, org_id);
    }
}
