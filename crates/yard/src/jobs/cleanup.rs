//! A landed or abandoned attempt's clone, boxes, transcripts and logs are
//! removed whole. Rows remain.

use super::spawn;
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::store::{attempts, executions};
use std::sync::Arc;

pub fn next(daemon: &Arc<Daemon>, project: &Arc<Project>) -> Result<(), Fail> {
    for attempt in project.read(attempts::uncleaned)? {
        let busy = project.read(|conn| {
            Ok(executions::for_attempt(conn, attempt.id)?
                .iter()
                .any(|row| row.status == "running"))
        })?;
        if busy {
            continue;
        }
        let execution = project.tx(|tx| {
            executions::start(
                tx,
                executions::Start {
                    attempt: attempt.id,
                    kind: "cleanup",
                    reason: attempt.outcome.as_deref(),
                    ticket: Some(attempt.ticket),
                    ..Default::default()
                },
            )
        })?;
        spawn(daemon, project, "cleanup", execution);
    }
    Ok(())
}

pub async fn run(daemon: &Arc<Daemon>, project: &Arc<Project>, execution: i64) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    let others = project.read(|conn| executions::for_attempt(conn, row.attempt))?;
    for other in &others {
        if other.kind != "cleanup" && other.handle.is_some() {
            daemon
                .pinfold
                .down(&super::supervise::box_name(project, other.id))
                .await?;
        }
        if other.kind == "landing" {
            remove(&super::queue::landing_dir(project, other.id))?;
        }
    }
    remove(&project.attempt_dir(row.attempt))?;
    project.tx(|tx| {
        tx.execute(
            "UPDATE attempt SET cleaned = 1 WHERE id = ?1",
            [row.attempt],
        )?;
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: "removed",
                ..Default::default()
            },
        )
    })
}

fn remove(dir: &std::path::Path) -> Result<(), Fail> {
    match std::fs::remove_dir_all(dir) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Fail::from(format!("remove {}: {error}", dir.display()))),
    }
}
