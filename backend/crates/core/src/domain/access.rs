//! Control de acceso por rol y por datos registrados.
//!
//! Fuente de verdad: `openspec/specs/access-control/action-authorization` y
//! `openspec/specs/identity/progressive-profiling`. Este módulo concentra dos cosas:
//!
//! - El **catálogo de datos** ([`CATALOG`]): qué dato aplica a qué rol y qué acciones lo
//!   vuelven obligatorio. Ningún dato es obligatorio solo por figurar en él.
//! - La **regla de autorización** ([`authorize`]): una función pura que decide si una
//!   acción procede según la membresía, el estado de la clínica y los datos presentes.
//!
//! Vive en el núcleo compartido porque la decisión cruza identidad (`user`) y
//! administración (`administration`), y ninguno de esos contextos debe depender del otro.

use std::collections::HashSet;
use uuid::Uuid;

/// Rol preferido de la cuenta. Solo orienta la interfaz: no concede permisos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Role {
    #[default]
    Patient,
    Practitioner,
    ClinicOwner,
    /// Recepción de una clínica existente: nunca crea clínicas.
    Receptionist,
}

impl Role {
    /// Código persistido en `identity.user_account.preferred_role`.
    pub fn code(self) -> &'static str {
        match self {
            Role::Patient => "PATIENT",
            Role::Practitioner => "PRACTITIONER",
            Role::ClinicOwner => "CLINIC_OWNER",
            Role::Receptionist => "RECEPTIONIST",
        }
    }

    /// Lee el código persistido; un valor desconocido se trata como paciente.
    pub fn from_code(code: &str) -> Self {
        match code {
            "PRACTITIONER" => Role::Practitioner,
            "CLINIC_OWNER" => Role::ClinicOwner,
            "RECEPTIONIST" => Role::Receptionist,
            _ => Role::Patient,
        }
    }
}

/// Dato del perfil (personal, profesional o de la clínica).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileField {
    GivenName,
    FamilyName,
    SecondFamilyName,
    Email,
    Phone,
    Document,
    BirthDate,
    Sex,
    Address,
    /// Colegio profesional y número de colegiatura.
    ProfessionalLicense,
    /// Especialidad y RNE.
    Specialty,
    /// Colegiatura confirmada por la clínica. No se llena en un formulario.
    LicenseConfirmed,
    ClinicTaxId,
    ClinicLegalName,
    ClinicTradeName,
    IpressCode,
}

/// De quién es el dato: del usuario que actúa, de la clínica o de su membresía.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSubject {
    Person,
    Clinic,
    Membership,
}

/// Acción protegida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectedAction {
    ConfirmAppointment,
    JoinClinic,
    ActivatePractitioner,
    SearchPatients,
    ViewPatient,
    OpenFirstEncounter,
    RecordEncounter,
    IssuePrescription,
    CreateClinic,
    ConfigureClinic,
    InviteStaff,
    SuspendMember,
    ConfirmLicense,
    ActivateClinic,
    /// Recepción registra en su clínica a un paciente, aunque no tenga cuenta.
    RegisterPatient,
    /// Un usuario pide entrar a una clínica existente como médico o recepcionista.
    RequestMembership,
    /// El dueño o un administrador aprueba una solicitud de ingreso.
    ApproveMember,
    /// El médico define su horario y sus excepciones en la clínica.
    ManageSchedule,
    /// Ver la agenda de la clínica (el médico, solo la suya).
    ViewClinicAgenda,
    /// Reservar en nombre de un paciente (recepción, o el médico en su agenda).
    BookForPatient,
    /// Cancelar o reprogramar como clínica.
    ManageAppointment,
    /// Registrar llegada, verificación del documento e inasistencia.
    CheckIn,
    /// Sugerir y confirmar el vínculo de un paciente sin cuenta con su cuenta.
    LinkPatientAccount,
    /// Deshacer un vínculo equivocado.
    UnlinkPatientAccount,
}

/// Rol de un usuario dentro de una clínica.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MembershipRole {
    Owner,
    Admin,
    Practitioner,
    /// Recepción: registra y busca pacientes. No ve la historia clínica ni configura la clínica.
    Receptionist,
}

impl MembershipRole {
    pub fn code(self) -> &'static str {
        match self {
            MembershipRole::Owner => "OWNER",
            MembershipRole::Admin => "ADMIN",
            MembershipRole::Practitioner => "PRACTITIONER",
            MembershipRole::Receptionist => "RECEPTIONIST",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "OWNER" => Some(MembershipRole::Owner),
            "ADMIN" => Some(MembershipRole::Admin),
            "PRACTITIONER" => Some(MembershipRole::Practitioner),
            "RECEPTIONIST" => Some(MembershipRole::Receptionist),
            _ => None,
        }
    }
}

