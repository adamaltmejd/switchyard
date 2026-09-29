use super::{Target, all, audit, now, ticket_name};
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
}

const COLUMNS: &str = "id, title, body, priority, workflow, state, parked, revision, origin";

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
    all(
        conn,
        &format!("SELECT {COLUMNS} FROM ticket ORDER BY id"),
        [],
        row,
    )
}

pub fn dependencies(conn: &Connection, id: i64) -> Result<Vec<i64>, Fail> {
    all(
        conn,
        "SELECT depends_on FROM dependency WHERE ticket = ?1 ORDER BY depends_on",
        [id],
        |row| row.get(0),
    )
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

pub fn create(tx: &Connection, new: &NewTicket) -> Result<i64, Fail> {
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
        None,
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

/// Why ticket `t` is not ready, or NULL when it is: open, not parked, no
/// live attempt, every dependency done, no open proposal to edit it.
const BLOCKER: &str = "CASE
    WHEN t.state != 'open' THEN 'it is ' || t.state
    WHEN t.parked THEN 'it is parked'
    WHEN EXISTS (SELECT 1 FROM attempt a WHERE a.ticket = t.id AND a.state = 'live')
        THEN 'it has a live attempt'
    WHEN EXISTS (SELECT 1 FROM dependency d JOIN ticket o ON o.id = d.depends_on
                 WHERE d.ticket = t.id AND o.state != 'done')
        THEN 'it depends on ' || (SELECT group_concat('Y-' || o.id, ', ')
                                  FROM dependency d JOIN ticket o ON o.id = d.depends_on
                                  WHERE d.ticket = t.id AND o.state != 'done')
    WHEN EXISTS (SELECT 1 FROM attention n WHERE n.ticket = t.id AND n.kind = 'proposal'
                 AND n.state = 'open' AND n.reason = 'edit')
        THEN 'a proposal to edit it is open'
END";

/// Ready tickets in priority order.
pub fn ready(conn: &Connection) -> Result<Vec<Ticket>, Fail> {
    all(
        conn,
        &format!("SELECT {COLUMNS} FROM ticket t WHERE ({BLOCKER}) IS NULL ORDER BY priority, id"),
        [],
        row,
    )
}

/// Why a ticket is not ready; `None` when it is.
pub fn blocker(conn: &Connection, id: i64) -> Result<Option<String>, Fail> {
    Ok(conn.query_row(
        &format!("SELECT {BLOCKER} FROM ticket t WHERE t.id = ?1"),
        [id],
        |row| row.get(0),
    )?)
}
