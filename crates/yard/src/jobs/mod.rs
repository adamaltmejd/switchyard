//! Admission, the execution state machine, the merge queue and
//! reconcile-on-start. `step` is the scheduler tick: it reads rows, decides
//! the next executions, records each before anything external runs, and
//! spawns it.

pub mod admit;
pub mod capacity;
pub mod cleanup;
pub mod proof;
pub mod queue;
pub mod reconcile;
pub mod report;
pub mod review;
pub mod supervise;

use crate::api::Fail;
use crate::config::{Approve, Config};
use crate::daemon::{Daemon, Project};
use crate::store::{self, Target, attempts, checks, executions, tickets};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

/// Validated configuration with exact source identities.
#[derive(Clone)]
pub struct Loaded {
    pub config: Config,
    pub branch: String,
    pub head: String,
    pub gate_digest: String,
    pub admission_inputs: Value,
}

#[derive(Default)]
pub struct Configurations {
    pub targets: HashMap<String, Arc<Loaded>>,
    pub default: Option<String>,
    pub sources: HashMap<String, Arc<Loaded>>,
}

/// Automatic readers cannot restore an input an uncertain mutation invalidated.
pub async fn refresh_default(
    daemon: &Daemon,
    project: &Project,
    _canonical: &tokio::sync::MutexGuard<'_, ()>,
) -> Result<Arc<Loaded>, Fail> {
    if project
        .loaded
        .lock()
        .expect("loaded lock")
        .default
        .is_none()
    {
        return Err(Fail::refused(
            "the project default configuration is not loaded",
        ));
    }
    read_default(daemon, project).await
}

/// Startup or a proved mutation publishes while its target is still held.
pub async fn publish_default(
    daemon: &Daemon,
    project: &Project,
    canonical: &tokio::sync::MutexGuard<'_, ()>,
    branch: &str,
    _target: &tokio::sync::MutexGuard<'_, ()>,
) -> Result<Arc<Loaded>, Fail> {
    if daemon.git.head_branch(&project.canonical_dir()).await? != branch {
        return refresh_default(daemon, project, canonical).await;
    }
    read_default(daemon, project).await
}

async fn read_default(daemon: &Daemon, project: &Project) -> Result<Arc<Loaded>, Fail> {
    project.loaded.lock().expect("loaded lock").default = None;
    let loaded = load(daemon, project).await?;
    let mut configs = project.loaded.lock().expect("loaded lock");
    configs.default = Some(loaded.branch.clone());
    configs
        .targets
        .insert(loaded.branch.clone(), loaded.clone());
    drop(configs);
    project.events.notify_waiters();
    Ok(loaded)
}

impl Loaded {
    pub fn review_digest(&self, workflow: &str) -> Result<String, Fail> {
        Ok(self.config.review_digest(self.config.workflow(workflow)?))
    }
}

pub async fn load(daemon: &Daemon, project: &Project) -> Result<Arc<Loaded>, Fail> {
    let branch = daemon.git.head_branch(&project.canonical_dir()).await?;
    load_target(daemon, project, &branch).await
}

pub async fn load_target(
    daemon: &Daemon,
    project: &Project,
    branch: &str,
) -> Result<Arc<Loaded>, Fail> {
    let canonical = project.canonical_dir();
    if !daemon.git.valid_branch(&canonical, branch).await? {
        return Err(
            Fail::invalid("target requires a local branch name").with(json!({ "target": branch }))
        );
    }
    let head = daemon
        .git
        .rev_parse(&canonical, &crate::git::target_ref(branch))
        .await?;
    let head = head.ok_or_else(|| {
        Fail::refused(format!(
            "canonical has no target {branch}; run `yard sync {branch}`"
        ))
        .with(json!({ "reason": "missing_target", "target": branch }))
    })?;
    if let Some(loaded) = project
        .loaded
        .lock()
        .expect("loaded lock")
        .targets
        .get(branch)
        && loaded.head == head
        && loaded.branch == branch
    {
        return Ok(loaded.clone());
    }
    let loaded = Arc::new(read_config(daemon, &canonical, branch, None, &head).await?);
    let mut configs = project.loaded.lock().expect("loaded lock");
    // Only canonical's reader/writer publishes the current default input.
    // A concurrent cache miss must not replace it with an earlier head.
    if configs.default.as_deref() != Some(branch) {
        configs.targets.insert(branch.to_string(), loaded.clone());
    }
    Ok(loaded)
}

