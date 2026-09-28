//! Admission, the execution state machine, the merge queue and
//! reconcile-on-start. `step` is the scheduler tick: it reads rows, decides
//! the next executions, records each before anything external runs, and
//! spawns it.

pub mod admit;
pub mod cleanup;
pub mod queue;
pub mod reconcile;
pub mod review;
pub mod supervise;

use crate::api::Fail;
use crate::config::{Approve, Config, Stage};
use crate::daemon::{Daemon, Project};
use crate::store::{self, Target, attempts, checks, executions, tickets};
use serde_json::{Value, json};
use std::sync::Arc;

/// The one configuration: canonical's target head's.
pub struct Loaded {
    pub config: Config,
    pub branch: String,
    pub head: String,
    pub gate_digest: String,
}

impl Loaded {
    pub fn review_digest(&self, workflow: &str) -> Result<String, Fail> {
        Ok(self.config.review_digest(self.config.workflow(workflow)?))
    }
}

pub async fn load(daemon: &Daemon, project: &Project) -> Result<Arc<Loaded>, Fail> {
    let canonical = project.canonical_dir();
    let branch = daemon.git.head_branch(&canonical).await?;
    let head = daemon
        .git
        .rev_parse(&canonical, &crate::git::target_ref(&branch))
        .await?
        .ok_or_else(|| {
            Fail::refused(format!(
                "canonical has no {branch} yet; commit .yard in the checkout and run `yard sync`"
            ))
        })?;
    if let Some(loaded) = project.loaded.lock().expect("loaded lock").as_ref()
        && loaded.head == head
    {
        return Ok(loaded.clone());
    }
    let loaded = Arc::new(read_config(daemon, &canonical, &branch, &head).await?);
    *project.loaded.lock().expect("loaded lock") = Some(loaded.clone());
    Ok(loaded)
}

/// The configuration at `commit`, validated.
pub async fn read_config(
    daemon: &Daemon,
    repo: &std::path::Path,
    branch: &str,
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
    if config.target.branch != branch {
        return Err(Fail::invalid(format!(
            ".yard/config.toml names target {:?}; this project's target is {branch:?}",
            config.target.branch
        )));
    }
    let dockerfile = daemon
        .git
        .show(repo, commit, crate::config::DOCKERFILE_PATH)
        .await?
        .ok_or_else(|| Fail::invalid(format!("{commit} has no .yard/Dockerfile")))?;
    Ok(Loaded {
        gate_digest: config.gate_digest(dockerfile.as_bytes()),
        config,
        branch: branch.to_string(),
        head: commit.to_string(),
    })
}

pub async fn step(daemon: &Arc<Daemon>, project: &Arc<Project>) -> Result<(), Fail> {
    let Ok(loaded) = load(daemon, project).await else {
        return Ok(());
    };
    admit::scheduled(daemon, project, &loaded)?;
    let live = project.read(attempts::live)?;
    for attempt in live {
        advance(daemon, project, &loaded, &attempt).await?;
    }
    queue::next(daemon, project, &loaded).await?;
    cleanup::next(daemon, project)?;
    Ok(())
}

/// Spawn the work of an execution already recorded.
pub fn spawn(daemon: &Arc<Daemon>, project: &Arc<Project>, kind: &str, execution: i64) {
    let daemon = daemon.clone();
    let project = project.clone();
    let kind = kind.to_string();
    tokio::spawn(async move {
        let result = match kind.as_str() {
            "implementation" => supervise::implement(&daemon, &project, execution).await,
            "gate" => supervise::gate(&daemon, &project, execution).await,
            "review" => review::run(&daemon, &project, execution).await,
            "landing" => queue::land(&daemon, &project, execution).await,
            "cleanup" => cleanup::run(&daemon, &project, execution).await,
            _ => unreachable!("five kinds"),
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
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: "error",
                            detail: Some(&fail.message),
                            ..Default::default()
                        },
                    )?;
                    if row.kind != "cleanup" {
                        let attempt = attempts::get(tx, row.attempt)?;
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
        daemon.stops.lock().expect("stops lock").remove(&execution);
        daemon.wake.notify_one();
    });
}

