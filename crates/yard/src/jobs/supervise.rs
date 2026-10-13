//! Box-backed work: the implementer execution, gates in a box or on the
//! host, the project image, and the worker's `yard_context`.

use super::{Loaded, proof};
use crate::api::Fail;
use crate::r#box::{BoxSpec, Egress, EnvValue, Mount, Route};
use crate::config::RunsIn;
use crate::daemon::{Daemon, Project};
use crate::harness::{Harness, ModelRoute, Stage};
use crate::store::{self, attempts, checks, executions, ticket_name, tickets};
use serde_json::{Value, json};
use std::collections::BTreeMap;
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
    reason: &str,
    feedback: Option<&Value>,
) -> Result<i64, Fail> {
    // The revision and body are read here: the row names what the prompt carries.
    let attempt = attempts::get(tx, attempt.id)?;
    let implementer = attempt.implementation_settings()?;
    let ticket = tickets::get(tx, attempt.ticket)?;
    let name = implementer["name"].as_str().unwrap_or("default");
    let harness_version = implementer["harness"]
        .as_str()
        .and_then(crate::harness::get)
        .map(|harness| harness.version());
    let id = executions::start(
        tx,
        executions::Start {
            attempt: attempt.id,
            kind: "implementation",
            reason: Some(reason),
            base: Some(&attempt.base),
            head: attempt.head.as_deref(),
            proof: attempt.proof.as_deref(),
            ticket_revision: Some(ticket.revision),
            ticket: Some(ticket.id),
            agent: Some((name, &implementer)),
            harness_version,
            feedback,
            ..Default::default()
        },
    )?;
    tx.execute(
        "UPDATE execution SET body = ?2 WHERE id = ?1",
        rusqlite::params![id, ticket.body],
    )?;
    for item in attempts::open_for_attempt(tx, attempt.id)? {
        if item.kind == "proposal"
            && item.reason == "edit"
            && item.payload["revision"]
                .as_i64()
                .is_some_and(|revision| revision < ticket.revision)
        {
            attempts::resolve(tx, &item, "stale", None)?;
        }
    }
    Ok(id)
}

/// Build from the admitted source, independently of observed runtime image IDs.
/// Live containment can revoke admitted access, never expand it.
pub async fn egress(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
) -> Result<Vec<String>, Fail> {
    let policy = super::load_target(daemon, project, &loaded.branch).await?;
    Ok(loaded
        .config
        .egress
        .iter()
        .filter(|host| policy.config.egress.contains(host))
        .cloned()
        .collect())
}

pub async fn image(daemon: &Daemon, project: &Project, loaded: &Loaded) -> Result<String, Fail> {
    let _building = daemon.image_build.lock().await;
    // `forget` retires the name under this lock; a build that queued behind it must not recreate it.
    if !daemon
        .projects
        .lock()
        .expect("projects lock")
        .contains_key(&project.root)
    {
        return Err(Fail::new("image", "the project was forgotten"));
    }
    if let Some((branch, head, reference)) =
        daemon.images.lock().expect("images lock").get(&project.key)
        && *head == loaded.image_commit
        && *branch == loaded.branch
    {
        return Ok(reference.clone());
    }
    let context = project.local().join("build").join(&loaded.image_commit);
    let _ = std::fs::remove_dir_all(&context);
    daemon
        .git
        .clone_detached(&project.canonical_dir(), &context, &loaded.image_commit)
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
    daemon.images.lock().expect("images lock").insert(
        project.key.clone(),
        (
            loaded.branch.clone(),
            loaded.image_commit.clone(),
            built.clone(),
        ),
    );
    Ok(built)
}

