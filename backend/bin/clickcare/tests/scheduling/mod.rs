//! Pruebas de integración de la agenda y las citas (`scheduling-appointments`).
//!
//! Usan el mismo contenedor `clickcare-test` que el resto de `user_api_test`.

use crate::{TestEnv, test_env};
use rstest::*;

/// Ejecuta una consulta de un solo valor en la base de pruebas.
pub(crate) async fn scalar<T>(env: &TestEnv, sql: &'static str) -> T
where
    T: for<'a> ::postgres::types::FromSql<'a> + Send + 'static,
{
    let conn = env.pg_connection_string.clone();
    tokio::task::spawn_blocking(move || {
        ::postgres::Client::connect(&conn, ::postgres::NoTls)
            .expect("conexión")
            .query_one(sql, &[])
            .expect("consulta")
            .get::<_, T>(0)
    })
    .await
    .expect("tarea bloqueante")
}

/// Tarea 1.2: el contenedor queda migrado hasta `V8__agenda.sql`, con la exclusión de
/// citas que se cruzan dentro de la organización.
#[rstest]
#[tokio::test]
async fn migra_la_agenda_hasta_v8(#[future(awt)] test_env: &'static TestEnv) {
    let version: i32 = scalar(
        test_env,
        "select max(version) from public.refinery_schema_history",
    )
    .await;
    assert!(version >= 8, "versión aplicada: {version}");
    let exclusion: i64 = scalar(
        test_env,
        "select count(*) from pg_constraint where conname = 'ex_appointment_practitioner_overlap'",
    )
    .await;
    assert_eq!(exclusion, 1);
}