/// Decide the next execution for one live attempt, if any.
async fn advance(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
) -> Result<(), Fail> {
    let executions = project.read(|conn| executions::for_attempt(conn, attempt.id))?;
    if executions.iter().any(|row| row.status == "running") {
        return Ok(());
    }
    let open = project.read(|conn| attempts::open_for_attempt(conn, attempt.id))?;
    if open
        .iter()
        .any(|item| item.kind != "proposal" || item.reason == "edit")
    {
        return Ok(());
    }
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    if project
        .read(|conn| checks::active_for(conn, attempt.id))?
        .is_some()
    {
        return Ok(());
    }
    if !attempt.lane && !admit::take_lane(daemon, project, loaded, attempt.id)? {
        return Ok(());
    }
    if let Some(next) = &attempt.next {
        let reason = next["reason"].as_str().unwrap_or("repair").to_string();
        let execution =
            project.tx(|tx| supervise::start_implementation(tx, attempt, &ticket, &reason))?;
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

    let gate_input = checks::Input {
        attempt: attempt.id,
        base: base.to_string(),
        head: head.to_string(),
        ticket_revision: ticket.revision,
        digest: loaded.gate_digest.clone(),
    };
    let mut passed = Vec::new();
    for gate in loaded
        .config
        .gates
        .iter()
        .filter(|gate| gate.stage == Stage::Candidate)
    {
        match project.read(|conn| checks::current(conn, "gate", &gate.name, &gate_input))? {
            None => {
                let execution = project.tx(|tx| {
                    executions::start(
                        tx,
                        executions::Start {
                            attempt: attempt.id,
                            kind: "gate",
                            reason: Some("candidate"),
                            base: Some(base),
                            head: Some(head),
                            ticket_revision: Some(ticket.revision),
                            digest: Some(&loaded.gate_digest),
                            name: Some(&gate.name),
                            ticket: Some(ticket.id),
                            ..Default::default()
                        },
                    )
                })?;
                spawn(daemon, project, "gate", execution);
                return Ok(());
            }
            Some(check) if check.verdict == "pass" => passed.push(check.id),
            Some(check) => {
                if check.execution > last_implementation {
                    let detail = project
                        .read(|conn| executions::get(conn, check.execution))?
                        .detail
                        .unwrap_or_default();
                    let next = json!({ "reason": "repair", "gate": gate.name, "detail": detail });
                    project.tx(|tx| attempts::set_next(tx, attempt.id, Some(&next)))?;
                }
                return Ok(());
            }
        }
    }

    let review_digest = loaded.config.review_digest(workflow);
    let review_input = checks::Input {
        digest: review_digest.clone(),
        ..gate_input.clone()
    };
    let mut blocked = Vec::new();
    for seat in &workflow.review {
        match project.read(|conn| checks::current(conn, "review", seat, &review_input))? {
            None => {
                let agent = &loaded.config.seats[seat].agent;
                let settings =
                    serde_json::to_value(&loaded.config.agents[agent]).expect("agent serializes");
                let execution = project.tx(|tx| {
                    executions::start(
                        tx,
                        executions::Start {
                            attempt: attempt.id,
                            kind: "review",
                            reason: Some("first"),
                            base: Some(base),
                            head: Some(head),
                            ticket_revision: Some(ticket.revision),
                            digest: Some(&review_digest),
                            name: Some(seat),
                            round: Some(attempt.rounds + 1),
                            ticket: Some(ticket.id),
                            agent: Some((agent, &settings)),
                            ..Default::default()
                        },
                    )
                })?;
                spawn(daemon, project, "review", execution);
                return Ok(());
            }
            Some(check) if check.verdict == "pass" => passed.push(check.id),
            Some(check) => blocked.push(check),
        }
    }
    if !blocked.is_empty() {
        if blocked
            .iter()
            .all(|check| check.execution > last_implementation)
        {
            review::blocked(project, loaded, attempt, &ticket, &blocked)?;
        }
        return Ok(());
    }

    let protected = protected_paths(daemon, project, loaded, base, head).await?;
    if loaded.config.approve == Approve::Auto && protected.is_empty() {
        project.tx(|tx| {
            checks::approve(
                tx,
                checks::Approve {
                    attempt: attempt.id,
                    ticket: ticket.id,
                    base,
                    head,
                    ticket_revision: ticket.revision,
                    gate_digest: &loaded.gate_digest,
                    review_digest: &review_digest,
                    checks: &passed,
                    actor: "auto",
                    text: None,
                },
            )?;
            attempts::set_lane(tx, attempt.id, false)
        })?;
    } else if !open.iter().any(|item| item.kind == "approval") {
        project.tx(|tx| {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "approval",
                    reason: if protected.is_empty() {
                        "verified"
                    } else {
                        "protected"
                    },
                    ticket: Some(ticket.id),
                    attempt: Some(attempt.id),
                    execution: None,
                    payload: json!({ "base": base, "head": head, "revision": ticket.revision,
                                     "checks": passed, "protected": protected,
                                     "unreviewed": workflow.review.is_empty() }),
                    text: None,
                },
            )
        })?;
    }
    Ok(())
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
        "status" => admit::status(&project),
        "sync" => admit::sync(daemon, &project).await,
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
        "attempt.nudge" => admit::attempt_nudge(daemon, &project, &params),
        "attempt.approve" => admit::attempt_approve(daemon, &project, &params).await,
        "attempt.reject" => admit::attempt_reject(&project, &params),
        "attempt.abandon" => admit::attempt_abandon(daemon, &project, &params),
        "attempt.show" => admit::attempt_show(&project, &params),
        "attempt.diff" => admit::attempt_diff(daemon, &project, &params).await,
        "attempt.tail" => admit::attempt_tail(&project, &params),
        "proposal.accept" => admit::proposal_answer(daemon, &project, &params, true).await,
        "proposal.reject" => admit::proposal_answer(daemon, &project, &params, false).await,
        _ => Err(Fail::invalid(format!("unknown method {method:?}"))),
    }
}