/// The configuration at `commit`, validated.
pub async fn read_config(
    daemon: &Daemon,
    repo: &std::path::Path,
    branch: &str,
    default: Option<&str>,
    commit: &str,
) -> Result<Loaded, Fail> {
    let text = daemon
        .git
        .show(repo, commit, crate::config::CONFIG_PATH)
        .await?
        .ok_or_else(|| Fail::invalid(format!("{commit} has no .yard/config.toml")))?;
    let config = Config::parse(&text).map_err(|message| {
        Fail::invalid(format!(".yard/config.toml: {message}")).with(json!({ "config": message }))
    })?;
    if let Some(default) = default
        && config.target.branch != default
    {
        return Err(Fail::invalid(format!(
            ".yard/config.toml names target {:?}; this project's default is {default:?}",
            config.target.branch
        )).with(json!({ "reason": "default_mismatch", "target": config.target.branch, "default": default })));
    }
    let dockerfile = daemon
        .git
        .show(repo, commit, crate::config::DOCKERFILE_PATH)
        .await?
        .ok_or_else(|| Fail::invalid(format!("{commit} has no .yard/Dockerfile")))?;
    let tree = daemon.git.root_tree(repo, commit).await?;
    let entries = daemon
        .git
        .ls_tree(
            repo,
            commit,
            &[crate::config::CONFIG_PATH, crate::config::DOCKERFILE_PATH],
        )
        .await?;
    let objects: std::collections::BTreeMap<_, _> = entries
        .into_iter()
        .map(|entry| (entry.path, entry.oid))
        .collect();
    let staged: Vec<Value> = daemon
        .git
        .ls_tree(repo, commit, &crate::agent_env::PATHS)
        .await?
        .iter()
        .filter(|entry| crate::agent_env::staged_input(entry))
        .map(|entry| json!({"path": entry.path, "blob": entry.oid, "executable": entry.exec}))
        .collect();
    Ok(Loaded {
        admission_inputs: json!({
            "source_commit": commit,
            "build_context_tree": tree,
            "config_blob": objects[crate::config::CONFIG_PATH],
            "dockerfile_blob": objects[crate::config::DOCKERFILE_PATH],
            "trusted_staged_sources": staged,
        }),
        gate_digest: config.gate_digest(dockerfile.as_bytes()),
        config,
        branch: branch.to_string(),
        head: commit.to_string(),
    })
}

/// Resolve recorded work from its immutable admission source. Legacy work
/// retains supported exits without invented historical evidence.
pub async fn load_attempt(
    daemon: &Daemon,
    project: &Project,
    attempt: &attempts::Attempt,
) -> Result<Arc<Loaded>, Fail> {
    let Some(admission) = &attempt.admission else {
        return load_target(daemon, project, attempt.target_branch()?).await;
    };
    let head = admission["inputs"]["source_commit"]
        .as_str()
        .ok_or_else(|| Fail::refused("missing admitted source"))?;
    let cached = project
        .loaded
        .lock()
        .expect("loaded lock")
        .sources
        .get(head)
        .cloned();
    let loaded = match cached {
        Some(loaded) => loaded,
        None => {
            let loaded = Arc::new(
                read_config(
                    daemon,
                    &project.canonical_dir(),
                    attempt.target_branch()?,
                    None,
                    head,
                )
                .await?,
            );
            project
                .loaded
                .lock()
                .expect("loaded lock")
                .sources
                .insert(head.to_string(), loaded.clone());
            loaded
        }
    };
    let admitted =
        crate::plan::Plan::admitted(admission).map_err(|error| Fail::refused(error.to_string()))?;
    let expected = if admission["runtime"] == "ordinary-linear" {
        let mut selection = admission["selection"].clone();
        selection["project"] = json!(project.root);
        let request =
            serde_json::from_value(selection).map_err(|error| Fail::refused(error.to_string()))?;
        loaded
            .config
            .resolve_plan(&request)
            .map_err(|error| Fail::refused(error.to_string()))?
    } else {
        loaded.config.workflow_plan(&attempt.workflow)?
    };
    if loaded.admission_inputs != admission["inputs"]
        || expected != admitted
        || loaded
            .config
            .runnable_workflow(&admitted, Some(&attempt.workflow))
            .is_none()
    {
        return Err(Fail::refused(
            "admitted source does not match recorded inputs",
        ));
    }
    Ok(Arc::new(resolved(&loaded, attempt)?))
}

