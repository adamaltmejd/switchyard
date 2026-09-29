//! Review: one execution per seat, each in its own box on a fresh read-only
//! checkout of the head. A seat publishes once; the publication is the
//! decision.

use super::load;
use super::supervise::{self, Worker};
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::store::{attempts, checks, executions, ticket_name, tickets};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

pub async fn run(daemon: &Arc<Daemon>, project: &Arc<Project>, execution: i64) -> Result<(), Fail> {
    let loaded = load(daemon, project).await?;
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let seat_name = row.name.clone().unwrap_or_default();
    let seat = loaded
        .config
        .seats
        .get(&seat_name)
        .ok_or_else(|| Fail::invalid(format!("seat {seat_name} is no longer configured")))?;
    let agent = &loaded.config.agents[&seat.agent];
    let connection = crate::harness::connection(&agent.provider)
        .ok_or_else(|| Fail::invalid(format!("connection {:?} is unknown", agent.provider)))?;
    let harness = crate::harness::get(&agent.harness)
        .ok_or_else(|| Fail::invalid(format!("harness {:?} is unknown", agent.harness)))?;
    let head = row.head.clone().unwrap_or_default();
    let base = row.base.clone().unwrap_or_default();

    let dir = project
        .attempt_dir(attempt.id)
        .join("reviews")
        .join(execution.to_string());
    let checkout = dir.join("checkout");
    let state = dir.join("state");
    let input = dir.join("input");
    let _ = std::fs::remove_dir_all(&dir);
    daemon
        .git
        .clone_detached(&project.canonical_dir(), &checkout, &head)
        .await?;
    let stage = crate::harness::Stage {
        state: &state,
        input: &input,
        connection,
    };
    harness.stage(&stage).map_err(|error| error.to_string())?;

    let stat = daemon
        .git
        .diff_stat(&project.canonical_dir(), &base, &head)
        .await?;
    let prompt = format!(
        "You are the {seat_name} reviewer for ticket {}: {}\n\n{}\n\n{}\n\n\
         The candidate is checked out read-only in /workspace at {head}; its base is {base}.\n\
         Changed files:\n{stat}\n\
         Read the change (`git diff {base} {head}`), then publish your review exactly once with \
         yard_publish_review. Give every finding a priority from P0 (worst) to P3, and a file and \
         line where you can. Publish an empty findings list if you find nothing.\n",
        ticket_name(ticket.id),
        ticket.title,
        ticket.body,
        seat.instructions
    );
    let argv = harness
        .argv(&crate::harness::Launch {
            provider: &agent.provider,
            model: &agent.model,
            effort: agent.effort.as_deref(),
            resume: None,
            prompt: &prompt,
        })
        .map_err(Fail::invalid)?;
    let image = supervise::image(daemon, project, &loaded).await?;
    let bearer = daemon.grants.issue(crate::mcp::Grant {
        project: project.clone(),
        execution,
        kind: crate::mcp::Kind::Review,
    });
    let model = harness
        .route(&stage, &daemon.machine)
        .map_err(Fail::refused)?;
    let secrets = vec![
        (crate::harness::BEARER_VAR.to_string(), bearer),
        model.secret.clone(),
    ];
    let spec = supervise::worker_spec(
        daemon,
        project,
        &Worker {
            execution,
            image: &image,
            workspace: &checkout,
            read_only: true,
            proof: None,
            harness,
            stage,
            model: &model,
            egress: &[],
        },
    );
    let live = supervise::up_worker(daemon, project, execution, &spec, &secrets).await?;
    let timeout = Duration::from_secs(loaded.config.review.timeout_minutes * 60);
    let run = supervise::run_harness(
        daemon,
        project,
        execution,
        crate::mcp::Kind::Review,
        harness,
        &argv,
        &supervise::transcript(project, attempt.id, execution),
        timeout,
        tokio::time::Instant::now() + timeout,
    )
    .await;
    daemon.grants.revoke(project, execution);
    let _ = live.down(supervise::DOWN_TIMEOUT).await;
    let _ = std::fs::remove_dir_all(&dir);
    let run = run?;

    // The box is gone: only now does the verdict count toward anything.
    project.tx(|tx| {
        let published = checks::for_execution(tx, execution)?;
        let failure = match (&published, &run.registered, &run.terminal) {
            (Some(_), _, _) => None,
            (None, Some(crate::harness::Registration::Refused(reason)), _) => Some(reason.clone()),
            (None, _, _) if run.timed_out => {
                Some("the seat ran past its timeout without publishing".to_string())
            }
            (None, _, Some(crate::harness::Event::Failed { message, .. })) => Some(message.clone()),
            (None, _, _) => Some("the seat ended without publishing".to_string()),
        };
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: published
                    .as_ref()
                    .map_or("error", |check| check.verdict.as_str()),
                detail: failure.as_deref(),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            },
        )?;
        if let Some(detail) = failure {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "red",
                    reason: "review",
                    ticket: Some(ticket.id),
                    attempt: Some(attempt.id),
                    execution: Some(execution),
                    payload: json!({ "seat": seat_name, "detail": detail }),
                    text: Some(&detail),
                },
            )?;
        }
        Ok(())
    })
}