/// Estado de una membresía.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MembershipStatus {
    Invited,
    /// La persona pidió entrar y espera que el dueño o un administrador la apruebe.
    Requested,
    Pending,
    Active,
    Suspended,
}

impl MembershipStatus {
    pub fn code(self) -> &'static str {
        match self {
            MembershipStatus::Invited => "INVITED",
            MembershipStatus::Requested => "REQUESTED",
            MembershipStatus::Pending => "PENDING",
            MembershipStatus::Active => "ACTIVE",
            MembershipStatus::Suspended => "SUSPENDED",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "INVITED" => Some(MembershipStatus::Invited),
            "REQUESTED" => Some(MembershipStatus::Requested),
            "PENDING" => Some(MembershipStatus::Pending),
            "ACTIVE" => Some(MembershipStatus::Active),
            "SUSPENDED" => Some(MembershipStatus::Suspended),
            _ => None,
        }
    }
}

/// Estado de una clínica (RFC-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClinicStatus {
    PendingSetup,
    Active,
    Suspended,
}

impl ClinicStatus {
    pub fn code(self) -> &'static str {
        match self {
            ClinicStatus::PendingSetup => "PENDING_SETUP",
            ClinicStatus::Active => "ACTIVE",
            ClinicStatus::Suspended => "SUSPENDED",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "PENDING_SETUP" => Some(ClinicStatus::PendingSetup),
            "ACTIVE" => Some(ClinicStatus::Active),
            "SUSPENDED" => Some(ClinicStatus::Suspended),
            _ => None,
        }
    }
}

/// Entrada del catálogo de datos.
#[derive(Debug, Clone, Copy)]
pub struct FieldSpec {
    pub field: ProfileField,
    pub subject: FieldSubject,
    /// Roles a los que aplica el dato.
    pub roles: &'static [Role],
    /// Acciones que lo vuelven obligatorio. Vacío: siempre opcional.
    pub required_for: &'static [ProtectedAction],
    /// Se muestra pero el usuario no lo edita (viene del proveedor o de la clínica).
    pub user_editable: bool,
}

const ALL_ROLES: &[Role] = &[
    Role::Patient,
    Role::Practitioner,
    Role::ClinicOwner,
    Role::Receptionist,
];

/// Catálogo único de datos de perfil. Ver la tabla de `design.md` del cambio
/// `role-based-access-by-profile-data`.
pub const CATALOG: &[FieldSpec] = {
    use ProtectedAction as A;
    &[
        FieldSpec {
            field: ProfileField::GivenName,
            subject: FieldSubject::Person,
            roles: ALL_ROLES,
            required_for: &[
                A::ConfirmAppointment,
                A::JoinClinic,
                A::CreateClinic,
                A::ActivatePractitioner,
                A::ActivateClinic,
                A::RequestMembership,
            ],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::FamilyName,
            subject: FieldSubject::Person,
            roles: ALL_ROLES,
            required_for: &[
                A::ConfirmAppointment,
                A::ActivatePractitioner,
                A::ActivateClinic,
                A::RequestMembership,
            ],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::SecondFamilyName,
            subject: FieldSubject::Person,
            roles: ALL_ROLES,
            required_for: &[],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::Email,
            subject: FieldSubject::Person,
            roles: ALL_ROLES,
            required_for: &[],
            user_editable: false,
        },
        FieldSpec {
            field: ProfileField::Phone,
            subject: FieldSubject::Person,
            roles: ALL_ROLES,
            required_for: &[
                A::ConfirmAppointment,
                A::ActivatePractitioner,
                A::ActivateClinic,
                A::RequestMembership,
            ],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::Document,
            subject: FieldSubject::Person,
            roles: ALL_ROLES,
            required_for: &[
                A::ConfirmAppointment,
                A::ActivatePractitioner,
                A::ActivateClinic,
                A::RequestMembership,
            ],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::BirthDate,
            subject: FieldSubject::Person,
            roles: &[Role::Patient],
            required_for: &[A::OpenFirstEncounter],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::Sex,
            subject: FieldSubject::Person,
            roles: &[Role::Patient],
            required_for: &[A::OpenFirstEncounter],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::Address,
            subject: FieldSubject::Person,
            roles: &[Role::Patient],
            required_for: &[],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::ProfessionalLicense,
            subject: FieldSubject::Person,
            roles: &[Role::Practitioner],
            required_for: &[A::ActivatePractitioner],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::Specialty,
            subject: FieldSubject::Person,
            roles: &[Role::Practitioner],
            required_for: &[],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::LicenseConfirmed,
            subject: FieldSubject::Membership,
            roles: &[Role::Practitioner],
            required_for: &[A::RecordEncounter, A::IssuePrescription, A::ManageSchedule],
            user_editable: false,
        },
        FieldSpec {
            field: ProfileField::ClinicTaxId,
            subject: FieldSubject::Clinic,
            roles: &[Role::ClinicOwner],
            required_for: &[A::ActivateClinic],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::ClinicLegalName,
            subject: FieldSubject::Clinic,
            roles: &[Role::ClinicOwner],
            required_for: &[A::ActivateClinic],
            user_editable: true,
        },
        FieldSpec {
            field: ProfileField::ClinicTradeName,
            subject: FieldSubject::Clinic,
            roles: &[Role::ClinicOwner],
            required_for: &[],
            user_editable: true,
        },
        // Exigencia para activar la clínica pendiente de validación legal: hoy opcional.
        FieldSpec {
            field: ProfileField::IpressCode,
            subject: FieldSubject::Clinic,
            roles: &[Role::ClinicOwner],
            required_for: &[],
            user_editable: true,
        },
    ]
};

