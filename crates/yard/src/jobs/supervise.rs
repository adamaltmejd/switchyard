//! Box-backed work: the implementer execution, gates in a box or on the
//! host, the project image, and the worker's `yard_context`.

use super::{Loaded, load, proof};
use crate::api::Fail;
use crate::r#box::{BoxSpec, Egress, EnvValue, Mount, Route};
use crate::config::RunsIn;
use crate::daemon::{Daemon, Project};
use crate::harness::{Harness, ModelRoute, Stage};
use crate::store::{self, attempts, checks, executions, ticket_name, tickets};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const UP_TIMEOUT: Duration = Duration::from_secs(600);
pub const DOWN_TIMEOUT: Duration = Duration::from_secs(60);
const BUILD_TIMEOUT: Duration = Duration::from_secs(3600);
const LOG_TAIL: usize = 8000;

pub fn box_name(project: &Project, execution: i64) -> String {
    format!("yard-{}-{execution}", project.key)
}

pub fn transcript(project: &Project, attempt: i64, execution: i64) -> PathBuf {
    project
        .attempt_dir(attempt)
        .join("transcripts")
        .join(format!("{execution}.jsonl"))
}

/// Record the next implementer execution. Its reason says why it runs.
pub fn start_implementation(
    tx: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    reason: &str,
) -> Result<i64, Fail> {
    let name = attempt.implementer["name"].as_str().unwrap_or("default");
    let harness_version = attempt.implementer["harness"]
        .as_str()
        .and_then(crate::harness::get)
        .map(|harness| harness.version());
    executions::start(
        tx,
        executions::Start {
            attempt: attempt.id,
            kind: "implementation",
            reason: Some(reason),
            base: Some(&attempt.base),
            head: attempt.head.as_deref(),
            ticket_revision: Some(ticket.revision),
            ticket: Some(ticket.id),
            agent: Some((name, &attempt.implementer)),
            harness_version,
            ..Default::default()
        },
    )
}

/// The project image, built from canonical's target head.
pub async fn image(daemon: &Daemon, project: &Project, loaded: &Loaded) -> Result<String, Fail> {
    let _building = daemon.image_build.lock().await;
    if let Some((head, reference)) = daemon.images.lock().expect("images lock").get(&project.key)
        && *head == loaded.head
    {
        return Ok(reference.clone());
    }
    let context = project.local().join("build").join(&loaded.head);
    let _ = std::fs::remove_dir_all(&context);
    daemon
        .git
        .clone_detached(&project.canonical_dir(), &context, &loaded.head)
        .await?;
    std::fs::remove_dir_all(context.join(".git")).map_err(|error| error.to_string())?;
    let built = daemon
        .pinfold
        .image_build(
            &format!("yard-{}", project.key),
            &context.join(crate::config::DOCKERFILE_PATH),
            &context,
            BUILD_TIMEOUT,
        )
        .await;
    let _ = std::fs::remove_dir_all(&context);
    let built = built
        .map_err(|error| Fail::new("image", format!("the project image did not build: {error}")))?;
    daemon
        .images
        .lock()
        .expect("images lock")
        .insert(project.key.clone(), (loaded.head.clone(), built.clone()));
    Ok(built)
}

/// Bring a worker's box up and record its handle. A box that does not come
/// up revokes the execution's grant.
pub async fn up_worker(
    daemon: &Daemon,
    project: &Project,
    execution: i64,
    spec: &BoxSpec,
    secrets: &[(String, String)],
) -> Result<crate::r#box::LiveBox, Fail> {
    let live = match daemon.pinfold.up(spec, secrets, UP_TIMEOUT).await {
        Ok(live) => live,
        Err(error) => {
            daemon.grants.revoke(project, execution);
            return Err(Fail::new(
                "box",
                format!("the box did not come up: {error}"),
            ));
        }
    };
    project.read(|conn| {
        executions::set_handle(conn, execution, &live.name, live.image_id.as_deref())
    })?;
    Ok(live)
}

/// A worker box: the harness, the model route and the MCP route.
pub struct Worker<'a> {
    pub execution: i64,
    pub image: &'a str,
    pub workspace: &'a Path,
    pub read_only: bool,
    /// The worker-written live proof directory, mounted at `/yard/proof`
    /// unless the workflow is read-only.
    pub proof: Option<&'a Path>,
    pub harness: &'static dyn Harness,
    pub stage: Stage<'a>,
    pub model: &'a ModelRoute,
    pub egress: &'a [String],
}

