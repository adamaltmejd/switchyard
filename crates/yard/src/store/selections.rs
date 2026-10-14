//! Choices refer to immutable publications; they never create candidates.
use super::{Target, attempts, audit, checks, executions, tickets};
use crate::api::Fail;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

pub fn inputs(conn: &Connection, attempt: &attempts::Attempt) -> Result<Option<Value>, Fail> {
    executions::binding(
        conn,
        attempt,
        &executions::Start {
            kind: "review",
            node: Some("selector"),
            ticket_revision: Some(tickets::get(conn, attempt.ticket)?.revision),
            ..Default::default()
        },
    )
    .map(|binding| binding.map(|(_, inputs)| inputs))
}

pub fn publication(row: &executions::Execution, candidate: &str) -> Result<Value, Fail> {
    let input = row.inputs.as_ref().and_then(|inputs| inputs.get(candidate))
        .filter(|input| input["kind"] == "candidate")
        .ok_or_else(|| Fail::refused("selection_input").with(json!({
            "reason": "selection_input", "identity": "candidate", "expected": candidate,
            "current": row.inputs.as_ref().and_then(Value::as_object).map(|inputs| inputs.keys().collect::<Vec<_>>())
        })))?;
    Ok(json!({"input": candidate, "candidate": input, "picker": {
        "execution": row.id, "node": row.node, "settings": row.settings,
        "source": if row.reason.as_deref() == Some("operator") { "operator" } else { "worker" }
    }}))
}

pub fn current(
    conn: &Connection,
    attempt: &attempts::Attempt,
) -> Result<Option<executions::Execution>, Fail> {
    let Some(id) = attempt.choice else {
        return Ok(None);
    };
    let row = executions::get(conn, id)?;
    if row.attempt != Some(attempt.id)
        || row.node.as_deref() != Some("selector")
        || row.plan_revision != attempt.plan_revision()
        || row.ticket_revision != Some(tickets::get(conn, attempt.ticket)?.revision)
        || !row.result_eligible
        || row.result.as_ref().is_none_or(|result| {
            result["infrastructure"]["outcome"] != "completed"
                || result["outputs"]["result"]["kind"] != "selection"
        })
        || (row.reason.as_deref() != Some("operator")
            && conn.query_row(
                "SELECT COALESCE(mcp, '') FROM execution WHERE id = ?1",
                [id],
                |r| r.get::<_, String>(0),
            )? != "registered")
        || row.inputs != inputs(conn, attempt)?
    {
        return Ok(None);
    }
    let value = &row.result.as_ref().expect("eligible choice")["outputs"]["result"]["value"];
    let Some(name) = value["input"].as_str() else {
        return Ok(None);
    };
    if publication(&row, name).ok().as_ref() != Some(value) {
        return Ok(None);
    }
    Ok(Some(row))
}

pub fn invalidate(tx: &Connection, attempt: &attempts::Attempt) -> Result<(), Fail> {
    if attempt.choice.is_none() {
        return Ok(());
    }
    for approval in checks::queue(tx)?
        .into_iter()
        .filter(|approval| approval.attempt == attempt.id)
    {
        checks::set_approval_state(
            tx,
            &approval,
            "withdrawn",
            attempt.ticket,
            Some("choice superseded"),
        )?;
    }
    for item in attempts::open_for_attempt(tx, attempt.id)? {
        if matches!(item.kind.as_str(), "approval" | "candidate")
            || (item.kind == "stopped" && item.reason == "limit")
        {
            attempts::resolve(tx, &item, "superseded", None)?;
        }
    }
    tx.execute(
        "UPDATE attempt SET choice = NULL, head = NULL, proof = NULL WHERE id = ?1",
        [attempt.id],
    )?;
    Ok(())
}

pub fn activate(tx: &Connection, attempt: &attempts::Attempt, id: i64) -> Result<(), Fail> {
    let mut selected = attempt.clone();
    selected.choice = Some(id);
    let row = current(tx, &selected)?.ok_or_else(|| Fail::refused("superseded_selection"))?;
    if attempt.choice == Some(id) {
        return Ok(());
    }
    invalidate(tx, attempt)?;
    let candidate =
        &row.result.as_ref().expect("choice")["outputs"]["result"]["value"]["candidate"]["value"];
    tx.execute(
        "UPDATE attempt SET choice = ?2, head = ?3, proof = ?4, base = ?5 WHERE id = ?1",
        params![
            attempt.id,
            id,
            candidate["head"].as_str(),
            candidate["proof"].as_str(),
            candidate["base"].as_str()
        ],
    )?;
    audit(
        tx,
        "candidate.selected",
        Target {
            attempt: Some(attempt.id),
            ticket: Some(attempt.ticket),
            execution: Some(id),
            ..Default::default()
        },
        None,
        row.result.unwrap(),
    )?;
    Ok(())
}
