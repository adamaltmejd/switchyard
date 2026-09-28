use super::{Target, audit, now};
use crate::api::Fail;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct Execution {
    pub id: i64,
    pub attempt: i64,
    pub parent: Option<i64>,
    pub kind: String,
    pub reason: Option<String>,
    pub status: String,
    pub outcome: Option<String>,
    pub detail: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub ticket_revision: Option<i64>,
    pub digest: Option<String>,
    pub name: Option<String>,
    pub round: Option<i64>,
    pub handle: Option<String>,
    pub session_id: Option<String>,
    pub approval: Option<i64>,
    pub intent_old: Option<String>,
    pub intent_merged: Option<String>,
    pub intent_state: Option<String>,
    pub progress: Option<String>,
    pub harness_version: Option<String>,
    pub resumed: Option<i64>,
}

const COLUMNS: &str = "id, attempt, parent, kind, reason, status, outcome, detail, base, head,
    ticket_revision, digest, name, round, handle, session_id, approval, intent_old, intent_merged,
    intent_state, progress, harness_version, resumed";

fn row(row: &Row) -> rusqlite::Result<Execution> {
    Ok(Execution {
        id: row.get(0)?,
        attempt: row.get(1)?,
        parent: row.get(2)?,
        kind: row.get(3)?,
        reason: row.get(4)?,
        status: row.get(5)?,
        outcome: row.get(6)?,
        detail: row.get(7)?,
        base: row.get(8)?,
        head: row.get(9)?,
        ticket_revision: row.get(10)?,
        digest: row.get(11)?,
        name: row.get(12)?,
        round: row.get(13)?,
        handle: row.get(14)?,
        session_id: row.get(15)?,
        approval: row.get(16)?,
        intent_old: row.get(17)?,
        intent_merged: row.get(18)?,
        intent_state: row.get(19)?,
        progress: row.get(20)?,
        harness_version: row.get(21)?,
        resumed: row.get(22)?,
    })
}

impl Execution {
    pub fn to_json(&self) -> Value {
        json!({
            "execution": self.id,
            "attempt": self.attempt,
            "parent": self.parent,
            "kind": self.kind,
            "reason": self.reason,
            "status": self.status,
            "outcome": self.outcome,
            "detail": self.detail,
            "base": self.base,
            "head": self.head,
            "name": self.name,
            "round": self.round,
            "intent": self.intent_state.as_ref().map(|state| json!({
                "state": state, "old": self.intent_old, "merged": self.intent_merged,
            })),
        })
    }
}

pub fn get(conn: &Connection, id: i64) -> Result<Execution, Fail> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM execution WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| Fail::not_found(format!("execution {id} does not exist")))
}

pub fn running(conn: &Connection) -> Result<Vec<Execution>, Fail> {
    query(conn, "status = 'running' ORDER BY id", [])
}

pub fn for_attempt(conn: &Connection, attempt: i64) -> Result<Vec<Execution>, Fail> {
    query(conn, "attempt = ?1 ORDER BY id", [attempt])
}

/// The attempt's session to resume: the most recent implementation before
/// `current` that reported a session id, and how many executions have run on
/// that session. A run that began fresh (`resumed` is NULL) is a boundary: an
/// older session is not resumed across it. The caller checks the session
/// owner's version, so a harness version change also ends continuity.
pub fn session(
    conn: &Connection,
    attempt: i64,
    current: i64,
) -> Result<Option<(Execution, i64)>, Fail> {
    let mut owner: Option<Execution> = None;
    for execution in query(
        conn,
        "attempt = ?1 AND kind = 'implementation' AND id != ?2 ORDER BY id DESC",
        params![attempt, current],
    )? {
        if execution.session_id.is_some() {
            owner = Some(execution);
            break;
        }
        // A run that reported no session but began fresh broke the chain:
        // an older session is not resumed across it.
        if execution.resumed.is_none() {
            break;
        }
    }
    let Some(owner) = owner else {
        return Ok(None);
    };
    let mut count = 0;
    let mut next = Some(owner.id);
    while let Some(id) = next {
        count += 1;
        next = conn.query_row("SELECT resumed FROM execution WHERE id = ?1", [id], |row| {
            row.get(0)
        })?;
    }
    Ok(Some((owner, count)))
}

/// Landings whose intent is still open.
pub fn open_intents(conn: &Connection) -> Result<Vec<Execution>, Fail> {
    query(
        conn,
        "kind = 'landing' AND intent_state = 'open' ORDER BY id",
        [],
    )
}

pub fn query<P: rusqlite::Params>(
    conn: &Connection,
    filter: &str,
    params: P,
) -> Result<Vec<Execution>, Fail> {
    super::all(
        conn,
        &format!("SELECT {COLUMNS} FROM execution WHERE {filter}"),
        params,
        row,
    )
}

/// What an execution is started on.
#[derive(Default)]
pub struct Start<'a> {
    pub attempt: i64,
    pub parent: Option<i64>,
    pub kind: &'a str,
    pub reason: Option<&'a str>,
    pub base: Option<&'a str>,
    pub head: Option<&'a str>,
    pub ticket_revision: Option<i64>,
    pub digest: Option<&'a str>,
    pub name: Option<&'a str>,
    pub round: Option<i64>,
    pub approval: Option<i64>,
    pub ticket: Option<i64>,
    /// Worker settings: agent name and its `{harness, provider login, model,
    /// effort}`.
    pub agent: Option<(&'a str, &'a Value)>,
    /// The pinfold pin of the worker's harness, recorded with the intent.
    pub harness_version: Option<&'a str>,
}

