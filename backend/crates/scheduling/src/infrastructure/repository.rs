//! Repositorios de la agenda sobre el esquema `scheduling` (SQL directo con toasty).
//!
//! La exclusión de citas vive en la base (`ex_appointment_practitioner_overlap`): si
//! una escritura la viola, se responde [`SchedulingError::SlotUnavailable`].

use crate::domain::appointment::{Appointment, AppointmentStatus, DocumentCheck};
use crate::domain::calendar::{Instant, format_local_time, parse_local_time};
use crate::domain::error::SchedulingError;
use crate::domain::reminder::{Reminder, ReminderStatus};
use crate::domain::repository::{
    AppointmentRepository, ClaimedReminder, OverlapCheck, ReminderRepository, ScheduleRepository,
};
use crate::domain::schedule::{Schedule, ScheduleException, TimeBlock};
use app_core::domain::error::ClickCareError;
use async_trait::async_trait;
use toasty::Db;
use toasty::db::Executor;
use toasty::stmt::{Type, Value};
use tracing::error;
use uuid::Uuid;

/// Nombre de la restricción de exclusión de citas.
const EXCLUSION: &str = "ex_appointment_practitioner_overlap";

fn db_error(context: &str, error: impl std::fmt::Display) -> SchedulingError {
    let text = error.to_string();
    if text.contains(EXCLUSION) || text.contains("23P01") {
        return SchedulingError::SlotUnavailable;
    }
    error!("{context}: {text}");
    SchedulingError::Unknown(ClickCareError::generic(format!("{context} ({text})")))
}

/// Opcional como texto: el SQL lo convierte con `nullif($n, '')::tipo`. Así no hace
/// falta enviar un `NULL` sin tipo, que el driver no sabe codificar.
fn opt(value: Option<impl ToString>) -> Value {
    Value::String(value.map(|v| v.to_string()).unwrap_or_default())
}

/// Lector de columnas de una fila.
struct Row(std::vec::IntoIter<Value>);

impl Row {
    fn new(value: Value) -> Self {
        Self(value.into_record().fields.into_iter())
    }

    fn next(&mut self) -> Result<Value, SchedulingError> {
        self.0
            .next()
            .ok_or_else(|| SchedulingError::Unknown(ClickCareError::generic("Falta una columna")))
    }

    fn uuid(&mut self) -> Result<Uuid, SchedulingError> {
        Uuid::try_from(self.next()?).map_err(|e| conversion(&e))
    }

    fn opt_uuid(&mut self) -> Result<Option<Uuid>, SchedulingError> {
        match self.next()? {
            Value::Null => Ok(None),
            value => Uuid::try_from(value).map(Some).map_err(|e| conversion(&e)),
        }
    }

    fn string(&mut self) -> Result<String, SchedulingError> {
        String::try_from(self.next()?).map_err(|e| conversion(&e))
    }

    fn opt_string(&mut self) -> Result<Option<String>, SchedulingError> {
        match self.next()? {
            Value::Null => Ok(None),
            value => String::try_from(value)
                .map(Some)
                .map_err(|e| conversion(&e)),
        }
    }

    fn instant(&mut self) -> Result<Instant, SchedulingError> {
        Instant::try_from(self.next()?).map_err(|e| conversion(&e))
    }

    fn opt_instant(&mut self) -> Result<Option<Instant>, SchedulingError> {
        match self.next()? {
            Value::Null => Ok(None),
            value => Instant::try_from(value)
                .map(Some)
                .map_err(|e| conversion(&e)),
        }
    }

    fn i64(&mut self) -> Result<i64, SchedulingError> {
        match self.next()? {
            Value::I16(v) => Ok(i64::from(v)),
            Value::I32(v) => Ok(i64::from(v)),
            Value::I64(v) => Ok(v),
            other => Err(conversion(&format!("se esperaba un entero: {other:?}"))),
        }
    }
}

fn conversion(error: &dyn std::fmt::Display) -> SchedulingError {
    SchedulingError::Unknown(ClickCareError::generic(format!(
        "Columna con un tipo inesperado ({error})"
    )))
}