/// Consume selected definitions from the persisted resolver evidence.
fn resolved(loaded: &Loaded, attempt: &attempts::Attempt) -> Result<Loaded, Fail> {
    let mut loaded = loaded.clone();
    loaded.branch = attempt.target_branch()?.to_string();
    if let Some(admission) = &attempt.admission {
        loaded.config.workflows.insert(
            attempt.workflow.clone(),
            serde_json::from_value(admission["workflow"].clone())
                .map_err(|error| Fail::refused(error.to_string()))?,
        );
        loaded.config.gates = serde_json::from_value(admission["gates"].clone())
            .map_err(|error| Fail::refused(error.to_string()))?;
        if !admission["review"].is_null() {
            loaded.config.review = serde_json::from_value(admission["review"].clone())
                .map_err(|error| Fail::refused(error.to_string()))?;
        }
        loaded.config.seats = serde_json::from_value(admission["seats"].clone())
            .map_err(|error| Fail::refused(error.to_string()))?;
        for (name, agent) in admission["agents"]
            .as_object()
            .ok_or_else(|| Fail::refused("invalid admitted agents"))?
        {
            loaded.config.agents.insert(
                name.clone(),
                serde_json::from_value(agent["settings"].clone())
                    .map_err(|error| Fail::refused(error.to_string()))?,
            );
        }
    }
    Ok(loaded)
}

pub async fn step(daemon: &Arc<Daemon>, project: &Arc<Project>) -> Result<(), Fail> {
    capacity::reap(daemon, project).await?;
    let canonical = project.canonical.lock().await;
    let Ok(_loaded) = refresh_default(daemon, project, &canonical).await else {
        return Ok(());
    };
    // Live attempts reacquire a freed lane before new tickets are admitted
    // (the scheduler also reserves lanes for them under the admission lock),
    // so a repair returning from the queue never starves behind admissions.
    drop(canonical);
    let live = project.read(attempts::live)?;
    for attempt in live {
        let selected = load_attempt(daemon, project, &attempt).await?;
        advance(daemon, project, &selected, &attempt).await?;
    }
    let canonical = project.canonical.lock().await;
    let loaded = refresh_default(daemon, project, &canonical).await?;
    admit::scheduled(daemon, project, &loaded).await?;
    drop(canonical);
    queue::next(daemon, project).await?;
    cleanup::next(daemon, project)?;
    Ok(())
}