/// Entrada del catálogo para un dato.
pub fn spec_of(field: ProfileField) -> &'static FieldSpec {
    CATALOG
        .iter()
        .find(|spec| spec.field == field)
        .unwrap_or(&CATALOG[0])
}

/// Datos que exige una acción, en el orden del catálogo.
pub fn required_fields(action: ProtectedAction) -> Vec<ProfileField> {
    CATALOG
        .iter()
        .filter(|spec| spec.required_for.contains(&action))
        .map(|spec| spec.field)
        .collect()
}

/// Acción habitual de cada rol preferido: la que alimenta el aviso al iniciar sesión.
pub fn habitual_action(role: Role) -> ProtectedAction {
    match role {
        Role::Patient => ProtectedAction::ConfirmAppointment,
        Role::Practitioner => ProtectedAction::ActivatePractitioner,
        Role::ClinicOwner => ProtectedAction::ActivateClinic,
        Role::Receptionist => ProtectedAction::RequestMembership,
    }
}

/// Datos personales que le faltan a alguien para una acción (sin los de clínica ni membresía).
pub fn missing_person_fields(
    action: ProtectedAction,
    present: &HashSet<ProfileField>,
) -> Vec<ProfileField> {
    required_fields(action)
        .into_iter()
        .filter(|field| spec_of(*field).subject == FieldSubject::Person)
        .filter(|field| !present.contains(field))
        .collect()
}

/// Membresía tal como la ve la regla de autorización.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipView {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub role: MembershipRole,
    pub status: MembershipStatus,
    pub license_confirmed: bool,
}

/// Quién llama: su identidad, los datos que tiene registrados y sus membresías.
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub user_id: Uuid,
    pub network_id: Uuid,
    pub preferred_role: Role,
    pub present_fields: HashSet<ProfileField>,
    pub memberships: Vec<MembershipView>,
}

impl RequestContext {
    /// Roles que aplican al usuario para mostrarle datos: todos pueden ser pacientes,
    /// más su rol preferido y los que le dan sus membresías.
    pub fn roles(&self) -> Vec<Role> {
        let mut roles = vec![Role::Patient];
        let mut push = |role: Role| {
            if !roles.contains(&role) {
                roles.push(role);
            }
        };
        push(self.preferred_role);
        for membership in &self.memberships {
            if membership.status == MembershipStatus::Suspended {
                continue;
            }
            match membership.role {
                MembershipRole::Owner => push(Role::ClinicOwner),
                MembershipRole::Practitioner => push(Role::Practitioner),
                MembershipRole::Receptionist => push(Role::Receptionist),
                MembershipRole::Admin => {}
            }
        }
        roles
    }

    /// Membresía del usuario en una clínica con un rol dado.
    pub fn membership(
        &self,
        organization_id: Uuid,
        role: MembershipRole,
    ) -> Option<&MembershipView> {
        self.memberships
            .iter()
            .find(|m| m.organization_id == organization_id && m.role == role)
    }
}

