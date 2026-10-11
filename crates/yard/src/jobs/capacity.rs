//! One project limit for execution effects. Ended rows retain reservations
//! until the external owner is proved gone.

use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::store::{attempts, executions};
use serde_json::json;
use std::future::Future;
use std::os::fd::AsRawFd;
use std::path::PathBuf;

tokio::task_local! {
    pub static HOST_LOCK: std::os::fd::RawFd;
}

pub fn dir(project: &Project, row: &executions::Execution) -> PathBuf {
    if row.kind == "landing" {
        super::queue::landing_dir(project, row.id)
    } else if row.kind == "cleanup" {
        // Cleanup removes its attempt, so its current lock must survive it.
        project.local().join("effects").join(row.id.to_string())
    } else {
        project
            .attempt_dir(row.attempt)
            .join("effects")
            .join(row.id.to_string())
    }
}

// Callers acquire target locks inside work, after reserving capacity. Holding
// a target while waiting for a slot can deadlock a reserved box launch.
pub async fn effect<T>(
    daemon: &Daemon,
    project: &Project,
    execution: i64,
    work: impl Future<Output = Result<T, Fail>>,
) -> Result<T, Fail> {
    let mut announced = false;
    loop {
        // Register before reading: a release or sync between read and wait
        // must not be lost. A refused reservation writes nothing.
        let changed = project.events.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let canonical = project.canonical.lock().await;
        let limit = {
            let configs = project.loaded.lock().expect("loaded lock");
            configs
                .default
                .as_ref()
                .and_then(|branch| configs.targets.get(branch))
                .map(|loaded| loaded.config.max_executions)
        };
        // Invalidation excludes effects until validated publication wakes us.
        let spare = match limit {
            Some(limit) => project.read(|conn| {
                Ok((conn.query_row(
                    "SELECT count(*) FROM execution WHERE reserved = 1",
                    [],
                    |row| row.get::<_, i64>(0),
                )? as u64)
                    < limit)
            })?,
            None => false,
        };
        let row = project.read(|conn| executions::get(conn, execution))?;
        let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
        if row.status != "running" || (row.kind != "cleanup" && attempt.state != "live") {
            return Err(Fail::refused("execution no longer eligible to act"));
        }
        if spare {
            project.tx(|tx| {
                // Stop and reservation race on this row, not on a missing
                // in-memory stop handle.
                let current = executions::get(tx, execution)?;
                if current.status != "running"
                    || (current.kind != "cleanup"
                        && attempts::get(tx, current.attempt)?.state != "live")
                {
                    return Err(Fail::refused("execution no longer eligible to act"));
                }
                executions::reserve(tx, execution, true)
            })?;
            drop(canonical);
            break;
        }
        if !announced {
            project.tx(|tx| {
                crate::store::audit(
                    tx,
                    "execution.waiting",
                    crate::store::Target {
                        ticket: Some(attempt.ticket),
                        attempt: Some(row.attempt),
                        execution: Some(row.id),
                        ..Default::default()
                    },
                    None,
                    json!({ "reason": "capacity" }),
                )?;
                Ok(())
            })?;
            announced = true;
        }
        drop(canonical);
        changed.await;
    }
    let row = project.read(|conn| executions::get(conn, execution))?;
    let lock = match super::supervise::hold_lock(&dir(project, &row)) {
        Ok(lock) => lock,
        Err(error) => {
            release(daemon, project, &row).await?;
            return Err(Fail::from(error));
        }
    };
    let result = HOST_LOCK.scope(lock.as_raw_fd(), work).await;
    drop(lock);
    if !release(daemon, project, &row).await? {
        return Err(Fail::from(
            "execution ownership is unknown; capacity retained".to_string(),
        ));
    }
    result
}

/// Unknown owners are probed again by scheduler ticks. These rows have no
/// controller: only a finished run or reconciliation raises ownership.
pub async fn reap(daemon: &Daemon, project: &Project) -> Result<(), Fail> {
    let rows = project.read(|conn| executions::query(conn,
        "reserved = 1 AND EXISTS (SELECT 1 FROM attention WHERE attention.execution = execution.id AND reason = 'ownership')", []))?;
    for row in rows {
        if !release(daemon, project, &row).await? {
            continue;
        }
        if row.intent_state.as_deref() == Some("open") {
            super::reconcile::decide_intent(daemon, project, &row).await?;
        } else if row.kind == "landing" && row.intent_state.as_deref() == Some("resolved") {
            let target_ref = row.intent_ref.as_deref().expect("a landing intent target");
            let branch = target_ref
                .strip_prefix("refs/heads/")
                .expect("a target ref");
            let target = project.target_lock(target_ref);
            let held = target.lock().await;
            let published = project.canonical.lock().await;
            let _ = super::publish_default(daemon, project, &published, branch, &held).await;
        } else if row.status == "running" {
            project.tx(|tx| {
                executions::end(
                    tx,
                    row.id,
                    executions::End {
                        outcome: "interrupted",
                        detail: Some("external owner is now proved gone"),
                        ..Default::default()
                    },
                )
            })?;
        }
    }
    Ok(())
}