/// Spawn the work of an execution already recorded.
pub fn spawn(daemon: &Arc<Daemon>, project: &Arc<Project>, kind: &str, execution: i64) {
    let daemon = daemon.clone();
    let project = project.clone();
    let kind = kind.to_string();
    if kind == "implementation" {
        daemon.stops.lock().expect("stops lock").insert(
            (project.key.clone(), execution),
            Arc::new(tokio::sync::Notify::new()),
        );
    }
    tokio::spawn(async move {
        let result = if kind == "landing" {
            queue::land(&daemon, &project, execution).await
        } else {
            capacity::effect(&daemon, &project, execution, async {
                match kind.as_str() {
                    "implementation" => supervise::implement(&daemon, &project, execution).await,
                    "gate" => supervise::gate(&daemon, &project, execution).await,
                    "review" => review::run(&daemon, &project, execution).await,
                    "cleanup" => cleanup::run(&daemon, &project, execution).await,
                    _ => unreachable!("five kinds"),
                }
            })
            .await
        };
        if let Err(fail) = result {
            eprintln!(
                "yard daemon: {}: execution {execution}: {}",
                project.root.display(),
                fail.message
            );
            // The execution could not record its own end: record it as an
            // error so nothing waits on it.
            let _ = project.tx(|tx| {
                let row = executions::get(tx, execution)?;
                if row.status == "running" {
                    let ended = row.kind != "cleanup"
                        && attempts::get(tx, row.attempt)?.outcome.as_deref() == Some("abandoned");
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: if ended { "abandoned" } else { "error" },
                            detail: Some(&fail.message),
                            ..Default::default()
                        },
                    )?;
                    let attempt = attempts::get(tx, row.attempt)?;
                    // A gate or seat that read a pre-edit candidate records nothing.
                    let edited = row.parent.is_none()
                        && matches!(row.kind.as_str(), "gate" | "review")
                        && row.ticket_revision.is_some_and(|revision| {
                            attempts::superseded_by_edit(tx, attempt.id, attempt.ticket, revision)
                                .unwrap_or(false)
                        });
                    // An edit withdrew the approval a landing ran on.
                    let withdrawn = match (row.kind.as_str(), row.approval) {
                        ("landing", Some(id)) => {
                            crate::store::checks::approval(tx, id)?.ticket_revision
                                != crate::store::tickets::get(tx, attempt.ticket)?.revision
                        }
                        _ => false,
                    };
                    if row.kind != "cleanup" && !ended && !edited && !withdrawn {
                        attempts::raise(
                            tx,
                            attempts::Raise {
                                kind: "red",
                                reason: "error",
                                ticket: Some(attempt.ticket),
                                attempt: (row.kind != "landing").then_some(row.attempt),
                                execution: Some(execution),
                                payload: json!({ "detail": fail.message }),
                                text: Some(&fail.message),
                            },
                        )?;
                    }
                }
                Ok(())
            });
        }
        daemon
            .stops
            .lock()
            .expect("stops lock")
            .remove(&(project.key.clone(), execution));
        daemon.wake.notify_one();
    });
}

/// Whether a live attempt could start its next execution once it holds a
/// lane. `Some` if so, carrying the stale approval item it must supersede.
fn runnable(
    conn: &rusqlite::Connection,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
) -> Result<Option<Option<attempts::Attention>>, Fail> {
    let executions = executions::for_attempt(conn, attempt.id)?;
    if executions
        .iter()
        .any(|row| row.status == "running" || row.reserved)
    {
        return Ok(None);
    }
    let open = attempts::open_for_attempt(conn, attempt.id)?;
    // A pending edit outranks an edit proposal made from the pre-edit text.
    let edited = attempts::edit_pending(conn, attempt.id)?;
    if open.iter().any(|item| {
        (item.kind != "proposal" && item.kind != "approval")
            || (item.reason == "edit" && !(edited && item.kind == "proposal"))
    }) {
        return Ok(None);
    }
    let ticket = tickets::get(conn, attempt.ticket)?;
    if checks::active_for(conn, attempt.id)?.is_some() {
        return Ok(None);
    }
    // An approval item counts while everything it binds is current; a new
    // head, ticket revision or digest supersedes it.
    match open.into_iter().find(|item| item.kind == "approval") {
        Some(item) if stale_part(&item.payload, attempt, &ticket, loaded)?.is_none() => Ok(None),
        item => Ok(Some(item)),
    }
}