async fn query(
    executor: &mut dyn Executor,
    sql: &str,
    binds: Vec<Value>,
    types: Vec<Type>,
) -> Result<Vec<Row>, SchedulingError> {
    let mut q = toasty::sql::query(sql.to_string());
    for value in binds {
        q = q.bind(value);
    }
    q.column_types(types)
        .exec(executor)
        .await
        .map(|rows| rows.into_iter().map(Row::new).collect())
        .map_err(|e| db_error("Error al consultar la agenda", e))
}

async fn execute(
    executor: &mut dyn Executor,
    sql: &str,
    binds: Vec<Value>,
) -> Result<u64, SchedulingError> {
    let mut statement = toasty::sql::statement(sql.to_string());
    for value in binds {
        statement = statement.bind(value);
    }
    statement
        .exec(executor)
        .await
        .map_err(|e| db_error("Error al escribir en la agenda", e))
}

// ─── Horarios ───────────────────────────────────────────────────────────────

pub struct PgScheduleRepository {
    pub(crate) db: Db,
}

const SCHEDULE_COLUMNS: &str = "select id, network_id, organization_id, practitioner_user_id, slot_minutes from scheduling.schedule";

fn schedule_types() -> Vec<Type> {
    vec![Type::Uuid, Type::Uuid, Type::Uuid, Type::Uuid, Type::I16]
}