/// Sobre qué recurso se intenta la acción.
#[derive(Debug, Clone, Default)]
pub struct AccessTarget {
    /// Clínica en la que se actúa.
    pub organization_id: Option<Uuid>,
    /// Estado de esa clínica.
    pub clinic_status: Option<ClinicStatus>,
    /// Datos de la clínica ya registrados (RUC, razón social…).
    pub clinic_fields: HashSet<ProfileField>,
    /// Membresía sobre la que se actúa (unirse, activar perfil).
    pub membership_id: Option<Uuid>,
}

/// Motivo de un rechazo. El orden de evaluación es el de las variantes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDenied {
    /// Sin credencial válida. Lo decide el interceptor, nunca [`authorize`].
    Unauthenticated,
    /// Sin el rol o la membresía que exige la acción.
    Forbidden,
    /// La clínica no está en un estado que permita la acción.
    ClinicNotActive,
    /// Tiene el permiso pero le faltan datos.
    MissingData {
        action: ProtectedAction,
        fields: Vec<ProfileField>,
    },
}

/// Decide si `ctx` puede ejecutar `action` sobre `target`.
///
/// Orden de evaluación: rol o membresía → estado de la clínica → datos. Así nunca se
/// revelan datos faltantes a quien no tendría permiso de todos modos.
pub fn authorize(
    action: ProtectedAction,
    ctx: &RequestContext,
    target: &AccessTarget,
) -> Result<(), AccessDenied> {
    use ProtectedAction as A;

    // 1. Rol y membresía.
    // Una cuenta de recepción entra a clínicas existentes: nunca funda una.
    if action == A::CreateClinic && ctx.preferred_role == Role::Receptionist {
        return Err(AccessDenied::Forbidden);
    }
    let membership = match action {
        // Pedir el ingreso no exige membresía: que no tenga ya una en la clínica lo
        // revisa el caso de uso.
        A::ConfirmAppointment | A::CreateClinic | A::RequestMembership => None,
        A::JoinClinic => Some(own_membership(
            ctx,
            target,
            &[MembershipStatus::Invited],
            &[
                MembershipRole::Practitioner,
                MembershipRole::Admin,
                MembershipRole::Receptionist,
            ],
        )?),
        A::ActivatePractitioner => Some(own_membership(
            ctx,
            target,
            &[MembershipStatus::Invited, MembershipStatus::Pending],
            &[MembershipRole::Practitioner],
        )?),
        // Buscar y ver los datos de contacto: el médico y recepción.
        A::SearchPatients | A::ViewPatient => Some(active_membership(
            ctx,
            target,
            &[MembershipRole::Practitioner, MembershipRole::Receptionist],
        )?),
        A::RegisterPatient => Some(active_membership(
            ctx,
            target,
            &[
                MembershipRole::Owner,
                MembershipRole::Admin,
                MembershipRole::Receptionist,
            ],
        )?),
        A::ManageSchedule => Some(active_membership(
            ctx,
            target,
            &[MembershipRole::Practitioner],
        )?),
        A::ViewClinicAgenda | A::BookForPatient | A::ManageAppointment => Some(active_membership(
            ctx,
            target,
            &[
                MembershipRole::Practitioner,
                MembershipRole::Owner,
                MembershipRole::Admin,
                MembershipRole::Receptionist,
            ],
        )?),
        A::CheckIn | A::LinkPatientAccount => Some(active_membership(
            ctx,
            target,
            &[
                MembershipRole::Owner,
                MembershipRole::Admin,
                MembershipRole::Receptionist,
            ],
        )?),
        A::UnlinkPatientAccount => Some(active_membership(ctx, target, &[MembershipRole::Owner])?),
        A::OpenFirstEncounter | A::RecordEncounter | A::IssuePrescription => Some(
            active_membership(ctx, target, &[MembershipRole::Practitioner])?,
        ),
        A::ConfigureClinic
        | A::InviteStaff
        | A::SuspendMember
        | A::ConfirmLicense
        | A::ApproveMember => Some(active_membership(
            ctx,
            target,
            &[MembershipRole::Owner, MembershipRole::Admin],
        )?),
        A::ActivateClinic => Some(active_membership(ctx, target, &[MembershipRole::Owner])?),
    };

    // 2. Estado de la clínica.
    let allowed: &[ClinicStatus] = match action {
        A::CreateClinic | A::JoinClinic => &[],
        A::ConfigureClinic
        | A::InviteStaff
        | A::SuspendMember
        | A::ConfirmLicense
        | A::ApproveMember => &[ClinicStatus::PendingSetup, ClinicStatus::Active],
        A::ActivateClinic => &[ClinicStatus::PendingSetup],
        _ => &[ClinicStatus::Active],
    };
    if !allowed.is_empty() {
        match target.clinic_status {
            Some(status) if allowed.contains(&status) => {}
            _ => return Err(AccessDenied::ClinicNotActive),
        }
    }

    // 3. Datos.
    let missing: Vec<ProfileField> = required_fields(action)
        .into_iter()
        .filter(|field| {
            let present = match spec_of(*field).subject {
                FieldSubject::Person => ctx.present_fields.contains(field),
                FieldSubject::Clinic => target.clinic_fields.contains(field),
                FieldSubject::Membership => membership.is_some_and(|m| m.license_confirmed),
            };
            !present
        })
        // Los datos de la primera atención son del paciente, no del médico: se validan
        // al abrir la historia clínica, fuera de esta regla.
        .filter(|_| action != A::OpenFirstEncounter)
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(AccessDenied::MissingData {
            action,
            fields: missing,
        })
    }
}