/// Decide the next execution for one live attempt, if any.
async fn advance(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
) -> Result<(), Fail> {
    let Some(superseded) = project.read(|conn| runnable(conn, loaded, attempt))? else {
        return Ok(());
    };
    if let Some(item) = superseded {
        project.tx(|tx| attempts::resolve(tx, &item, "superseded", None))?;
    }
    let executions = project.read(|conn| executions::for_attempt(conn, attempt.id))?;
    let edited = project.read(|conn| attempts::edit_pending(conn, attempt.id))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    if !attempt.lane {
        let canonical = project.canonical.lock().await;
        let Ok(default) = refresh_default(daemon, project, &canonical).await else {
            return Ok(());
        };
        if !admit::take_lane(daemon, project, &default, attempt.id)? {
            return Ok(());
        }
    }
    if let Some(next) = &attempt.next {
        let reason = next["reason"].as_str().unwrap_or("repair").to_string();
        let execution =
            project.tx(|tx| supervise::start_implementation(tx, attempt, &reason, None))?;
        spawn(daemon, project, "implementation", execution);
        return Ok(());
    }
    // A pending edit is derived: the ticket is newer than the last implementer's read.
    if edited {
        let execution =
            project.tx(|tx| supervise::start_implementation(tx, attempt, "edit", None))?;
        spawn(daemon, project, "implementation", execution);
        return Ok(());
    }
    let Some((base, head)) = attempt.candidate() else {
        return Ok(());
    };
    let workflow = loaded.config.workflow(&attempt.workflow)?;
    let last_implementation = executions
        .iter()
        .filter(|row| row.kind == "implementation")
        .map(|row| row.id)
        .max()
        .unwrap_or(0);
    if executions
        .iter()
        .rev()
        .find(|row| row.kind == "implementation" && row.outcome.as_deref() == Some("candidate"))
        .is_some_and(|row| row.node.is_some() && !row.result_eligible)
    {
        return Ok(());
    }

    let shared = executions::Start {
        attempt: attempt.id,
        base: Some(base),
        head: Some(head),
        proof: attempt.proof.as_deref(),
        ticket_revision: Some(ticket.revision),
        ticket: Some(ticket.id),
        ..Default::default()
    };
    let gate_input = checks::Input {
        plan_revision: attempt.plan_revision(),
        attempt: attempt.id,
        base: base.to_string(),
        head: head.to_string(),
        proof: attempt.proof.clone().unwrap_or_default(),
        ticket_revision: ticket.revision,
        digest: loaded.gate_digest.clone(),
    };
    let review_digest = loaded.config.review_digest(workflow);
    let plan = match &attempt.admission {
        Some(admission) => crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?,
        None => loaded.config.workflow_plan(&attempt.workflow)?,
    };
    let mut passed = Vec::new();
    let mut blocked = Vec::new();
    let mut current = "implementer";
    let routed = attempt
        .admission
        .as_ref()
        .is_some_and(|admission| admission["plan"]["inputs"].get("feedback").is_some());
    // Only the candidate/review portion consumes routes here. The queue owns
    // the external merged input and all effects beyond this boundary.
    while let Some(route) = plan.nodes[current]
        .routes
        .get("complete")
        .and_then(|routes| routes.first())
    {
        current = &route.to;
        let node = &plan.nodes[current];
        if node
            .inputs
            .values()
            .any(|input| input.node.is_none() && input.output == "merged")
        {
            break;
        }
        let (kind, name, digest) = match &node.runner {
            crate::plan::Runner::Command { gate } => {
                ("gate", gate.as_str(), loaded.gate_digest.as_str())
            }
            crate::plan::Runner::Agent { .. } => {
                let index = current
                    .strip_prefix("review-")
                    .and_then(|index| index.parse::<usize>().ok())
                    .ok_or_else(|| Fail::refused("unsupported admitted review node"))?;
                (
                    "review",
                    workflow.review[index].as_str(),
                    review_digest.as_str(),
                )
            }
        };
        let input = checks::Input {
            digest: digest.to_string(),
            ..gate_input.clone()
        };
        let check = project.read(|conn| checks::current(conn, kind, name, &input))?;
        let Some(check) = check else {
            let agent = match &node.runner {
                crate::plan::Runner::Agent { agent, .. } => Some(agent.as_str()),
                _ => None,
            };
            let settings = agent.map(|agent| {
                serde_json::to_value(&loaded.config.agents[agent]).expect("agent serializes")
            });
            let harness_version = agent
                .and_then(|agent| crate::harness::get(&loaded.config.agents[agent].harness))
                .map(|harness| harness.version());
            let execution = project.tx(|tx| {
                if attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)? {
                    return Ok(None);
                }
                executions::start(
                    tx,
                    executions::Start {
                        kind,
                        name: Some(name),
                        digest: Some(digest),
                        reason: Some(if kind == "gate" { "candidate" } else { "first" }),
                        round: (kind == "review").then_some(attempt.rounds + 1),
                        agent: agent.zip(settings.as_ref()),
                        harness_version,
                        ..shared
                    },
                )
                .map(Some)
            })?;
            if let Some(execution) = execution {
                spawn(daemon, project, kind, execution);
            }
            return Ok(());
        };
        let result = project.read(|conn| executions::get(conn, check.execution))?;
        if check.verdict == "pass" {
            passed.push(check.id);
        } else if kind == "review" {
            blocked.push(check.clone());
        }
        // Intermediate seats complete their panel input even when they block.
        // The final seat's bounded outcome is the accumulated panel decision.
        // Old complete-only results retain the same decision without rewriting history.
        let outcome = if routed {
            result
                .result
                .as_ref()
                .and_then(|result| result["outcome"].as_str())
                .ok_or_else(|| Fail::refused("eligible judgment has no outcome"))?
        } else if kind == "gate" && check.verdict != "pass" {
            "failed"
        } else if node.outcomes.iter().any(|outcome| outcome == "blocked") && !blocked.is_empty() {
            "blocked"
        } else {
            "complete"
        };
        if outcome == "complete" {
            continue;
        }
        let route = node
            .routes
            .get(outcome)
            .and_then(|routes| routes.first())
            .ok_or_else(|| Fail::refused("judgment has no admitted outcome route"))?;
        if route.to != "implementer" || route.repeat.is_none() {
            return Err(Fail::refused("unsupported admitted repair route"));
        }
        // One eligible result buys one back edge, never a reversal of a result.
        if check.execution <= last_implementation {
            return Ok(());
        }
        let execution = if kind == "review" {
            review::blocked(
                project,
                loaded,
                attempt,
                &ticket,
                &blocked,
                route.repeat.expect("bounded"),
                &check,
            )?
        } else {
            let feedback = json!({"kind": "verdict", "source": {
                "external": "feedback", "execution": check.execution, "node": current, "output": "result"},
                "value": {"verdict": "fail", "gate": name, "detail": result.detail.unwrap_or_default()}});
            project.tx(|tx| {
                if attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)? {
                    return Ok(None);
                }
                supervise::start_implementation(tx, attempt, "repair", Some(&feedback)).map(Some)
            })?
        };
        if let Some(execution) = execution {
            spawn(daemon, project, "implementation", execution);
        }
        return Ok(());
    }

    let target_lock = project.target_lock(&crate::git::target_ref(attempt.target_branch()?));
    let _target = target_lock.lock().await;
    let policy = load_target(daemon, project, attempt.target_branch()?).await?;
    let protected = protected_paths(daemon, project, &policy, base, head).await?;
    let payload = json!({
        "target": attempt.target_branch, "base": base, "head": head, "proof": attempt.proof.clone().unwrap_or_default(),
        "revision": ticket.revision, "plan_revision": attempt.plan_revision(),
        "gate_digest": loaded.gate_digest, "review_digest": review_digest,
        "checks": passed, "protected": protected, "unreviewed": workflow.review.is_empty(),
    });
    project.tx(|tx| {
        // The paths were read outside this transaction: the candidate must
        // still be current, or an ended attempt gets an item nothing answers.
        let now = attempts::get(tx, attempt.id)?;
        if now.state != "live"
            || attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)?
            || stale_part(&payload, &now, &tickets::get(tx, ticket.id)?, loaded)?.is_some()
        {
            return Ok(());
        }
        if policy.config.approve == Approve::Auto && protected.is_empty() {
            let proof = attempt.proof.clone().unwrap_or_default();
            checks::approve(
                tx,
                checks::Approve {
                    attempt: attempt.id,
                    ticket: ticket.id,
                    base,
                    head,
                    proof: &proof,
                    ticket_revision: ticket.revision,
                    gate_digest: &loaded.gate_digest,
                    review_digest: &review_digest,
                    checks: &passed,
                    actor: "auto",
                    text: None,
                    overrode: false,
                },
            )?;
            attempts::set_lane(tx, attempt.id, false)
        } else {
            raise_approval(tx, attempt, &payload).map(drop)
        }
    })
}

