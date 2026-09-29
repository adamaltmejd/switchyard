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
    pub input: Input,
    pub verdict: String,
    pub round: Option<i64>,
}

const COLUMNS: &str = "id, execution, attempt, kind, name, base, head, ticket_revision, digest, verdict, round, proof";

fn row(row: &Row) -> rusqlite::Result<Check> {
    Ok(Check {
        id: row.get(0)?,
        execution: row.get(1)?,
        kind: row.get(3)?,
        name: row.get(4)?,
        input: Input {
            attempt: row.get(2)?,
            base: row.get(5)?,
            head: row.get(6)?,
            ticket_revision: row.get(7)?,
            digest: row.get(8)?,
            proof: row.get(11)?,
        },
        verdict: row.get(9)?,
        round: row.get(10)?,
    })
}

impl Check {
    pub fn to_json(&self) -> Value {
        json!({
            "check": self.id,
            "execution": self.execution,
            "kind": self.kind,
            "name": self.name,
            "head": self.input.head,
            "base": self.input.base,
            "proof": self.input.proof,
            "ticket_revision": self.input.ticket_revision,
            "verdict": self.verdict,
            "round": self.round,
        })
    }
}

/// The latest check of `kind` and `name` on exactly `input`.
pub fn current(
    conn: &Connection,
    kind: &str,
    name: &str,
    input: &Input,
) -> Result<Option<Check>, Fail> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {COLUMNS} FROM \"check\" WHERE kind = ?1 AND name = ?2 AND attempt = ?3
                 AND base = ?4 AND head = ?5 AND ticket_revision = ?6 AND digest = ?7 AND proof = ?8
                 ORDER BY id DESC LIMIT 1"
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
            &format!("SELECT {COLUMNS} FROM \"check\" WHERE execution = ?1"),
            [execution],
            row,
        )
        .optional()?)
}

pub fn for_attempt(conn: &Connection, attempt: i64) -> Result<Vec<Check>, Fail> {
    super::all(
        conn,
        &format!("SELECT {COLUMNS} FROM \"check\" WHERE attempt = ?1 ORDER BY id"),
        [attempt],
        row,
    )
}

pub struct Record<'a> {
    pub execution: i64,
    pub kind: &'a str,
    pub name: &'a str,
    pub input: &'a Input,
    pub verdict: &'a str,
    pub image_id: Option<&'a str>,
    pub round: Option<i64>,
    pub ticket: i64,
}

pub fn record(tx: &Connection, record: Record) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO \"check\" (execution, attempt, kind, name, base, head, proof, ticket_revision, digest,
            verdict, image_id, round, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            record.execution,
            record.input.attempt,
            record.kind,
            record.name,
            record.input.base,
            record.input.head,
            record.input.proof,
            record.input.ticket_revision,
            record.input.digest,
            record.verdict,
            record.image_id,
            record.round,
            now()
        ],
    )?;
    let id = tx.last_insert_rowid();
    audit(
        tx,
        "check.recorded",
        Target {
            ticket: Some(record.ticket),
            attempt: Some(record.input.attempt),
            execution: Some(record.execution),
            attention: None,
        },
        None,
        json!({ "check": id, "kind": record.kind, "name": record.name, "verdict": record.verdict,
                "head": record.input.head, "proof": record.input.proof, "round": record.round }),
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
}

pub fn approve(tx: &Connection, approve: Approve) -> Result<i64, Fail> {
    tx.execute(
        "INSERT INTO approval (attempt, base, head, proof, ticket_revision, gate_digest, review_digest,
            checks, actor, text, state, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'active', ?11)",
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
            now()
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
                "proof": approve.proof, "checks": approve.checks }),
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
