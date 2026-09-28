//! SQLite, WAL, foreign keys, one writer. Every decision is one short
//! transaction that appends its audit event. The store knows nothing of
//! processes.

mod schema;

use crate::api::Fail;
use rusqlite::{Connection, Row, Transaction, params};

pub mod attempts;
pub mod checks;
pub mod executions;
pub mod tickets;
use serde_json::{Value, json};
use std::fs::File;
use std::path::Path;

pub struct Store {
    conn: Connection,
    /// Held for the daemon's life: the single-writer proof.
    _lock: nix::fcntl::Flock<File>,
}

/// What an audit event names.
#[derive(Default, Clone, Copy)]
pub struct Target {
    pub ticket: Option<i64>,
    pub attempt: Option<i64>,
    pub execution: Option<i64>,
    pub attention: Option<i64>,
}

impl Store {
    pub fn open(dir: &Path) -> Result<Store, String> {
        std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
        let lock = File::create(dir.join("store.lock")).map_err(|error| error.to_string())?;
        let lock = nix::fcntl::Flock::lock(lock, nix::fcntl::FlockArg::LockExclusiveNonblock)
            .map_err(|_| {
                format!(
                    "{} is held by another daemon",
                    dir.join("store.lock").display()
                )
            })?;
        let conn = Connection::open(dir.join("store.sqlite")).map_err(|error| error.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| error.to_string())?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| error.to_string())?;
        conn.pragma_update(None, "synchronous", "FULL")
            .map_err(|error| error.to_string())?;
        let mut store = Store { conn, _lock: lock };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<(), String> {
        let version: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| error.to_string())?;
        let version = version as usize;
        if version > schema::MIGRATIONS.len() {
            return Err(format!(
                "the store is at schema {version}, newer than this Yard's {}",
                schema::MIGRATIONS.len()
            ));
        }
        for (index, migration) in schema::MIGRATIONS.iter().enumerate().skip(version) {
            let tx = self.conn.transaction().map_err(|error| error.to_string())?;
            tx.execute_batch(migration)
                .map_err(|error| error.to_string())?;
            tx.pragma_update(None, "user_version", (index + 1) as i64)
                .map_err(|error| error.to_string())?;
            tx.commit().map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// One transaction. Rolled back unless `f` succeeds.
    pub fn tx<T>(&mut self, f: impl FnOnce(&Transaction) -> Result<T, Fail>) -> Result<T, Fail> {
        let tx = self.conn.transaction()?;
        let value = f(&tx)?;
        tx.commit()?;
        Ok(value)
    }

    pub fn read(&self) -> &Connection {
        &self.conn
    }
}

/// Append one decision's event. Call it inside the decision's transaction.
pub fn audit(
    tx: &Connection,
    event: &str,
    target: Target,
    text: Option<&str>,
    data: Value,
) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO audit (at, event, ticket, attempt, execution, attention, text, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            now(),
            event,
            target.ticket,
            target.attempt,
            target.execution,
            target.attention,
            text,
            data.to_string()
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

/// Every row of one query.
pub fn all<T>(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
    row: impl FnMut(&Row) -> rusqlite::Result<T>,
) -> Result<Vec<T>, Fail> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map(params, row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Audit events after `seq`, oldest first, at most `limit`.
pub fn events_since(conn: &Connection, seq: i64, limit: i64) -> Result<Vec<Value>, Fail> {
    all(
        conn,
        "SELECT seq, at, event, ticket, attempt, execution, attention, text, data
         FROM audit WHERE seq > ?1 ORDER BY seq LIMIT ?2",
        params![seq, limit],
        |row| {
            let data: String = row.get(8)?;
            Ok(json!({
                "seq": row.get::<_, i64>(0)?,
                "at": row.get::<_, String>(1)?,
                "event": row.get::<_, String>(2)?,
                "ticket": row.get::<_, Option<i64>>(3)?.map(ticket_name),
                "attempt": row.get::<_, Option<i64>>(4)?,
                "execution": row.get::<_, Option<i64>>(5)?,
                "attention": row.get::<_, Option<i64>>(6)?,
                "text": row.get::<_, Option<String>>(7)?,
                "data": serde_json::from_str::<Value>(&data).unwrap_or(Value::Null),
            }))
        },
    )
}

pub fn last_seq(conn: &Connection) -> Result<i64, Fail> {
    Ok(
        conn.query_row("SELECT COALESCE(MAX(seq), 0) FROM audit", [], |row| {
            row.get(0)
        })?,
    )
}

pub fn ticket_name(id: i64) -> String {
    format!("Y-{id}")
}

/// Parse `Y-<n>` or `<n>`.
pub fn ticket_id(name: &str) -> Option<i64> {
    name.strip_prefix("Y-")
        .unwrap_or(name)
        .parse()
        .ok()
        .filter(|id| *id > 0)
}

/// UTC now, RFC 3339 with milliseconds.
pub fn now() -> String {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = elapsed.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60,
        elapsed.subsec_millis()
    )
}

/// Milliseconds since the epoch, for clocks.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}