/// Ask the operator to approve the candidate `payload` names.
pub fn raise_approval(
    tx: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    payload: &Value,
) -> Result<i64, Fail> {
    let protected = payload["protected"]
        .as_array()
        .is_some_and(|paths| !paths.is_empty());
    attempts::raise(
        tx,
        attempts::Raise {
            kind: "approval",
            reason: if protected { "protected" } else { "verified" },
            ticket: Some(attempt.ticket),
            attempt: Some(attempt.id),
            execution: None,
            payload: payload.clone(),
            text: None,
        },
    )
}

/// The first part of an approval item's candidate, ticket revision and
/// digests that is no longer current: `(key, recorded, current)`.
pub fn stale_part(
    payload: &Value,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    loaded: &Loaded,
) -> Result<Option<(&'static str, Value, Value)>, Fail> {
    let (base, head) = attempt.candidate().unwrap_or_default();
    for (key, current) in [
        ("target", json!(attempt.target_branch)),
        ("base", json!(base)),
        ("head", json!(head)),
        ("proof", json!(attempt.proof.clone().unwrap_or_default())),
        ("revision", json!(ticket.revision)),
        ("plan_revision", json!(attempt.plan_revision())),
        ("gate_digest", json!(loaded.gate_digest)),
        (
            "review_digest",
            json!(loaded.review_digest(&attempt.workflow)?),
        ),
    ] {
        if payload[key] != current {
            return Ok(Some((key, payload[key].clone(), current)));
        }
    }
    Ok(None)
}