async fn load_schedules(
    executor: &mut dyn Executor,
    filter: &str,
    binds: Vec<Value>,
) -> Result<Vec<Schedule>, SchedulingError> {
    let rows = query(
        executor,
        &format!("{SCHEDULE_COLUMNS} {filter}"),
        binds,
        schedule_types(),
    )
    .await?;
    let mut schedules = Vec::with_capacity(rows.len());
    for mut row in rows {
        let id = row.uuid()?;
        let network_id = row.uuid()?;
        let organization_id = row.uuid()?;
        let practitioner_user_id = row.uuid()?;
        let slot_minutes = u16::try_from(row.i64()?).unwrap_or(30);
        let blocks = query(
            executor,
            "select weekday, to_char(start_time, 'HH24:MI'), to_char(end_time, 'HH24:MI') \
             from scheduling.schedule_block where schedule_id = $1 order by weekday, start_time",
            vec![id.into()],
            vec![Type::I16, Type::String, Type::String],
        )
        .await?
        .into_iter()
        .map(|mut block| {
            let weekday = u8::try_from(block.i64()?).unwrap_or(1);
            let start = parse_local_time(&block.string()?);
            let end = parse_local_time(&block.string()?);
            match (start, end) {
                (Some(start), Some(end)) => Ok(TimeBlock {
                    weekday,
                    start,
                    end,
                }),
                _ => Err(conversion(&"hora inválida en schedule_block")),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
        schedules.push(Schedule {
            id,
            network_id,
            organization_id,
            practitioner_user_id,
            slot_minutes,
            blocks,
        });
    }
    Ok(schedules)
}

#[async_trait]
impl ScheduleRepository for PgScheduleRepository {
    async fn find(
        &self,
        organization_id: &Uuid,
        practitioner_user_id: &Uuid,
    ) -> Result<Option<Schedule>, SchedulingError> {
        Ok(load_schedules(
            &mut self.db.clone(),
            "where organization_id = $1 and practitioner_user_id = $2",
            vec![(*organization_id).into(), (*practitioner_user_id).into()],
        )
        .await?
        .pop())
    }

    async fn by_clinic(&self, organization_id: &Uuid) -> Result<Vec<Schedule>, SchedulingError> {
        load_schedules(
            &mut self.db.clone(),
            "where organization_id = $1",
            vec![(*organization_id).into()],
        )
        .await
    }

    async fn save(
        &self,
        schedule: &Schedule,
        check: OverlapCheck<'_>,
    ) -> Result<(), SchedulingError> {
        let mut db = self.db.clone();
        let mut tx = db
            .transaction()
            .await
            .map_err(|e| db_error("Error al abrir la transacción del horario", e))?;
        // Un solo guardado a la vez por médico dentro de su red: así dos clínicas no
        // aceptan a la vez horarios que se cruzan.
        query(
            &mut tx,
            "select 1 from (select pg_advisory_xact_lock(hashtextextended($1, 0))) l",
            vec![format!("{}:{}", schedule.network_id, schedule.practitioner_user_id).into()],
            vec![Type::I32],
        )
        .await?;
        let others = load_schedules(
            &mut tx,
            "where network_id = $1 and practitioner_user_id = $2 and organization_id <> $3",
            vec![
                schedule.network_id.into(),
                schedule.practitioner_user_id.into(),
                schedule.organization_id.into(),
            ],
        )
        .await?;
        check(&others)?;
        let id = query(
            &mut tx,
            "insert into scheduling.schedule \
               (id, network_id, organization_id, practitioner_user_id, slot_minutes) \
             values ($1, $2, $3, $4, $5) \
             on conflict (organization_id, practitioner_user_id) \
             do update set slot_minutes = excluded.slot_minutes, updated_at = now() \
             returning id",
            vec![
                schedule.id.into(),
                schedule.network_id.into(),
                schedule.organization_id.into(),
                schedule.practitioner_user_id.into(),
                Value::I16(i16::try_from(schedule.slot_minutes).unwrap_or(30)),
            ],
            vec![Type::Uuid],
        )
        .await?
        .pop()
        .ok_or_else(|| conversion(&"sin id de horario"))?
        .uuid()?;
        execute(
            &mut tx,
            "delete from scheduling.schedule_block where schedule_id = $1",
            vec![id.into()],
        )
        .await?;
        for block in &schedule.blocks {
            execute(
                &mut tx,
                "insert into scheduling.schedule_block (schedule_id, weekday, start_time, end_time) \
                 values ($1, $2, $3::time, $4::time)",
                vec![
                    id.into(),
                    Value::I16(i16::from(block.weekday)),
                    format_local_time(block.start).into(),
                    format_local_time(block.end).into(),
                ],
            )
            .await?;
        }
        tx.commit()
            .await
            .map_err(|e| db_error("Error al guardar el horario", e))
    }

    async fn exceptions(
        &self,
        schedule_id: &Uuid,
    ) -> Result<Vec<ScheduleException>, SchedulingError> {
        query(
            &mut self.db.clone(),
            "select id, schedule_id, lower(period), upper(period), reason \
             from scheduling.schedule_exception where schedule_id = $1 order by lower(period)",
            vec![(*schedule_id).into()],
            vec![
                Type::Uuid,
                Type::Uuid,
                Type::Timestamp,
                Type::Timestamp,
                Type::String,
            ],
        )
        .await?
        .into_iter()
        .map(|mut row| {
            Ok(ScheduleException {
                id: row.uuid()?,
                schedule_id: row.uuid()?,
                start: row.instant()?,
                end: row.instant()?,
                reason: row.opt_string()?,
            })
        })
        .collect()
    }

    async fn add_exception(&self, exception: &ScheduleException) -> Result<(), SchedulingError> {
        execute(
            &mut self.db.clone(),
            "insert into scheduling.schedule_exception (id, schedule_id, period, reason) \
             values ($1, $2, tstzrange($3, $4, '[)'), nullif($5, ''))",
            vec![
                exception.id.into(),
                exception.schedule_id.into(),
                exception.start.into(),
                exception.end.into(),
                opt(exception.reason.clone()),
            ],
        )
        .await
        .map(|_| ())
    }

    async fn remove_exception(
        &self,
        schedule_id: &Uuid,
        exception_id: &Uuid,
    ) -> Result<bool, SchedulingError> {
        Ok(execute(
            &mut self.db.clone(),
            "delete from scheduling.schedule_exception where id = $1 and schedule_id = $2",
            vec![(*exception_id).into(), (*schedule_id).into()],
        )
        .await?
            > 0)
    }
}

// ─── Citas ──────────────────────────────────────────────────────────────────

pub struct PgAppointmentRepository {
    pub(crate) db: Db,
}

const APPOINTMENT_COLUMNS: &str = "select id, network_id, organization_id, practitioner_user_id, \
     patient_id, lower(period), upper(period), status, version, booked_by, cancelled_by, \
     cancelled_at, arrived_by, arrived_at, document_check, noshow_by, noshow_at \
     from scheduling.appointment";

fn appointment_types() -> Vec<Type> {
    vec![
        Type::Uuid,
        Type::Uuid,
        Type::Uuid,
        Type::Uuid,
        Type::Uuid,
        Type::Timestamp,
        Type::Timestamp,
        Type::String,
        Type::I32,
        Type::Uuid,
        Type::Uuid,
        Type::Timestamp,
        Type::Uuid,
        Type::Timestamp,
        Type::String,
        Type::Uuid,
        Type::Timestamp,
    ]
}

fn map_appointment(mut row: Row) -> Result<Appointment, SchedulingError> {
    Ok(Appointment {
        id: row.uuid()?,
        network_id: row.uuid()?,
        organization_id: row.uuid()?,
        practitioner_user_id: row.uuid()?,
        patient_id: row.uuid()?,
        start: row.instant()?,
        end: row.instant()?,
        status: {
            let code = row.string()?;
            AppointmentStatus::from_code(&code)
                .ok_or_else(|| conversion(&format!("estado de cita desconocido: {code}")))?
        },
        version: i32::try_from(row.i64()?).unwrap_or(1),
        booked_by: row.uuid()?,
        cancelled_by: row.opt_uuid()?,
        cancelled_at: row.opt_instant()?,
        arrived_by: row.opt_uuid()?,
        arrived_at: row.opt_instant()?,
        document_check: row
            .opt_string()?
            .and_then(|code| DocumentCheck::from_code(&code)),
        noshow_by: row.opt_uuid()?,
        noshow_at: row.opt_instant()?,
    })
}

async fn load_appointments(
    db: &Db,
    filter: &str,
    binds: Vec<Value>,
) -> Result<Vec<Appointment>, SchedulingError> {
    query(
        &mut db.clone(),
        &format!("{APPOINTMENT_COLUMNS} {filter}"),
        binds,
        appointment_types(),
    )
    .await?
    .into_iter()
    .map(map_appointment)
    .collect()
}

#[async_trait]
impl AppointmentRepository for PgAppointmentRepository {
    async fn insert(
        &self,
        appointment: &Appointment,
        reminder: &Reminder,
    ) -> Result<(), SchedulingError> {
        // Una sola sentencia: la cita y su recordatorio se guardan juntos o nada.
        execute(
            &mut self.db.clone(),
            "with a as ( \
               insert into scheduling.appointment \
                 (id, network_id, organization_id, practitioner_user_id, patient_id, period, \
                  status, version, booked_by) \
               values ($1, $2, $3, $4, $5, tstzrange($6, $7, '[)'), $8, 1, $9) \
               returning id) \
             insert into scheduling.reminder (id, network_id, appointment_id, due_at, status) \
             select $10, $2, a.id, $11, $12 from a",
            vec![
                appointment.id.into(),
                appointment.network_id.into(),
                appointment.organization_id.into(),
                appointment.practitioner_user_id.into(),
                appointment.patient_id.into(),
                appointment.start.into(),
                appointment.end.into(),
                appointment.status.code().to_string().into(),
                appointment.booked_by.into(),
                reminder.id.into(),
                reminder.due_at.into(),
                reminder.status.code().to_string().into(),
            ],
        )
        .await
        .map(|_| ())
    }

    async fn find(
        &self,
        network_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<Appointment>, SchedulingError> {
        Ok(load_appointments(
            &self.db,
            "where network_id = $1 and id = $2",
            vec![(*network_id).into(), (*id).into()],
        )
        .await?
        .pop())
    }

    async fn update(&self, a: &Appointment) -> Result<(), SchedulingError> {
        let changed = execute(
            &mut self.db.clone(),
            "update scheduling.appointment set period = tstzrange($3, $4, '[)'), status = $5, \
               version = version + 1, cancelled_by = nullif($6, '')::uuid, \
               cancelled_at = nullif($7, '')::timestamptz, arrived_by = nullif($8, '')::uuid, \
               arrived_at = nullif($9, '')::timestamptz, document_check = nullif($10, ''), \
               noshow_by = nullif($11, '')::uuid, noshow_at = nullif($12, '')::timestamptz \
             where network_id = $1 and id = $2 and version = $13",
            vec![
                a.network_id.into(),
                a.id.into(),
                a.start.into(),
                a.end.into(),
                a.status.code().to_string().into(),
                opt(a.cancelled_by),
                opt(a.cancelled_at),
                opt(a.arrived_by),
                opt(a.arrived_at),
                opt(a.document_check.map(|check| check.code())),
                opt(a.noshow_by),
                opt(a.noshow_at),
                Value::I32(a.version),
            ],
        )
        .await?;
        if changed == 0 {
            return Err(SchedulingError::Conflict);
        }
        Ok(())
    }

    async fn busy(
        &self,
        network_id: &Uuid,
        practitioner_user_id: &Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<(Instant, Instant)>, SchedulingError> {
        query(
            &mut self.db.clone(),
            "select lower(period), upper(period) from scheduling.appointment \
             where network_id = $1 and practitioner_user_id = $2 \
               and status in ('BOOKED', 'ARRIVED') and period && tstzrange($3, $4, '[)')",
            vec![
                (*network_id).into(),
                (*practitioner_user_id).into(),
                from.into(),
                to.into(),
            ],
            vec![Type::Timestamp, Type::Timestamp],
        )
        .await?
        .into_iter()
        .map(|mut row| Ok((row.instant()?, row.instant()?)))
        .collect()
    }

    async fn count_active(
        &self,
        organization_id: &Uuid,
        practitioner_user_id: &Uuid,
        from: Instant,
        to: Instant,
    ) -> Result<usize, SchedulingError> {
        let count = query(
            &mut self.db.clone(),
            "select count(*) from scheduling.appointment \
             where organization_id = $1 and practitioner_user_id = $2 \
               and status in ('BOOKED', 'ARRIVED') and period && tstzrange($3, $4, '[)')",
            vec![
                (*organization_id).into(),
                (*practitioner_user_id).into(),
                from.into(),
                to.into(),
            ],
            vec![Type::I64],
        )
        .await?
        .pop()
        .map(|mut row| row.i64())
        .transpose()?
        .unwrap_or(0);
        Ok(usize::try_from(count).unwrap_or(0))
    }

    async fn by_patients(
        &self,
        network_id: &Uuid,
        patient_ids: &[Uuid],
    ) -> Result<Vec<Appointment>, SchedulingError> {
        if patient_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids = patient_ids
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(",");
        // Los ids son `Uuid` ya validados: no hay texto del usuario en la consulta.
        load_appointments(
            &self.db,
            &format!("where network_id = $1 and patient_id in ({ids}) order by lower(period)"),
            vec![(*network_id).into()],
        )
        .await
    }

    async fn by_clinic(
        &self,
        organization_id: &Uuid,
        practitioner_user_id: Option<&Uuid>,
        from: Instant,
        to: Instant,
    ) -> Result<Vec<Appointment>, SchedulingError> {
        let mut binds: Vec<Value> = vec![(*organization_id).into(), from.into(), to.into()];
        let mut filter =
            "where organization_id = $1 and period && tstzrange($2, $3, '[)')".to_string();
        if let Some(practitioner) = practitioner_user_id {
            binds.push((*practitioner).into());
            filter.push_str(" and practitioner_user_id = $4");
        }
        filter.push_str(" order by lower(period)");
        load_appointments(&self.db, &filter, binds).await
    }
}

// ─── Recordatorios ──────────────────────────────────────────────────────────

pub struct PgReminderRepository {
    pub(crate) db: Db,
}

const REMINDER_COLUMNS: &str =
    "select id, network_id, appointment_id, due_at, status, attempts from scheduling.reminder";

fn reminder_types() -> Vec<Type> {
    vec![
        Type::Uuid,
        Type::Uuid,
        Type::Uuid,
        Type::Timestamp,
        Type::String,
        Type::I16,
    ]
}

fn map_reminder(mut row: Row) -> Result<Reminder, SchedulingError> {
    Ok(Reminder {
        id: row.uuid()?,
        network_id: row.uuid()?,
        appointment_id: row.uuid()?,
        due_at: row.instant()?,
        status: {
            let code = row.string()?;
            ReminderStatus::from_code(&code)
                .ok_or_else(|| conversion(&format!("estado de recordatorio desconocido: {code}")))?
        },
        attempts: i16::try_from(row.i64()?).unwrap_or(0),
    })
}

#[async_trait]
impl ReminderRepository for PgReminderRepository {
    async fn of_appointment(
        &self,
        appointment_id: &Uuid,
    ) -> Result<Option<Reminder>, SchedulingError> {
        query(
            &mut self.db.clone(),
            &format!("{REMINDER_COLUMNS} where appointment_id = $1"),
            vec![(*appointment_id).into()],
            reminder_types(),
        )
        .await?
        .pop()
        .map(map_reminder)
        .transpose()
    }

    async fn reset(
        &self,
        appointment_id: &Uuid,
        due_at: Instant,
        status: ReminderStatus,
    ) -> Result<Option<Reminder>, SchedulingError> {
        query(
            &mut self.db.clone(),
            "update scheduling.reminder set due_at = $2, status = $3, attempts = 0, \
               claimed_at = null, last_error = null \
             where appointment_id = $1 and status <> 'SENT' \
             returning id, network_id, appointment_id, due_at, status, attempts",
            vec![
                (*appointment_id).into(),
                due_at.into(),
                status.code().to_string().into(),
            ],
            reminder_types(),
        )
        .await?
        .pop()
        .map(map_reminder)
        .transpose()
    }

    async fn claim(
        &self,
        id: &Uuid,
        due_at: Instant,
        now: Instant,
    ) -> Result<Option<ClaimedReminder>, SchedulingError> {
        let reminder = query(
            &mut self.db.clone(),
            "update scheduling.reminder \
               set status = 'SENDING', attempts = attempts + 1, claimed_at = $3 \
             where id = $1 and status = 'PENDING' and due_at = $2 \
             returning id, network_id, appointment_id, due_at, status, attempts",
            vec![(*id).into(), due_at.into(), now.into()],
            reminder_types(),
        )
        .await?
        .pop()
        .map(map_reminder)
        .transpose()?;
        let Some(reminder) = reminder else {
            return Ok(None);
        };
        let appointment = load_appointments(
            &self.db,
            "where network_id = $1 and id = $2",
            vec![reminder.network_id.into(), reminder.appointment_id.into()],
        )
        .await?
        .pop()
        .ok_or_else(|| SchedulingError::NotFound("La cita del recordatorio no existe".into()))?;
        Ok(Some(ClaimedReminder {
            reminder,
            appointment,
        }))
    }

    async fn finish(
        &self,
        id: &Uuid,
        status: ReminderStatus,
        due_at: Instant,
        error: Option<String>,
        now: Instant,
    ) -> Result<(), SchedulingError> {
        execute(
            &mut self.db.clone(),
            "update scheduling.reminder set status = $2, due_at = $3, last_error = nullif($4, ''), \
               sent_at = case when $2 = 'SENT' then $5 else sent_at end, claimed_at = null \
             where id = $1 and status = 'SENDING'",
            vec![
                (*id).into(),
                status.code().to_string().into(),
                due_at.into(),
                opt(error),
                now.into(),
            ],
        )
        .await
        .map(|_| ())
    }

    async fn release_stale(&self, stale_before: Instant) -> Result<usize, SchedulingError> {
        let released = execute(
            &mut self.db.clone(),
            "update scheduling.reminder set status = 'PENDING', claimed_at = null \
             where status = 'SENDING' and claimed_at < $1",
            vec![stale_before.into()],
        )
        .await?;
        Ok(usize::try_from(released).unwrap_or(0))
    }

    async fn pending_due(&self, now: Instant) -> Result<Vec<Reminder>, SchedulingError> {
        query(
            &mut self.db.clone(),
            &format!(
                "{REMINDER_COLUMNS} where status = 'PENDING' and due_at <= $1 order by due_at"
            ),
            vec![now.into()],
            reminder_types(),
        )
        .await?
        .into_iter()
        .map(map_reminder)
        .collect()
    }
}