pub fn worker_spec(daemon: &Daemon, project: &Project, worker: &Worker) -> BoxSpec {
    let mut env: BTreeMap<String, EnvValue> = worker
        .harness
        .env(&worker.stage)
        .into_iter()
        .map(|(name, value)| (name, EnvValue::Value(value)))
        .collect();
    env.insert(
        crate::harness::BEARER_VAR.into(),
        EnvValue::From(crate::harness::BEARER_VAR.into()),
    );
    for (name, value) in [
        ("GIT_AUTHOR_NAME", "Yard worker"),
        ("GIT_AUTHOR_EMAIL", "worker@yard.invalid"),
        ("GIT_COMMITTER_NAME", "Yard worker"),
        ("GIT_COMMITTER_EMAIL", "worker@yard.invalid"),
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "safe.directory"),
        ("GIT_CONFIG_VALUE_0", "/workspace"),
    ] {
        env.insert(name.into(), EnvValue::Value(value.into()));
    }
    let mut routes = BTreeMap::new();
    routes.insert(
        crate::harness::MCP_ROUTE.to_string(),
        Route::Service(format!("127.0.0.1:{}", daemon.mcp_port)),
    );
    routes.insert(worker.model.name.clone(), worker.model.route.clone());
    let mut mounts = vec![
        Mount {
            host: worker.workspace.into(),
            guest: "/workspace".into(),
            readonly: worker.read_only,
        },
        Mount {
            host: worker.stage.state.into(),
            guest: crate::harness::STATE_GUEST.into(),
            readonly: false,
        },
        Mount {
            host: worker.stage.input.into(),
            guest: crate::harness::INPUT_GUEST.into(),
            readonly: true,
        },
    ];
    if !worker.read_only
        && let Some(proof) = worker.proof
    {
        mounts.push(Mount {
            host: proof.into(),
            guest: "/yard/proof".into(),
            readonly: false,
        });
    }
    BoxSpec {
        name: box_name(project, worker.execution),
        labels: labels(project),
        harness: Some(worker.harness.name().into()),
        image: worker.image.into(),
        mounts,
        env,
        egress: Some(Egress {
            allow: worker.egress.to_vec(),
            routes,
        }),
        memory: daemon.machine.box_memory.clone(),
    }
}

pub fn labels(project: &Project) -> BTreeMap<String, String> {
    BTreeMap::from([("dev.yard.project".to_string(), project.key.clone())])
}

/// What a worker run ended as.
#[derive(Default)]
pub struct Run {
    pub terminal: Option<crate::harness::Event>,
    pub registered: Option<crate::harness::Registration>,
    pub session_id: Option<String>,
    pub stopped: bool,
    pub timed_out: bool,
    pub exit_code: Option<i32>,
}

/// Run the harness in a live box, streaming its frames to the transcript,
/// until it ends, the stop signal fires or a clock runs out. The caller
/// takes the box down, which is how a run is cancelled.
// Each argument is a separate input of the one run; a struct would only
// rename them.
#[allow(clippy::too_many_arguments)]
pub async fn run_harness(
    daemon: &Daemon,
    project: &Project,
    execution: i64,
    kind: crate::mcp::Kind,
    harness: &'static dyn Harness,
    argv: &[String],
    transcript: &Path,
    inactivity: Duration,
    deadline: tokio::time::Instant,
) -> Result<Run, Fail> {
    let stop = Arc::new(tokio::sync::Notify::new());
    daemon
        .stops
        .lock()
        .expect("stops lock")
        .insert((project.key.clone(), execution), stop.clone());
    std::fs::create_dir_all(transcript.parent().expect("transcript dir"))
        .map_err(|error| error.to_string())?;
    let mut file = tokio::fs::File::create(transcript)
        .await
        .map_err(|error| error.to_string())?;
    let mut child = daemon
        .pinfold
        .exec_streaming(&box_name(project, execution), Some("/workspace"), argv)
        .map_err(|error| error.to_string())?;
    let mut stdout = BufReader::new(child.stdout.take().expect("piped")).lines();
    let mut stderr = BufReader::new(child.stderr.take().expect("piped")).lines();
    let mut reader = harness.reader();
    let mut run = Run::default();
    let mut usage = None;
    let mut stdout_open = true;
    let mut stderr_open = true;
    while stdout_open || stderr_open {
        let idle = tokio::time::sleep(inactivity);
        tokio::select! {
            line = stdout.next_line(), if stdout_open => match line {
                Ok(Some(line)) => {
                    let _ = file.write_all(line.as_bytes()).await;
                    let _ = file.write_all(b"\n").await;
                    if let Some(event) = reader.stdout(&line) {
                        match &event {
                            crate::harness::Event::Started { session_id } => {
                                run.session_id = Some(session_id.clone());
                                project.read(|conn| executions::set_worker(conn, execution, Some(session_id), None))?;
                            }
                            crate::harness::Event::Finished { usage: spent, .. } | crate::harness::Event::Failed { usage: spent, .. } => {
                                usage = Some((spent.input, spent.output, spent.cost));
                                run.terminal = Some(event.clone());
                            }
                        }
                    }
                }
                _ => stdout_open = false,
            },
            line = stderr.next_line(), if stderr_open => match line {
                Ok(Some(line)) => {
                    if let Some(registration) = reader.stderr(&line) {
                        run.registered = Some(registration);
                    }
                }
                _ => stderr_open = false,
            },
            _ = stop.notified() => { run.stopped = true; break; }
            _ = idle => { run.timed_out = true; break; }
            _ = tokio::time::sleep_until(deadline) => { run.timed_out = true; break; }
        }
    }
    if !run.stopped && !run.timed_out {
        run.exit_code = child.wait().await.ok().map(crate::git::exit_code);
    }
    project.read(|conn| executions::set_worker(conn, execution, None, usage))?;
    // A granted tool the fetched list lacks is a failed registration.
    if let Some(crate::harness::Registration::Registered(tools)) = &run.registered
        && let Some(missing) = crate::mcp::tool_names(kind)
            .into_iter()
            .find(|tool| !tools.contains(tool))
    {
        run.registered = Some(crate::harness::Registration::Refused(format!(
            "the MCP client registered no {missing}"
        )));
    }
    Ok(run)
}