/// The protected paths `base..head` touches.
pub async fn protected_paths(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    base: &str,
    head: &str,
) -> Result<Vec<String>, Fail> {
    let paths = daemon
        .git
        .changed_paths(&project.canonical_dir(), base, head)
        .await?;
    Ok(paths
        .into_iter()
        .filter(|path| loaded.config.is_protected(path))
        .collect())
}

pub async fn command(daemon: &Arc<Daemon>, method: &str, params: Value) -> Result<Value, Fail> {
    let project = crate::daemon::project(daemon, &params)?;
    match method {
        "plan.resolve" | "plan.validate" => inspect_plan(daemon, &project, method, params).await,
        "status" => admit::status(daemon, &project).await,
        "sync" => admit::sync(daemon, &project, &params).await,
        "doctor" => admit::doctor(daemon, &project).await,
        "ticket.new" => admit::ticket_new(daemon, &project, &params).await,
        "ticket.show" => admit::ticket_show(&project, &params),
        "ticket.list" => admit::ticket_list(&project),
        "ticket.edit" => admit::ticket_edit(daemon, &project, &params).await,
        "ticket.park" => admit::ticket_park(&project, &params, true),
        "ticket.unpark" => admit::ticket_park(&project, &params, false),
        "ticket.depend" => admit::ticket_depend(&project, &params),
        "ticket.done" => admit::ticket_close(&project, &params, "done"),
        "ticket.abandon" => admit::ticket_close(&project, &params, "abandoned"),
        "attempt.start" => admit::attempt_start(daemon, &project, &params).await,
        "attempt.stop" => admit::attempt_stop(daemon, &project, &params),
        "attempt.approve" => admit::attempt_approve(daemon, &project, &params).await,
        "attempt.reject" => admit::attempt_reject(&project, &params),
        "attempt.abandon" => admit::attempt_abandon(daemon, &project, &params),
        "attempt.show" => admit::attempt_show(&project, &params),
        "attempt.stats" => admit::attempt_stats(&project, &params),
        "attempt.diff" => admit::attempt_diff(daemon, &project, &params).await,
        "attempt.tail" => admit::attempt_tail(&project, &params),
        "proposal.accept" => admit::proposal_answer(daemon, &project, &params, true).await,
        "proposal.reject" => admit::proposal_answer(daemon, &project, &params, false).await,
        _ => Err(Fail::invalid(format!("unknown method {method:?}"))),
    }
}

