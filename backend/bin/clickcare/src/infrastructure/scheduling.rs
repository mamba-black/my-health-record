//! Adaptadores de los puertos de la agenda (`crates/scheduling`) sobre
//! `administration` y `user`. Así la agenda no depende de esos crates.

use crate::infrastructure::grpc::access::AccessContextBuilder;
use administration::application::membership_usecase::MembershipError;
use administration::application::patient_usecase::{NewPatient, PatientUseCases};
use administration::domain::membership::membership_repository::MembershipRepository;
use administration::domain::organization::Organization;
use administration::domain::organization::organization_repository::OrganizationRepository;
use administration::domain::patient::Patient;
use administration::domain::patient::patient_link_repository::{
    PatientLink, PatientLinkRepository,
};
use administration::domain::patient::patient_repository::PatientRepository;
use app_core::domain::access::{MembershipRole, MembershipStatus};
use app_core::domain::error::ClickCareError;
use app_core::domain::fhir::Identifier;
use async_trait::async_trait;
use scheduling::application::integration::{
    ClinicDirectory, ClinicInfo, IdentityVerification, LinkSuggestion, NewPatientData,
    NotificationSender, PatientRef, PatientRegistry, PractitionerInfo,
};
use scheduling::domain::calendar::Zone;
use scheduling::domain::error::SchedulingError;
use scheduling::domain::reminder::ReminderMessage;
use std::sync::Arc;
use tracing::{info, warn};
use user::domain::user::user_repository::UserRepository;
use uuid::Uuid;

fn unknown(error: ClickCareError) -> SchedulingError {
    SchedulingError::Unknown(error)
}

/// Zona IANA de la clínica; si no se reconoce, Lima.
fn zone_of(name: &str) -> Zone {
    Zone::get(name).unwrap_or_else(|error| {
        warn!("Zona horaria desconocida '{name}' ({error}); se usa America/Lima");
        Zone::get("America/Lima").unwrap_or(Zone::UTC)
    })
}

// ─── ClinicDirectory ────────────────────────────────────────────────────────

pub struct ClinicDirectoryAdapter {
    pub organizations: Arc<dyn OrganizationRepository>,
    pub memberships: Arc<dyn MembershipRepository>,
    pub users: Arc<dyn UserRepository>,
}

fn clinic_info(organization: Organization) -> ClinicInfo {
    ClinicInfo {
        zone: zone_of(&organization.timezone),
        id: organization.id,
        network_id: organization.network_id,
        subdomain: organization.subdomain,
        name: organization.trade_name.unwrap_or(organization.name),
        status: organization.status,
        address: organization.address,
    }
}

#[async_trait]
impl ClinicDirectory for ClinicDirectoryAdapter {
    async fn by_subdomain(&self, subdomain: &str) -> Result<Option<ClinicInfo>, SchedulingError> {
        let Some((id, _)) = self
            .organizations
            .find_organization_and_network_by_subdomain(&subdomain.trim().to_lowercase())
            .await
            .map_err(unknown)?
        else {
            return Ok(None);
        };
        self.by_id(&id).await
    }

    async fn by_id(&self, organization_id: &Uuid) -> Result<Option<ClinicInfo>, SchedulingError> {
        Ok(self
            .organizations
            .find_by_id(organization_id)
            .await
            .map_err(unknown)?
            .map(clinic_info))
    }

    async fn active_practitioners(
        &self,
        organization_id: &Uuid,
    ) -> Result<Vec<PractitionerInfo>, SchedulingError> {
        let mut practitioners = Vec::new();
        for membership in self
            .memberships
            .find_by_organization(organization_id)
            .await
            .map_err(unknown)?
            .into_iter()
            .filter(|m| {
                m.role == MembershipRole::Practitioner && m.status == MembershipStatus::Active
            })
        {
            if let Some(user) = self
                .users
                .find_user_by_id(&membership.user_id)
                .await
                .map_err(unknown)?
            {
                practitioners.push(PractitionerInfo {
                    user_id: user.id,
                    name: user.person.name().text().clone(),
                    specialty: user.professional.specialty.clone(),
                });
            }
        }
        Ok(practitioners)
    }

