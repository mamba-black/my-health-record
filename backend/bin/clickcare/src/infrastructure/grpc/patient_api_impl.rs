use crate::infrastructure::grpc::access::{AccessContextBuilder, claims_of, denied_to_status};
use crate::infrastructure::grpc::patient_api_server::*;
use crate::infrastructure::grpc::subdomain_resolver::SubdomainResolver;
use crate::infrastructure::grpc::user_api_impl::identifier_to_proto;
use crate::infrastructure::grpc::*;
use administration::application::membership_usecase::{
    MembershipError, MembershipUseCases, clinic_target,
};
use administration::application::patient_usecase::{NewPatient, PatientUseCases};
use administration::domain::patient::Patient as DomainPatient;
use administration::domain::patient::patient_repository::PatientRepository;
use app_core::domain::access::{
    AccessDenied, MembershipStatus, ProtectedAction, RequestContext, authorize,
};
use app_core::domain::fhir::{ContactPointSystem, Identifier};
use std::sync::Arc;
use tonic::*;
use uuid::Uuid;

/// Adaptador del servicio gRPC `PatientApi`.
///
/// Busca y muestra pacientes al médico o a recepción con membresía activa en una
/// clínica activa, y deja que recepción registre pacientes sin cuenta. Siempre
/// limitado a los expedientes de esa clínica: un expediente de otra clínica se trata
/// como inexistente (`openspec/specs/access-control/action-authorization`).
pub struct PatientApiImpl {
    patient_repository: Arc<dyn PatientRepository>,
    patient_use_cases: Arc<PatientUseCases>,
    membership_use_cases: Arc<MembershipUseCases>,
    access: Arc<AccessContextBuilder>,
    subdomain_resolver: Arc<SubdomainResolver>,
}

impl PatientApiImpl {
    pub fn new(
        patient_repository: Arc<dyn PatientRepository>,
        patient_use_cases: Arc<PatientUseCases>,
        membership_use_cases: Arc<MembershipUseCases>,
        access: Arc<AccessContextBuilder>,
        subdomain_resolver: Arc<SubdomainResolver>,
    ) -> Self {
        Self {
            patient_repository,
            patient_use_cases,
            membership_use_cases,
            access,
            subdomain_resolver,
        }
    }

    /// Autoriza la acción y devuelve la clínica en la que se actúa.
    ///
    /// La clínica sale del subdominio de la petición. Sin subdominio, se usa la única
    /// clínica donde la persona está activa; con varias, debe indicar el subdominio.
    async fn authorize<T>(
        &self,
        request: &Request<T>,
        action: ProtectedAction,
    ) -> Result<(RequestContext, Uuid), Status> {
        let claims = claims_of(request)?;
        let (_user, ctx) = self.access.build(&claims).await?;
        let (_network_id, resolved) = self.subdomain_resolver.resolve(request).await?;

        let active_clinics: Vec<Uuid> = ctx
            .memberships
            .iter()
            .filter(|m| m.status == MembershipStatus::Active)
            .map(|m| m.organization_id)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let organization_id = if ctx
            .memberships
            .iter()
            .any(|m| m.organization_id == resolved)
        {
            resolved
        } else {
            match active_clinics.as_slice() {
                [only] => *only,
                [] => return Err(denied_to_status(AccessDenied::Forbidden, None)),
                _ => {
                    return Err(Status::invalid_argument(
                        "Trabajas en varias clínicas: entra por el subdominio de la clínica",
                    ));
                }
            }
        };

        let clinic = self
            .membership_use_cases
            .clinic(&organization_id)
            .await
            .map_err(|_| denied_to_status(AccessDenied::Forbidden, None))?;
        authorize(action, &ctx, &clinic_target(&clinic))
            .map_err(|denied| denied_to_status(denied, Some(organization_id)))?;
        Ok((ctx, organization_id))
    }
}

#[async_trait]
impl PatientApi for PatientApiImpl {
    async fn search_patient(
        &self,
        request: Request<SearchPatientRequest>,
    ) -> Result<Response<SearchPatientResponse>, Status> {
        let (_ctx, organization_id) = self
            .authorize(&request, ProtectedAction::SearchPatients)
            .await?;
        let name = request.into_inner().name.unwrap_or_default();
        let patients = self
            .patient_repository
            .search(&organization_id, &name)
            .await
            .map_err(|error| Status::internal(error.to_string()))?;

        Ok(Response::new(SearchPatientResponse {
            first_name: None,
            last_name: None,
            second_last_name: None,
            email: None,
            patients: patients.iter().map(patient_to_proto).collect(),
        }))
    }

