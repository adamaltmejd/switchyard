use super::{Target, all, audit, now, ticket_name};
use crate::api::Fail;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct Attempt {
    pub id: i64,
    pub ticket: i64,
    pub workflow: String,
    pub target_branch: Option<String>,
    /// The implementer frozen at admission: `{name, harness, provider login,
    /// model, effort}`.
    pub implementer: Value,
    /// Current plan evidence; original admission is never overwritten.
    pub admission: Option<Value>,
    pub original_admission: Option<Value>,
    pub base: String,
    pub head: Option<String>,
    /// The digest of the candidate's proof snapshot.
    pub proof: Option<String>,
    /// The latest target handed to the attempt for a repair.
    pub target: Option<String>,
    pub state: String,
    pub outcome: Option<String>,
    pub lane: bool,
    pub lane_since: Option<i64>,
    pub work_ms: i64,
    /// What the next implementer execution is for: `{reason, detail}`.
    pub next: Option<Value>,
    pub rounds: i64,
    pub extra_rounds: i64,
    pub landing_reds: i64,
}

const COLUMNS: &str =
    "id, ticket, workflow, implementer, base, head, proof, target, state, outcome,
    lane, lane_since, work_ms, next, rounds, extra_rounds, landing_reds, target_branch, admission,
    COALESCE((SELECT evidence FROM plan_revision WHERE plan_revision.attempt = attempt.id
              ORDER BY revision DESC LIMIT 1), admission)";

