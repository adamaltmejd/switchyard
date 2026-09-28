use super::{Target, audit, now, ticket_name};
use crate::api::Fail;
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub priority: i64,
    pub workflow: String,
    pub state: String,
    pub parked: bool,
    pub revision: i64,
    pub origin: String,
    pub started_once: bool,
}

const COLUMNS: &str =
    "id, title, body, priority, workflow, state, parked, revision, origin, started_once";

fn row(row: &Row) -> rusqlite::Result<Ticket> {
    Ok(Ticket {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        priority: row.get(3)?,
        workflow: row.get(4)?,
        state: row.get(5)?,
        parked: row.get(6)?,
        revision: row.get(7)?,
        origin: row.get(8)?,
        started_once: row.get(9)?,
    })
}

impl Ticket {
    pub fn to_json(&self) -> Value {
        json!({
            "ticket": ticket_name(self.id),
            "title": self.title,
            "body": self.body,
            "priority": format!("P{}", self.priority),
            "workflow": self.workflow,
            "state": self.state,
            "parked": self.parked,
            "revision": self.revision,
            "origin": self.origin,
        })
    }
}

pub fn get(conn: &Connection, id: i64) -> Result<Ticket, Fail> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM ticket WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| Fail::not_found(format!("{} does not exist", ticket_name(id))))
}

pub fn list(conn: &Connection) -> Result<Vec<Ticket>, Fail> {
    let mut statement = conn.prepare(&format!("SELECT {COLUMNS} FROM ticket ORDER BY id"))?;
    let rows = statement.query_map([], row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn dependencies(conn: &Connection, id: i64) -> Result<Vec<i64>, Fail> {
    let mut statement =
        conn.prepare("SELECT depends_on FROM dependency WHERE ticket = ?1 ORDER BY depends_on")?;
    let rows = statement.query_map([id], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub struct NewTicket<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub priority: i64,
    pub workflow: &'a str,
    pub depends_on: &'a [i64],
    pub parked: bool,
    pub origin: &'a str,
}

pub fn create(tx: &Connection, new: &NewTicket, text: Option<&str>) -> Result<i64, Fail> {
    if new.title.trim().is_empty() {
        return Err(Fail::invalid("a ticket needs a title"));
    }
    tx.execute(
        "INSERT INTO ticket (title, body, priority, workflow, state, parked, origin, created_at)
         VALUES (?1, ?2, ?3, ?4, 'open', ?5, ?6, ?7)",
        params![
            new.title,
            new.body,
            new.priority,
            new.workflow,
            new.parked,
            new.origin,
            now()
        ],
    )?;
    let id = tx.last_insert_rowid();
    for dependency in new.depends_on {
        get(tx, *dependency)?;
        tx.execute(
            "INSERT INTO dependency (ticket, depends_on) VALUES (?1, ?2)",
            params![id, dependency],
        )?;
    }
    audit(
        tx,
        "ticket.new",
        Target {
            ticket: Some(id),
            ..Target::default()
        },
        text,
        json!({ "title": new.title, "workflow": new.workflow, "origin": new.origin,
                "depends_on": new.depends_on.iter().map(|id| ticket_name(*id)).collect::<Vec<_>>(),
                "body_bytes": new.body.len(), "parked": new.parked }),
    )?;
    Ok(id)
}

/// Would adding `ticket depends_on on` close a cycle?
pub fn would_cycle(conn: &Connection, ticket: i64, on: i64) -> Result<bool, Fail> {
    if ticket == on {
        return Ok(true);
    }
    let found: Option<i64> = conn
        .query_row(
            "WITH RECURSIVE reach(id) AS (
                SELECT depends_on FROM dependency WHERE ticket = ?1
                UNION SELECT d.depends_on FROM dependency d JOIN reach r ON d.ticket = r.id)
             SELECT id FROM reach WHERE id = ?2",
            params![on, ticket],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

/// Ready tickets in priority order: open, not parked, every dependency done,
/// no live attempt, no open proposal blocking it, and a read-only workflow's
/// ticket only once.
pub fn ready(conn: &Connection) -> Result<Vec<Ticket>, Fail> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM ticket t
         WHERE state = 'open' AND parked = 0
           AND NOT EXISTS (SELECT 1 FROM dependency d JOIN ticket o ON o.id = d.depends_on
                           WHERE d.ticket = t.id AND o.state != 'done')
           AND NOT EXISTS (SELECT 1 FROM attempt a WHERE a.ticket = t.id AND a.state = 'live')
           AND NOT EXISTS (SELECT 1 FROM attention n WHERE n.ticket = t.id AND n.kind = 'proposal'
                           AND n.state = 'open' AND n.reason = 'edit')
         ORDER BY priority, id"
    ))?;
    let rows = statement.query_map([], row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
