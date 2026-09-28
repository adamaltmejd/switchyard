//! The merge queue: approved candidates in approval order, landed one at a
//! time on their own full gate run.

use super::{Loaded, load, spawn};
use crate::api::Fail;
use crate::config::Approve;
use crate::daemon::{Daemon, Project};
use crate::git::{Merge, Opts};
use crate::store::{self, attempts, checks, executions, ticket_name, tickets};
use serde_json::json;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::sync::Arc;

pub fn landing_dir(project: &Project, landing: i64) -> PathBuf {
    project.local().join("landings").join(landing.to_string())
}

/// Start the next landing, if the queue is free.
pub async fn next(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
) -> Result<(), Fail> {
    let busy = project.read(|conn| {
        Ok(
            !executions::query(conn, "kind = 'landing' AND status = 'running'", [])?.is_empty()
                || !executions::open_intents(conn)?.is_empty(),
        )
    })?;
    if busy {
        return Ok(());
    }
    for approval in project.read(checks::queue)? {
        let attempt = project.read(|conn| attempts::get(conn, approval.attempt))?;
        let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
        if let Err(reason) = holds(daemon, project, loaded, &approval, &attempt, &ticket).await? {
            project.tx(|tx| {
                checks::set_approval_state(tx, &approval, "withdrawn", ticket.id, Some(&reason))
            })?;
            continue;
        }
        let blocked = project.read(|conn| {
            Ok(attempts::open_attention(conn)?
                .iter()
                .any(|item| item.ticket == Some(ticket.id) && item.kind == "red"))
        })?;
        if blocked {
            continue;
        }
        let execution = project.tx(|tx| {
            executions::start(
                tx,
                executions::Start {
                    attempt: attempt.id,
                    kind: "landing",
                    reason: Some("queue"),
                    base: Some(&approval.base),
                    head: Some(&approval.head),
                    ticket_revision: Some(approval.ticket_revision),
                    approval: Some(approval.id),
                    ticket: Some(ticket.id),
                    ..Default::default()
                },
            )
        })?;
        spawn(daemon, project, "landing", execution);
        return Ok(());
    }
    Ok(())
}

/// Whether an approval still holds against current rows and policy.
async fn holds(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    approval: &checks::Approval,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
) -> Result<Result<(), String>, Fail> {
    if attempt.state != "live" {
        return Ok(Err("the attempt ended".into()));
    }
    if attempt.candidate() != Some((approval.base.as_str(), approval.head.as_str())) {
        return Ok(Err("the candidate changed".into()));
    }
    if ticket.revision != approval.ticket_revision {
        return Ok(Err("the ticket changed".into()));
    }
    if approval.gate_digest != loaded.gate_digest {
        return Ok(Err("the gate configuration changed".into()));
    }
    if approval.review_digest != loaded.review_digest(&attempt.workflow)? {
        return Ok(Err("the review configuration changed".into()));
    }
    if approval.actor == "auto" {
        if loaded.config.approve != Approve::Auto {
            return Ok(Err("approve is now manual".into()));
        }
        let protected =
            super::protected_paths(daemon, project, loaded, &approval.base, &approval.head).await?;
        if !protected.is_empty() {
            return Ok(Err(format!(
                "the candidate touches protected paths: {}",
                protected.join(", ")
            )));
        }
    }
    Ok(Ok(()))
}

