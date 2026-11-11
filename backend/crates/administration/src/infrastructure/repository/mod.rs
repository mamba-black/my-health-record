pub(crate) mod clinical_network_repository_impl;
pub(crate) mod membership_repository_impl;
pub(crate) mod organization_repository_impl;
pub(crate) mod patient_link_repository_impl;
pub(crate) mod patient_repository_impl;
pub(crate) mod practitioner_repository_impl;

use app_core::domain::error::ClickCareError;
use toasty::stmt::Value;
use tracing::error;
use uuid::Uuid;

/// Convierte un error de toasty en `ClickCareError`, dejando el detalle en el log.
pub(crate) fn db_error(context: String) -> impl FnOnce(toasty::Error) -> ClickCareError {
    move |error| {
        error!("{context}: {error}");
        ClickCareError::generic(format!("{context} ({error})"))
    }
}

/// Lee la columna `uuid` de cada fila de un `select <uuid> from …` en SQL crudo.
///
/// Las consultas que toasty no puede expresar (joins, filtros dentro del JSON de la
/// `Person`) solo devuelven ids; las filas completas se cargan después con el Model.
pub(crate) fn uuid_column(rows: Vec<Value>) -> Result<Vec<Uuid>, ClickCareError> {
    rows.into_iter()
        .map(|row| {
            let value = row.into_record().fields.into_iter().next().ok_or_else(|| {
                ClickCareError::generic("La consulta no devolvió ninguna columna".to_string())
            })?;
            Uuid::try_from(value).map_err(|error| {
                ClickCareError::generic(format!("La columna no es un UUID ({error})"))
            })
        })
        .collect()
}
