//! Autenticación de cada llamada y armado del contexto de autorización.
//!
//! - [`Authenticator`] valida `authorization: Bearer <token>` con el servicio de sesión.
//!   Las APIs donde todo es protegido (`ClinicApi`, `PatientApi`) lo usan como
//!   interceptor de tonic; `UserApi`, que mezcla operaciones públicas y protegidas, lo
//!   llama dentro de cada operación protegida.
//! - [`AccessContextBuilder`] arma el [`RequestContext`] (datos presentes y membresías)
//!   con una caché de pocos segundos que se invalida al cambiar una membresía.
//! - [`denied_to_status`] traduce un rechazo de la regla a un estado gRPC. Los datos
//!   faltantes viajan como `ActionRequirements` en `grpc-status-details-bin`.

use crate::infrastructure::grpc;
use administration::application::membership_usecase::MembershipUseCases;
use app_core::domain::access::{
    AccessDenied, MembershipView, ProfileField, ProtectedAction, RequestContext,
};
use prost::Message;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tonic::metadata::MetadataMap;
use tonic::{Code, Request, Status};
use tracing::debug;
use user::application::integration::service::session_service::{
    SessionClaims, SessionError, SessionService,
};
use user::domain::user::User;
use user::domain::user::user_repository::UserRepository;
use uuid::Uuid;

/// Vida de las membresías en caché. Corta, para que una suspensión surta efecto pronto
/// incluso entre procesos; dentro del mismo proceso se invalida al instante.
const MEMBERSHIP_CACHE_TTL: Duration = Duration::from_secs(5);

/// Valida el token de sesión de una llamada.
#[derive(Clone)]
pub struct Authenticator {
    session_service: Arc<dyn SessionService>,
}

impl Authenticator {
    pub fn new(session_service: Arc<dyn SessionService>) -> Self {
        Self { session_service }
    }

    /// Identidad de la sesión. Sin credencial válida responde `UNAUTHENTICATED` sin
    /// revelar nada del recurso pedido.
    pub fn authenticate(&self, metadata: &MetadataMap) -> Result<SessionClaims, Status> {
        let token = metadata
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| Status::unauthenticated("Inicia sesión para continuar"))?;
        self.session_service
            .verify_access(token)
            .map_err(|error| match error {
                SessionError::Expired => Status::unauthenticated("La sesión venció"),
                SessionError::Invalid(reason) => {
                    debug!("Token de sesión rechazado: {reason}");
                    Status::unauthenticated("La sesión no es válida")
                }
            })
    }

    /// Interceptor de tonic que deja la identidad en las extensiones de la petición.
    pub fn interceptor(
        &self,
    ) -> impl FnMut(Request<()>) -> Result<Request<()>, Status> + Clone + Send + Sync + 'static
    {
        let authenticator = self.clone();
        move |mut request: Request<()>| {
            let claims = authenticator.authenticate(request.metadata())?;
            request.extensions_mut().insert(claims);
            Ok(request)
        }
    }
}

/// Identidad que dejó el interceptor en la petición.
pub fn claims_of<T>(request: &Request<T>) -> Result<SessionClaims, Status> {
    request
        .extensions()
        .get::<SessionClaims>()
        .copied()
        .ok_or_else(|| Status::unauthenticated("Inicia sesión para continuar"))
}

/// Arma el contexto de quien llama: su cuenta, sus datos y sus membresías.
pub struct AccessContextBuilder {
    user_repository: Arc<dyn UserRepository>,
    membership_use_cases: Arc<MembershipUseCases>,
    cache: Mutex<HashMap<Uuid, (Instant, Vec<MembershipView>)>>,
}

impl AccessContextBuilder {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        membership_use_cases: Arc<MembershipUseCases>,
    ) -> Self {
        Self {
            user_repository,
            membership_use_cases,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Cuenta de la sesión. Si ya no existe o está inactiva, la sesión deja de valer.
    pub async fn user(&self, claims: &SessionClaims) -> Result<User, Status> {
        self.user_repository
            .find_user_by_id(&claims.user_id)
            .await
            .map_err(|error| Status::internal(error.to_string()))?
            .filter(|user| user.active && user.network_id == claims.network_id)
            .ok_or_else(|| Status::unauthenticated("La cuenta de la sesión ya no está activa"))
    }

    /// Contexto completo para la regla de autorización.
    pub async fn build(&self, claims: &SessionClaims) -> Result<(User, RequestContext), Status> {
        let user = self.user(claims).await?;
        let memberships = self.memberships(&claims.user_id).await?;
        let ctx = RequestContext {
            user_id: user.id,
            network_id: user.network_id,
            preferred_role: user.preferred_role,
            present_fields: user.present_fields(),
            memberships,
        };
        Ok((user, ctx))
    }

    /// Olvida las membresías en caché de un usuario (tras invitarlo, activarlo o suspenderlo).
    pub fn invalidate(&self, user_id: &Uuid) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(user_id);
        }
    }

    async fn memberships(&self, user_id: &Uuid) -> Result<Vec<MembershipView>, Status> {
        if let Ok(cache) = self.cache.lock()
            && let Some((cached_at, memberships)) = cache.get(user_id)
            && cached_at.elapsed() < MEMBERSHIP_CACHE_TTL
        {
            return Ok(memberships.clone());
        }
        let memberships: Vec<MembershipView> = self
            .membership_use_cases
            .list_for_user(user_id)
            .await
            .map_err(|error| Status::internal(error.to_string()))?
            .into_iter()
            .map(|entry| entry.membership.view())
            .collect();
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(*user_id, (Instant::now(), memberships.clone()));
        }
        Ok(memberships)
    }
}

