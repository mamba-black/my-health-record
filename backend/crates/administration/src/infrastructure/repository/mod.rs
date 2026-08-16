pub(crate) mod organization_repository_impl;
pub(crate) mod patient_repository_impl;
pub(crate) mod practitioner_repository_impl;

use app_core::domain::error::ClickCareError;
use toasty::stmt::Value;
use uuid::Uuid;

/// Extrae el `Uuid` de la primera columna de la primera fila devuelta por una
/// consulta SQL cruda, o `None` si no hubo filas.
///
/// Toasty devuelve las filas de SQL crudo como `Value::Record` dinámicos, así que
/// el destructurado es el mismo para cualquier `select id from …`. Vive aquí para
/// que los repositorios no lo repitan.
pub(crate) fn first_uuid_column(rows: Vec<Value>) -> Result<Option<Uuid>, ClickCareError> {
    let Some(row) = rows.into_iter().next() else {
        return Ok(None);
    };

    let id_column = row.into_record().fields.into_iter().next().ok_or_else(|| {
        ClickCareError::generic("La consulta no devolvió ninguna columna".to_string())
    })?;

    Uuid::try_from(id_column)
        .map(Some)
        .map_err(|error| ClickCareError::generic(format!("La columna no es un UUID ({error})")))
}
