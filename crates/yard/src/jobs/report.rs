//! Read-only publication bound by the daemon to the execution's frozen inputs.

use crate::api::Fail;
use crate::mcp::Grant;
use crate::store::{self, attempts, executions};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    outcome: String,
    report: Report,
}

#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Report {
    summary: String,
    findings: Vec<String>,
}

pub fn publish(grant: &Grant, arguments: &Value) -> Result<Value, Fail> {
    let publication: Publication =
        serde_json::from_value(arguments.clone()).map_err(|_| Fail::invalid("invalid_report"))?;
    let text_valid = |text: &str| !text.trim().is_empty() && text.len() <= 8000;
    if !text_valid(&publication.report.summary)
        || publication.report.findings.len() > 200
        || !publication
            .report
            .findings
            .iter()
            .all(|text| text_valid(text))
        || arguments.to_string().len() > 262_144
    {
        return Err(Fail::invalid("invalid_report"));
    }
    grant.project.tx(|tx| {
        let row = executions::get(tx, grant.execution)?;
        if let Some(scope) = &row.scope {
            if row.status != "running" {
                return Err(Fail::refused("execution_ended"));
            }
            if row.kind != "report"
                || scope["policy"]["node"]["access"] != "read-only"
                || scope["policy"]["node"]["outputs"]["result"] != "report"
                || publication.outcome != "complete"
            {
                return Err(Fail::refused("standalone_authority"));
            }
            if row.report.is_some() {
                return Err(Fail::refused("duplicate_report"));
            }
            let report = json!({"execution": row.id, "attempt": row.attempt, "scope": scope,
                "outcome": publication.outcome, "report": publication.report});
            tx.execute(
                "UPDATE execution SET report = ?2 WHERE id = ?1 AND report IS NULL",
                rusqlite::params![row.id, report.to_string()],
            )?;
            store::audit(
                tx,
                "report.published",
                store::Target {
                    execution: Some(row.id),
                    attempt: row.attempt,
                    ..Default::default()
                },
                None,
                json!({"outcome": publication.outcome}),
            )?;
            return Ok(json!({"published": row.id}));
        }
        let attempt = attempts::get(tx, row.required_attempt()?)?;
        if row.status != "running" || attempt.state != "live" {
            return Err(Fail::refused("execution_ended"));
        }
        if row.report.is_some() {
            return Err(Fail::refused("duplicate_report"));
        }
        let frozen = attempts::for_execution(tx, &row)?;
        let plan = crate::plan::Plan::admitted(
            frozen
                .admission
                .as_ref()
                .ok_or_else(|| Fail::refused("unbound_report"))?,
        )
        .map_err(|_| Fail::refused("unbound_report"))?;
        let node = row
            .node
            .as_ref()
            .and_then(|name| plan.nodes.get(name))
            .ok_or_else(|| Fail::refused("unbound_report"))?;
        if node.outputs.get("result") != Some(&crate::plan::Output::Report)
            || !node.outcomes.contains(&publication.outcome)
        {
            return Err(Fail::invalid("undeclared_outcome"));
        }
        if executions::plan_superseded(tx, &attempt, &row)?
            || row.ticket_revision.is_some_and(|revision| {
                attempts::superseded_by_edit(tx, attempt.id, attempt.ticket, revision)
                    .unwrap_or(true)
            })
        {
            return Err(Fail::refused("superseded_report"));
        }
        let report = json!({
            "execution": row.id, "node": row.node, "plan_revision": row.plan_revision,
            "ticket_revision": row.ticket_revision, "inputs": row.inputs,
            "base": row.base, "head": row.head, "proof": row.proof,
            "outcome": publication.outcome, "report": publication.report,
        });
        tx.execute(
            "UPDATE execution SET report = ?2 WHERE id = ?1",
            rusqlite::params![row.id, report.to_string()],
        )?;
        store::audit(
            tx,
            "report.published",
            store::Target {
                execution: Some(row.id),
                attempt: row.attempt,
                ticket: Some(attempt.ticket),
                ..Default::default()
            },
            None,
            json!({"outcome": publication.outcome}),
        )?;
        Ok(json!({"published": row.id}))
    })
}