/// `yard_propose`: a proposal is an attention item. A proposal to edit the
/// worker's own ticket pauses its attempt until decided.
pub fn propose(
    _daemon: &Arc<Daemon>,
    grant: &crate::mcp::Grant,
    arguments: &Value,
) -> Result<Value, Fail> {
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
    let reason = match kind {
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
            "ticket"
        }
        "edit" => {
            if text("title").is_none() && text("body").is_none() {
                return Err(Fail::invalid("an edit proposal needs a title or a body"));
            }
            "edit"
        }
        "link" => {
            if text("ticket").is_none() || object.get("depends_on").is_none() {
                return Err(Fail::invalid("a link proposal needs ticket and depends_on"));
            }
            "link"
        }
        other => {
            return Err(Fail::invalid(format!(
                "kind {other:?} is ticket, edit or link"
            )));
        }
    };
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
        attempts::raise(
            tx,
            attempts::Raise {
                kind: "proposal",
                reason,
                ticket: Some(attempt.ticket),
                attempt: Some(attempt.id),
                execution: Some(grant.execution),
                payload: Value::Object(object.clone()),
                text: text("reason"),
            },
        )
    })?;
    Ok(json!({ "proposal": id }))
}

/// The attempt's audit target.
pub fn target(attempt: &attempts::Attempt) -> Target {
    Target {
        ticket: Some(attempt.ticket),
        attempt: Some(attempt.id),
        ..Target::default()
    }
}

pub fn audit_attempt(
    tx: &rusqlite::Connection,
    event: &str,
    attempt: &attempts::Attempt,
    text: Option<&str>,
    data: Value,
) -> Result<i64, Fail> {
    store::audit(tx, event, target(attempt), text, data)
}
