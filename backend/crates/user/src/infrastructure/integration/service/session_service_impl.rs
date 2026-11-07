use crate::application::integration::service::session_service::{
    SessionClaims, SessionError, SessionService, SessionTokens,
};
use app_core::domain::error::ClickCareError;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

/// Vida del token de acceso por defecto: 15 minutos.
const DEFAULT_ACCESS_TTL_SECS: i64 = 15 * 60;
/// Vida del token de renovación por defecto: 7 días.
const DEFAULT_REFRESH_TTL_SECS: i64 = 7 * 24 * 60 * 60;

const ACCESS: &str = "access";
const REFRESH: &str = "refresh";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// `user_id`.
    sub: String,
    /// `network_id`.
    net: String,
    /// `access` o `refresh`: un token de renovación no sirve como token de acceso.
    typ: String,
    iat: i64,
    exp: i64,
}

/// Tokens de sesión JWT firmados con HS256.
///
/// Configuración por entorno:
/// - `SESSION_SECRET`: clave de firma. Sin ella se genera una al azar en cada arranque
///   y todas las sesiones caducan al reiniciar el servidor.
/// - `SESSION_ACCESS_TTL_SECS` y `SESSION_REFRESH_TTL_SECS`: vida de cada token.
pub struct JwtSessionServiceImpl {
    secret: Vec<u8>,
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
}

impl JwtSessionServiceImpl {
    /// Construye el servicio leyendo la configuración del entorno.
    pub fn from_env() -> Self {
        let secret = match std::env::var("SESSION_SECRET") {
            Ok(secret) if !secret.trim().is_empty() => secret.into_bytes(),
            _ => {
                warn!(
                    "SESSION_SECRET no está definido: se usa una clave aleatoria y las sesiones \
                     caducan al reiniciar el servidor"
                );
                let mut random = Uuid::new_v4().as_bytes().to_vec();
                random.extend_from_slice(Uuid::new_v4().as_bytes());
                random
            }
        };
        let ttl = |name: &str, default: i64| {
            std::env::var(name)
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(default)
        };
        Self::new(
            secret,
            ttl("SESSION_ACCESS_TTL_SECS", DEFAULT_ACCESS_TTL_SECS),
            ttl("SESSION_REFRESH_TTL_SECS", DEFAULT_REFRESH_TTL_SECS),
        )
    }

    pub fn new(secret: Vec<u8>, access_ttl_secs: i64, refresh_ttl_secs: i64) -> Self {
        Self {
            secret,
            access_ttl_secs,
            refresh_ttl_secs,
        }
    }

    fn sign(
        &self,
        user_id: Uuid,
        network_id: Uuid,
        typ: &str,
        now: i64,
        ttl: i64,
    ) -> Result<(String, i64), ClickCareError> {
        let exp = now + ttl;
        let claims = Claims {
            sub: user_id.to_string(),
            net: network_id.to_string(),
            typ: typ.to_string(),
            iat: now,
            exp,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map(|token| (token, exp))
        .map_err(|error| ClickCareError::generic(format!("No se pudo firmar la sesión: {error}")))
    }

    fn verify(&self, token: &str, expected_typ: &str) -> Result<SessionClaims, SessionError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        validation.set_required_spec_claims(&["exp", "sub"]);
        let data = decode::<Claims>(
            token.trim(),
            &DecodingKey::from_secret(&self.secret),
            &validation,
        )
        .map_err(|error| match error.kind() {
            ErrorKind::ExpiredSignature => SessionError::Expired,
            _ => SessionError::Invalid(error.to_string()),
        })?;
        if data.claims.typ != expected_typ {
            return Err(SessionError::Invalid(format!(
                "Se esperaba un token de tipo {expected_typ}"
            )));
        }
        let parse = |value: &str| {
            Uuid::parse_str(value).map_err(|error| SessionError::Invalid(error.to_string()))
        };
        Ok(SessionClaims {
            user_id: parse(&data.claims.sub)?,
            network_id: parse(&data.claims.net)?,
        })
    }
}

impl SessionService for JwtSessionServiceImpl {
    fn issue(&self, user_id: Uuid, network_id: Uuid) -> Result<SessionTokens, ClickCareError> {
        let now = jiff::Timestamp::now().as_second();
        let (access_token, access_expires_at) =
            self.sign(user_id, network_id, ACCESS, now, self.access_ttl_secs)?;
        let (refresh_token, _) =
            self.sign(user_id, network_id, REFRESH, now, self.refresh_ttl_secs)?;
        Ok(SessionTokens {
            access_token,
            access_expires_at,
            refresh_token,
        })
    }

    fn verify_access(&self, token: &str) -> Result<SessionClaims, SessionError> {
        self.verify(token, ACCESS)
    }

    fn verify_refresh(&self, token: &str) -> Result<SessionClaims, SessionError> {
        self.verify(token, REFRESH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(access_ttl: i64) -> JwtSessionServiceImpl {
        JwtSessionServiceImpl::new(b"secreto-de-prueba".to_vec(), access_ttl, 3600)
    }

    #[test]
    fn el_token_lleva_la_cuenta_y_la_red() {
        let (user_id, network_id) = (Uuid::now_v7(), Uuid::now_v7());
        let tokens = service(60).issue(user_id, network_id).expect("firma");
        let claims = service(60)
            .verify_access(&tokens.access_token)
            .expect("válido");
        assert_eq!(
            claims,
            SessionClaims {
                user_id,
                network_id
            }
        );
        assert!(tokens.access_expires_at > jiff::Timestamp::now().as_second());
    }

    #[test]
    fn un_token_de_renovacion_no_sirve_como_acceso() {
        let tokens = service(60)
            .issue(Uuid::now_v7(), Uuid::now_v7())
            .expect("firma");
        assert!(matches!(
            service(60).verify_access(&tokens.refresh_token),
            Err(SessionError::Invalid(_))
        ));
        assert!(service(60).verify_refresh(&tokens.refresh_token).is_ok());
    }

    #[test]
    fn un_token_vencido_se_rechaza() {
        let svc = service(60);
        let (token, _) = svc
            .sign(
                Uuid::now_v7(),
                Uuid::now_v7(),
                ACCESS,
                jiff::Timestamp::now().as_second() - 120,
                60,
            )
            .expect("firma");
        assert_eq!(svc.verify_access(&token), Err(SessionError::Expired));
    }

    #[test]
    fn otra_clave_no_valida_el_token() {
        let tokens = service(60)
            .issue(Uuid::now_v7(), Uuid::now_v7())
            .expect("firma");
        let other = JwtSessionServiceImpl::new(b"otra-clave".to_vec(), 60, 60);
        assert!(matches!(
            other.verify_access(&tokens.access_token),
            Err(SessionError::Invalid(_))
        ));
    }
}