    async fn get_patient_by_id(
        &self,
        request: Request<PatientIdRequest>,
    ) -> Result<Response<Patient>, Status> {
        let not_found = || Status::not_found("El paciente no existe");
        // Un expediente que no se puede ver se trata como inexistente: así no se
        // revela si existe (escenario «Expediente ajeno»).
        let (_ctx, organization_id) = self
            .authorize(&request, ProtectedAction::ViewPatient)
            .await
            .map_err(|status| match status.code() {
                Code::PermissionDenied => not_found(),
                _ => status,
            })?;
        let patient_id = request
            .into_inner()
            .id
            .and_then(|id| Uuid::parse_str(id.trim()).ok())
            .ok_or_else(not_found)?;
        self.patient_repository
            .find_by_id(&organization_id, &patient_id)
            .await
            .map_err(|error| Status::internal(error.to_string()))?
            .map(|patient| Response::new(patient_to_proto(&patient)))
            .ok_or_else(not_found)
    }

    async fn register_patient(
        &self,
        request: Request<RegisterPatientRequest>,
    ) -> Result<Response<Patient>, Status> {
        let (ctx, organization_id) = self
            .authorize(&request, ProtectedAction::RegisterPatient)
            .await?;
        let request = request.into_inner();
        let identifier = request
            .identifier
            .and_then(|identifier| identifier.identifier_type)
            .map(|kind| match kind {
                identifier::IdentifierType::Dni(value) => Identifier::DNI(value.trim().to_string()),
                identifier::IdentifierType::Ce(value) => Identifier::CE(value.trim().to_string()),
                identifier::IdentifierType::Passport(value) => {
                    Identifier::Passport(value.trim().to_string())
                }
            });
        let patient = self
            .patient_use_cases
            .register(
                &ctx,
                organization_id,
                NewPatient {
                    given_name: request.given_name,
                    family_name: request.family_name,
                    second_family_name: request.second_family_name,
                    identifier,
                    phone: request.phone,
                    birth_date: request.birth_date,
                },
            )
            .await
            .map_err(|error| match error {
                MembershipError::Denied(denied) => denied_to_status(denied, Some(organization_id)),
                MembershipError::NotFound(message) => Status::not_found(message),
                MembershipError::AlreadyExists(message) => Status::already_exists(message),
                MembershipError::InvalidInput(message) => Status::invalid_argument(message),
                other => Status::internal(other.to_string()),
            })?;
        Ok(Response::new(patient_to_proto(&patient)))
    }

    /// Todavía no persiste: solo exige el mismo permiso que ver un paciente.
    async fn save(&self, request: Request<Patient>) -> Result<Response<Patient>, Status> {
        self.authorize(&request, ProtectedAction::ViewPatient)
            .await?;
        Ok(Response::new(request.into_inner()))
    }
}

fn patient_to_proto(patient: &DomainPatient) -> Patient {
    let person = &patient.person;
    let mut family = Vec::new();
    family.extend(person.name().family().clone());
    family.extend(person.name().second_family().clone());
    Patient {
        id: patient.id.to_string(),
        identifier: person
            .identifier()
            .as_ref()
            .map(identifier_to_proto)
            .into_iter()
            .collect(),
        active: patient.active,
        name: vec![HumanName {
            r#use: "official".to_string(),
            family,
            given: person.name().given().clone(),
        }],
        telecom: person
            .telecom()
            .iter()
            .map(|contact| ContactPoint {
                system: match contact.system {
                    ContactPointSystem::Email => contact_point::System::Email,
                    ContactPointSystem::Phone => contact_point::System::Phone,
                    _ => contact_point::System::Unspecified,
                } as i32,
                value: contact.value.clone(),
                r#use: contact_point::Use::Unspecified as i32,
            })
            .collect(),
        gender: patient::Gender::Unknown as i32,
        birth_date: person.birth_date().clone().unwrap_or_default(),
        deceased_status: None,
        managing_organization_reference: format!("Organization/{}", patient.organization_id),
        photo: vec![],
    }
}
