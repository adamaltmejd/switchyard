use super::{Target, audit, now};
use crate::api::Fail;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde_json::{Value, json};

/// What a judgment binds: a candidate, the ticket revision and one digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub attempt: i64,
    pub base: String,
    pub head: String,
    pub proof: String,
    pub ticket_revision: i64,
    pub digest: String,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub id: i64,
    pub execution: i64,
    pub kind: String,
    pub name: String,
    pub base: String,
    pub head: String,
    pub proof: String,
    pub ticket_revision: i64,
    pub verdict: String,
    pub round: Option<i64>,
}

// A check is its execution's input plus a verdict, read through the join.
const SELECT: &str = "SELECT c.id, c.execution, c.kind, COALESCE(e.name, ''), COALESCE(e.base, ''),
    COALESCE(e.head, ''), COALESCE(e.proof, ''), COALESCE(e.ticket_revision, 0), c.verdict, e.round
    FROM \"check\" c JOIN execution e ON e.id = c.execution";

fn row(row: &Row) -> rusqlite::Result<Check> {
    Ok(Check {
        id: row.get(0)?,
        execution: row.get(1)?,
        kind: row.get(2)?,
        name: row.get(3)?,
        base: row.get(4)?,
        head: row.get(5)?,
        proof: row.get(6)?,
        ticket_revision: row.get(7)?,
        verdict: row.get(8)?,
        round: row.get(9)?,
    })
}

impl Check {
    pub fn to_json(&self) -> Value {
        json!({
            "check": self.id,
            "execution": self.execution,
            "kind": self.kind,
            "name": self.name,
            "head": self.head,
            "base": self.base,
            "proof": self.proof,
            "ticket_revision": self.ticket_revision,
            "verdict": self.verdict,
            "round": self.round,
        })
    }
}

/// The latest check of `kind` and `name` on exactly `input`. A review check
/// counts only when its execution row records the harness's registration
/// proof, so a seat that never proved itself never counts, before or after a
/// restart; a seat whose proof was recorded keeps its publication even if a
/// crash later interrupts it. Gate checks carry no registration.
pub fn current(
    conn: &Connection,
    kind: &str,
    name: &str,
    input: &Input,
) -> Result<Option<Check>, Fail> {
    Ok(conn
        .query_row(
            &format!(
                "{SELECT}
                 WHERE c.kind = ?1 AND e.name = ?2 AND e.attempt = ?3
                 AND e.base = ?4 AND e.head = ?5 AND e.ticket_revision = ?6 AND e.digest = ?7
                 AND COALESCE(e.proof, '') = ?8
                 AND (c.kind != 'review' OR e.mcp = 'registered')
                 ORDER BY c.id DESC LIMIT 1"
            ),
            params![
                kind,
                name,
                input.attempt,
                input.base,
                input.head,
                input.ticket_revision,
                input.digest,
                input.proof
            ],
            row,
        )
        .optional()?)
}

pub fn for_execution(conn: &Connection, execution: i64) -> Result<Option<Check>, Fail> {
    Ok(conn
        .query_row(
            &format!("{SELECT} WHERE c.execution = ?1"),
            [execution],
            row,
        )
        .optional()?)
}

pub fn for_attempt(conn: &Connection, attempt: i64) -> Result<Vec<Check>, Fail> {
    super::all(
        conn,
        &format!("{SELECT} WHERE e.attempt = ?1 ORDER BY c.id"),
        [attempt],
        row,
    )
}

pub struct Record<'a> {
    pub execution: i64,
    pub kind: &'a str,
    pub verdict: &'a str,
    pub ticket: i64,
}

pub fn record(tx: &Connection, record: Record) -> Result<i64, Fail> {
    let row = super::executions::get(tx, record.execution)?;
    tx.execute(
        "INSERT INTO \"check\" (execution, kind, verdict, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![record.execution, record.kind, record.verdict, now()],
    )?;
    let id = tx.last_insert_rowid();
    audit(
        tx,
        "check.recorded",
        Target {
            ticket: Some(record.ticket),
            attempt: Some(row.attempt),
            execution: Some(record.execution),
            attention: None,
        },
        None,
        json!({ "check": id, "kind": record.kind, "name": row.name, "verdict": record.verdict,
                "head": row.head.unwrap_or_default(), "proof": row.proof.unwrap_or_default(), "round": row.round }),
    )?;
    Ok(id)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Finding {
    pub priority: String,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub category: Option<String>,
    pub body: String,
}

pub fn add_finding(tx: &Connection, check: i64, finding: &Finding) -> Result<(), Fail> {
    tx.execute(
        "INSERT INTO finding (\"check\", priority, file, line, category, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            check,
            crate::config::priority(&finding.priority).unwrap_or(3),
            finding.file,
            finding.line,
            finding.category,
            finding.body
        ],
    )?;
    Ok(())
}