pub async fn implement(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    execution: i64,
) -> Result<(), Fail> {
    let loaded = load(daemon, project).await?;
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let workflow = loaded.config.workflow(&attempt.workflow)?.clone();
    let provider = attempt.implementer["provider"].as_str().unwrap_or_default();
    let connection = crate::harness::connection(provider)
        .ok_or_else(|| Fail::invalid(format!("connection {provider:?} is unknown")))?;
    let harness = crate::harness::get(attempt.implementer["harness"].as_str().unwrap_or_default())
        .ok_or_else(|| {
            Fail::invalid(format!(
                "harness {:?} is unknown",
                attempt.implementer["harness"]
            ))
        })?;
    let dir = project.attempt_dir(attempt.id);
    let clone = dir.join("clone");
    if !clone.exists() {
        daemon
            .git
            .clone_branch(
                &project.canonical_dir(),
                &clone,
                &attempt.branch(),
                &attempt.base,
            )
            .await?;
    }
    let state = dir.join("state");
    let input = dir.join("input");
    // The worker-written proof directory exists before the box so the mount
    // always has a host directory. A read-only workflow gets none.
    let proof_dir = dir.join("proof");
    if !workflow.read_only {
        std::fs::create_dir_all(&proof_dir).map_err(|error| error.to_string())?;
    }
    let stage = Stage {
        state: &state,
        input: &input,
        connection,
    };
    harness.stage(&stage).map_err(|error| error.to_string())?;

    // Resume the session unless it has run its executions or the harness
    // version changed under it.
    let resume = match project.read(|conn| executions::session(conn, attempt.id, execution))? {
        Some((previous, count))
            if count < i64::from(workflow.max_session_executions)
                && previous.harness_version.as_deref() == Some(harness.version()) =>
        {
            previous.session_id.map(|session| (previous.id, session))
        }
        _ => None,
    };
    project.read(|conn| {
        conn.execute(
            "UPDATE attempt SET next = NULL, nudge = NULL WHERE id = ?1",
            [attempt.id],
        )?;
        conn.execute(
            "UPDATE execution SET resumed = ?2 WHERE id = ?1",
            rusqlite::params![execution, resume.as_ref().map(|(id, _)| *id)],
        )?;
        Ok(())
    })?;
    let prompt = implementer_prompt(
        daemon,
        project,
        &loaded,
        &attempt,
        &ticket,
        row.reason.as_deref().unwrap_or("first"),
        resume.is_some(),
    )
    .await?;
    let argv = harness
        .argv(&crate::harness::Launch {
            provider,
            model: attempt.implementer["model"].as_str().unwrap_or_default(),
            effort: attempt.implementer["effort"].as_str(),
            resume: resume.as_ref().map(|(_, session)| session.as_str()),
            prompt: &prompt,
        })
        .map_err(Fail::invalid)?;

    let image = image(daemon, project, &loaded).await?;
    let bearer = daemon.grants.issue(crate::mcp::Grant {
        project: project.clone(),
        execution,
        kind: crate::mcp::Kind::Implementation,
    });
    let model = harness
        .route(&stage, &daemon.machine)
        .map_err(Fail::refused)?;
    let secrets = vec![
        (crate::harness::BEARER_VAR.to_string(), bearer),
        model.secret.clone(),
    ];
    let spec = worker_spec(
        daemon,
        project,
        &Worker {
            execution,
            image: &image,
            workspace: &clone,
            read_only: workflow.read_only,
            proof: (!workflow.read_only).then_some(proof_dir.as_path()),
            harness,
            stage,
            model: &model,
            egress: &loaded.config.egress,
        },
    );
    let live = up_worker(daemon, project, execution, &spec, &secrets).await?;

    let deadline = {
        let spent = attempt.work_ms
            + attempt
                .lane_since
                .map(|since| store::now_ms() - since)
                .unwrap_or(0);
        let total = (workflow.total_work_timeout_minutes * 60_000) as i64;
        tokio::time::Instant::now() + Duration::from_millis((total - spent).max(0) as u64)
    };
    let run = run_harness(
        daemon,
        project,
        execution,
        crate::mcp::Kind::Implementation,
        harness,
        &argv,
        &transcript(project, attempt.id, execution),
        Duration::from_secs(workflow.inactivity_timeout_minutes * 60),
        deadline,
    )
    .await;
    let run = match run {
        Ok(run) => run,
        Err(fail) => {
            daemon.grants.revoke(project, execution);
            let _ = live.down(DOWN_TIMEOUT).await;
            return Err(fail);
        }
    };
    // Ask git inside the box whether the clone is clean, before it comes down.
    // A timeout never reaches the candidate. A stopped run is deferred whole:
    // its status would be taken while the worker may still write, so it never
    // admits a candidate and keeps its tree for the next execution.
    let listing = if run.stopped || run.timed_out {
        None
    } else {
        daemon
            .pinfold
            .exec(
                &live.name,
                Some("/workspace"),
                &["git", "status", "--porcelain", "--untracked-files=all"].map(String::from),
                Duration::from_secs(120),
            )
            .await
            .ok()
            .filter(|out| out.code == 0)
            .map(|out| out.stdout)
    };
    let oom = daemon.pinfold.oom_kills(&live.name).await;
    daemon.grants.revoke(project, execution);
    // The proof is only read once the box is confirmed down; a failed teardown
    // refuses the candidate below.
    let teardown = live.down(DOWN_TIMEOUT).await;

    // Take the candidate's objects by a fetch run in canonical.
    let fetched = {
        let _canonical = project.canonical.lock().await;
        daemon
            .git
            .fetch(
                &project.canonical_dir(),
                &clone,
                &format!(
                    "+refs/heads/{}:{}",
                    attempt.branch(),
                    crate::git::candidate_ref(attempt.id)
                ),
            )
            .await
    };
    let head = match &fetched {
        Ok(()) => {
            daemon
                .git
                .rev_parse(
                    &project.canonical_dir(),
                    &crate::git::candidate_ref(attempt.id),
                )
                .await?
        }
        Err(_) => None,
    };
    // The base moves to a target handed to the attempt once the head
    // contains it.
    let base = match (&attempt.target, &head) {
        (Some(target), Some(head))
            if daemon
                .git
                .is_ancestor(&project.canonical_dir(), target, head, None)
                .await? =>
        {
            target.clone()
        }
        _ => attempt.base.clone(),
    };
    let touches_yard = match &head {
        Some(head) if Some(head.as_str()) != attempt.head.as_deref() && *head != base => daemon
            .git
            .changed_paths(&project.canonical_dir(), &base, head)
            .await?
            .iter()
            .any(|path| path == ".yard" || path.starts_with(".yard/")),
        _ => false,
    };
    // The proof directory is worker-written. Snapshot it only once the box is
    // down, the clone is known clean and the head is not refused for `.yard`,
    // and never follow a link. A changed proof with an unchanged head is a new
    // candidate.
    let clean = listing
        .as_deref()
        .is_some_and(|listing| listing.trim().is_empty());
    let snapshots = dir.join("proof-snapshots");
    let proof_snapshot =
        (teardown.is_ok() && fetched.is_ok() && clean && !touches_yard && !workflow.read_only)
            .then(|| proof::snapshot(&proof_dir, &snapshots));
    project.tx(|tx| {
        let current = attempts::get(tx, attempt.id)?;
        if current.state != "live" {
            return executions::end(tx, execution, executions::End {
                outcome: "abandoned",
                ticket: Some(ticket.id),
                ..Default::default()
            });
        }
        let failure = match (&run.registered, &run.terminal) {
            (Some(crate::harness::Registration::Refused(reason)), _) => Some(("failed", Some("mcp"), reason.clone())),
            _ if run.timed_out => Some(("timeout", Some("timeout"), "a clock ran out".to_string())),
            (_, Some(crate::harness::Event::Failed { message, .. })) if !run.stopped => {
                Some(("failed", Some("harness"), message.clone()))
            }
            (_, None) if !run.stopped => {
                let rose = oom.is_some_and(|count| count > 0);
                Some((
                    "failed",
                    Some(if rose { "oom" } else if oom.is_none() { "unknown" } else { "harness" }),
                    format!("the worker's stream ended without a terminal frame (exit {:?})", run.exit_code),
                ))
            }
            _ => None,
        };
        let stop = |tx: &rusqlite::Connection, reason: &str, detail: &str| -> Result<(), Fail> {
            if current.nudge.is_some() && reason != "timeout" {
                return attempts::set_next(tx, attempt.id, Some(&json!({ "reason": "nudge" })));
            }
            attempts::raise(tx, attempts::Raise {
                kind: "stopped",
                reason,
                ticket: Some(ticket.id),
                attempt: Some(attempt.id),
                execution: Some(execution),
                payload: json!({ "detail": detail }),
                text: Some(detail),
            })?;
            Ok(())
        };
        if let Some((outcome, cause, detail)) = failure {
            executions::end(tx, execution, executions::End {
                outcome,
                detail: Some(&detail),
                exit_cause: cause,
                exit_code: run.exit_code,
                oom_kills: oom,
                ticket: Some(ticket.id),
            })?;
            return stop(tx, if outcome == "timeout" { "timeout" } else { "failed" }, &detail);
        }
        if let Err(error) = &teardown {
            let detail = format!("the box did not come down, so the clone cannot be trusted: {error}");
            executions::end(tx, execution, executions::End {
                outcome: "failed",
                detail: Some(&detail),
                exit_cause: Some("box"),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            return stop(tx, "failed", &detail);
        }
        if listing.is_none() && !run.stopped {
            let detail = "the clone's status could not be read, so the candidate is refused";
            executions::end(tx, execution, executions::End {
                outcome: "refused",
                detail: Some(detail),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            return stop(tx, "failed", detail);
        }
        if let Err(error) = &fetched {
            let detail = format!("the clone was refused: {error}");
            executions::end(tx, execution, executions::End {
                outcome: "refused",
                detail: Some(&detail),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            return stop(tx, "failed", &detail);
        }
        let dirty = listing.as_deref().filter(|listing| !listing.trim().is_empty());
        if let Some(listing) = dirty
            && !workflow.read_only
            && !run.stopped
        {
            executions::end(tx, execution, executions::End {
                outcome: "dirty",
                detail: Some(listing),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            // One return per dirty tree: a worker told once and still
            // leaving files is stopped, not looped.
            if row.reason.as_deref() == Some("dirty") {
                return stop(tx, "dirty", &format!("the clone is still dirty:\n{listing}"));
            }
            return attempts::set_next(tx, attempt.id, Some(&json!({ "reason": "dirty", "detail": listing })));
        }
        if touches_yard {
            let detail = ".yard changes only through the operator's `yard sync`; the candidate touching it is refused. Take the .yard change out of your commits.";
            executions::end(tx, execution, executions::End {
                outcome: "refused",
                detail: Some(detail),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            return attempts::set_next(tx, attempt.id, Some(&json!({ "reason": "repair", "detail": detail })));
        }
        if let Some(Err(error)) = &proof_snapshot {
            let detail = error.clone();
            executions::end(tx, execution, executions::End {
                outcome: "refused",
                detail: Some(&detail),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            return stop(tx, "failed", &detail);
        }
        let previous = current.head.clone().unwrap_or_else(|| current.base.clone());
        let new_head = head.clone().unwrap_or_else(|| previous.clone());
        let current_proof = current
            .proof
            .clone()
            .unwrap_or_else(proof::empty_digest);
        let digest = match &proof_snapshot {
            Some(Ok(digest)) => digest.clone(),
            _ => current_proof.clone(),
        };
        // A stopped run never admits a candidate: the tree is kept and the
        // next execution continues. Otherwise a changed proof is a new
        // candidate even when the head is unchanged.
        let advanced = !run.stopped
            && (new_head != previous || (!workflow.read_only && current_proof != digest));
        if !advanced {
            executions::end(tx, execution, executions::End {
                outcome: "unchanged",
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            if workflow.read_only {
                attempts::end(tx, attempt.id, "planned")?;
                return super::audit_attempt(tx, "attempt.ended", &current, None, json!({ "outcome": "planned" })).map(|_| ());
            }
            return stop(tx, "unchanged", "the worker stopped without a new commit");
        }
        tx.execute(
            "UPDATE attempt SET head = ?2, base = ?3, proof = ?4 WHERE id = ?1",
            rusqlite::params![attempt.id, new_head, base, digest],
        )?;
        executions::end(tx, execution, executions::End {
            outcome: "candidate",
            exit_code: run.exit_code,
            ticket: Some(ticket.id),
            ..Default::default()
        })?;
        super::audit_attempt(tx, "attempt.candidate", &current, None, json!({
            "base": base, "head": new_head, "proof": digest,
            "proof_path": proof::snapshot_path(project, attempt.id, &digest).display().to_string(),
        }))?;
        // A nudge queued during the execution reaches the implementer before
        // anything judges the candidate.
        if current.nudge.is_some() {
            attempts::set_next(tx, attempt.id, Some(&json!({ "reason": "nudge" })))?;
        }
        Ok(())
    })
}

async fn implementer_prompt(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    reason: &str,
    resumed: bool,
) -> Result<String, Fail> {
    let next = attempt.next.clone().unwrap_or(json!({}));
    let nudge = attempt.nudge.as_deref();
    let workflow = loaded.config.workflow(&attempt.workflow)?;
    let mut prompt = String::new();
    if !resumed {
        prompt.push_str(&format!(
            "You are working on ticket {}: {}\n\n{}\n\n",
            ticket_name(ticket.id),
            ticket.title,
            ticket.body
        ));
        if !workflow.instructions.is_empty() {
            prompt.push_str(&workflow.instructions);
            prompt.push_str("\n\n");
        }
        if workflow.read_only {
            prompt.push_str("Your workspace is read-only. Plan the work: propose child tickets and an edit of this ticket's body with yard_propose, then stop.\n\n");
        } else {
            prompt.push_str("Work in /workspace on the current branch. Commit your work with git and leave the tree clean; uncommitted or untracked files send the work back to you. Do not change .yard/. Record progress with yard_progress. Files under /yard/proof are snapshotted with the candidate and are not merged. Propose follow-up tickets with yard_propose; if the ticket is too large, propose the split and stop.\n\n");
        }
        if reason != "first"
            && let Some((base, head)) = attempt.candidate()
        {
            let stat = daemon
                .git
                .diff_stat(&project.canonical_dir(), base, head)
                .await?;
            prompt.push_str(&format!("The attempt so far, {base}..{head}:\n{stat}\n"));
            let notes = project.read(|conn| {
                Ok(executions::for_attempt(conn, attempt.id)?
                    .into_iter()
                    .filter_map(|row| row.progress)
                    .collect::<Vec<_>>())
            })?;
            if !notes.is_empty() {
                prompt.push_str("Progress notes:\n");
                for note in notes {
                    prompt.push_str(&format!("- {note}\n"));
                }
                prompt.push('\n');
            }
        }
    }
    let detail = next["detail"].as_str().unwrap_or_default();
    match reason {
        "dirty" => prompt.push_str(&format!(
            "Your clone was not clean when you stopped. Commit or remove these files:\n{detail}\n"
        )),
        "repair" => {
            if let Some(gate) = next["gate"].as_str() {
                prompt.push_str(&format!(
                    "The gate {gate} failed on your candidate:\n{detail}\n"
                ));
            } else if let Some(findings) = next.get("findings") {
                prompt.push_str(&format!(
                    "Review blocked your candidate. Findings:\n{}\n",
                    serde_json::to_string_pretty(findings).unwrap_or_default()
                ));
            } else if let Some(target) = next["target"].as_str() {
                prompt.push_str(&format!(
                    "Your candidate failed to land on the target {target}. {detail}\n"
                ));
                // The bundle exists only when the target is not the base.
                if target != attempt.base {
                    prompt.push_str(&format!(
                        "Your clone lacks the target: `git fetch {}/target.bundle refs/heads/{}` and merge FETCH_HEAD.\n",
                        crate::harness::INPUT_GUEST,
                        next["branch"].as_str().unwrap_or_default()
                    ));
                }
            } else {
                prompt.push_str(&format!("Repair the candidate: {detail}\n"));
            }
        }
        "restart" | "retry" if resumed => prompt.push_str("Continue the work.\n"),
        _ => {}
    }
    if let Some(nudge) = nudge {
        prompt.push_str(&format!("\nThe operator says:\n{nudge}\n"));
    }
    if prompt.trim().is_empty() {
        prompt.push_str("Continue the work.\n");
    }
    Ok(prompt)
}

/// One gate on one commit: a candidate-stage gate, or a landing's child.
/// Run one gate execution. A landing's child passes the landing's lock,
/// which its host child inherits; a candidate gate takes the attempt's.
pub async fn gate(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    execution: i64,
    lock: Option<&Lock>,
) -> Result<(), Fail> {
    let loaded = load(daemon, project).await?;
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
    let name = row.name.clone().unwrap_or_default();
    let commit = row.head.clone().unwrap_or_default();
    let base = row.base.clone().unwrap_or_default();
    let dir = match row.parent {
        Some(landing) => super::queue::landing_dir(project, landing),
        None => project.attempt_dir(attempt.id),
    };
    let checkout = dir.join("gates").join(execution.to_string());
    let outcome = match loaded.config.gates.iter().find(|gate| gate.name == name) {
        None => Err(format!("gate {name} is no longer configured")),
        Some(gate) => {
            let _ = std::fs::remove_dir_all(&checkout);
            match daemon
                .git
                .clone_detached(&project.canonical_dir(), &checkout, &commit)
                .await
            {
                Err(error) => Err(error),
                Ok(()) => match gate.runs_in {
                    RunsIn::Box => {
                        box_gate(daemon, project, &loaded, execution, gate, &checkout, &base).await
                    }
                    RunsIn::Host => {
                        host_gate(project, execution, gate, &checkout, &dir, lock, &base).await
                    }
                },
            }
        }
    };
    let _ = std::fs::remove_dir_all(&checkout);
    let log = dir.join("gates").join(format!("{execution}.log"));
    let (verdict, detail, code, oom, image) = match outcome {
        Ok(result) => {
            let _ = std::fs::write(&log, &result.output);
            let tail = tail(&result.output);
            let verdict = if result.code == 0 { "pass" } else { "fail" };
            (verdict, tail, Some(result.code), result.oom, result.image)
        }
        Err(error) => ("error", error, None, None, None),
    };
    project.tx(|tx| {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: verdict,
                detail: Some(&detail),
                exit_code: code,
                oom_kills: oom,
                ticket: Some(attempt.ticket),
                ..Default::default()
            },
        )?;
        if row.parent.is_some() {
            return Ok(());
        }
        if verdict == "error" {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "red",
                    reason: "gate",
                    ticket: Some(attempt.ticket),
                    attempt: Some(attempt.id),
                    execution: Some(execution),
                    payload: json!({ "gate": name, "detail": detail }),
                    text: Some(&detail),
                },
            )?;
            return Ok(());
        }
        checks::record(
            tx,
            checks::Record {
                execution,
                kind: "gate",
                name: &name,
                input: &checks::Input {
                    attempt: attempt.id,
                    base: row.base.clone().unwrap_or_default(),
                    head: commit.clone(),
                    proof: row.proof.clone().unwrap_or_default(),
                    ticket_revision: row.ticket_revision.unwrap_or_default(),
                    digest: row.digest.clone().unwrap_or_default(),
                },
                verdict,
                image_id: image.as_deref(),
                round: None,
                ticket: attempt.ticket,
            },
        )?;
        Ok(())
    })
}

pub struct GateResult {
    pub code: i32,
    pub output: String,
    pub oom: Option<u64>,
    pub image: Option<String>,
}

fn tail(output: &str) -> String {
    output[output.ceil_char_boundary(output.len().saturating_sub(LOG_TAIL))..].to_string()
}

async fn box_gate(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    execution: i64,
    gate: &crate::config::Gate,
    checkout: &Path,
    base: &str,
) -> Result<GateResult, String> {
    let image = image(daemon, project, loaded)
        .await
        .map_err(|fail| fail.message)?;
    let spec = BoxSpec {
        name: box_name(project, execution),
        labels: labels(project),
        harness: None,
        image,
        mounts: vec![Mount {
            host: checkout.into(),
            guest: "/workspace".into(),
            readonly: false,
        }],
        env: BTreeMap::from([
            ("HOME".to_string(), EnvValue::Value("/tmp".into())),
            (
                "PATH".to_string(),
                EnvValue::Value(
                    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
                ),
            ),
            ("YARD_BASE".to_string(), EnvValue::Value(base.into())),
        ]),
        egress: (!loaded.config.egress.is_empty()).then(|| Egress {
            allow: loaded.config.egress.clone(),
            routes: BTreeMap::new(),
        }),
        memory: daemon.machine.box_memory.clone(),
    };
    let live = daemon
        .pinfold
        .up(&spec, &[], UP_TIMEOUT)
        .await
        .map_err(|error| format!("the gate box did not come up: {error}"))?;
    let _ = project
        .read(|conn| executions::set_handle(conn, execution, &live.name, live.image_id.as_deref()));
    let result = daemon
        .pinfold
        .exec(
            &live.name,
            Some("/workspace"),
            &["sh".to_string(), "-c".to_string(), gate.command.clone()],
            Duration::from_secs(gate.timeout_minutes * 60),
        )
        .await;
    let oom = daemon.pinfold.oom_kills(&live.name).await;
    let image = live.image_id.clone();
    let _ = live.down(DOWN_TIMEOUT).await;
    match result {
        Ok(out) => Ok(GateResult {
            code: out.code,
            output: format!("{}{}", out.stdout, out.stderr),
            oom,
            image,
        }),
        Err(crate::r#box::ExecError::Timeout) => Ok(GateResult {
            code: 124,
            output: format!("the gate ran past its {} minutes", gate.timeout_minutes),
            oom,
            image,
        }),
        Err(error) => Err(format!("the gate could not run: {error}")),
    }
}

/// A host gate: a child in its own process group holding the directory's
/// lock descriptor, with only `PATH`, `HOME`, `YARD_BASE` and the variables
/// it names. Its command starts once its handle is recorded: the child
/// waits for a line on stdin, and exits without running if the daemon dies
/// first.
async fn host_gate(
    project: &Project,
    execution: i64,
    gate: &crate::config::Gate,
    checkout: &Path,
    dir: &Path,
    lock: Option<&Lock>,
    base: &str,
) -> Result<GateResult, String> {
    let own;
    let lock = match lock {
        Some(lock) => lock,
        None => {
            own = hold_lock(dir)?;
            &own
        }
    };
    let mut command = tokio::process::Command::new("sh");
    command
        .args([
            "-c",
            "read -r _ || exit 125; exec sh -c \"$1\" < /dev/null",
            "gate",
        ])
        .arg(&gate.command)
        .current_dir(checkout)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", std::env::var("HOME").unwrap_or_default())
        .env("YARD_BASE", base)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    for name in &gate.env {
        if let Ok(value) = std::env::var(name) {
            command.env(name, value);
        }
    }
    crate::git::inherit(&mut command, lock.as_raw_fd());
    let mut child = command
        .spawn()
        .map_err(|error| format!("spawn the host gate: {error}"))?;
    let pid = child.id().unwrap_or_default();
    let birth = birth(pid).await;
    project
        .read(|conn| {
            executions::set_handle(
                conn,
                execution,
                &json!({ "pgid": pid, "birth": birth }).to_string(),
                None,
            )
        })
        .map_err(|fail| fail.message)?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    tokio::io::AsyncWriteExt::write_all(&mut stdin, b"go\n")
        .await
        .map_err(|error| format!("start the host gate: {error}"))?;
    drop(stdin);
    let limit = Duration::from_secs(gate.timeout_minutes * 60);
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let run = async {
        let (stdout, stderr) = tokio::join!(
            crate::r#box::read_capped(stdout, crate::r#box::OUTPUT_CAP),
            crate::r#box::read_capped(stderr, crate::r#box::OUTPUT_CAP)
        );
        (stdout + &stderr, child.wait().await)
    };
    match tokio::time::timeout(limit, run).await {
        Ok((output, Ok(status))) => Ok(GateResult {
            code: crate::git::exit_code(status),
            output,
            oom: None,
            image: None,
        }),
        Ok((_, Err(error))) => Err(format!("the host gate could not run: {error}")),
        Err(_) => {
            let _ = nix::sys::signal::killpg(
                nix::unistd::Pid::from_raw(pid as i32),
                nix::sys::signal::Signal::SIGKILL,
            );
            Ok(GateResult {
                code: 124,
                output: format!("the gate ran past its {} minutes", gate.timeout_minutes),
                oom: None,
                image: None,
            })
        }
    }
}

pub type Lock = nix::fcntl::Flock<std::fs::File>;

/// Open the directory's lock file and take `flock` on it. Every host child
/// inherits the descriptor, so the lock is held while any child lives.
pub fn hold_lock(dir: &Path) -> Result<Lock, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let file = std::fs::File::create(dir.join("lock")).map_err(|error| error.to_string())?;
    nix::fcntl::Flock::lock(file, nix::fcntl::FlockArg::LockExclusiveNonblock).map_err(|_| {
        format!(
            "{} is held by a child of an earlier daemon",
            dir.join("lock").display()
        )
    })
}

/// Whether nothing holds the directory's lock: taken proves no child can
/// still act. The probe releases it again.
pub fn lock_free(dir: &Path) -> Result<bool, String> {
    let path = dir.join("lock");
    let Ok(file) = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
    else {
        return Ok(true);
    };
    match nix::fcntl::Flock::lock(file, nix::fcntl::FlockArg::LockExclusiveNonblock) {
        Ok(held) => {
            drop(held);
            Ok(true)
        }
        Err((_, nix::errno::Errno::EWOULDBLOCK)) => Ok(false),
        Err((_, error)) => Err(error.to_string()),
    }
}

/// A process's start time as `ps` reports it: its birth identity against
/// pid reuse.
pub async fn birth(pid: u32) -> Option<String> {
    let ps = tokio::process::Command::new("ps")
        .args(["-o", "lstart=", "-p", &pid.to_string()])
        .current_dir("/")
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("LC_ALL", "C")
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(10), ps)
        .await
        .ok()?
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// `yard_context`: the ticket, a brief and the candidate.
pub async fn context(daemon: &Arc<Daemon>, grant: &crate::mcp::Grant) -> Result<Value, Fail> {
    let project = &grant.project;
    let row = project.read(|conn| executions::get(conn, grant.execution))?;
    let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let (base, head) = match grant.kind {
        crate::mcp::Kind::Review => (row.base.clone().unwrap_or_default(), row.head.clone()),
        crate::mcp::Kind::Implementation => (attempt.base.clone(), attempt.head.clone()),
    };
    let stat = match &head {
        Some(head) => daemon
            .git
            .diff_stat(&project.canonical_dir(), &base, head)
            .await
            .unwrap_or_default(),
        None => String::new(),
    };
    Ok(json!({
        "ticket": ticket_name(ticket.id),
        "title": ticket.title,
        "body": ticket.body,
        "revision": ticket.revision,
        "base": base,
        "head": head,
        "brief": stat,
    }))
}
