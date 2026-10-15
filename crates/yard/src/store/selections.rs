//! Choices refer to immutable publications; they never create candidates.
use super::{Target, attempts, audit, checks, executions, tickets};
use crate::api::Fail;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

pub fn inputs(conn: &Connection, attempt: &attempts::Attempt) -> Result<Option<Value>, Fail> {
    let findings = attempt
        .admission
        .as_ref()
        .map(crate::plan::Plan::admitted)
        .transpose()
        .map_err(|error| Fail::refused(error.to_string()))?
        .is_some_and(|plan| plan.finding_selector().is_some());
    executions::binding(
        conn,
        attempt,
        &executions::Start {
            kind: "review",
            node: Some("selector"),
            ticket_revision: Some(tickets::get(conn, attempt.ticket)?.revision),
            base: findings.then_some(attempt.base.as_str()),
            head: findings
                .then(|| attempt.candidate().map(|(_, head)| head))
                .flatten(),
            proof: findings.then_some(attempt.proof.as_deref().unwrap_or_default()),
            ..Default::default()
        },
    )
    .map(|binding| binding.map(|(_, inputs)| inputs))
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FindingChoice {
    groups: Vec<FindingGroup>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct FindingGroup {
    reason: String,
    sources: Vec<Value>,
}

pub fn finding_publication(row: &executions::Execution, arguments: &Value) -> Result<Value, Fail> {
    let eligible: Vec<_> = row
        .inputs
        .as_ref()
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|inputs| inputs.values())
        .filter(|input| input["kind"] == "verdict")
        .flat_map(|input| {
            input["value"]["review"]["findings"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .map(|finding| &finding["source"])
        .collect();
    let refused = |identity: &str, expected: Value| {
        Fail::refused("selection_input")
            .with(json!({"reason": "selection_input", "identity": identity,
            "expected": expected, "current": eligible}))
    };
    let choice: FindingChoice = serde_json::from_value(arguments.clone())
        .map_err(|_| refused("groups", arguments.clone()))?;
    if choice.groups.len() > 200 {
        return Err(refused("groups", arguments.clone()));
    }
    let mut seen = Vec::new();
    for group in &choice.groups {
        if group.reason.trim().is_empty() || group.reason.len() > 32768 || group.sources.is_empty()
        {
            return Err(refused("group", json!(group)));
        }
        for source in &group.sources {
            if !eligible.contains(&source) || seen.contains(&source) {
                return Err(refused("source", source.clone()));
            }
            seen.push(source);
        }
    }
    Ok(json!({"groups": choice.groups, "inputs": row.inputs,
        "picker": {"execution": row.id, "node": row.node, "settings": row.settings, "source": "worker"}}))
}

pub fn finding_current(
    conn: &Connection,
    attempt: &attempts::Attempt,
    row: &executions::Execution,
) -> Result<bool, Fail> {
    if row.attempt != Some(attempt.id)
        || row.node.as_deref() != Some("selector")
        || row.plan_revision != attempt.plan_revision()
        || row.ticket_revision != Some(tickets::get(conn, attempt.ticket)?.revision)
        || !row.result_eligible
        || row.inputs != inputs(conn, attempt)?
        || conn.query_row(
            "SELECT COALESCE(mcp, '') FROM execution WHERE id = ?1",
            [row.id],
            |r| r.get::<_, String>(0),
        )? != "registered"
    {
        return Ok(false);
    }
    let Some(result) = &row.result else {
        return Ok(false);
    };
    let output = &result["outputs"]["result"];
    Ok(result["infrastructure"]["outcome"] == "completed"
        && output["kind"] == "finding-selection"
        && finding_publication(row, &json!({"groups": output["value"]["groups"]}))
            .ok()
            .as_ref()
            == Some(&output["value"]))
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
    let attempt = attempts::get(tx, attempt.id)?;
    let mut selected = attempt.clone();
    selected.choice = Some(id);
    let row = current(tx, &selected)?.ok_or_else(|| Fail::refused("superseded_selection"))?;
    if attempt.choice == Some(id) {
        return Ok(());
    }
    if attempt.next.is_some() {
        let previous = attempt
            .choice
            .map(|choice| executions::get(tx, choice))
            .transpose()?;
        let expected = previous.as_ref().and_then(|choice| {
            choice
                .result
                .as_ref()
                .map(|result| &result["outputs"]["result"]["value"]["candidate"])
        });
        let candidate =
            &row.result.as_ref().expect("choice")["outputs"]["result"]["value"]["candidate"];
        if expected != Some(candidate) {
            return Err(Fail::refused("pending_continuation").with(json!({
                "reason": "pending_continuation", "identity": "pending_continuation",
                "expected": expected, "current": candidate
            })));
        }
    }
    invalidate(tx, &attempt)?;
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