pub fn findings(conn: &Connection, check: i64) -> Result<Vec<Finding>, Fail> {
    super::all(
        conn,
        "SELECT priority, file, line, category, body FROM finding WHERE \"check\" = ?1 ORDER BY priority, id",
        [check],
        |row| {
            Ok(Finding {
                priority: format!("P{}", row.get::<_, i64>(0)?),
                file: row.get(1)?,
                line: row.get(2)?,
                category: row.get(3)?,
                body: row.get(4)?,
            })
        },
    )
}

#[derive(Debug, Clone)]
pub struct Approval {
    pub id: i64,
    pub attempt: i64,
    pub base: String,
    pub head: String,
    pub proof: String,
    pub ticket_revision: i64,
    pub gate_digest: String,
    pub review_digest: String,
    pub checks: Vec<i64>,
    pub actor: String,
    pub state: String,
}

const APPROVAL_COLUMNS: &str = "id, attempt, base, head, ticket_revision, gate_digest, review_digest, checks, actor, state, proof";

fn approval_row(row: &Row) -> rusqlite::Result<Approval> {
    Ok(Approval {
        id: row.get(0)?,
        attempt: row.get(1)?,
        base: row.get(2)?,
        head: row.get(3)?,
        ticket_revision: row.get(4)?,
        gate_digest: row.get(5)?,
        review_digest: row.get(6)?,
        checks: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        actor: row.get(8)?,
        state: row.get(9)?,
        proof: row.get(10)?,
    })
}

pub fn approval(conn: &Connection, id: i64) -> Result<Approval, Fail> {
    conn.query_row(
        &format!("SELECT {APPROVAL_COLUMNS} FROM approval WHERE id = ?1"),
        [id],
        approval_row,
    )
    .optional()?
    .ok_or_else(|| Fail::not_found(format!("approval {id} does not exist")))
}

/// Active approvals in approval order: the queue.
pub fn queue(conn: &Connection) -> Result<Vec<Approval>, Fail> {
    super::all(
        conn,
        &format!("SELECT {APPROVAL_COLUMNS} FROM approval WHERE state = 'active' ORDER BY id"),
        [],
        approval_row,
    )
}

pub fn active_for(conn: &Connection, attempt: i64) -> Result<Option<Approval>, Fail> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {APPROVAL_COLUMNS} FROM approval WHERE attempt = ?1 AND state = 'active'"
            ),
            [attempt],
            approval_row,
        )
        .optional()?)
}

pub struct Approve<'a> {
    pub attempt: i64,
    pub ticket: i64,
    pub base: &'a str,
    pub head: &'a str,
    pub proof: &'a str,
    pub ticket_revision: i64,
    pub gate_digest: &'a str,
    pub review_digest: &'a str,
    pub checks: &'a [i64],
    pub actor: &'a str,
    pub text: Option<&'a str>,
    pub overrode: bool,
}

pub fn approve(tx: &Connection, approve: Approve) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO approval (attempt, base, head, proof, ticket_revision, gate_digest, review_digest,
            checks, actor, text, state, created_at, overrode)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'active', ?11, ?12)",
        params![
            approve.attempt,
            approve.base,
            approve.head,
            approve.proof,
            approve.ticket_revision,
            approve.gate_digest,
            approve.review_digest,
            json!(approve.checks).to_string(),
            approve.actor,
            approve.text,
            now(),
            approve.overrode
        ],
    )?;
    let id = tx.last_insert_rowid();
    audit(
        tx,
        "approval.given",
        Target {
            ticket: Some(approve.ticket),
            attempt: Some(approve.attempt),
            ..Target::default()
        },
        approve.text,
        json!({ "approval": id, "actor": approve.actor, "head": approve.head, "base": approve.base,
                "proof": approve.proof, "checks": approve.checks, "overrode": approve.overrode }),
    )?;
    Ok(id)
}

/// Move an approval out of the queue: `withdrawn`, `landed` or `retired`.
pub fn set_approval_state(
    tx: &Connection,
    approval: &Approval,
    state: &str,
    ticket: i64,
    text: Option<&str>,
) -> Result<(), Fail> {
    tx.execute(
        "UPDATE approval SET state = ?2 WHERE id = ?1",
        params![approval.id, state],
    )?;
    if state != "landed" {
        audit(
            tx,
            "approval.ended",
            Target {
                ticket: Some(ticket),
                attempt: Some(approval.attempt),
                ..Target::default()
            },
            text,
            json!({ "approval": approval.id, "state": state }),
        )?;
    }
    Ok(())
}