/// Probe only after the controller is over, or during startup reconciliation.
pub async fn release(
    daemon: &Daemon,
    project: &Project,
    row: &executions::Execution,
) -> Result<bool, Fail> {
    if !row.reserved {
        return Ok(true);
    }
    let label = format!("dev.yard.project={}", project.key);
    let boxes_gone = daemon
        .pinfold
        .list(&label)
        .await
        .is_ok_and(|boxes| !boxes.contains(&super::supervise::box_name(project, row.id)));
    let host_gone = if row.lock_contract {
        (!matches!(row.kind.as_str(), "cleanup" | "review" | "gate")
            || dir(project, row).join("lock").is_file())
            && super::supervise::lock_free(&dir(project, row)).unwrap_or(false)
    } else {
        // The old binary never inherited effects/<id>/lock. Only paths
        // whose recorded state reached an old ownership proof may use it.
        let old_lock = if row.kind == "landing" && row.intent_state.as_deref() == Some("open") {
            Some(super::queue::landing_dir(project, row.id))
        } else if row.kind == "gate" && row.handle.as_deref().is_some_and(|h| h.starts_with('{')) {
            Some(match row.parent {
                Some(parent) => super::queue::landing_dir(project, parent),
                None => project.attempt_dir(row.attempt),
            })
        } else {
            None
        };
        match old_lock {
            Some(path) => {
                path.join("lock").is_file() && super::supervise::lock_free(&path).unwrap_or(false)
            }
            // These terminal box judgments finished their pre-box Git.
            // Box absence proves the remaining controller cannot act.
            None => {
                row.status == "ended"
                    && matches!(row.kind.as_str(), "gate" | "review")
                    && matches!(row.outcome.as_deref(), Some("pass" | "fail"))
                    && row.handle.as_deref().is_some_and(|h| !h.starts_with('{'))
            }
        }
    };
    let gone = boxes_gone && host_gone;
    let detail = if !row.lock_contract && !host_gone {
        "legacy execution has no supported host ownership proof; stop the upgrade and resolve outside Yard"
    } else {
        "external owner not proved gone; capacity retained"
    };
    let completed_cleanup = project.tx(|tx| {
        let current = executions::get(tx, row.id)?;
        let completed_cleanup = gone
            && current.kind == "cleanup"
            && current.lock_contract
            && current.status == "ended"
            && current.outcome.as_deref() == Some("removed");
        if gone {
            tx.execute(
                "UPDATE execution SET teardown = 1 WHERE id = ?1 AND node IS NOT NULL",
                [row.id],
            )?;
            executions::reserve(tx, row.id, false)?;
            let released = executions::get(tx, row.id)?;
            let completed_candidate = released.kind == "implementation"
                && released.lock_contract
                && released.outcome.as_deref() == Some("candidate")
                && released.result_eligible
                && dir(project, &released).join("lock").is_file();
            let completed_judgment = matches!(released.kind.as_str(), "review" | "gate")
                && released.lock_contract
                && matches!(released.outcome.as_deref(), Some("pass" | "fail"))
                && released.result_eligible
                && released
                    .result
                    .as_ref()
                    .is_some_and(|result| result["infrastructure"]["outcome"] == "completed")
                && dir(project, &released).join("lock").is_file();
            if completed_cleanup || completed_candidate || completed_judgment {
                for item in attempts::open_for_attempt(tx, current.attempt)? {
                    if item.kind == "red"
                        && item.reason == "ownership"
                        && item.execution == Some(current.id)
                    {
                        attempts::resolve(tx, &item, "reconciled", None)?;
                    }
                }
            }
        } else if !attempts::open_attention(tx)?
            .iter()
            .any(|a| a.reason == "ownership" && a.execution == Some(row.id))
        {
            let attempt = attempts::get(tx, row.attempt)?;
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "red",
                    reason: "ownership",
                    ticket: Some(attempt.ticket),
                    attempt: (row.kind != "landing").then_some(row.attempt),
                    execution: Some(row.id),
                    payload: json!({ "detail": detail, "lock_contract": row.lock_contract }),
                    text: None,
                },
            )?;
        }
        Ok(completed_cleanup)
    })?;
    if completed_cleanup {
        super::cleanup::remove(&dir(project, row))?;
    }
    Ok(gone)
}