    async fn person_name(&self, user_id: &Uuid) -> Result<Option<String>, SchedulingError> {
        Ok(self
            .users
            .find_user_by_id(user_id)
            .await
            .map_err(unknown)?
            .map(|user| user.person.name().text().clone()))
    }
}

// ─── PatientRegistry ────────────────────────────────────────────────────────

pub struct PatientRegistryAdapter {
    pub patients: Arc<dyn PatientRepository>,
    pub links: Arc<dyn PatientLinkRepository>,
    pub patient_use_cases: Arc<PatientUseCases>,
    pub users: Arc<dyn UserRepository>,
}

fn patient_ref(patient: &Patient) -> PatientRef {
    PatientRef {
        id: patient.id,
        organization_id: patient.organization_id,
        user_id: patient.user_id,
        name: patient.person.name().text().clone(),
    }
}

impl PatientRegistryAdapter {
    async fn find_patient(
        &self,
        organization_id: &Uuid,
        patient_id: &Uuid,
    ) -> Result<Option<Patient>, SchedulingError> {
        self.patients
            .find_by_id(organization_id, patient_id)
            .await
            .map_err(unknown)
    }
}

#[async_trait]
impl PatientRegistry for PatientRegistryAdapter {
    async fn ensure_for_account(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<PatientRef, SchedulingError> {
        if let Some(id) = self
            .links
            .record_of_account(organization_id, user_id)
            .await
            .map_err(unknown)?
            && let Some(patient) = self.find_patient(organization_id, &id).await?
        {
            return Ok(patient_ref(&patient));
        }
        let user = self
            .users
            .find_user_by_id(user_id)
            .await
            .map_err(unknown)?
            .ok_or_else(|| SchedulingError::NotFound("La cuenta no existe".to_string()))?;
        let patient = Patient::new(Uuid::now_v7(), *organization_id, *user_id, user.person);
        if let Err(error) = self.patients.save(&patient).await {
            // Otra reserva simultánea lo creó: se usa ese.
            let id = self
                .links
                .record_of_account(organization_id, user_id)
                .await
                .map_err(unknown)?
                .ok_or(SchedulingError::Unknown(error))?;
            let existing = self
                .find_patient(organization_id, &id)
                .await?
                .ok_or_else(|| SchedulingError::NotFound("El paciente no existe".into()))?;
            return Ok(patient_ref(&existing));
        }
        info!(
            "Paciente registrado en su primera cita: patient={}",
            patient.id
        );
        Ok(patient_ref(&patient))
    }

    async fn register(
        &self,
        organization_id: &Uuid,
        data: NewPatientData,
    ) -> Result<PatientRef, SchedulingError> {
        let identifier = Identifier::from_parts(&data.document_type, data.document_value.clone());
        if identifier.is_none() && !data.document_value.trim().is_empty() {
            return Err(SchedulingError::Invalid(
                "El tipo de documento debe ser DNI, CE o PASSPORT".to_string(),
            ));
        }
        self.patient_use_cases
            .register_authorized(
                *organization_id,
                NewPatient {
                    given_name: data.given_name,
                    family_name: data.family_name,
                    second_family_name: data.second_family_name,
                    identifier,
                    phone: data.phone,
                    birth_date: data.birth_date,
                },
            )
            .await
            .map(|patient| patient_ref(&patient))
            .map_err(|error| match error {
                MembershipError::InvalidInput(message) => {
                    match message.strip_prefix("Faltan datos del paciente: ") {
                        Some(fields) => SchedulingError::MissingPatientData(fields.to_string()),
                        None => SchedulingError::Invalid(message),
                    }
                }
                MembershipError::AlreadyExists(message) => SchedulingError::Invalid(format!(
                    "{message}: búscalo y resérvale con su registro"
                )),
                MembershipError::NotFound(message) => SchedulingError::NotFound(message),
                MembershipError::Unknown(error) => SchedulingError::Unknown(error),
                other => SchedulingError::Invalid(other.to_string()),
            })
    }

    async fn find(
        &self,
        organization_id: &Uuid,
        patient_id: &Uuid,
    ) -> Result<Option<PatientRef>, SchedulingError> {
        Ok(self
            .find_patient(organization_id, patient_id)
            .await?
            .map(|patient| patient_ref(&patient)))
    }

    async fn records_of_account(
        &self,
        network_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<Uuid>, SchedulingError> {
        self.links
            .records_of_account(network_id, user_id)
            .await
            .map_err(unknown)
    }

    async fn account_of(&self, patient_id: &Uuid) -> Result<Option<Uuid>, SchedulingError> {
        self.links.account_of(patient_id).await.map_err(unknown)
    }

    async fn mark_document_verified(
        &self,
        patient_id: &Uuid,
        verified_by: &Uuid,
    ) -> Result<(), SchedulingError> {
        self.links
            .mark_document_verified(patient_id, verified_by)
            .await
            .map_err(unknown)
    }

    async fn suggest_links(
        &self,
        network_id: &Uuid,
        organization_id: &Uuid,
        patient_id: &Uuid,
    ) -> Result<Vec<LinkSuggestion>, SchedulingError> {
        let Some(patient) = self.find_patient(organization_id, patient_id).await? else {
            return Ok(Vec::new());
        };
        let Some(identifier) = patient.person.identifier().clone() else {
            return Ok(Vec::new());
        };
        if patient.user_id.is_some()
            || self
                .links
                .active_link(patient_id)
                .await
                .map_err(unknown)?
                .is_some()
        {
            return Ok(Vec::new());
        }
        Ok(self
            .users
            .find_users_by_document(network_id, &identifier)
            .await
            .map_err(unknown)?
            .into_iter()
            .map(|user| LinkSuggestion {
                user_id: user.id,
                email: user.email(),
                name: user.person.name().text().clone(),
            })
            .collect())
    }

    async fn link(
        &self,
        network_id: &Uuid,
        organization_id: &Uuid,
        patient_id: &Uuid,
        user_id: &Uuid,
        linked_by: &Uuid,
    ) -> Result<Uuid, SchedulingError> {
        let patient = self
            .find_patient(organization_id, patient_id)
            .await?
            .ok_or_else(|| SchedulingError::NotFound("El paciente no existe".to_string()))?;
        if patient.user_id.is_some()
            || self
                .links
                .active_link(patient_id)
                .await
                .map_err(unknown)?
                .is_some()
        {
            return Err(SchedulingError::Invalid(
                "El paciente ya está vinculado a una cuenta".to_string(),
            ));
        }
        let user = self
            .users
            .find_user_by_id(user_id)
            .await
            .map_err(unknown)?
            .filter(|user| user.network_id == *network_id)
            .ok_or_else(|| SchedulingError::NotFound("La cuenta no existe".to_string()))?;
        if patient.person.identifier().is_none()
            || patient.person.identifier() != user.person.identifier()
        {
            return Err(SchedulingError::Invalid(
                "El documento de la cuenta no coincide con el del paciente".to_string(),
            ));
        }
        let link = PatientLink {
            id: Uuid::now_v7(),
            network_id: *network_id,
            organization_id: *organization_id,
            patient_id: *patient_id,
            user_id: *user_id,
            linked_by: *linked_by,
        };
        self.links.insert(&link).await.map_err(unknown)?;
        Ok(link.id)
    }

    async fn unlink(
        &self,
        organization_id: &Uuid,
        link_id: &Uuid,
        unlinked_by: &Uuid,
        reason: &str,
    ) -> Result<Uuid, SchedulingError> {
        self.links
            .close(organization_id, link_id, unlinked_by, reason)
            .await
            .map_err(unknown)?
            .ok_or_else(|| SchedulingError::NotFound("El vínculo no existe".to_string()))
    }
}

// ─── IdentityVerification ───────────────────────────────────────────────────

pub struct IdentityVerificationAdapter {
    pub users: Arc<dyn UserRepository>,
    pub access: Arc<AccessContextBuilder>,
}

#[async_trait]
impl IdentityVerification for IdentityVerificationAdapter {
    async fn verify_document(&self, user_id: &Uuid) -> Result<(), SchedulingError> {
        self.users
            .set_document_verified(user_id)
            .await
            .map_err(unknown)?;
        // Sus datos cambiaron: la próxima petición vuelve a leer su contexto.
        self.access.invalidate(user_id);
        Ok(())
    }

    async fn email_of(&self, user_id: &Uuid) -> Result<Option<String>, SchedulingError> {
        Ok(self
            .users
            .find_user_by_id(user_id)
            .await
            .map_err(unknown)?
            .map(|user| user.email()))
    }
}

// ─── NotificationSender ─────────────────────────────────────────────────────

/// Sin servidor de correo (`SMTP_HOST`): solo deja constancia en el log, sin datos
/// personales (ni correo ni nombres).
pub struct LogNotificationSender;

#[async_trait]
impl NotificationSender for LogNotificationSender {
    async fn send(&self, _to: &str, message: &ReminderMessage) -> Result<(), String> {
        info!(
            "Recordatorio de cita (sin servidor de correo, solo log) para {}",
            message.start
        );
        Ok(())
    }
}

/// Envío por SMTP con `lettre`. Se usa cuando hay `SMTP_HOST`.
pub struct SmtpNotificationSender {
    transport: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    from: lettre::message::Mailbox,
}

impl SmtpNotificationSender {
    /// Lee `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` y `SMTP_FROM`.
    /// `None` sin `SMTP_HOST`: entonces se usa [`LogNotificationSender`].
    pub fn from_env() -> Result<Option<Self>, ClickCareError> {
        let Some(host) = std::env::var("SMTP_HOST")
            .ok()
            .filter(|h| !h.trim().is_empty())
        else {
            return Ok(None);
        };
        let mut builder =
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(&host)
                .map_err(|e| ClickCareError::generic(format!("SMTP_HOST inválido ({e})")))?;
        if let Some(port) = std::env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()) {
            builder = builder.port(port);
        }
        if let (Ok(user), Ok(password)) =
            (std::env::var("SMTP_USER"), std::env::var("SMTP_PASSWORD"))
        {
            builder = builder.credentials(
                lettre::transport::smtp::authentication::Credentials::new(user, password),
            );
        }
        let from = std::env::var("SMTP_FROM")
            .unwrap_or_else(|_| "My Health Record <no-reply@localhost>".to_string())
            .parse()
            .map_err(|e| ClickCareError::generic(format!("SMTP_FROM inválido ({e})")))?;
        Ok(Some(Self {
            transport: builder.build(),
            from,
        }))
    }
}

#[async_trait]
impl NotificationSender for SmtpNotificationSender {
    async fn send(&self, to: &str, message: &ReminderMessage) -> Result<(), String> {
        use lettre::AsyncTransport;
        let to = to
            .parse()
            .map_err(|_| "correo del paciente inválido".to_string())?;
        let email = lettre::Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(message.subject())
            .header(lettre::message::header::ContentType::TEXT_PLAIN)
            .body(message.body())
            .map_err(|e| format!("no se pudo armar el correo ({e})"))?;
        self.transport
            .send(email)
            .await
            .map(|_| ())
            .map_err(|e| format!("el servidor de correo rechazó el envío ({e})"))
    }
}

/// El canal de envío según la configuración.
pub fn notification_sender() -> Result<Arc<dyn NotificationSender>, ClickCareError> {
    Ok(match SmtpNotificationSender::from_env()? {
        Some(smtp) => {
            info!("Recordatorios por correo SMTP");
            Arc::new(smtp)
        }
        None => {
            info!("Sin SMTP_HOST: los recordatorios solo se registran en el log");
            Arc::new(LogNotificationSender)
        }
    })
}