/// The intent: the row exists before any effect.
pub fn start(tx: &Connection, start: Start) -> Result<i64, Fail> {
    let settings = start.agent.map(|(_, settings)| settings);
    let field = |key: &str| settings.and_then(|value| value[key].as_str().map(str::to_string));
    tx.execute(
        "INSERT INTO execution (attempt, parent, kind, reason, status, base, head, ticket_revision,
            digest, name, round, approval, agent, harness, harness_version, provider, model, effort, started_at)
         VALUES (?1, ?2, ?3, ?4, 'running', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            start.attempt,
            start.parent,
            start.kind,
            start.reason,
            start.base,
            start.head,
            start.ticket_revision,
            start.digest,
            start.name,
            start.round,
            start.approval,
            start.agent.map(|(name, _)| name),
            field("harness"),
            start.harness_version,
            field("provider"),
            field("model"),
            field("effort"),
            now()
        ],
    )?;
    let id = tx.last_insert_rowid();
    audit(
        tx,
        "execution.started",
        Target {
            ticket: start.ticket,
            attempt: Some(start.attempt),
            execution: Some(id),
            attention: None,
        },
        None,
        json!({ "kind": start.kind, "reason": start.reason, "name": start.name,
                "head": start.head, "parent": start.parent }),
    )?;
    Ok(id)
}

/// Observational: no event.
pub fn set_handle(
    conn: &Connection,
    id: i64,
    handle: &str,
    image_id: Option<&str>,
) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET handle = ?2, image_id = COALESCE(?3, image_id) WHERE id = ?1",
        params![id, handle, image_id],
    )?;
    Ok(())
}

/// Observational: a worker's session and usage.
pub fn set_worker(
    conn: &Connection,
    id: i64,
    session_id: Option<&str>,
    usage: Option<(u64, u64, f64)>,
) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET session_id = COALESCE(?2, session_id),
            tokens_in = COALESCE(?3, tokens_in), tokens_out = COALESCE(?4, tokens_out),
            cost = COALESCE(?5, cost)
         WHERE id = ?1",
        params![
            id,
            session_id,
            usage.map(|usage| usage.0 as i64),
            usage.map(|usage| usage.1 as i64),
            usage.map(|usage| usage.2)
        ],
    )?;
    Ok(())
}

pub fn set_progress(conn: &Connection, id: i64, note: &str) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET progress = ?2 WHERE id = ?1",
        params![id, note],
    )?;
    Ok(())
}

/// Observational: the harness's registration proof, recorded once its reader
/// yields a complete one. `mcp` is `registered` or `refused`.
pub fn set_mcp(conn: &Connection, id: i64, mcp: &str) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET mcp = ?2 WHERE id = ?1",
        params![id, mcp],
    )?;
    Ok(())
}

/// How an execution ended.
#[derive(Default)]
pub struct End<'a> {
    pub outcome: &'a str,
    pub detail: Option<&'a str>,
    pub exit_cause: Option<&'a str>,
    pub exit_code: Option<i32>,
    pub oom_kills: Option<u64>,
    pub ticket: Option<i64>,
}

pub fn end(tx: &Connection, id: i64, end: End) -> Result<(), Fail> {
    let execution = get(tx, id)?;
    if execution.status != "running" {
        return Err(Fail::new(
            "internal",
            format!("execution {id} has already ended"),
        ));
    }
    tx.execute(
        "UPDATE execution SET status = 'ended', outcome = ?2, detail = ?3, exit_cause = ?4,
            exit_code = ?5, oom_kills = ?6, ended_at = ?7
         WHERE id = ?1",
        params![
            id,
            end.outcome,
            end.detail,
            end.exit_cause,
            end.exit_code,
            end.oom_kills.map(|count| count as i64),
            now()
        ],
    )?;
    audit(
        tx,
        "execution.ended",
        Target {
            ticket: end.ticket,
            attempt: Some(execution.attempt),
            execution: Some(id),
            attention: None,
        },
        end.detail,
        json!({ "kind": execution.kind, "outcome": end.outcome, "name": execution.name,
                "exit_cause": end.exit_cause }),
    )?;
    Ok(())
}

pub fn set_intent(
    tx: &Connection,
    id: i64,
    old: &str,
    merged: &str,
    ticket: i64,
    attempt: i64,
) -> Result<(), Fail> {
    tx.execute(
        "UPDATE execution SET intent_old = ?2, intent_merged = ?3, intent_state = 'open' WHERE id = ?1",
        params![id, old, merged],
    )?;
    audit(
        tx,
        "landing.intent",
        Target {
            ticket: Some(ticket),
            attempt: Some(attempt),
            execution: Some(id),
            attention: None,
        },
        None,
        json!({ "old": old, "merged": merged }),
    )?;
    Ok(())
}

pub fn resolve_intent(tx: &Connection, id: i64) -> Result<(), Fail> {
    tx.execute(
        "UPDATE execution SET intent_state = 'resolved' WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