/// The image may be gone from the runtime; forget it so the next attempt
/// rebuilds. Rebuilding a present image is a cache hit.
fn forget_image(daemon: &Daemon, reference: &str) {
    daemon
        .images
        .lock()
        .expect("images lock")
        .retain(|_, (_, _, cached)| cached != reference);
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
            forget_image(daemon, &spec.image);
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
    /// The proof directory mounted at `/yard/proof`: the implementer's live
    /// worker-written directory (writable), or a review/audit snapshot
    /// (read-only, matching the workspace).
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
    if let Some(proof) = worker.proof {
        mounts.push(Mount {
            host: proof.into(),
            guest: "/yard/proof".into(),
            readonly: worker.read_only,
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
    state: &Path,
    transcript: &Path,
    inactivity: Duration,
    deadline: tokio::time::Instant,
) -> Result<Run, Fail> {
    let stop = daemon
        .stops
        .lock()
        .expect("stops lock")
        .entry((project.key.clone(), execution))
        .or_insert_with(|| Arc::new(tokio::sync::Notify::new()))
        .clone();
    if kind == crate::mcp::Kind::Review {
        notify_if_edited(project, execution, &stop);
    }
    if harness_stopped(daemon, project, execution).await? {
        return Ok(Run {
            stopped: true,
            ..Default::default()
        });
    }
    std::fs::create_dir_all(transcript.parent().expect("transcript dir"))
        .map_err(|error| error.to_string())?;
    let mut file = tokio::fs::File::create(transcript)
        .await
        .map_err(|error| error.to_string())?;
    if harness_stopped(daemon, project, execution).await? {
        return Ok(Run {
            stopped: true,
            ..Default::default()
        });
    }
    let predecessor = project.read(|conn| {
        executions::get(conn, execution)?
            .resumed
            .map(|id| executions::get(conn, id))
            .transpose()
    })?;
    let mut reader = harness.reader(predecessor.is_some());
    reader.prepare(
        state,
        predecessor
            .as_ref()
            .and_then(|row| row.session_id.as_deref()),
    );
    let mut child = daemon
        .pinfold
        .exec_streaming(&box_name(project, execution), Some("/workspace"), argv)
        .map_err(|error| error.to_string())?;
    let mut stdout = BufReader::new(child.stdout.take().expect("piped")).lines();
    let mut stderr = BufReader::new(child.stderr.take().expect("piped")).lines();
    let mut run = Run::default();
    let mut usage = None;
    let mut stdout_open = true;
    let mut stderr_open = true;
    let mut observations = tokio::time::interval(Duration::from_millis(100));
    let mut idle_deadline = tokio::time::Instant::now() + inactivity;
    let activity_key = (project.key.clone(), execution);
    daemon
        .activity
        .lock()
        .expect("activity lock")
        .insert(activity_key.clone(), std::time::Instant::now());
    while stdout_open || stderr_open {
        tokio::select! {
            line = stdout.next_line(), if stdout_open => match line {
                Ok(Some(line)) => {
                    idle_deadline = tokio::time::Instant::now() + inactivity;
                    daemon.activity.lock().expect("activity lock")
                        .insert(activity_key.clone(), std::time::Instant::now());
                    let _ = file.write_all(line.as_bytes()).await;
                    let _ = file.write_all(b"\n").await;
                    let mut refused = false;
                    for event in reader.stdout(&line) {
                        match &event {
                            crate::harness::Event::Started { session_id } => {
                                run.session_id = Some(session_id.clone());
                                project.read(|conn| executions::set_worker(conn, execution, Some(session_id), None))?;
                            }
                            crate::harness::Event::Registered(registration) => {
                                let registration = complete(harness, registration.clone(), kind);
                                project.read(|conn| executions::set_mcp(conn, execution, proof(&registration)))?;
                                refused = matches!(registration, crate::harness::Registration::Refused(_));
                                run.registered = Some(registration);
                            }
                            crate::harness::Event::Usage { usage: spent } => {
                                project.read(|conn| executions::set_worker(conn, execution, None, Some(spent)))?;
                                usage = Some(spent.clone());
                            }
                            crate::harness::Event::Finished { usage: spent, .. } | crate::harness::Event::Failed { usage: spent, .. } => {
                                usage = Some(spent.clone());
                                // A required-server harness reaches Yard's MCP server
                                // over TCP while its frames come over the box's stdout
                                // pipe; neither channel orders the other. The terminal
                                // turn is the last point to see the handshake, and a
                                // run without one is ungated and fails.
                                if harness.registration_is_connection()
                                    && matches!(run.registered, Some(crate::harness::Registration::Registered(_)))
                                    && !daemon.grants.connected(project, execution)
                                {
                                    let registration = crate::harness::Registration::Refused(
                                        "the Yard MCP server was never contacted".into(),
                                    );
                                    project.read(|conn| executions::set_mcp(conn, execution, proof(&registration)))?;
                                    run.registered = Some(registration);
                                    refused = true;
                                }
                                run.terminal = Some(event.clone());
                            }
                        }
                    }
                    // A refused registration ends the run before its first
                    // turn can do anything; the caller revokes the grant.
                    if refused {
                        break;
                    }
                }
                _ => stdout_open = false,
            },
            line = stderr.next_line(), if stderr_open => match line {
                Ok(Some(line)) => {
                    idle_deadline = tokio::time::Instant::now() + inactivity;
                    daemon.activity.lock().expect("activity lock")
                        .insert(activity_key.clone(), std::time::Instant::now());
                    if let Some(registration) = reader.stderr(&line) {
                        let registration = complete(harness, registration, kind);
                        let refused = matches!(registration, crate::harness::Registration::Refused(_));
                        project.read(|conn| executions::set_mcp(conn, execution, proof(&registration)))?;
                        run.registered = Some(registration);
                        if refused {
                            break;
                        }
                    }
                }
                _ => stderr_open = false,
            },
            _ = stop.notified() => { run.stopped = true; break; }
            _ = observations.tick(), if harness.name() == "codex" => {
                if let Some(spent) = reader.observe() {
                    project.read(|conn| executions::set_worker(conn, execution, None, Some(&spent)))?;
                    usage = Some(spent);
                }
            }
            _ = tokio::time::sleep_until(idle_deadline) => { run.timed_out = true; break; }
            _ = tokio::time::sleep_until(deadline) => { run.timed_out = true; break; }
        }
    }
    daemon
        .activity
        .lock()
        .expect("activity lock")
        .remove(&activity_key);
    if !run.stopped && !run.timed_out && !refused(&run.registered) {
        run.exit_code = child.wait().await.ok().map(crate::git::exit_code);
    }
    if let Some(spent) = reader.observe() {
        usage = Some(spent);
    }
    if (run.stopped || run.timed_out)
        && let Some(spent) = &mut usage
    {
        for field in [
            &mut spent.fields.input,
            &mut spent.fields.output,
            &mut spent.fields.cost,
        ] {
            if field.completeness == crate::harness::Completeness::Complete {
                field.completeness = crate::harness::Completeness::Partial;
            }
        }
    }
    project.read(|conn| executions::set_worker(conn, execution, None, usage.as_ref()))?;
    Ok(run)
}

fn refused(registration: &Option<crate::harness::Registration>) -> bool {
    matches!(registration, Some(crate::harness::Registration::Refused(_)))
}

/// The durable marker for one registration. `checks::current` counts a
/// review only from an execution whose row says `registered`.
fn proof(registration: &crate::harness::Registration) -> &'static str {
    match registration {
        crate::harness::Registration::Registered(_) => "registered",
        crate::harness::Registration::Refused(_) => "refused",
    }
}

/// A registration is complete only when every tool the grant names is in the
/// fetched list. An incomplete one is refused, and gates the run. A harness
/// whose required MCP server is the whole proof has not listed its tools: it
/// records the tools the server serves, and the terminal turn checks that it
/// reached the server at all.
fn complete(
    harness: &'static dyn Harness,
    registration: crate::harness::Registration,
    kind: crate::mcp::Kind,
) -> crate::harness::Registration {
    let crate::harness::Registration::Registered(tools) = registration else {
        return registration;
    };
    let tools = if tools.is_empty() && harness.registration_is_connection() {
        crate::mcp::tool_names(kind)
    } else {
        tools
    };
    match crate::mcp::tool_names(kind)
        .into_iter()
        .find(|tool| !tools.contains(tool))
    {
        Some(missing) => {
            crate::harness::Registration::Refused(format!("the MCP client registered no {missing}"))
        }
        None => crate::harness::Registration::Registered(tools),
    }
}

pub async fn implement(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    execution: i64,
) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::for_execution(conn, &row))?;
    let Some(loaded) = preparation_result(
        daemon,
        project,
        execution,
        super::load_attempt(daemon, project, &attempt).await,
    )
    .await?
    else {
        return Ok(());
    };
    // The prompt carries the body the start transaction recorded; a later edit
    // stays pending for the next execution.
    let mut ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    if let Some(body) = project.read(|conn| {
        Ok(conn.query_row(
            "SELECT body FROM execution WHERE id = ?1",
            [execution],
            |row| row.get::<_, Option<String>>(0),
        )?)
    })? {
        ticket.body = body;
    }
    let workflow = loaded.config.workflow(&attempt.workflow)?.clone();
    let plan = match &attempt.admission {
        Some(admission) => crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?,
        None => loaded.config.workflow_plan(&attempt.workflow)?,
    };
    let packaged = plan.packaged();
    let implementer = attempt.implementation_settings()?;
    let provider = implementer["provider"].as_str();
    let connection = match provider {
        Some(name) => Some(
            crate::harness::connection(name)
                .ok_or_else(|| Fail::invalid(format!("connection {name:?} is unknown")))?,
        ),
        None => None,
    };
    let harness: &'static dyn Harness = if packaged.is_some() {
        &crate::pi::PackagedAudit
    } else {
        crate::harness::get(implementer["harness"].as_str().unwrap_or_default()).ok_or_else(
            || Fail::invalid(format!("harness {:?} is unknown", implementer["harness"])),
        )?
    };
    let dir = project.attempt_dir(attempt.id);
    let audit_dir = dir.join("audits").join(execution.to_string());
    let clone = if workflow.read_only {
        audit_dir.join("checkout")
    } else {
        dir.join("clone")
    };
    if stop_preparation(daemon, project, execution, None).await? {
        return Ok(());
    }
    let cloned = async {
        if workflow.read_only {
            // No predecessor can mount this directory. Refuse an existing path
            // rather than stage into guest-written state after an interruption.
            std::fs::create_dir_all(audit_dir.parent().expect("audit parent"))
                .map_err(|error| error.to_string())?;
            std::fs::create_dir(&audit_dir).map_err(|error| error.to_string())?;
            daemon
                .git
                .clone_detached(
                    &project.canonical_dir(),
                    &clone,
                    row.head.as_deref().unwrap_or(&attempt.base),
                )
                .await?;
        } else if !clone.exists() {
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
        Ok(())
    }
    .await;
    if preparation_result(daemon, project, execution, cloned)
        .await?
        .is_none()
    {
        return Ok(());
    }
    let storage = if workflow.read_only { &audit_dir } else { &dir };
    let state = storage.join("state");
    let input = storage.join("input");
    let proof_dir = (|| {
        if workflow.read_only {
            let snapshots = audit_dir.join("proof-snapshots");
            let source = row
                .proof
                .as_deref()
                .filter(|digest| !digest.is_empty())
                .map(|digest| proof::snapshot_path(project, attempt.id, digest));
            let empty = audit_dir.join("empty-proof");
            if source.is_none() {
                std::fs::create_dir(&empty).map_err(|error| error.to_string())?;
            }
            let digest = proof::snapshot(source.as_deref().unwrap_or(&empty), &snapshots)?;
            Ok::<_, Fail>(snapshots.join(digest))
        } else {
            let live = dir.join("proof");
            std::fs::create_dir_all(&live).map_err(|error| error.to_string())?;
            Ok(live)
        }
    })();
    let Some(proof_dir) = preparation_result(daemon, project, execution, proof_dir).await? else {
        return Ok(());
    };
    let env = if packaged.is_some() {
        Ok(crate::agent_env::AgentEnv::empty(&clone))
    } else {
        crate::agent_env::AgentEnv::load(
            &daemon.git,
            &project.canonical_dir(),
            if attempt.admission.is_some() {
                &loaded.staging_commit
            } else {
                &attempt.base
            },
            &clone,
        )
        .await
        .map_err(Fail::refused)
    };
    let Some(env) = preparation_result(daemon, project, execution, env).await? else {
        return Ok(());
    };
    let stage = Stage {
        state: &state,
        input: &input,
        connection,
        env: &env,
    };
    if preparation_result(
        daemon,
        project,
        execution,
        harness
            .stage(&stage)
            .map_err(|error| Fail::from(error.to_string())),
    )
    .await?
    .is_none()
    {
        return Ok(());
    }
    if let Some(crate::plan::Runner::Packaged {
        skill,
        references,
        files,
        instructions,
        ..
    }) = packaged
    {
        let staged = async {
            let trusted = input.join("trusted");
            std::fs::create_dir(&trusted).map_err(|error| error.to_string())?;
            let admitted = attempt.admission.as_ref().expect("admitted plan");
            for source in admitted["inputs"]["trusted_staged_sources"]
                .as_array()
                .ok_or_else(|| Fail::refused("missing selected inputs"))?
            {
                let path = source["path"]
                    .as_str()
                    .ok_or_else(|| Fail::refused("invalid selected path"))?;
                let blob = source["blob"]
                    .as_str()
                    .ok_or_else(|| Fail::refused("invalid selected blob"))?;
                let bytes = daemon.git.blob(&project.canonical_dir(), blob).await?;
                if path == skill && std::str::from_utf8(&bytes).is_err() {
                    return Err(Fail::refused(
                        "invalid_input: selected Markdown is not UTF-8",
                    ));
                }
                let dest = trusted.join(path);
                std::fs::create_dir_all(dest.parent().expect("selected parent"))
                    .map_err(|error| error.to_string())?;
                std::fs::write(dest, bytes).map_err(|error| error.to_string())?;
            }
            let mut declared = files.clone();
            declared.extend(std::iter::once(skill).chain(references).map(|path| {
                crate::plan::AuditFile {
                    root: "trusted".into(),
                    path: path.clone(),
                }
            }));
            let mut prompt = format!("{instructions}\n{}\n{}", ticket.title, ticket.body);
            prompt.push_str(&plan_handoff(daemon, project, &attempt, &row).await?);
            if prompt.len() > 32768 {
                return Err(Fail::refused("invalid_input: audit prompt bound"));
            }
            let launch = json!({"provider": provider, "model": implementer["model"],
            "prompt": prompt, "files": declared, "skill": format!("trusted:{skill}"),
            "identity": admitted["inputs"]["package"]});
            let bytes = launch.to_string();
            if bytes.len() > 131072 {
                return Err(Fail::refused("invalid_input: audit launch bound"));
            }
            std::fs::write(input.join("audit.json"), bytes).map_err(|error| error.to_string())?;
            Ok::<_, Fail>(())
        }
        .await;
        if preparation_result(daemon, project, execution, staged)
            .await?
            .is_none()
        {
            return Ok(());
        }
    }

    // Writable sessions resume until their execution bound or version changes.
    let resume = match project.read(|conn| {
        if workflow.read_only {
            Ok(None)
        } else {
            executions::session(conn, attempt.id, execution)
        }
    })? {
        Some((previous, count))
            if count < i64::from(workflow.max_session_executions)
                && previous.harness_version.as_deref() == Some(harness.version()) =>
        {
            previous.session_id.map(|session| (previous.id, session))
        }
        _ => None,
    };
    project.read(|conn| {
        conn.execute("UPDATE attempt SET next = NULL WHERE id = ?1", [attempt.id])?;
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
        resume.as_ref().map(|(id, _)| *id),
        &row,
    )
    .await;
    let Some(prompt) = preparation_result(daemon, project, execution, prompt).await? else {
        return Ok(());
    };
    let argv = harness
        .argv(&crate::harness::Launch {
            provider,
            model: implementer["model"].as_str().unwrap_or_default(),
            effort: implementer["effort"].as_str(),
            resume: resume.as_ref().map(|(_, session)| session.as_str()),
            prompt: &prompt,
            env: &env,
        })
        .map_err(Fail::invalid);
    let Some(argv) = preparation_result(daemon, project, execution, argv).await? else {
        return Ok(());
    };

    let Some(image) = preparation_result(
        daemon,
        project,
        execution,
        image(daemon, project, &loaded).await,
    )
    .await?
    else {
        return Ok(());
    };
    // Resolve the route, including a login's token, before any bearer is
    // issued, so a missing credential never leaves a live grant.
    let model = harness
        .route(&stage, &daemon.machine)
        .map_err(Fail::refused);
    let Some(model) = preparation_result(daemon, project, execution, model).await? else {
        return Ok(());
    };
    let target_lock = project.target_lock(&crate::git::target_ref(attempt.target_branch()?));
    let target = target_lock.lock().await;
    let Some(egress) = preparation_result(
        daemon,
        project,
        execution,
        egress(daemon, project, &loaded).await,
    )
    .await?
    else {
        return Ok(());
    };
    let kind = if workflow.read_only {
        crate::mcp::Kind::Audit
    } else {
        crate::mcp::Kind::Implementation
    };
    let bearer = daemon.grants.issue(crate::mcp::Grant {
        project: project.clone(),
        execution,
        kind,
    });
    let mut secrets = vec![(crate::harness::BEARER_VAR.to_string(), bearer)];
    secrets.extend(model.secret.clone());
    let spec = worker_spec(
        daemon,
        project,
        &Worker {
            execution,
            image: &image,
            workspace: &clone,
            read_only: workflow.read_only,
            proof: Some(proof_dir.as_path()),
            harness,
            stage,
            model: &model,
            egress: &egress,
        },
    );
    let live = match up_worker(daemon, project, execution, &spec, &secrets).await {
        Ok(live) => live,
        Err(fail) => {
            return preparation_result::<()>(daemon, project, execution, Err(fail))
                .await
                .map(|_| ());
        }
    };
    // A successful startup owns a live controller. Let run_harness consume
    // any pending stop, then use the normal teardown path below.
    drop(target);
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
        kind,
        harness,
        &argv,
        &state,
        &transcript(project, attempt.id, execution),
        Duration::from_secs(workflow.inactivity_timeout_minutes * 60),
        deadline,
    )
    .await;
    let mut run = match run {
        Ok(run) => run,
        Err(fail) => {
            daemon.grants.revoke(project, execution);
            let _ = live.down(DOWN_TIMEOUT).await;
            return Err(fail);
        }
    };
    // The harness has ended; pull its execution-scoped bearer before any
    // host-side inspection, so a refused run cannot call MCP tools meanwhile.
    daemon.grants.revoke(project, execution);
    // Ask git inside the box whether the clone is clean, before it comes down.
    // A timeout never reaches the candidate. A stopped run is deferred whole:
    // its status would be taken while the worker may still write, so it never
    // admits a candidate and keeps its tree for the next execution.
    let listing = if run.stopped || run.timed_out || refused(&run.registered) {
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
    // A refused registration is already a failure; no box inspection needed.
    let oom = if refused(&run.registered) {
        None
    } else {
        daemon.pinfold.oom_kills(&live.name).await
    };
    // The proof is only read once the box is confirmed down; a failed teardown
    // refuses the candidate below.
    let teardown = live.down(DOWN_TIMEOUT).await;

    // Take the candidate's objects by a fetch run in canonical.
    let fetched = if workflow.read_only {
        Ok(())
    } else {
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
    let head = if workflow.read_only {
        Some(row.head.clone().unwrap_or_else(|| attempt.base.clone()))
    } else {
        match &fetched {
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
        }
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
    run.stopped |= take_stop(daemon, project, execution).await;
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
            // A run that finished with no registration proof at all was not
            // gated; it fails like a refused one.
            (None, Some(_)) if !run.stopped => Some((
                "failed",
                Some("mcp"),
                "the worker registered no MCP client".to_string(),
            )),
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
        let report = executions::get(tx, execution)?.report;
        let edited = attempts::edit_pending(tx, attempt.id)?;
        let failure = failure.or_else(|| {
            (workflow.read_only && !run.stopped && !edited && report.is_none())
                .then(|| ("failed", Some("publication"), "missing_report".to_string()))
        });
        let stop = |tx: &rusqlite::Connection, reason: &str, detail: &str| -> Result<(), Fail> {
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
                ..Default::default()
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
        // An edit run that leaves the existing candidate as it is sends it to
        // judgment at the new revision.
        let rejudge = row.reason.as_deref() == Some("edit")
            && current.head.is_some()
            && !workflow.read_only
            && !edited;
        let advanced = !run.stopped
            && (new_head != previous || (!workflow.read_only && current_proof != digest) || rejudge);
        if !advanced {
            executions::end(tx, execution, executions::End {
                outcome: "unchanged",
                output: (workflow.read_only && !run.stopped && !edited).then(|| json!({"kind": "report", "value": report})),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            })?;
            // With an edit pending the implementer runs again.
            if edited && !run.stopped {
                return Ok(());
            }
            if workflow.read_only && !run.stopped {
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
            output: Some(json!({"kind": "candidate", "value": {"base": base, "head": new_head, "proof": digest}})),
            exit_code: run.exit_code,
            ticket: Some(ticket.id),
            ..Default::default()
        })?;
        super::audit_attempt(tx, "attempt.candidate", &current, None, json!({
            "base": base, "head": new_head, "proof": digest,
            "proof_path": proof::snapshot_path(project, attempt.id, &digest).display().to_string(),
        }))?;
        Ok(())
    })
}

async fn take_stop(daemon: &Daemon, project: &Project, execution: i64) -> bool {
    let stop = daemon
        .stops
        .lock()
        .expect("stops lock")
        .get(&(project.key.clone(), execution))
        .cloned();
    let Some(stop) = stop else {
        return false;
    };
    tokio::select! {
        biased;
        _ = stop.notified() => true,
        _ = std::future::ready(()) => false,
    }
}

async fn harness_stopped(daemon: &Daemon, project: &Project, execution: i64) -> Result<bool, Fail> {
    let stopped = take_stop(daemon, project, execution).await;
    project.read(|conn| {
        let row = executions::get(conn, execution)?;
        Ok(stopped || row.status != "running" || attempts::get(conn, row.attempt)?.state != "live")
    })
}

// Check stop after the handoff returns under capacity::effect's lock.
// A successful box startup keeps its LiveBox on the normal teardown path.
async fn preparation_result<T>(
    daemon: &Daemon,
    project: &Project,
    execution: i64,
    result: Result<T, Fail>,
) -> Result<Option<T>, Fail> {
    let detail = result.as_ref().err().map(|fail| fail.message.as_str());
    if stop_preparation(daemon, project, execution, detail).await? {
        return Ok(None);
    }
    result.map(Some)
}

// A preparation child finishes under the effect lock before this row ends.
// capacity::effect still requires ownership proof before releasing its slot.
async fn stop_preparation(
    daemon: &Daemon,
    project: &Project,
    execution: i64,
    detail: Option<&str>,
) -> Result<bool, Fail> {
    let stopped = take_stop(daemon, project, execution).await;
    project.tx(|tx| {
        let row = executions::get(tx, execution)?;
        if row.status != "running" {
            return Ok(true);
        }
        let attempt = attempts::get(tx, row.attempt)?;
        if attempt.state != "live" {
            executions::end(tx, execution, executions::End {
                outcome: "abandoned", ticket: Some(attempt.ticket), detail, ..Default::default()
            })?;
            return Ok(true);
        }
        if !stopped {
            return Ok(false);
        }
        executions::end(tx, execution, executions::End {
            outcome: "unchanged", ticket: Some(attempt.ticket), detail, ..Default::default()
        })?;
        attempts::raise(tx, attempts::Raise {
            kind: "stopped", reason: "unchanged", ticket: Some(attempt.ticket),
            attempt: Some(attempt.id), execution: Some(execution),
            payload: json!({ "detail": detail.unwrap_or("the operator stopped the execution during preparation") }),
            text: None,
        })?;
        Ok(true)
    })
}

// An edit that landed before this execution's stop handle existed found nothing
// to notify; the revision the execution started on tells.
fn notify_if_edited(project: &Project, execution: i64, stop: &tokio::sync::Notify) {
    let moved = project
        .read(|conn| {
            let row = executions::get(conn, execution)?;
            let attempt = attempts::get(conn, row.attempt)?;
            let ticket = tickets::get(conn, attempt.ticket)?;
            Ok(row.parent.is_none()
                && row
                    .ticket_revision
                    .is_some_and(|revision| revision != ticket.revision)
                || (row.parent.is_none() && executions::plan_superseded(conn, &attempt, &row)?))
        })
        .unwrap_or(false);
    if moved {
        stop.notify_one();
    }
}

async fn implementer_prompt(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    resumed: Option<i64>,
    execution: &executions::Execution,
) -> Result<String, Fail> {
    let reason = execution.reason.as_deref().unwrap_or("first");
    let next = execution
        .inputs
        .as_ref()
        .map(|inputs| &inputs["feedback"]["value"])
        .filter(|value| !value.is_null())
        .cloned()
        .or_else(|| attempt.next.clone())
        .unwrap_or(json!({}));
    let workflow = loaded.config.workflow(&attempt.workflow)?;
    let plan = attempt
        .admission
        .as_ref()
        .map(crate::plan::Plan::admitted)
        .transpose()
        .map_err(|error| Fail::refused(error.to_string()))?;
    let instructions = plan
        .as_ref()
        .and_then(|plan| match &plan.nodes["implementer"].runner {
            crate::plan::Runner::Agent { instructions, .. } => Some(instructions.as_str()),
            _ => None,
        })
        .unwrap_or(&workflow.instructions);
    let mut prompt = String::new();
    // The resumed session's owner launched its harness with this body.
    let body_read: Option<String> = match resumed {
        Some(owner) => project.read(|conn| {
            Ok(
                conn.query_row("SELECT body FROM execution WHERE id = ?1", [owner], |row| {
                    row.get(0)
                })?,
            )
        })?,
        None => None,
    };
    let resumed = resumed.is_some();
    if !resumed || body_read.is_none() {
        prompt.push_str(&format!(
            "You are working on ticket {}: {}\n\n{}\n\n",
            ticket_name(ticket.id),
            ticket.title,
            ticket.body
        ));
        if !instructions.is_empty() {
            prompt.push_str(instructions);
            prompt.push_str("\n\n");
        }
        prompt.push_str(&format!("Target branch: {}.\n\n", attempt.target_branch()?));
        if workflow.read_only {
            prompt.push_str("Your workspace is read-only. Plan the work: propose child tickets and an edit of this ticket's body with yard_propose. Publish exactly once with yard_publish_report: outcome complete, report with a nonempty summary and a findings list (empty if none), then stop.\n\n");
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
    if resumed && let Some(read) = body_read.as_deref().filter(|read| *read != ticket.body) {
        let diff = body_diff(daemon, project, attempt.id, read, &ticket.body).await?;
        prompt.push_str(&format!("\nThe operator edited the ticket:\n{diff}\n"));
    }
    prompt.push_str(&plan_handoff(daemon, project, attempt, execution).await?);
    if prompt.trim().is_empty() {
        prompt.push_str("Continue the work.\n");
    }
    Ok(prompt)
}

async fn plan_handoff(
    daemon: &Daemon,
    project: &Project,
    attempt: &attempts::Attempt,
    execution: &executions::Execution,
) -> Result<String, Fail> {
    let previous = project.read(|conn| {
        Ok(executions::for_attempt(conn, attempt.id)?
            .into_iter()
            .rev()
            .find(|row| row.kind == "implementation" && row.id < execution.id))
    })?;
    if let Some(previous) = previous
        && previous.plan_revision != execution.plan_revision
    {
        let before = project.read(|conn| attempts::for_execution(conn, &previous))?;
        let evidence = |attempt: &attempts::Attempt| -> Result<String, Fail> {
            let Some(admission) = &attempt.admission else {
                return Ok("null".into());
            };
            let plan = crate::plan::Plan::admitted(admission)
                .map_err(|error| Fail::refused(error.to_string()))?;
            let definitions: BTreeMap<_, _> = plan
                .nodes
                .iter()
                .map(|(name, node)| {
                    (
                        name,
                        executions::consumed_definition(
                            if name.starts_with("landing-gate-") {
                                attempt.original_admission.as_ref().unwrap_or(admission)
                            } else {
                                admission
                            },
                            &node.runner,
                            name.starts_with("review-"),
                        ),
                    )
                })
                .collect();
            Ok(
                serde_json::to_string_pretty(&json!({"plan": plan, "definitions": definitions}))
                    .expect("plan evidence serializes"),
            )
        };
        let old = evidence(&before)?;
        let new = evidence(attempt)?;
        let diff = body_diff(daemon, project, attempt.id, &old, &new).await?;
        return Ok(format!("\nThe operator revised the plan:\n{diff}\n"));
    }
    Ok(String::new())
}

/// The unified diff from the body the previous execution read to the current one.
async fn body_diff(
    daemon: &Daemon,
    project: &Project,
    attempt: i64,
    before: &str,
    after: &str,
) -> Result<String, Fail> {
    let dir = project.attempt_dir(attempt).join("edit");
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    std::fs::write(dir.join("before"), before).map_err(|error| error.to_string())?;
    std::fs::write(dir.join("after"), after).map_err(|error| error.to_string())?;
    let out = daemon
        .git
        .run(
            &dir,
            &["diff", "--no-index", "--no-color", "--", "before", "after"],
        )
        .await?;
    let _ = std::fs::remove_dir_all(&dir);
    if out.code > 1 {
        return Err(Fail::from(format!(
            "git diff exited {}: {}",
            out.code,
            out.stderr.trim()
        )));
    }
    Ok(out.stdout)
}

/// One gate on one commit: a candidate-stage gate, or a landing's child.
/// Run one gate execution inside its reserved effect scope.
pub async fn gate(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    execution: i64,
) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::for_execution(conn, &row))?;
    let loaded = super::load_attempt(daemon, project, &attempt).await?;
    let name = row.name.clone().unwrap_or_default();
    let commit = row.head.clone().unwrap_or_default();
    let base = row.base.clone().unwrap_or_default();
    // The snapshot the execution was judged on: the candidate's proof, or a
    // landing child's approval proof. Both are worker-written bytes already
    // copied into the attempt's `proof-snapshots`.
    let proof = row
        .proof
        .as_deref()
        .filter(|digest| !digest.is_empty())
        .map(|digest| proof::snapshot_path(project, attempt.id, digest));
    let dir = match row.parent {
        Some(landing) => super::queue::landing_dir(project, landing),
        None => project.attempt_dir(attempt.id),
    };
    let stop = Arc::new(tokio::sync::Notify::new());
    daemon
        .stops
        .lock()
        .expect("stops lock")
        .insert((project.key.clone(), execution), stop.clone());
    notify_if_edited(project, execution, &stop);
    let checkout = dir.join("gates").join(execution.to_string());
    let gates = if row.parent.is_some() {
        loaded.landing_gates(&attempt)?
    } else {
        loaded.config.gates.clone()
    };
    let outcome = match gates.iter().find(|gate| gate.name == name) {
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
                        box_gate(
                            daemon,
                            project,
                            &loaded,
                            execution,
                            gate,
                            &checkout,
                            &base,
                            proof.as_deref(),
                            &stop,
                        )
                        .await
                    }
                    RunsIn::Host => {
                        host_gate(
                            project,
                            execution,
                            gate,
                            &checkout,
                            &base,
                            proof.as_deref(),
                            &stop,
                        )
                        .await
                    }
                },
            }
        }
    };
    let _ = std::fs::remove_dir_all(&checkout);
    let log = dir.join("gates").join(format!("{execution}.log"));
    let stopped = matches!(&outcome, Ok(result) if result.stopped);
    let (verdict, detail, code, oom, infrastructure) = match outcome {
        Ok(result) if result.stopped => {
            ("stopped", String::new(), None, None, result.infrastructure)
        }
        Ok(result) => {
            let _ = std::fs::write(&log, &result.output);
            let tail = tail(&result.output);
            let verdict = if result.infrastructure != "completed" {
                "error"
            } else if result.code == 0 {
                "pass"
            } else {
                "fail"
            };
            (
                verdict,
                tail,
                Some(result.code),
                result.oom,
                result.infrastructure,
            )
        }
        Err(error) => ("error", error, None, None, "error"),
    };
    project.tx(|tx| {
        if attempts::get(tx, attempt.id)?.state != "live" {
            return executions::end(
                tx,
                execution,
                executions::End {
                    outcome: "abandoned",
                    ticket: Some(attempt.ticket),
                    ..Default::default()
                },
            );
        }
        // An edit that landed while the gate ran superseded its check: no
        // decision, red or check comes of it.
        let moved = row.parent.is_none()
            && (executions::plan_superseded(tx, &attempts::get(tx, attempt.id)?, &row)?
                || row.ticket_revision.is_some_and(|read| {
                    attempts::superseded_by_edit(tx, attempt.id, attempt.ticket, read)
                        .unwrap_or(true)
                }));
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: verdict,
                infrastructure: Some(infrastructure),
                detail: Some(&detail),
                exit_code: code,
                oom_kills: oom,
                ticket: Some(attempt.ticket),
                ..Default::default()
            },
        )?;
        if row.parent.is_some() || stopped || moved {
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
                verdict,
                ticket: attempt.ticket,
            },
        )?;
        Ok(())
    })
}