pub async fn land(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    execution: i64,
) -> Result<(), Fail> {
    let loaded = load(daemon, project).await?;
    let row = project.read(|conn| executions::get(conn, execution))?;
    let approval = project
        .read(|conn| checks::approval(conn, row.approval.expect("a landing binds an approval")))?;
    let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let canonical = project.canonical_dir();
    let target_ref = crate::git::target_ref(&loaded.branch);
    let dir = landing_dir(project, execution);
    let lock = super::supervise::hold_lock(&dir)?;
    let opts = Opts {
        inherit: Some(lock.as_raw_fd()),
        never_kill: true,
    };
    let head = approval.head.clone();

    let (target, merged) = {
        let _canonical = project.canonical.lock().await;
        let target = daemon
            .git
            .rev_parse(&canonical, &target_ref)
            .await?
            .ok_or_else(|| Fail::refused("canonical has no target"))?;
        let merged = if daemon.git.is_ancestor(&canonical, &target, &head).await? {
            head.clone()
        } else {
            match daemon.git.merge_tree(&canonical, &target, &head).await? {
                Merge::Conflict { paths } => {
                    return returned(
                        daemon,
                        project,
                        &loaded,
                        execution,
                        &attempt,
                        &ticket,
                        &approval,
                        &target,
                        "conflict",
                        &format!(
                            "The candidate does not merge cleanly. Conflicting paths:\n{}",
                            paths.join("\n")
                        ),
                    )
                    .await;
                }
                Merge::Clean { tree } => {
                    let message = format!("Merge {}: {}", ticket_name(ticket.id), ticket.title);
                    daemon
                        .git
                        .commit_tree(&canonical, &tree, &[&target, &head], &message)
                        .await?
                }
            }
        };
        // Keep the merged commit reachable for the gate checkouts.
        daemon
            .git
            .update_ref(
                &canonical,
                &format!("refs/yard/landings/{execution}"),
                &merged,
                None,
                Opts::default(),
            )
            .await?;
        (target, merged)
    };

    if let Err(reason) = holds(daemon, project, &loaded, &approval, &attempt, &ticket).await? {
        return withdraw(
            daemon, project, &loaded, execution, &approval, &ticket, &reason,
        )
        .await;
    }

    for gate in &loaded.config.gates {
        let child = project.tx(|tx| {
            executions::start(
                tx,
                executions::Start {
                    attempt: attempt.id,
                    parent: Some(execution),
                    kind: "gate",
                    reason: Some("landing"),
                    base: Some(&target),
                    head: Some(&merged),
                    ticket_revision: Some(ticket.revision),
                    digest: Some(&loaded.gate_digest),
                    name: Some(&gate.name),
                    ticket: Some(ticket.id),
                    ..Default::default()
                },
            )
        })?;
        super::supervise::gate(daemon, project, child).await?;
        let result = project.read(|conn| executions::get(conn, child))?;
        match result.outcome.as_deref() {
            Some("pass") => {}
            Some("fail") => {
                let detail = format!(
                    "The gate {} failed on the merged ref {merged}:\n{}",
                    gate.name,
                    result.detail.unwrap_or_default()
                );
                return returned(
                    daemon, project, &loaded, execution, &attempt, &ticket, &approval, &target,
                    "red", &detail,
                )
                .await;
            }
            _ => {
                let detail = format!(
                    "The gate {} could not run: {}",
                    gate.name,
                    result.detail.unwrap_or_default()
                );
                return project.tx(|tx| {
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: "error",
                            detail: Some(&detail),
                            ticket: Some(ticket.id),
                            ..Default::default()
                        },
                    )?;
                    attempts::raise(
                        tx,
                        attempts::Raise {
                            kind: "red",
                            reason: "landing",
                            ticket: Some(ticket.id),
                            attempt: None,
                            execution: Some(execution),
                            payload: json!({ "detail": detail, "gate": gate.name }),
                            text: Some(&detail),
                        },
                    )?;
                    Ok(())
                });
            }
        }
    }

    let _canonical = project.canonical.lock().await;
    // Re-read the approval and record the intent, serialised with every
    // command that could invalidate it.
    let policy = holds(daemon, project, &loaded, &approval, &attempt, &ticket).await?;
    let recorded = project.tx(|tx| {
        let current = checks::approval(tx, approval.id)?;
        let ticket_now = tickets::get(tx, ticket.id)?;
        let attempt_now = attempts::get(tx, attempt.id)?;
        let reason = match &policy {
            Err(reason) => Some(reason.clone()),
            Ok(()) if current.state != "active" => Some("the approval was withdrawn".into()),
            Ok(()) if ticket_now.revision != approval.ticket_revision => {
                Some("the ticket changed".into())
            }
            Ok(()) if attempt_now.state != "live" => Some("the attempt ended".into()),
            Ok(()) => None,
        };
        if reason.is_some() {
            return Ok(reason);
        }
        executions::set_intent(tx, execution, &target, &merged, ticket.id, attempt.id)?;
        Ok(None)
    })?;
    if let Some(reason) = recorded {
        return withdraw(
            daemon, project, &loaded, execution, &approval, &ticket, &reason,
        )
        .await;
    }
    let moved = daemon
        .git
        .update_ref(&canonical, &target_ref, &merged, Some(&target), opts)
        .await;
    match moved {
        Ok(false) => {
            // The target moved under the landing: retired silently, re-queued.
            return project.tx(|tx| {
                executions::resolve_intent(tx, execution)?;
                executions::end(
                    tx,
                    execution,
                    executions::End {
                        outcome: "retired",
                        ticket: Some(ticket.id),
                        ..Default::default()
                    },
                )
            });
        }
        Ok(true) => {}
        Err(_) => {
            // Command success is never read as a landing: decide from canonical.
            drop(lock);
            let row = project.read(|conn| executions::get(conn, execution))?;
            super::reconcile::decide_intent(daemon, project, &row).await?;
            return Ok(());
        }
    }
    if !daemon
        .git
        .is_ancestor_with(&canonical, &head, &target_ref, opts)
        .await?
    {
        return Err(Fail::new(
            "internal",
            format!("canonical does not contain {head} after the landing"),
        ));
    }
    project.tx(|tx| record_landing(tx, execution))
}

