use super::{Target, audit, now, ticket_name};
use crate::api::Fail;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct Attempt {
    pub id: i64,
    pub ticket: i64,
    pub workflow: String,
    /// The implementer frozen at admission: `{name, harness, provider, model, effort}`.
    pub implementer: Value,
    pub branch: String,
    pub base: String,
    pub head: Option<String>,
    /// The latest target handed to the attempt for a repair.
    pub target: Option<String>,
    pub state: String,
    pub outcome: Option<String>,
    pub lane: bool,
    pub lane_since: Option<i64>,
    pub work_ms: i64,
    /// What the next implementer execution is for: `{reason, detail}`.
    pub next: Option<Value>,
    pub nudge: Option<String>,
    pub session_execution: Option<i64>,
    pub session_count: i64,
    pub rounds: i64,
    pub extra_rounds: i64,
    pub landing_reds: i64,
}

const COLUMNS: &str =
    "id, ticket, workflow, implementer, branch, base, head, target, state, outcome,
    lane, lane_since, work_ms, next, nudge, session_execution, session_count, rounds, extra_rounds,
    landing_reds";

fn row(row: &Row) -> rusqlite::Result<Attempt> {
    let implementer: String = row.get(3)?;
    let next: Option<String> = row.get(13)?;
    let lane_since: Option<String> = row.get(11)?;
    Ok(Attempt {
        id: row.get(0)?,
        ticket: row.get(1)?,
        workflow: row.get(2)?,
        implementer: serde_json::from_str(&implementer).unwrap_or(Value::Null),
        branch: row.get(4)?,
        base: row.get(5)?,
        head: row.get(6)?,
        target: row.get(7)?,
        state: row.get(8)?,
        outcome: row.get(9)?,
        lane: row.get(10)?,
        lane_since: lane_since.and_then(|value| value.parse().ok()),
        work_ms: row.get(12)?,
        next: next.and_then(|text| serde_json::from_str(&text).ok()),
        nudge: row.get(14)?,
        session_execution: row.get(15)?,
        session_count: row.get(16)?,
        rounds: row.get(17)?,
        extra_rounds: row.get(18)?,
        landing_reds: row.get(19)?,
    })
}

impl Attempt {
    pub fn to_json(&self) -> Value {
        json!({
            "attempt": self.id,
            "ticket": ticket_name(self.ticket),
            "workflow": self.workflow,
            "implementer": self.implementer,
            "branch": self.branch,
            "base": self.base,
            "head": self.head,
            "state": self.state,
            "outcome": self.outcome,
            "lane": self.lane,
            "next": self.next,
            "nudge": self.nudge,
            "rounds": self.rounds,
            "landing_reds": self.landing_reds,
        })
    }

    /// The candidate, once there is one.
    pub fn candidate(&self) -> Option<(&str, &str)> {
        self.head.as_deref().map(|head| (self.base.as_str(), head))
    }
}

pub fn get(conn: &Connection, id: i64) -> Result<Attempt, Fail> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM attempt WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| Fail::not_found(format!("attempt {id} does not exist")))
}

pub fn live_for(conn: &Connection, ticket: i64) -> Result<Option<Attempt>, Fail> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM attempt WHERE ticket = ?1 AND state = 'live'"),
            [ticket],
            row,
        )
        .optional()?)
}

pub fn latest_for(conn: &Connection, ticket: i64) -> Result<Option<Attempt>, Fail> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM attempt WHERE ticket = ?1 ORDER BY id DESC LIMIT 1"),
            [ticket],
            row,
        )
        .optional()?)
}

pub fn live(conn: &Connection) -> Result<Vec<Attempt>, Fail> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM attempt WHERE state = 'live' ORDER BY id"
    ))?;
    let rows = statement.query_map([], row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Ended attempts whose files are not yet removed.
pub fn uncleaned(conn: &Connection) -> Result<Vec<Attempt>, Fail> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM attempt WHERE state = 'ended' AND cleaned = 0 ORDER BY id"
    ))?;
    let rows = statement.query_map([], row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Attempts holding a lane in this project.
pub fn lanes_held(conn: &Connection) -> Result<i64, Fail> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM attempt WHERE state = 'live' AND lane = 1",
        [],
        |row| row.get(0),
    )?)
}

pub fn insert(
    tx: &Connection,
    ticket: i64,
    workflow: &str,
    implementer: &Value,
    base: &str,
) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO attempt (ticket, workflow, implementer, branch, base, state, lane, lane_since, created_at)
         VALUES (?1, ?2, ?3, '', ?4, 'live', 1, ?5, ?6)",
        params![
            ticket,
            workflow,
            implementer.to_string(),
            base,
            super::now_ms().to_string(),
            now()
        ],
    )?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE attempt SET branch = ?2 WHERE id = ?1",
        params![id, format!("yard/{}/{id}", ticket_name(ticket))],
    )?;
    Ok(id)
}

pub fn set_next(tx: &Connection, attempt: i64, next: Option<&Value>) -> Result<(), Fail> {
    tx.execute(
        "UPDATE attempt SET next = ?2 WHERE id = ?1",
        params![attempt, next.map(Value::to_string)],
    )?;
    Ok(())
}