pub struct GateResult {
    pub infrastructure: &'static str,
    /// An edit stopped the gate; nothing it produced counts.
    pub stopped: bool,
    pub code: i32,
    pub output: String,
    pub oom: Option<u64>,
}

fn tail(output: &str) -> String {
    output[output.ceil_char_boundary(output.len().saturating_sub(LOG_TAIL))..].to_string()
}

// Each argument is a separate input of the one gate run; a struct would
// only rename them.
#[allow(clippy::too_many_arguments)]
async fn box_gate(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    execution: i64,
    gate: &crate::config::Gate,
    checkout: &Path,
    base: &str,
    proof: Option<&Path>,
    stop: &tokio::sync::Notify,
) -> Result<GateResult, String> {
    let image = image(daemon, project, loaded)
        .await
        .map_err(|fail| fail.message)?;
    let mut mounts = vec![Mount {
        host: checkout.into(),
        guest: "/workspace".into(),
        readonly: false,
    }];
    if let Some(proof) = proof {
        mounts.push(Mount {
            host: proof.into(),
            guest: "/yard/proof".into(),
            readonly: true,
        });
    }
    let target_lock = project.target_lock(&crate::git::target_ref(&loaded.branch));
    let target = target_lock.lock().await;
    let egress = egress(daemon, project, loaded)
        .await
        .map_err(|fail| fail.message)?;
    let spec = BoxSpec {
        name: box_name(project, execution),
        labels: labels(project),
        harness: None,
        image,
        mounts,
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
        egress: (!egress.is_empty()).then(|| Egress {
            allow: egress,
            routes: BTreeMap::new(),
        }),
        memory: daemon.machine.box_memory.clone(),
    };
    let live = match daemon.pinfold.up(&spec, &[], UP_TIMEOUT).await {
        Ok(live) => live,
        Err(error) => {
            forget_image(daemon, &spec.image);
            return Err(format!("the gate box did not come up: {error}"));
        }
    };
    let _ = project
        .read(|conn| executions::set_handle(conn, execution, &live.name, live.image_id.as_deref()));
    drop(target);
    let argv = [
        "sh".to_string(),
        "-c".to_string(),
        "exec sh -c \"$1\" 2>&1".to_string(),
        "gate".to_string(),
        gate.command.clone(),
    ];
    let exec = daemon.pinfold.exec(
        &live.name,
        Some("/workspace"),
        &argv,
        Duration::from_secs(gate.timeout_minutes * 60),
    );
    let result = tokio::select! {
        result = exec => Some(result),
        _ = stop.notified() => None,
    };
    let oom = daemon.pinfold.oom_kills(&live.name).await;
    live.down(DOWN_TIMEOUT).await?;
    match result {
        None => Ok(GateResult {
            infrastructure: "stopped",
            stopped: true,
            code: 0,
            output: String::new(),
            oom,
        }),
        Some(Ok(out)) => Ok(GateResult {
            infrastructure: "completed",
            stopped: false,
            code: out.code,
            output: format!("{}{}", out.stdout, out.stderr),
            oom,
        }),
        Some(Err(crate::r#box::ExecError::Timeout)) => Ok(GateResult {
            infrastructure: "timeout",
            stopped: false,
            code: 124,
            output: format!("the gate ran past its {} minutes", gate.timeout_minutes),
            oom,
        }),
        Some(Err(error)) => Err(format!("the gate could not run: {error}")),
    }
}