/// Traduce un rechazo de la regla de autorización a un estado gRPC.
///
/// - Sin permiso: `PERMISSION_DENIED`, sin listar datos.
/// - Clínica en estado no válido: `FAILED_PRECONDITION` sin detalle.
/// - Datos faltantes: `FAILED_PRECONDITION` con `ActionRequirements` en los detalles.
pub fn denied_to_status(denied: AccessDenied, organization_id: Option<Uuid>) -> Status {
    match denied {
        AccessDenied::Unauthenticated => Status::unauthenticated("Inicia sesión para continuar"),
        AccessDenied::Forbidden => {
            Status::permission_denied("No tienes permiso para realizar esta acción")
        }
        AccessDenied::ClinicNotActive => {
            Status::failed_precondition("La clínica no está activa para esta acción")
        }
        AccessDenied::MissingData { action, fields } => {
            let details = grpc::ActionRequirements {
                action: action_to_proto(action) as i32,
                missing: fields
                    .iter()
                    .map(|field| field_to_proto(*field) as i32)
                    .collect(),
                organization_id: organization_id.map(|id| id.to_string()),
            };
            Status::with_details(
                Code::FailedPrecondition,
                "Faltan datos para realizar esta acción",
                details.encode_to_vec().into(),
            )
        }
    }
}

pub fn field_to_proto(field: ProfileField) -> grpc::ProfileField {
    use grpc::ProfileField as P;
    match field {
        ProfileField::GivenName => P::GivenName,
        ProfileField::FamilyName => P::FamilyName,
        ProfileField::SecondFamilyName => P::SecondFamilyName,
        ProfileField::Email => P::Email,
        ProfileField::Phone => P::Phone,
        ProfileField::Document => P::Document,
        ProfileField::BirthDate => P::BirthDate,
        ProfileField::Sex => P::Sex,
        ProfileField::Address => P::Address,
        ProfileField::ProfessionalLicense => P::ProfessionalLicense,
        ProfileField::Specialty => P::Specialty,
        ProfileField::LicenseConfirmed => P::LicenseConfirmed,
        ProfileField::ClinicTaxId => P::ClinicTaxId,
        ProfileField::ClinicLegalName => P::ClinicLegalName,
        ProfileField::ClinicTradeName => P::ClinicTradeName,
        ProfileField::IpressCode => P::IpressCode,
    }
}

pub fn action_to_proto(action: ProtectedAction) -> grpc::ProtectedAction {
    use grpc::ProtectedAction as P;
    match action {
        ProtectedAction::ConfirmAppointment => P::ActionConfirmAppointment,
        ProtectedAction::JoinClinic => P::ActionJoinClinic,
        ProtectedAction::ActivatePractitioner => P::ActionActivatePractitioner,
        ProtectedAction::SearchPatients => P::ActionSearchPatients,
        ProtectedAction::ViewPatient => P::ActionViewPatient,
        ProtectedAction::OpenFirstEncounter => P::ActionOpenFirstEncounter,
        ProtectedAction::RecordEncounter => P::ActionRecordEncounter,
        ProtectedAction::IssuePrescription => P::ActionIssuePrescription,
        ProtectedAction::CreateClinic => P::ActionCreateClinic,
        ProtectedAction::ConfigureClinic => P::ActionConfigureClinic,
        ProtectedAction::InviteStaff => P::ActionInviteStaff,
        ProtectedAction::SuspendMember => P::ActionSuspendMember,
        ProtectedAction::ConfirmLicense => P::ActionConfirmLicense,
        ProtectedAction::ActivateClinic => P::ActionActivateClinic,
        ProtectedAction::RegisterPatient => P::ActionRegisterPatient,
        ProtectedAction::RequestMembership => P::ActionRequestMembership,
        ProtectedAction::ApproveMember => P::ActionApproveMember,
    }
}
