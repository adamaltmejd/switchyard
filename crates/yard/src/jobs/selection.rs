//! Worker publication and trusted operator choice use the same frozen inputs.
use crate::api::Fail;
use crate::daemon::Project;
use crate::mcp::Grant;
use crate::store::{attempts, executions, selections, ticket_id, tickets};
use serde_json::{Value, json};

pub fn publish(grant: &Grant, arguments: &Value) -> Result<Value, Fail> {
    crate::mcp::strict(arguments, &["candidate"])?;
    let candidate = arguments["candidate"]
        .as_str()
        .ok_or_else(|| Fail::invalid("candidate is an input name"))?;
    grant.project.tx(|tx| {
        let row = executions::get(tx, grant.execution)?;
        let attempt = attempts::get(tx, row.required_attempt()?)?;
        if row.status != "running"
            || attempt.state != "live"
            || row.node.as_deref() != Some("selector")
        {
            return Err(Fail::refused("execution_ended"));
        }
        if row.report.is_some() {
            return Err(Fail::refused("duplicate_selection"));
        }
        let current = selections::inputs(tx, &attempt)?;
        if row.plan_revision != attempt.plan_revision() || row.inputs != current {
            return Err(Fail::stale("inputs", json!(row.inputs), json!(current))
                .with(json!({"identity": "inputs", "expected": row.inputs, "current": current})));
        }
        let value = selections::publication(&row, candidate)?;
        tx.execute(
            "UPDATE execution SET report = ?2 WHERE id = ?1 AND report IS NULL",
            rusqlite::params![row.id, value.to_string()],
        )?;
        Ok(json!({"published": row.id}))
    })
}

pub fn operator(project: &Project, params: &Value) -> Result<Value, Fail> {
    let ticket = params["ticket"]
        .as_str()
        .and_then(ticket_id)
        .ok_or_else(|| Fail::invalid("ticket is Y-N"))?;
    let name = params["candidate"]
        .as_str()
        .ok_or_else(|| Fail::invalid("candidate is an input name"))?;
    project.tx(|tx| {
        super::admit::refuse_during_intent(tx, ticket)?;
        let attempt = attempts::live_for(tx, ticket)?.ok_or_else(|| Fail::refused("no_live_attempt"))?;
        let current_ticket = tickets::get(tx, ticket)?;
        let plan = crate::plan::Plan::admitted(attempt.admission.as_ref().ok_or_else(|| Fail::refused("no_selector"))?)
            .map_err(|error| Fail::invalid(error.to_string()))?;
        if plan.selector().is_none() { return Err(Fail::refused("no_selector")); }
        for (key, current) in [
            ("attempt", json!(attempt.id)), ("revision", json!(current_ticket.revision)),
            ("plan_revision", json!(attempt.plan_revision())), ("target", json!(attempt.target_branch)),
            ("choice", json!(attempt.choice.unwrap_or(0))),
        ] {
            if params[key] != current {
                return Err(Fail::stale(key, params[key].clone(), current.clone())
                    .with(json!({"identity": key, "expected": params[key], "current": current})));
            }
        }
        let inputs = selections::inputs(tx, &attempt)?;
        if inputs.is_none() || params["inputs"] != json!(inputs) {
            return Err(Fail::stale("inputs", params["inputs"].clone(), json!(inputs))
                .with(json!({"identity": "inputs", "expected": params["inputs"], "current": inputs})));
        }
        if executions::attempt_work(tx, attempt.id)?.iter().any(|row| row.status == "running" || row.reserved) {
            return Err(Fail::refused("selection_owned"));
        }
        // Validate the reference before recording any intent or retiring an approval.
        let declared = inputs.as_ref().and_then(|inputs| inputs.get(name));
        if declared.is_none_or(|input| input["kind"] != "candidate") {
            return Err(Fail::refused("selection_input").with(json!({
                "reason": "selection_input", "identity": "candidate", "expected": name, "current": inputs
            })));
        }
        let id = executions::start(tx, executions::Start {
            attempt: Some(attempt.id), kind: "review", node: Some("selector"), name: Some("selector"),
            reason: Some("operator"), ticket_revision: Some(current_ticket.revision), ticket: Some(ticket),
            ..Default::default()
        })?;
        let row = executions::get(tx, id)?;
        let value = selections::publication(&row, name)?;
        executions::end(tx, id, executions::End {
            outcome: "selected", infrastructure: Some("completed"),
            output: Some(json!({"kind": "selection", "value": value})), ticket: Some(ticket),
            ..Default::default()
        })?;
        // This execution has no external effect, handle, grant or owner.
        tx.execute("UPDATE execution SET teardown = 1 WHERE id = ?1", [id])?;
        selections::activate(tx, &attempt, id)?;
        Ok(json!({"choice": id, "candidate": declared}))
    })
}