/// A host gate: a child in its own process group holding the directory's
/// lock descriptor, with only `PATH`, `HOME`, `YARD_BASE`, `YARD_PROOF` and
/// the variables it names. `YARD_PROOF` is the host path of the snapshot
/// this execution judges. Its command starts once its handle is recorded:
/// the child waits for a line on stdin, and exits without running if the
/// daemon dies first.
// Each argument is a separate input of the one gate run; a struct would
// only rename them.
#[allow(clippy::too_many_arguments)]
async fn host_gate(
    project: &Project,
    execution: i64,
    gate: &crate::config::Gate,
    checkout: &Path,
    base: &str,
    proof: Option<&Path>,
    stop: &tokio::sync::Notify,
) -> Result<GateResult, String> {
    let mut command = tokio::process::Command::new("sh");
    command
        .args([
            "-c",
            "read -r _ || exit 125; exec sh -c \"$1\" < /dev/null 2>&1",
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
        .stderr(std::process::Stdio::null())
        .process_group(0)
        .kill_on_drop(true);
    if let Some(proof) = proof {
        command.env("YARD_PROOF", proof);
    }
    for name in &gate.env {
        if let Ok(value) = std::env::var(name) {
            command.env(name, value);
        }
    }
    let fd = super::capacity::HOST_LOCK
        .try_with(|fd| *fd)
        .map_err(|_| "host gate has no execution reservation".to_string())?;
    crate::git::inherit(&mut command, fd);
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
    let run = async {
        let output = crate::r#box::read_tail(stdout, crate::r#box::OUTPUT_CAP).await;
        (output, child.wait().await)
    };
    let kill = || {
        let _ = nix::sys::signal::killpg(
            nix::unistd::Pid::from_raw(pid as i32),
            nix::sys::signal::Signal::SIGKILL,
        );
    };
    let timed = tokio::select! {
        result = tokio::time::timeout(limit, run) => result,
        _ = stop.notified() => {
            kill();
            return Ok(GateResult { infrastructure: "stopped", stopped: true, code: 0, output: String::new(), oom: None });
        }
    };
    match timed {
        Ok((output, Ok(status))) => Ok(GateResult {
            infrastructure: "completed",
            stopped: false,
            code: crate::git::exit_code(status),
            output,
            oom: None,
        }),
        Ok((_, Err(error))) => Err(format!("the host gate could not run: {error}")),
        Err(_) => {
            kill();
            Ok(GateResult {
                infrastructure: "timeout",
                stopped: false,
                code: 124,
                output: format!("the gate ran past its {} minutes", gate.timeout_minutes),
                oom: None,
            })
        }
    }
}

/// Open the directory's lock file and take `flock` on it. Every host child
/// inherits the descriptor, so the lock is held while any child lives.
pub fn hold_lock(dir: &Path) -> Result<std::fs::File, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let file = std::fs::File::create(dir.join("lock")).map_err(|error| error.to_string())?;
    // Flock's Drop explicitly unlocks the shared open file description.
    // Close only our descriptor; inherited owners must keep the lock.
    file.try_lock().map_err(|_| {
        format!(
            "{} is held by a child of an earlier daemon",
            dir.join("lock").display()
        )
    })?;
    Ok(file)
}

/// Whether nothing holds the directory's lock: taken proves no child can
/// still act. The probe releases it again.
pub fn lock_free(dir: &Path) -> Result<bool, String> {
    let path = dir.join("lock");
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(error.to_string()),
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
        crate::mcp::Kind::Implementation | crate::mcp::Kind::Audit => {
            (attempt.base.clone(), attempt.head.clone())
        }
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
        "target": attempt.target_branch,
        "base": base,
        "head": head,
        "brief": stat,
    }))
}