fn row(row: &Row) -> rusqlite::Result<Attempt> {
    let implementer: String = row.get(3)?;
    let next: Option<String> = row.get(13)?;
    Ok(Attempt {
        id: row.get(0)?,
        ticket: row.get(1)?,
        workflow: row.get(2)?,
        implementer: serde_json::from_str(&implementer).unwrap_or(Value::Null),
        base: row.get(4)?,
        head: row.get(5)?,
        proof: row.get(6)?,
        target: row.get(7)?,
        state: row.get(8)?,
        outcome: row.get(9)?,
        lane: row.get(10)?,
        lane_since: row.get(11)?,
        work_ms: row.get(12)?,
        next: next.and_then(|text| serde_json::from_str(&text).ok()),
        rounds: row.get(14)?,
        extra_rounds: row.get(15)?,
        landing_reds: row.get(16)?,
        target_branch: row.get(17)?,
        original_admission: row
            .get::<_, Option<String>>(18)?
            .map(|text| {
                serde_json::from_str(&text).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        18,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
        admission: row
            .get::<_, Option<String>>(19)?
            .map(|text| {
                serde_json::from_str(&text).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        19,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
    })
}

impl Attempt {
    /// Launch settings belong to the selected revision, not the original row.
    pub fn implementation_settings(&self) -> Result<Value, Fail> {
        let Some(admission) = &self.admission else {
            return Ok(self.implementer.clone());
        };
        let plan = crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?;
        let agent = match &plan.nodes["implementer"].runner {
            crate::plan::Runner::Agent { agent, .. }
            | crate::plan::Runner::Packaged { agent, .. } => agent,
            _ => return Err(Fail::refused("implementation runner has no agent")),
        };
        let mut settings = admission["agents"][agent]["settings"].clone();
        if !settings.is_object() {
            return Err(Fail::refused("missing recorded implementation settings"));
        }
        settings["name"] = json!(agent);
        Ok(settings)
    }

    pub fn plan_revision(&self) -> Option<i64> {
        self.admission
            .as_ref()
            .and_then(|input| input["revision"].as_i64())
    }

    pub fn to_json(&self) -> Value {
        json!({
            "attempt": self.id,
            "ticket": ticket_name(self.ticket),
            "workflow": self.workflow,
            "implementer": self.implementer,
            "admission": self.original_admission.clone().unwrap_or_else(|| json!({"status": "unknown"})),
            "plan": self.admission,
            "plan_revision": self.plan_revision(),
            "branch": self.branch(),
            "target": self.target_branch,
            "base": self.base,
            "head": self.head,
            "proof": self.proof,
            "state": self.state,
            "outcome": self.outcome,
            "lane": self.lane,
            "next": self.next,
            "rounds": self.rounds,
            "landing_reds": self.landing_reds,
        })
    }

    pub fn target_branch(&self) -> Result<&str, Fail> {
        self.target_branch
            .as_deref()
            .ok_or_else(|| Fail::refused("the attempt has no frozen target"))
    }

    /// The attempt\'s branch in its clone.
    pub fn branch(&self) -> String {
        format!("yard/{}/{}", ticket_name(self.ticket), self.id)
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

/// Read the definitions this execution consumed, including after a live revision.
/// Without a recorded revision, admission evidence stays unknown.
pub fn for_execution(
    conn: &Connection,
    execution: &super::executions::Execution,
) -> Result<Attempt, Fail> {
    let mut attempt = get(conn, execution.attempt)?;
    attempt.admission = match execution.plan_revision {
        Some(revision_id) => Some(
            revision(conn, attempt.id, revision_id)?
                .ok_or_else(|| Fail::refused("missing execution plan revision"))?,
        ),
        None => None,
    };
    Ok(attempt)
}

/// Only recorded evidence is returned; pre-admission history stays unknown.
pub fn revision(conn: &Connection, attempt: i64, revision: i64) -> Result<Option<Value>, Fail> {
    if revision == 1 {
        return Ok(get(conn, attempt)?.original_admission);
    }
    let raw: Option<String> = conn
        .query_row(
            "SELECT evidence FROM plan_revision WHERE attempt = ?1 AND revision = ?2",
            params![attempt, revision],
            |row| row.get(0),
        )
        .optional()?;
    raw.map(|raw| serde_json::from_str(&raw).map_err(|error| Fail::refused(error.to_string())))
        .transpose()
}

pub fn revisions(conn: &Connection, attempt: i64) -> Result<Vec<Value>, Fail> {
    let mut history: Vec<Value> = get(conn, attempt)?.original_admission.into_iter().collect();
    let rows: Vec<String> = all(
        conn,
        "SELECT evidence FROM plan_revision WHERE attempt = ?1 ORDER BY revision",
        [attempt],
        |row| row.get(0),
    )?;
    for raw in rows {
        history.push(serde_json::from_str(&raw).map_err(|error| Fail::refused(error.to_string()))?);
    }
    Ok(history)
}

pub fn require_plan(conn: &Connection, attempt: i64, expected: Option<i64>) -> Result<(), Fail> {
    let current = get(conn, attempt)?.plan_revision();
    if expected != current {
        return Err(
            Fail::stale("plan_revision", json!(expected), json!(current)).with(
                json!({"identity": "plan_revision", "expected": expected, "current": current}),
            ),
        );
    }
    Ok(())
}

pub fn append_plan(
    conn: &Connection,
    attempt: i64,
    expected: i64,
    evidence: &Value,
) -> Result<(), Fail> {
    require_plan(conn, attempt, Some(expected))?;
    conn.execute("INSERT INTO plan_revision (attempt, revision, evidence, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![attempt, expected + 1, evidence.to_string(), now()])?;
    Ok(())
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
    all(
        conn,
        &format!("SELECT {COLUMNS} FROM attempt WHERE state = 'live' ORDER BY id"),
        [],
        row,
    )
}

/// Ended attempts whose files are not yet removed.
pub fn uncleaned(conn: &Connection) -> Result<Vec<Attempt>, Fail> {
    all(
        conn,
        &format!("SELECT {COLUMNS} FROM attempt WHERE state = 'ended' AND cleaned = 0 ORDER BY id"),
        [],
        row,
    )
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
    target_branch: &str,
    admission: &Value,
) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO attempt (ticket, workflow, implementer, base, state, lane, lane_since, created_at, target_branch, admission)
         VALUES (?1, ?2, ?3, ?4, 'live', 1, ?5, ?6, ?7, ?8)",
        params![
            ticket,
            workflow,
            implementer.to_string(),
            base,
            super::now_ms(),
            now(),
            target_branch,
            admission.to_string()
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

pub fn set_next(tx: &Connection, attempt: i64, next: Option<&Value>) -> Result<(), Fail> {
    tx.execute(
        "UPDATE attempt SET next = ?2 WHERE id = ?1",
        params![attempt, next.map(Value::to_string)],
    )?;
    Ok(())
}

/// Renew the total-work clock: an operator's edit on a `timeout`.
pub fn renew_clock(tx: &Connection, attempt: i64) -> Result<(), Fail> {
    tx.execute(
        "UPDATE attempt SET work_ms = 0, lane_since = CASE WHEN lane = 1 THEN ?2 ELSE lane_since END WHERE id = ?1",
        params![attempt, super::now_ms()],
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
            params![attempt, now],
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
    /// The commands that answer the item.
    pub fn exits(&self) -> &'static [&'static str] {
        match (self.kind.as_str(), self.reason.as_str()) {
            ("candidate", "landing") if self.payload["evidence"]["nodes"].is_array() => {
                &["accept", "edit", "abandon"]
            }
            ("candidate", _) => &["accept", "repair", "edit", "abandon"],
            ("approval", _) => &["approve", "reject", "abandon"],
            ("proposal", _) => &["accept", "reject"],
            ("stopped", "limit") => &["approve", "edit", "abandon"],
            ("stopped", "timeout") => &["edit", "abandon"],
            ("stopped", _) => &["start", "edit", "abandon"],
            ("red", _) if self.attempt.is_none() => &["start"],
            ("red", _) => &["start", "edit", "abandon"],
            _ => &["start", "abandon"],
        }
    }

    /// The item with the commands that answer it.
    pub fn to_json(&self) -> Value {
        let ticket = self.ticket.map(ticket_name);
        json!({
            "attention": self.id,
            "kind": self.kind,
            "reason": self.reason,
            "ticket": ticket,
            "attempt": self.attempt,
            "execution": self.execution,
            "payload": self.payload,
            "state": self.state,
            "exits": self.exits(),
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
    all(
        conn,
        &format!("SELECT {ATTENTION_COLUMNS} FROM attention WHERE state = 'open' ORDER BY id"),
        [],
        attention_row,
    )
}

/// Open items on one attempt.
pub fn open_for_attempt(conn: &Connection, attempt: i64) -> Result<Vec<Attention>, Fail> {
    all(
        conn,
        &format!(
            "SELECT {ATTENTION_COLUMNS} FROM attention WHERE state = 'open' AND attempt = ?1 ORDER BY id"
        ),
        [attempt],
        attention_row,
    )
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

/// An edit is pending when the latest implementer has unread ticket changes,
/// changed policy or a live revision handoff. Derived from immutable rows.
pub fn edit_pending(tx: &Connection, attempt: i64) -> Result<bool, Fail> {
    let read: Option<i64> = tx
        .query_row(
            "SELECT ticket_revision FROM execution
             WHERE attempt = ?1 AND kind = 'implementation' ORDER BY id DESC LIMIT 1",
            [attempt],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let Some(read) = read else {
        return Ok(false);
    };
    let revision: i64 = tx.query_row(
        "SELECT revision FROM ticket WHERE id = (SELECT ticket FROM attempt WHERE id = ?1)",
        [attempt],
        |row| row.get(0),
    )?;
    if revision > read {
        return Ok(true);
    }
    let current = get(tx, attempt)?;
    let execution = super::executions::for_attempt(tx, attempt)?
        .into_iter()
        .rev()
        .find(|row| row.kind == "implementation")
        .expect("implementation read");
    if current
        .admission
        .as_ref()
        .and_then(|value| value["handoff_from"].as_i64())
        .is_some_and(|id| id >= execution.id)
    {
        return Ok(true);
    }
    super::executions::plan_superseded(tx, &current, &execution)
}

/// The one guard for every transaction that decides an attempt's next step: a
/// decision about a candidate that read ticket revision `read` records nothing
/// once the ticket moved past it or an edit is pending.
pub fn superseded_by_edit(
    tx: &Connection,
    attempt: i64,
    ticket: i64,
    read: i64,
) -> Result<bool, Fail> {
    let revision: i64 = tx.query_row(
        "SELECT revision FROM ticket WHERE id = ?1",
        [ticket],
        |row| row.get(0),
    )?;
    Ok(revision != read || edit_pending(tx, attempt)?)
}
