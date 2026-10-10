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
    } else {
        project
            .attempt_dir(row.attempt)
            .join("effects")
            .join(row.id.to_string())
    }
}

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
        let limit = super::load(daemon, project).await?.config.max_executions;
        let spare = project.read(|conn| {
            Ok((conn.query_row(
                "SELECT count(*) FROM execution WHERE reserved = 1",
                [],
                |row| row.get::<_, i64>(0),
            )? as u64)
                < limit)
        })?;
        let row = project.read(|conn| executions::get(conn, execution))?;
        let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
        if row.status != "running" || (row.kind != "cleanup" && attempt.state != "live") {
            return Err(Fail::refused("execution no longer eligible to act"));
        }
        if spare {
            project.tx(|tx| executions::reserve(tx, execution, true))?;
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
    let gone = daemon
        .pinfold
        .list(&label)
        .await
        .is_ok_and(|boxes| !boxes.contains(&super::supervise::box_name(project, row.id)))
        && super::supervise::lock_free(&dir(project, row)).unwrap_or(false);
    // Legacy host children inherited the attempt/parent lock.
    let legacy = if row.kind == "gate" && row.handle.as_deref().is_some_and(|h| h.starts_with('{'))
    {
        match row.parent {
            Some(parent) => super::queue::landing_dir(project, parent),
            None => project.attempt_dir(row.attempt),
        }
    } else {
        dir(project, row)
    };
    let gone = gone && super::supervise::lock_free(&legacy).unwrap_or(false);
    project.tx(|tx| {
        if gone {
            executions::reserve(tx, row.id, false)?;
        } else if !attempts::open_attention(tx)?.iter().any(|a| a.reason == "ownership" && a.execution == Some(row.id)) {
            let attempt = attempts::get(tx, row.attempt)?;
            attempts::raise(tx, attempts::Raise {
                kind: "red", reason: "ownership", ticket: Some(attempt.ticket),
                attempt: (row.kind != "landing").then_some(row.attempt), execution: Some(row.id),
                payload: json!({ "detail": "external owner not proved gone; capacity retained" }), text: None,
            })?;
        }
        Ok(gone)
    })
}