/// Membresía propia sobre la que se actúa, con un estado y rol permitidos.
fn own_membership<'a>(
    ctx: &'a RequestContext,
    target: &AccessTarget,
    statuses: &[MembershipStatus],
    roles: &[MembershipRole],
) -> Result<&'a MembershipView, AccessDenied> {
    ctx.memberships
        .iter()
        .find(|m| {
            Some(m.id) == target.membership_id
                && statuses.contains(&m.status)
                && roles.contains(&m.role)
        })
        .ok_or(AccessDenied::Forbidden)
}

/// Membresía activa del usuario en la clínica objetivo con alguno de los roles.
fn active_membership<'a>(
    ctx: &'a RequestContext,
    target: &AccessTarget,
    roles: &[MembershipRole],
) -> Result<&'a MembershipView, AccessDenied> {
    let organization_id = target.organization_id.ok_or(AccessDenied::Forbidden)?;
    ctx.memberships
        .iter()
        .find(|m| {
            m.organization_id == organization_id
                && m.status == MembershipStatus::Active
                && roles.contains(&m.role)
        })
        .ok_or(AccessDenied::Forbidden)
}

/// Valida el RUC peruano: 11 dígitos y dígito verificador módulo 11 (SUNAT).
pub fn is_valid_ruc(ruc: &str) -> bool {
    if ruc.len() != 11 || !ruc.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let digits: Vec<u32> = ruc.chars().filter_map(|c| c.to_digit(10)).collect();
    const WEIGHTS: [u32; 10] = [5, 4, 3, 2, 7, 6, 5, 4, 3, 2];
    let sum: u32 = WEIGHTS.iter().zip(&digits).map(|(w, d)| w * d).sum();
    let check = match 11 - (sum % 11) {
        10 => 0,
        11 => 1,
        other => other,
    };
    digits[10] == check
}

#[cfg(test)]
mod tests {
    use super::ProfileField as F;
    use super::ProtectedAction as A;
    use super::*;

    const ORG: Uuid = uuid::uuid!("0191eb44-4860-7000-8000-0000000000aa");
    const OTHER_ORG: Uuid = uuid::uuid!("0191eb44-4860-7000-8000-0000000000bb");
    const MEMBERSHIP: Uuid = uuid::uuid!("0191eb44-4860-7000-8000-0000000000cc");

    fn fields(list: &[F]) -> HashSet<F> {
        list.iter().copied().collect()
    }

    fn all_person_fields() -> HashSet<F> {
        fields(&[
            F::GivenName,
            F::FamilyName,
            F::Phone,
            F::Document,
            F::BirthDate,
            F::Sex,
            F::ProfessionalLicense,
        ])
    }

    fn ctx(present: HashSet<F>, memberships: Vec<MembershipView>) -> RequestContext {
        RequestContext {
            user_id: Uuid::now_v7(),
            network_id: Uuid::now_v7(),
            preferred_role: Role::Patient,
            present_fields: present,
            memberships,
        }
    }

    fn membership(
        role: MembershipRole,
        status: MembershipStatus,
        license_confirmed: bool,
    ) -> MembershipView {
        MembershipView {
            id: MEMBERSHIP,
            organization_id: ORG,
            role,
            status,
            license_confirmed,
        }
    }

    fn target(status: Option<ClinicStatus>) -> AccessTarget {
        AccessTarget {
            organization_id: Some(ORG),
            clinic_status: status,
            clinic_fields: HashSet::new(),
            membership_id: Some(MEMBERSHIP),
        }
    }