/// Take or give back a lane. Taking starts the total-work clock; giving it
/// back banks the time.
pub fn set_lane(tx: &Connection, attempt: i64, held: bool) -> Result<(), Fail> {
    let current = get(tx, attempt)?;
    if current.lane == held {
        return Ok(());
    }
    let now = super::now_ms();
    if held {
        tx.execute(
            "UPDATE attempt SET lane = 1, lane_since = ?2 WHERE id = ?1",
            params![attempt, now.to_string()],
        )?;
    } else {
        let spent = current.lane_since.map(|since| now - since).unwrap_or(0);
        tx.execute(
            "UPDATE attempt SET lane = 0, lane_since = NULL, work_ms = work_ms + ?2 WHERE id = ?1",
            params![attempt, spent.max(0)],
        )?;
    }
    Ok(())
}

pub fn end(tx: &Connection, attempt: i64, outcome: &str) -> Result<(), Fail> {
    set_lane(tx, attempt, false)?;
    tx.execute(
        "UPDATE attempt SET state = 'ended', outcome = ?2, ended_at = ?3, next = NULL WHERE id = ?1",
        params![attempt, outcome, now()],
    )?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Attention {
    pub id: i64,
    pub kind: String,
    pub reason: String,
    pub ticket: Option<i64>,
    pub attempt: Option<i64>,
    pub execution: Option<i64>,
    pub payload: Value,
    pub state: String,
}

const ATTENTION_COLUMNS: &str = "id, kind, reason, ticket, attempt, execution, payload, state";

fn attention_row(row: &Row) -> rusqlite::Result<Attention> {
    let payload: String = row.get(6)?;
    Ok(Attention {
        id: row.get(0)?,
        kind: row.get(1)?,
        reason: row.get(2)?,
        ticket: row.get(3)?,
        attempt: row.get(4)?,
        execution: row.get(5)?,
        payload: serde_json::from_str(&payload).unwrap_or(Value::Null),
        state: row.get(7)?,
    })
}

impl Attention {
    /// The item with the commands that answer it.
    pub fn to_json(&self) -> Value {
        let ticket = self.ticket.map(ticket_name);
        let exits: Vec<&str> = match (self.kind.as_str(), self.reason.as_str()) {
            ("approval", _) => vec!["approve", "reject", "abandon"],
            ("proposal", _) => vec!["accept", "reject"],
            ("stopped", "timeout" | "limit") => vec!["nudge", "abandon"],
            ("red", _) if self.attempt.is_none() => vec!["start"],
            _ => vec!["start", "nudge", "abandon"],
        };
        json!({
            "attention": self.id,
            "kind": self.kind,
            "reason": self.reason,
            "ticket": ticket,
            "attempt": self.attempt,
            "execution": self.execution,
            "payload": self.payload,
            "state": self.state,
            "exits": exits,
        })
    }
}

pub fn attention(conn: &Connection, id: i64) -> Result<Attention, Fail> {
    conn.query_row(
        &format!("SELECT {ATTENTION_COLUMNS} FROM attention WHERE id = ?1"),
        [id],
        attention_row,
    )
    .optional()?
    .ok_or_else(|| Fail::not_found(format!("attention {id} does not exist")))
}

pub fn open_attention(conn: &Connection) -> Result<Vec<Attention>, Fail> {
    let mut statement = conn.prepare(&format!(
        "SELECT {ATTENTION_COLUMNS} FROM attention WHERE state = 'open' ORDER BY id"
    ))?;
    let rows = statement.query_map([], attention_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Open items on one attempt.
pub fn open_for_attempt(conn: &Connection, attempt: i64) -> Result<Vec<Attention>, Fail> {
    let mut statement = conn.prepare(&format!(
        "SELECT {ATTENTION_COLUMNS} FROM attention WHERE state = 'open' AND attempt = ?1 ORDER BY id"
    ))?;
    let rows = statement.query_map([attempt], attention_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub struct Raise<'a> {
    pub kind: &'a str,
    pub reason: &'a str,
    pub ticket: Option<i64>,
    pub attempt: Option<i64>,
    pub execution: Option<i64>,
    pub payload: Value,
    pub text: Option<&'a str>,
}

pub fn raise(tx: &Connection, raise: Raise) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO attention (kind, reason, ticket, attempt, execution, payload, state, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', ?7)",
        params![
            raise.kind,
            raise.reason,
            raise.ticket,
            raise.attempt,
            raise.execution,
            raise.payload.to_string(),
            now()
        ],
    )?;
    let id = tx.last_insert_rowid();
    audit(
        tx,
        "attention.raised",
        Target {
            ticket: raise.ticket,
            attempt: raise.attempt,
            execution: raise.execution,
            attention: Some(id),
        },
        raise.text,
        json!({ "kind": raise.kind, "reason": raise.reason, "payload": raise.payload }),
    )?;
    Ok(id)
}

/// Resolve an item in the transaction of the command that answers it.
pub fn resolve(
    tx: &Connection,
    item: &Attention,
    resolution: &str,
    text: Option<&str>,
) -> Result<(), Fail> {
    let changed = tx.execute(
        "UPDATE attention SET state = 'resolved', resolution = ?2, resolved_at = ?3
         WHERE id = ?1 AND state = 'open'",
        params![item.id, resolution, now()],
    )?;
    if changed == 0 {
        return Err(Fail::stale(
            format!("attention {} is already resolved", item.id),
            json!(item.id),
            Value::Null,
        ));
    }
    audit(
        tx,
        "attention.resolved",
        Target {
            ticket: item.ticket,
            attempt: item.attempt,
            execution: item.execution,
            attention: Some(item.id),
        },
        text,
        json!({ "kind": item.kind, "reason": item.reason, "resolution": resolution }),
    )?;
    Ok(())
}
