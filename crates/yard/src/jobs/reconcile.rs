//! Reconcile-on-start: what a restart must prove before it serves. Unknown
//! liveness or outcome never duplicates work, lands, deletes or kills.

use super::queue::{landing_dir, record_landing};
use super::supervise::{birth, lock_free};
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::store::{attempts, executions};
use serde_json::{Value, json};

pub async fn project(daemon: &Daemon, project: &Project) -> Result<(), Fail> {
    // The old daemon's boxes: their `up` lost its stdin with the daemon.
    let label = format!("dev.yard.project={}", project.key);
    for name in daemon.pinfold.list(&label).await? {
        daemon.pinfold.down(&name).await?;
    }
    daemon.pinfold.prune().await?;

    let running = project.read(executions::running)?;
    // Children first: a landing's host gates end before the landing is read.
    for row in running.iter().filter(|row| row.kind != "landing") {
        if let Some(handle) = row
            .handle
            .as_deref()
            .and_then(|handle| serde_json::from_str::<Value>(handle).ok())
        {
            kill_group(&handle).await;
        }
        project
            .tx(|tx| {
                executions::end(tx, row.id, executions::End {
                    outcome: "interrupted",
                    detail: Some("the daemon restarted"),
                    ..Default::default()
                })?;
                if row.kind == "implementation" {
                    let attempt = attempts::get(tx, row.attempt)?;
                    if attempt.state == "live" {
                        attempts::raise(tx, attempts::Raise {
                            kind: "stopped",
                            reason: "interrupted",
                            ticket: Some(attempt.ticket),
                            attempt: Some(attempt.id),
                            execution: Some(row.id),
                            payload: json!({ "detail": "the daemon restarted during the execution" }),
                            text: None,
                        })?;
                    }
                }
                Ok(())
            })?;
    }
    for row in running
        .iter()
        .filter(|row| row.kind == "landing" && row.intent_state.is_none())
    {
        let free = lock_free(&landing_dir(project, row.id))?;
        project.tx(|tx| {
            executions::end(
                tx,
                row.id,
                executions::End {
                    outcome: "interrupted",
                    detail: Some("the daemon restarted before the landing's intent"),
                    ..Default::default()
                },
            )?;
            if !free {
                let attempt = attempts::get(tx, row.attempt)?;
                attempts::raise(
                    tx,
                    attempts::Raise {
                        kind: "red",
                        reason: "landing",
                        ticket: Some(attempt.ticket),
                        attempt: None,
                        execution: Some(row.id),
                        payload: json!({ "detail": "a child of the landing still holds its lock" }),
                        text: None,
                    },
                )?;
            }
            Ok(())
        })?;
    }
    for row in project.read(executions::open_intents)? {
        decide_intent(daemon, project, &row).await?;
    }
    Ok(())
}

/// Kill a recorded host group, if its leader is still the process we
/// started, and wait until the group is gone so its lock can be read.
async fn kill_group(handle: &Value) {
    let (Some(pgid), Some(recorded)) = (handle["pgid"].as_u64(), handle["birth"].as_str()) else {
        return;
    };
    if birth(pgid as u32).await.as_deref() != Some(recorded) {
        return;
    }
    let group = nix::unistd::Pid::from_raw(pgid as i32);
    let _ = nix::sys::signal::killpg(group, nix::sys::signal::Signal::SIGKILL);
    // Bounded: a group that outlives this leaves its lock held, and the
    // lock decides.
    for _ in 0..500 {
        if nix::sys::signal::killpg(group, None).is_err() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

/// Decide an unresolved landing intent by canonical alone, once the
/// landing's lock proves its git children are over. `Ok(false)` when
/// nothing can be decided: the intent stays, the queue is refused, and
/// `red` is raised.
pub async fn decide_intent(
    daemon: &Daemon,
    project: &Project,
    row: &executions::Execution,
) -> Result<bool, Fail> {
    let undecided = |detail: &str| -> Result<bool, Fail> {
        project.tx(|tx| {
            let open = attempts::open_attention(tx)?
                .into_iter()
                .any(|item| item.execution == Some(row.id) && item.kind == "red");
            if !open {
                let attempt = attempts::get(tx, row.attempt)?;
                attempts::raise(tx, attempts::Raise {
                    kind: "red",
                    reason: "intent",
                    ticket: Some(attempt.ticket),
                    attempt: None,
                    execution: Some(row.id),
                    payload: json!({ "detail": detail, "old": row.intent_old, "merged": row.intent_merged }),
                    text: Some(detail),
                })?;
            }
            Ok(false)
        })
    };
    if !lock_free(&landing_dir(project, row.id))? {
        return undecided("a git child of the landing still holds its lock");
    }
    let canonical = project.canonical_dir();
    let target = daemon.git.target_head(&canonical).await;
    let (Ok((_, Some(target))), Some(old), Some(merged)) = (
        target,
        row.intent_old.as_deref(),
        row.intent_merged.as_deref(),
    ) else {
        return undecided("canonical's target is unreadable");
    };
    if target == old {
        project.tx(|tx| {
            executions::resolve_intent(tx, row.id)?;
            if executions::get(tx, row.id)?.status == "running" {
                executions::end(
                    tx,
                    row.id,
                    executions::End {
                        outcome: "retired",
                        detail: Some("nothing landed"),
                        ..Default::default()
                    },
                )?;
            }
            Ok(())
        })?;
        return Ok(true);
    }
    if target == merged
        || daemon
            .git
            .is_ancestor(&canonical, merged, &target, None)
            .await?
    {
        project.tx(|tx| record_landing(tx, row.id))?;
        return Ok(true);
    }
    undecided("canonical's target is neither the expected old head nor contains the merged head")
}
