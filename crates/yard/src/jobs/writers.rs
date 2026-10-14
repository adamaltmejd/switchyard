//! Independent candidate producers share capacity, never a writable mount.
use super::{Loaded, admit, spawn, supervise};
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::plan::{Access, Plan, Runner};
use crate::store::{attempts, executions, tickets};
use serde_json::json;
use std::sync::Arc;

/// A ticket edit takes its stopped exits only after every old owner drains.
/// Ownership attention belongs to checked release, never to an edit.
pub fn steer(tx: &rusqlite::Connection, attempt: &attempts::Attempt) -> Result<Vec<i64>, Fail> {
    let rows = executions::attempt_work(tx, attempt.id)?;
    let readers = rows
        .iter()
        .filter(|row| row.status == "running" && row.kind == "review")
        .map(|row| row.id)
        .collect();
    if rows
        .iter()
        .any(|row| row.status == "running" || row.reserved)
    {
        return Ok(readers);
    }
    let ticket = tickets::get(tx, attempt.ticket)?;
    for item in attempts::open_for_attempt(tx, attempt.id)? {
        if !matches!(item.kind.as_str(), "stopped" | "red") || item.reason == "ownership" {
            continue;
        }
        let source = match item.execution {
            Some(id) => rows.iter().find(|row| row.id == id),
            None if item.reason == "candidates" => {
                rows.iter().rev().find(|row| row.kind == "implementation")
            }
            None => None,
        };
        if source.is_some_and(|row| {
            row.ticket_revision
                .is_some_and(|read| read < ticket.revision)
        }) {
            attempts::resolve(tx, &item, "edit", None)?;
            if item.kind == "stopped" && item.reason == "timeout" {
                attempts::renew_clock(tx, attempt.id)?;
            }
        }
    }
    Ok(readers)
}

pub async fn advance(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
    plan: &Plan,
) -> Result<(), Fail> {
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let rows = project.read(|conn| executions::attempt_work(conn, attempt.id))?;
    project.tx(|tx| steer(tx, attempt))?;
    if project.read(|conn| {
        Ok(attempts::open_for_attempt(conn, attempt.id)?
            .iter()
            .any(|item| item.kind != "proposal" || item.reason == "edit"))
    })? {
        return Ok(());
    }
    if !attempt.lane {
        let default = super::load(daemon, project).await?;
        if !admit::take_lane(daemon, project, &default, attempt.id)? {
            return Ok(());
        }
    }
    // Edits and live definition handoffs drain every old writer first. Each
    // successor then rereads the frozen revision in its own retained workspace.
    let changed = rows.iter().any(|row| {
        row.kind == "implementation"
            && (row.ticket_revision != Some(ticket.revision)
                || row.plan_revision != attempt.plan_revision())
    });
    if changed
        && rows
            .iter()
            .any(|row| row.status == "running" || row.reserved)
    {
        return Ok(());
    }
    let mut unfinished = false;
    for (name, node) in &plan.nodes {
        let current = rows.iter().rev().find(|row| {
            row.node.as_deref() == Some(name)
                && row.ticket_revision == Some(ticket.revision)
                && row.plan_revision == attempt.plan_revision()
        });
        if let Some(row) = current {
            if row.status == "running" || row.reserved {
                unfinished = true;
                continue;
            }
            if node.access == Access::WriteAllowed {
                if !row.result_eligible {
                    if row.outcome.as_deref() == Some("dirty")
                        && row.reason.as_deref() != Some("dirty")
                    {
                        let execution = project.tx(|tx| {
                            attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
                            supervise::start_writer(tx, attempt, name, "dirty", None)
                        })?;
                        spawn(daemon, project, "implementation", execution);
                    }
                    unfinished = true;
                }
                continue;
            }
            if row.result_eligible {
                let source = node.inputs["candidate"]
                    .node
                    .as_deref()
                    .expect("validated producer");
                let latest = rows.iter().rev().find(|producer| {
                    producer.node.as_deref() == Some(source)
                        && producer.ticket_revision == Some(ticket.revision)
                        && producer.plan_revision == attempt.plan_revision()
                        && producer.result_eligible
                });
                if latest.is_some_and(|producer| {
                    row.inputs.as_ref().is_some_and(|inputs| {
                        inputs["candidate"]["source"]["execution"] == producer.id
                    })
                }) {
                    continue;
                }
            }
        }
        if node.access == Access::WriteAllowed {
            let execution = project.tx(|tx| {
                attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
                supervise::start_writer(
                    tx,
                    attempt,
                    name,
                    if changed { "edit" } else { "first" },
                    None,
                )
            })?;
            spawn(daemon, project, "implementation", execution);
            unfinished = true;
            continue;
        }
        let source = node.inputs["candidate"]
            .node
            .as_deref()
            .expect("validated producer");
        let Some(producer) = rows.iter().rev().find(|row| {
            row.node.as_deref() == Some(source)
                && row.ticket_revision == Some(ticket.revision)
                && row.plan_revision == attempt.plan_revision()
                && row.result_eligible
        }) else {
            unfinished = true;
            continue;
        };
        let candidate =
            &producer.result.as_ref().expect("eligible result")["outputs"]["result"]["value"];
        let index = name
            .strip_prefix("review-")
            .and_then(|i| i.parse::<usize>().ok())
            .ok_or_else(|| Fail::refused("invalid reader"))?;
        let seat = loaded
            .config
            .workflow(&attempt.workflow)?
            .review
            .get(index)
            .ok_or_else(|| Fail::refused("undeclared reader seat"))?;
        let Runner::Agent { agent, .. } = &node.runner else {
            return Err(Fail::refused("invalid reader"));
        };
        let settings = serde_json::to_value(&loaded.config.agents[agent]).expect("agent settings");
        let execution = project.tx(|tx| {
            attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
            executions::start(
                tx,
                executions::Start {
                    attempt: Some(attempt.id),
                    kind: "review",
                    node: Some(name),
                    name: Some(seat),
                    reason: Some("first"),
                    base: candidate["base"].as_str(),
                    head: candidate["head"].as_str(),
                    proof: candidate["proof"].as_str(),
                    ticket_revision: Some(ticket.revision),
                    ticket: Some(ticket.id),
                    agent: Some((agent, &settings)),
                    digest: Some(
                        &loaded
                            .config
                            .review_digest(loaded.config.workflow(&attempt.workflow)?),
                    ),
                    harness_version: crate::harness::get(&loaded.config.agents[agent].harness)
                        .map(|h| h.version()),
                    ..Default::default()
                },
            )
        })?;
        spawn(daemon, project, "review", execution);
        unfinished = true;
    }
    if !unfinished
        && !rows
            .iter()
            .any(|row| row.status == "running" || row.reserved)
    {
        project.tx(|tx| {
            attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
            attempts::set_lane(tx, attempt.id, false)?;
            attempts::raise(tx, attempts::Raise {
                kind: "stopped", reason: "candidates", attempt: Some(attempt.id), ticket: Some(ticket.id),
                execution: None, payload: json!({"candidates": rows.iter().filter(|row| row.kind == "implementation"
                    && row.result_eligible && row.ticket_revision == Some(ticket.revision)
                    && row.plan_revision == attempt.plan_revision()).map(|row| row.id).collect::<Vec<_>>()}), text: None,
            })?;
            Ok(())
        })?;
    }
    Ok(())
}
