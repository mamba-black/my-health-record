use app_core::domain::access::{ClinicStatus, ProfileField};
use bon::Builder;
use derive_getters::Getters;
use std::collections::HashSet;
use uuid::Uuid;

/// Recurso de dominio que representa la Clínica u Organización Sanitaria (FHIR R4 Organization).
#[derive(Debug, Clone, Getters, Builder, PartialEq, Eq)]
pub struct Organization {
    /// Identificador único de la clínica u organización (UUID v7).
    pub id: Uuid,
    /// Red de clínicas a la que pertenece esta clínica (UUID v7).
    pub network_id: Uuid,
    /// Subdominio exclusivo y plano de la clínica (ej. "san-borja").
    pub subdomain: String,
    /// Razon social o nombre comercial de la clínica.
    pub name: String,
    /// Registro RUC o identificador fiscal (opcional).
    pub tax_id: Option<String>,
    /// Identificador del usuario propietario / administrador principal (UUID v7).
    pub owner_user_id: Uuid,
    /// Estado de la clínica (RFC-004). Solo `Active` puede atender.
    pub status: ClinicStatus,
    /// Razón social.
    pub legal_name: Option<String>,
    /// Nombre comercial.
    pub trade_name: Option<String>,
    /// Código IPRESS (RENIPRESS).
    pub ipress_code: Option<String>,
}

impl Organization {
    /// Crea una nueva organización de salud pendiente de activación (`PendingSetup`).
    ///
    /// # Parámetros
    /// - `id`: Identificador único de la organización (UUID v7).
    /// - `network_id`: Identificador de la red médica a la que pertenece (UUID v7).
    /// - `subdomain`: Subdominio web exclusivo para resolver el inquilino (ej. `"san-borja"`).
    /// - `name`: Nombre comercial o denominación de la clínica.
    /// - `tax_id`: Registro fiscal o RUC de la institución (opcional).
    /// - `owner_user_id`: Identificador del usuario propietario de la clínica (UUID v7).
    pub fn new(
        id: Uuid,
        network_id: Uuid,
        subdomain: String,
        name: String,
        tax_id: Option<String>,
        owner_user_id: Uuid,
    ) -> Self {
        Self {
            id,
            network_id,
            subdomain,
            name,
            tax_id,
            owner_user_id,
            status: ClinicStatus::PendingSetup,
            legal_name: None,
            trade_name: None,
            ipress_code: None,
        }
    }

    /// Datos legales que la clínica ya tiene registrados.
    pub fn present_fields(&self) -> HashSet<ProfileField> {
        let filled =
            |value: &Option<String>| value.as_deref().is_some_and(|v| !v.trim().is_empty());
        [
            (filled(&self.tax_id), ProfileField::ClinicTaxId),
            (filled(&self.legal_name), ProfileField::ClinicLegalName),
            (filled(&self.trade_name), ProfileField::ClinicTradeName),
            (filled(&self.ipress_code), ProfileField::IpressCode),
        ]
        .into_iter()
        .filter_map(|(present, field)| present.then_some(field))
        .collect()
    }
}