/// Record a landing once: close the ticket, end the attempt, resolve the
/// intent.
pub fn record_landing(tx: &rusqlite::Connection, execution: i64) -> Result<(), Fail> {
    let row = executions::get(tx, execution)?;
    let attempt = attempts::get(tx, row.attempt)?;
    let approval = checks::approval(tx, row.approval.expect("a landing binds an approval"))?;
    executions::resolve_intent(tx, execution)?;
    if row.status == "running" {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: "landed",
                ticket: Some(attempt.ticket),
                ..Default::default()
            },
        )?;
    } else {
        tx.execute(
            "UPDATE execution SET outcome = 'landed' WHERE id = ?1",
            [execution],
        )?;
    }
    checks::set_approval_state(tx, &approval, "landed", attempt.ticket, None)?;
    attempts::end(tx, attempt.id, "landed")?;
    tx.execute(
        "UPDATE ticket SET state = 'done', closed_at = ?2, close_reason = 'landed' WHERE id = ?1",
        rusqlite::params![attempt.ticket, store::now()],
    )?;
    store::audit(
        tx,
        "landing.recorded",
        store::Target {
            ticket: Some(attempt.ticket),
            attempt: Some(attempt.id),
            execution: Some(execution),
            attention: None,
        },
        None,
        json!({ "old": row.intent_old, "merged": row.intent_merged, "head": row.head }),
    )?;
    store::audit(
        tx,
        "ticket.done",
        store::Target {
            ticket: Some(attempt.ticket),
            ..Default::default()
        },
        None,
        json!({ "by": "landing" }),
    )?;
    Ok(())
}

async fn withdraw(
    daemon: &Daemon,
    project: &Project,
    _loaded: &Loaded,
    execution: i64,
    approval: &checks::Approval,
    ticket: &tickets::Ticket,
    reason: &str,
) -> Result<(), Fail> {
    let _ = daemon;
    project.tx(|tx| {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: "withdrawn",
                detail: Some(reason),
                ticket: Some(ticket.id),
                ..Default::default()
            },
        )?;
        let current = checks::approval(tx, approval.id)?;
        if current.state == "active" {
            checks::set_approval_state(tx, &current, "withdrawn", ticket.id, Some(reason))?;
        }
        Ok(())
    })
}

/// A red or conflicting landing is the candidate's: one repair with the
/// target made available, and a second consecutive red raises `red`.
#[allow(clippy::too_many_arguments)]
async fn returned(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    execution: i64,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    approval: &checks::Approval,
    target: &str,
    outcome: &str,
    detail: &str,
) -> Result<(), Fail> {
    let input = project.attempt_dir(attempt.id).join("input");
    std::fs::create_dir_all(&input).map_err(|error| error.to_string())?;
    let bundle = input.join("target.bundle");
    let _ = std::fs::remove_file(&bundle);
    if target != attempt.base {
        daemon
            .git
            .run(
                &project.canonical_dir(),
                &[
                    "bundle",
                    "create",
                    "--quiet",
                    bundle.to_str().unwrap_or_default(),
                    &crate::git::target_ref(&loaded.branch),
                    &format!("^{}", attempt.base),
                ],
            )
            .await?;
    }
    project.tx(|tx| {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome,
                detail: Some(detail),
                ticket: Some(ticket.id),
                ..Default::default()
            },
        )?;
        checks::set_approval_state(tx, approval, "retired", ticket.id, Some(outcome))?;
        let reds = if outcome == "red" {
            attempt.landing_reds + 1
        } else {
            attempt.landing_reds
        };
        tx.execute(
            "UPDATE attempt SET target = ?2, landing_reds = ?3 WHERE id = ?1",
            rusqlite::params![attempt.id, target, reds],
        )?;
        if reds >= 2 {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "red",
                    reason: "landing",
                    ticket: Some(ticket.id),
                    attempt: Some(attempt.id),
                    execution: Some(execution),
                    payload: json!({ "detail": detail, "target": target }),
                    text: Some(detail),
                },
            )?;
            return Ok(());
        }
        attempts::set_next(
            tx,
            attempt.id,
            Some(&json!({
                "reason": "repair", "detail": detail, "target": target, "branch": loaded.branch,
            })),
        )
    })
}