    use ClinicStatus::{Active as ClinicActive, PendingSetup};
    use MembershipRole::{Admin, Owner, Practitioner, Receptionist};
    use MembershipStatus::{Active, Invited, Pending, Suspended};

    /// Una fila de la matriz de `design.md` por caso, más los órdenes de error.
    #[rstest::rstest]
    // Cualquier usuario autenticado
    #[case::create_clinic_ok(A::CreateClinic, fields(&[F::GivenName]), vec![], None, Ok(()))]
    #[case::create_clinic_sin_nombre(A::CreateClinic, fields(&[]), vec![], None,
        Err(AccessDenied::MissingData { action: A::CreateClinic, fields: vec![F::GivenName] }))]
    // Paciente
    #[case::confirmar_cita_ok(A::ConfirmAppointment, all_person_fields(), vec![], Some(ClinicActive), Ok(()))]
    #[case::confirmar_cita_sin_documento(A::ConfirmAppointment,
        fields(&[F::GivenName, F::FamilyName, F::Phone]), vec![], Some(ClinicActive),
        Err(AccessDenied::MissingData { action: A::ConfirmAppointment, fields: vec![F::Document] }))]
    #[case::confirmar_cita_clinica_pendiente(A::ConfirmAppointment, all_person_fields(), vec![],
        Some(PendingSetup), Err(AccessDenied::ClinicNotActive))]
    #[case::paciente_busca_pacientes(A::SearchPatients, all_person_fields(), vec![], Some(ClinicActive),
        Err(AccessDenied::Forbidden))]
    // Médico
    #[case::unirse_ok(A::JoinClinic, fields(&[F::GivenName]),
        vec![membership(Practitioner, Invited, false)], None, Ok(()))]
    #[case::unirse_sin_invitacion(A::JoinClinic, fields(&[F::GivenName]),
        vec![membership(Practitioner, Active, false)], None, Err(AccessDenied::Forbidden))]
    #[case::activar_perfil_ok(A::ActivatePractitioner, all_person_fields(),
        vec![membership(Practitioner, Pending, false)], Some(ClinicActive), Ok(()))]
    #[case::activar_perfil_sin_colegiatura(A::ActivatePractitioner,
        fields(&[F::GivenName, F::FamilyName, F::Phone, F::Document]),
        vec![membership(Practitioner, Invited, false)], Some(ClinicActive),
        Err(AccessDenied::MissingData { action: A::ActivatePractitioner, fields: vec![F::ProfessionalLicense] }))]
    #[case::activar_perfil_paciente_sin_membresia(A::ActivatePractitioner, fields(&[]), vec![],
        Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::buscar_pacientes_ok(A::SearchPatients, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::buscar_pacientes_clinica_pendiente(A::SearchPatients, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(PendingSetup), Err(AccessDenied::ClinicNotActive))]
    #[case::buscar_pacientes_suspendido(A::SearchPatients, fields(&[]),
        vec![membership(Practitioner, Suspended, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::ver_paciente_ok(A::ViewPatient, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive), Ok(()))]
    // Recepcionista
    #[case::pedir_ingreso_ok(A::RequestMembership,
        fields(&[F::GivenName, F::FamilyName, F::Phone, F::Document]), vec![], Some(ClinicActive), Ok(()))]
    #[case::pedir_ingreso_sin_documento(A::RequestMembership,
        fields(&[F::GivenName, F::FamilyName, F::Phone]), vec![], Some(ClinicActive),
        Err(AccessDenied::MissingData { action: A::RequestMembership, fields: vec![F::Document] }))]
    #[case::pedir_ingreso_clinica_pendiente(A::RequestMembership, all_person_fields(), vec![],
        Some(PendingSetup), Err(AccessDenied::ClinicNotActive))]
    #[case::recepcion_registra_paciente(A::RegisterPatient, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::recepcion_pendiente_no_registra(A::RegisterPatient, fields(&[]),
        vec![membership(Receptionist, Pending, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::admin_registra_paciente(A::RegisterPatient, fields(&[]),
        vec![membership(Admin, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::medico_no_registra_paciente(A::RegisterPatient, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::recepcion_busca_pacientes(A::SearchPatients, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::recepcion_no_abre_historia(A::RecordEncounter, all_person_fields(),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::recepcion_no_invita(A::InviteStaff, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::recepcion_no_aprueba(A::ApproveMember, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::dueno_aprueba(A::ApproveMember, fields(&[]),
        vec![membership(Owner, Active, false)], Some(ClinicActive), Ok(()))]
    // Agenda
    #[case::horario_ok(A::ManageSchedule, fields(&[]),
        vec![membership(Practitioner, Active, true)], Some(ClinicActive), Ok(()))]
    #[case::horario_sin_colegiatura(A::ManageSchedule, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive),
        Err(AccessDenied::MissingData { action: A::ManageSchedule, fields: vec![F::LicenseConfirmed] }))]
    #[case::horario_por_el_dueno(A::ManageSchedule, fields(&[]),
        vec![membership(Owner, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::horario_clinica_pendiente(A::ManageSchedule, fields(&[]),
        vec![membership(Practitioner, Active, true)], Some(PendingSetup), Err(AccessDenied::ClinicNotActive))]
    #[case::agenda_medico(A::ViewClinicAgenda, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::agenda_recepcion(A::ViewClinicAgenda, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::agenda_paciente(A::ViewClinicAgenda, all_person_fields(), vec![], Some(ClinicActive),
        Err(AccessDenied::Forbidden))]
    #[case::reserva_admin(A::BookForPatient, fields(&[]),
        vec![membership(Admin, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::reserva_medico(A::BookForPatient, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::reserva_invitado(A::BookForPatient, fields(&[]),
        vec![membership(Practitioner, Invited, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::reserva_clinica_pendiente(A::BookForPatient, fields(&[]),
        vec![membership(Admin, Active, false)], Some(PendingSetup), Err(AccessDenied::ClinicNotActive))]
    #[case::gestiona_cita_recepcion(A::ManageAppointment, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::llegada_admin(A::CheckIn, fields(&[]),
        vec![membership(Admin, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::llegada_recepcion(A::CheckIn, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::llegada_medico(A::CheckIn, all_person_fields(),
        vec![membership(Practitioner, Active, true)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::vincular_recepcion(A::LinkPatientAccount, fields(&[]),
        vec![membership(Receptionist, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::vincular_paciente(A::LinkPatientAccount, all_person_fields(), vec![], Some(ClinicActive),
        Err(AccessDenied::Forbidden))]
    #[case::desvincular_dueno(A::UnlinkPatientAccount, fields(&[]),
        vec![membership(Owner, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::desvincular_admin(A::UnlinkPatientAccount, fields(&[]),
        vec![membership(Admin, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::receta_sin_confirmar(A::IssuePrescription, all_person_fields(),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive),
        Err(AccessDenied::MissingData { action: A::IssuePrescription, fields: vec![F::LicenseConfirmed] }))]
    #[case::atencion_confirmada(A::RecordEncounter, all_person_fields(),
        vec![membership(Practitioner, Active, true)], Some(ClinicActive), Ok(()))]
    // Dueño o administrador
    #[case::invitar_clinica_pendiente(A::InviteStaff, fields(&[]),
        vec![membership(Owner, Active, false)], Some(PendingSetup), Ok(()))]
    #[case::invitar_admin(A::InviteStaff, fields(&[]),
        vec![membership(Admin, Active, false)], Some(ClinicActive), Ok(()))]
    #[case::invitar_medico(A::InviteStaff, fields(&[]),
        vec![membership(Practitioner, Active, false)], Some(ClinicActive), Err(AccessDenied::Forbidden))]
    #[case::configurar_suspendida(A::ConfigureClinic, fields(&[]),
        vec![membership(Owner, Active, false)], Some(ClinicStatus::Suspended), Err(AccessDenied::ClinicNotActive))]
    #[case::admin_activa_clinica(A::ActivateClinic, all_person_fields(),
        vec![membership(Admin, Active, false)], Some(PendingSetup), Err(AccessDenied::Forbidden))]
    #[case::activar_clinica_sin_ruc(A::ActivateClinic, all_person_fields(),
        vec![membership(Owner, Active, false)], Some(PendingSetup),
        Err(AccessDenied::MissingData { action: A::ActivateClinic, fields: vec![F::ClinicTaxId, F::ClinicLegalName] }))]
    #[case::activar_clinica_ya_activa(A::ActivateClinic, all_person_fields(),
        vec![membership(Owner, Active, false)], Some(ClinicActive), Err(AccessDenied::ClinicNotActive))]
    // Orden de evaluación: sin permiso no se listan datos faltantes
    #[case::orden_permiso_antes_que_datos(A::ActivateClinic, fields(&[]), vec![], Some(PendingSetup),
        Err(AccessDenied::Forbidden))]
    #[case::orden_estado_antes_que_datos(A::SearchPatients, fields(&[]),
        vec![membership(Practitioner, Active, false)], None, Err(AccessDenied::ClinicNotActive))]
    fn matriz_de_acciones(
        #[case] action: ProtectedAction,
        #[case] present: HashSet<F>,
        #[case] memberships: Vec<MembershipView>,
        #[case] clinic_status: Option<ClinicStatus>,
        #[case] expected: Result<(), AccessDenied>,
    ) {
        assert_eq!(
            authorize(action, &ctx(present, memberships), &target(clinic_status)),
            expected
        );
    }

    #[test]
    fn una_cuenta_de_recepcion_no_crea_clinicas() {
        let mut receptionist = ctx(all_person_fields(), vec![]);
        receptionist.preferred_role = Role::Receptionist;
        assert_eq!(
            authorize(A::CreateClinic, &receptionist, &AccessTarget::default()),
            Err(AccessDenied::Forbidden)
        );
    }

    #[test]
    fn la_membresia_de_otra_clinica_no_autoriza() {
        let mut m = membership(Practitioner, Active, false);
        m.organization_id = OTHER_ORG;
        assert_eq!(
            authorize(
                A::SearchPatients,
                &ctx(fields(&[]), vec![m]),
                &target(Some(ClinicActive))
            ),
            Err(AccessDenied::Forbidden)
        );
    }

    #[test]
    fn activar_clinica_con_datos_completos() {
        let mut t = target(Some(PendingSetup));
        t.clinic_fields = fields(&[F::ClinicTaxId, F::ClinicLegalName]);
        assert_eq!(
            authorize(
                A::ActivateClinic,
                &ctx(all_person_fields(), vec![membership(Owner, Active, false)]),
                &t
            ),
            Ok(())
        );
    }

    /// Cada fila de la tabla del catálogo de `design.md`.
    #[rstest::rstest]
    #[case(F::GivenName, &[A::ConfirmAppointment, A::JoinClinic, A::CreateClinic, A::ActivatePractitioner, A::ActivateClinic, A::RequestMembership][..])]
    #[case(F::FamilyName, &[A::ConfirmAppointment, A::ActivatePractitioner, A::ActivateClinic, A::RequestMembership][..])]
    #[case(F::SecondFamilyName, &[][..])]
    #[case(F::Email, &[][..])]
    #[case(F::Phone, &[A::ConfirmAppointment, A::ActivatePractitioner, A::ActivateClinic, A::RequestMembership][..])]
    #[case(F::Document, &[A::ConfirmAppointment, A::ActivatePractitioner, A::ActivateClinic, A::RequestMembership][..])]
    #[case(F::BirthDate, &[A::OpenFirstEncounter][..])]
    #[case(F::Sex, &[A::OpenFirstEncounter][..])]
    #[case(F::Address, &[][..])]
    #[case(F::ProfessionalLicense, &[A::ActivatePractitioner][..])]
    #[case(F::LicenseConfirmed, &[A::RecordEncounter, A::IssuePrescription, A::ManageSchedule][..])]
    #[case(F::Specialty, &[][..])]
    #[case(F::ClinicTaxId, &[A::ActivateClinic][..])]
    #[case(F::ClinicLegalName, &[A::ActivateClinic][..])]
    #[case(F::ClinicTradeName, &[][..])]
    #[case(F::IpressCode, &[][..])]
    fn catalogo_por_dato(#[case] field: F, #[case] actions: &[ProtectedAction]) {
        assert_eq!(spec_of(field).field, field);
        assert_eq!(spec_of(field).required_for, actions);
    }

    #[test]
    fn el_catalogo_no_repite_datos() {
        let unique: HashSet<_> = CATALOG.iter().map(|spec| spec.field).collect();
        assert_eq!(unique.len(), CATALOG.len());
    }

    #[test]
    fn roles_de_un_medico_que_tambien_es_paciente() {
        let c = ctx(fields(&[]), vec![membership(Practitioner, Pending, false)]);
        assert_eq!(c.roles(), vec![Role::Patient, Role::Practitioner]);
    }

    #[rstest::rstest]
    #[case::valido("20100070970", true)]
    #[case::digito_incorrecto("20100070971", false)]
    #[case::corto("2010007097", false)]
    #[case::letras("2010007097A", false)]
    fn ruc_modulo_11(#[case] ruc: &str, #[case] expected: bool) {
        assert_eq!(is_valid_ruc(ruc), expected);
    }
}