/// `yard_propose`: a proposal is an attention item. A proposal to edit the
/// worker's own ticket pauses its attempt until decided.
pub fn propose(grant: &crate::mcp::Grant, arguments: &Value) -> Result<Value, Fail> {
    let object = crate::mcp::strict(
        arguments,
        &[
            "kind",
            "key",
            "title",
            "body",
            "priority",
            "workflow",
            "parked",
            "depends_on",
            "ticket",
            "reason",
        ],
    )?;
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| Fail::invalid("kind is required"))?;
    let text = |key: &str| object.get(key).and_then(Value::as_str);
    match kind {
        "ticket" => {
            if text("title").is_none_or(|title| title.trim().is_empty()) {
                return Err(Fail::invalid("a ticket proposal needs a title"));
            }
            if let Some(priority) = text("priority")
                && crate::config::priority(priority).is_none()
            {
                return Err(Fail::invalid(format!(
                    "priority {priority:?} is not P0 to P3"
                )));
            }
        }
        "edit" => {
            if text("title").is_none() && text("body").is_none() {
                return Err(Fail::invalid("an edit proposal needs a title or a body"));
            }
        }
        "link" => {
            if text("ticket").is_none() || object.get("depends_on").is_none() {
                return Err(Fail::invalid("a link proposal needs ticket and depends_on"));
            }
        }
        other => {
            return Err(Fail::invalid(format!(
                "kind {other:?} is ticket, edit or link"
            )));
        }
    }
    if let Some(depends) = object.get("depends_on")
        && !depends
            .as_array()
            .is_some_and(|items| items.iter().all(Value::is_string))
    {
        return Err(Fail::invalid(
            "depends_on is a list of ticket names or keys",
        ));
    }
    let id = grant.project.tx(|tx| {
        let execution = executions::get(tx, grant.execution)?;
        let attempt = attempts::get(tx, execution.attempt)?;
        let mut payload = Value::Object(object.clone());
        if kind == "edit" {
            // Bind the revision the proposing execution read, so accepting
            // it after the ticket moved is stale instead of overwriting (G2).
            let revision = execution
                .ticket_revision
                .ok_or_else(|| Fail::invalid("the proposing execution names no ticket revision"))?;
            payload["revision"] = json!(revision);
        }
        attempts::raise(
            tx,
            attempts::Raise {
                kind: "proposal",
                reason: kind,
                ticket: Some(attempt.ticket),
                attempt: Some(attempt.id),
                execution: Some(grant.execution),
                payload,
                text: text("reason"),
            },
        )
    })?;
    Ok(json!({ "proposal": id }))
}

pub fn audit_attempt(
    tx: &rusqlite::Connection,
    event: &str,
    attempt: &attempts::Attempt,
    text: Option<&str>,
    data: Value,
) -> Result<i64, Fail> {
    let target = Target {
        ticket: Some(attempt.ticket),
        attempt: Some(attempt.id),
        ..Target::default()
    };
    store::audit(tx, event, target, text, data)
}

/// Trusted inspection reads one target identity and creates no store rows.
async fn inspect_plan(
    daemon: &Daemon,
    project: &Project,
    method: &str,
    params: Value,
) -> Result<Value, Fail> {
    let request: crate::plan::Request =
        serde_json::from_value(params).map_err(|error| Fail::invalid(error.to_string()))?;
    let mut branch = match &request.target {
        Some(branch) => branch.clone(),
        None => daemon.git.head_branch(&project.canonical_dir()).await?,
    };
    let mut target_lock = project.target_lock(&crate::git::target_ref(&branch));
    let (_target, _canonical) = loop {
        let target = target_lock.lock().await;
        let canonical = project.canonical.lock().await;
        let default = daemon.git.head_branch(&project.canonical_dir()).await?;
        if request.target.is_none() && branch != default {
            drop(canonical);
            drop(target);
            branch = default;
            target_lock = project.target_lock(&crate::git::target_ref(&branch));
            continue;
        }
        break (target, canonical);
    };
    let loaded = load_target(daemon, project, &branch).await?;
    let plan = loaded
        .config
        .resolve_plan(&request)
        .map_err(|invalid| Fail::invalid("invalid plan").with(json!(invalid)))?;
    let workflow = loaded
        .config
        .runnable_workflow(&plan, request.workflow.as_deref());
    let shape = if workflow.is_some() {
        "ordinary-linear"
    } else {
        plan.unsupported_shape()
    };
    if method == "plan.validate" && workflow.is_none() {
        return Err(Fail::refused("plan shape has no runtime support")
            .with(json!({"shape": shape, "structurally_valid": true, "runnable": false})));
    }
    Ok(
        json!({"target": branch, "config_head": loaded.head, "plan": plan,
        "structurally_valid": true, "runnable": workflow.is_some(), "shape": shape,
        "workflow": workflow,
        "bindings": {"agents": loaded.config.agents, "gates": loaded.config.gates,
            "review": loaded.config.review,
            "workflow_settings": workflow.as_ref().and_then(|name| loaded.config.workflows.get(name))}}),
    )
}