/// `yard_publish_review`: findings, the seat's check and its event in one
/// transaction. Once per execution.
pub fn publish(grant: &crate::mcp::Grant, arguments: &Value) -> Result<Value, Fail> {
    let object = crate::mcp::strict(arguments, &["findings"])?;
    let findings: Vec<checks::Finding> = match object.get("findings") {
        Some(Value::Array(items)) => {
            let mut findings = Vec::new();
            for item in items {
                crate::mcp::strict(item, &["priority", "file", "line", "category", "body"])?;
                let finding: checks::Finding = serde_json::from_value(item.clone())
                    .map_err(|error| Fail::invalid(format!("finding: {error}")))?;
                if crate::config::priority(&finding.priority).is_none() {
                    return Err(Fail::invalid(format!(
                        "priority {:?} is not P0 to P3",
                        finding.priority
                    )));
                }
                if finding.line.is_some_and(|line| line < 1) {
                    return Err(Fail::invalid("line starts at 1"));
                }
                findings.push(finding);
            }
            findings
        }
        _ => return Err(Fail::invalid("findings is a list")),
    };
    let project = &grant.project;
    let loaded = project
        .loaded
        .lock()
        .expect("loaded lock")
        .clone()
        .ok_or_else(|| Fail::refused("the configuration is not loaded"))?;
    let blocking = loaded.config.review.blocking;
    let check = project.tx(|tx| {
        if checks::for_execution(tx, grant.execution)?.is_some() {
            return Err(Fail::refused("this seat has already published"));
        }
        let row = executions::get(tx, grant.execution)?;
        let attempt = attempts::get(tx, row.attempt)?;
        let image: Option<String> = tx.query_row(
            "SELECT image_id FROM execution WHERE id = ?1",
            [grant.execution],
            |row| row.get(0),
        )?;
        let blocked = findings.iter().any(|finding| {
            crate::config::priority(&finding.priority).is_some_and(|priority| priority <= blocking)
        });
        let check = checks::record(
            tx,
            checks::Record {
                execution: grant.execution,
                kind: "review",
                name: row.name.as_deref().unwrap_or_default(),
                input: &checks::Input {
                    attempt: attempt.id,
                    base: row.base.clone().unwrap_or_default(),
                    head: row.head.clone().unwrap_or_default(),
                    proof: row.proof.clone().unwrap_or_default(),
                    ticket_revision: row.ticket_revision.unwrap_or_default(),
                    digest: row.digest.clone().unwrap_or_default(),
                },
                verdict: if blocked { "fail" } else { "pass" },
                image_id: image.as_deref(),
                round: row.round,
                ticket: attempt.ticket,
            },
        )?;
        for finding in &findings {
            checks::add_finding(tx, check, finding)?;
        }
        Ok(check)
    })?;
    Ok(json!({ "published": check, "findings": findings.len() }))
}

/// A round that blocked: a repair, or `stopped:limit` at `max_rounds`.
pub fn blocked(
    project: &Project,
    loaded: &super::Loaded,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    blocked: &[checks::Check],
) -> Result<(), Fail> {
    let mut findings = Vec::new();
    for check in blocked {
        for finding in project.read(|conn| checks::findings(conn, check.id))? {
            findings.push(json!({ "seat": check.name, "finding": finding }));
        }
    }
    let rounds = attempt.rounds + 1;
    let limit = i64::from(loaded.config.review.max_rounds) + attempt.extra_rounds;
    project.tx(|tx| {
        tx.execute(
            "UPDATE attempt SET rounds = ?2 WHERE id = ?1",
            rusqlite::params![attempt.id, rounds],
        )?;
        if rounds >= limit {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "stopped",
                    reason: "limit",
                    ticket: Some(ticket.id),
                    attempt: Some(attempt.id),
                    execution: blocked.last().map(|check| check.execution),
                    payload: json!({ "rounds": rounds, "findings": findings }),
                    text: None,
                },
            )?;
            return Ok(());
        }
        attempts::set_next(
            tx,
            attempt.id,
            Some(&json!({ "reason": "repair", "findings": findings })),
        )
    })
}
