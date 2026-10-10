//! Reconcile-on-start: what a restart must prove before it serves. Unknown
//! liveness or outcome never duplicates work, lands, deletes or kills.

use super::queue::{landing_dir, record_landing};
use super::supervise::{birth, lock_free};
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::store::{attempts, executions};
use serde_json::{Value, json};

pub async fn project(daemon: &Daemon, project: &Project) -> Result<(), Fail> {
    // Old live attempts used HEAD; Y-82 refused switching it under them.
    // Ended history has no recorded branch, so it stays unknown.
    let branch = daemon.git.head_branch(&project.canonical_dir()).await?;
    project.tx(|tx| {
        tx.execute(
            "UPDATE attempt SET target_branch = ?1 WHERE target_branch IS NULL AND state = 'live'",
            [&branch],
        )?;
        tx.execute("UPDATE execution SET intent_ref = 'refs/heads/' || (SELECT target_branch FROM attempt WHERE attempt.id = execution.attempt) WHERE intent_state IS NOT NULL AND intent_ref IS NULL", [])?;
        // Legacy candidate items must keep their override exit after backfill.
        for attempt in attempts::live(tx)? {
            for item in attempts::open_for_attempt(tx, attempt.id)? {
                if (item.kind == "approval" || (item.kind == "stopped" && item.reason == "limit"))
                    && item.payload.get("target").is_none()
                {
                    let mut payload = item.payload;
                    payload["target"] = json!(attempt.target_branch()?);
                    tx.execute(
                        "UPDATE attention SET payload = ?2 WHERE id = ?1",
                        rusqlite::params![item.id, payload.to_string()],
                    )?;
                }
            }
        }
        Ok(())
    })?;
    // The old daemon's boxes: their `up` lost its stdin with the daemon.
    let label = format!("dev.yard.project={}", project.key);
    for name in daemon.pinfold.list(&label).await? {
        daemon.pinfold.down(&name).await?;
    }
    daemon.pinfold.prune().await?;

    let running = project.read(executions::running)?;
    for row in
        project.read(|conn| executions::query(conn, "reserved = 1 AND status = 'ended'", []))?
    {
        super::capacity::release(daemon, project, &row).await?;
    }
    // Children first: a landing's host gates end before the landing is read.
    for row in running.iter().filter(|row| row.kind != "landing") {
        if let Some(handle) = row
            .handle
            .as_deref()
            .and_then(|handle| serde_json::from_str::<Value>(handle).ok())
        {
            kill_group(&handle).await;
        }
        if !super::capacity::release(daemon, project, row).await? {
            continue;
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
        if !super::capacity::release(daemon, project, row).await? {
            continue;
        }
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
        if !super::capacity::release(daemon, project, &row).await? {
            continue;
        }
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
/// nothing can be decided: the intent stays, its target queue is refused, and
/// `red` is raised.
pub async fn decide_intent(
    daemon: &Daemon,
    project: &Project,
    row: &executions::Execution,
) -> Result<bool, Fail> {
    if project
        .read(|conn| Ok(executions::get(conn, row.id)?.intent_state))?
        .as_deref()
        != Some("open")
    {
        return Ok(true);
    }
    if row.reserved && !super::capacity::release(daemon, project, row).await? {
        return Ok(false);
    }
    let undecided = |detail: &str| -> Result<bool, Fail> {
        project.tx(|tx| {
            if executions::get(tx, row.id)?.intent_state.as_deref() != Some("open") {
                return Ok(true);
            }
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
                    payload: json!({ "detail": detail, "target_ref": row.intent_ref, "old": row.intent_old, "merged": row.intent_merged }),
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
    let Some(target_ref) = row.intent_ref.as_deref() else {
        return undecided("the landing intent has no recorded target ref");
    };
    let target_lock = project.target_lock(target_ref);
    let _held = target_lock.lock().await;
    let target = daemon.git.rev_parse(&canonical, target_ref).await;
    let (Ok(Some(target)), Some(old), Some(merged)) = (
        target,
        row.intent_old.as_deref(),
        row.intent_merged.as_deref(),
    ) else {
        return undecided("canonical's target is unreadable");
    };
    if target == old {
        project.tx(|tx| {
            let current = executions::get(tx, row.id)?;
            if current.intent_state.as_deref() != Some("open") {
                return Ok(());
            }
            executions::resolve_intent(tx, row.id)?;
            if current.status == "running" {
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
        let published = project.canonical.lock().await;
        let branch = target_ref
            .strip_prefix("refs/heads/")
            .expect("a target ref");
        let _ = super::publish_default(daemon, project, &published, branch, &_held).await;
        return Ok(true);
    }
    if target == merged
        || daemon
            .git
            .is_ancestor(&canonical, merged, &target, None)
            .await?
    {
        project.tx(|tx| record_landing(tx, row.id))?;
        let published = project.canonical.lock().await;
        let branch = target_ref
            .strip_prefix("refs/heads/")
            .expect("a target ref");
        let _ = super::publish_default(daemon, project, &published, branch, &_held).await;
        return Ok(true);
    }
    undecided("canonical's target is neither the expected old head nor contains the merged head")
}
