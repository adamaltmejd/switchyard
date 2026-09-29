//! Admission, capacity and the operator's commands. Every mutation carries
//! the smallest expected identity and a mismatch is a stale result.

use super::{Loaded, load, proof, spawn};
use crate::api::Fail;
use crate::config::Stage;
use crate::daemon::{Daemon, Project};
use crate::store::{self, attempts, checks, executions, ticket_id, ticket_name, tickets};
use serde_json::{Value, json};
use std::sync::Arc;

fn ticket_param(params: &Value) -> Result<i64, Fail> {
    let name = params["ticket"]
        .as_str()
        .ok_or_else(|| Fail::invalid("the call names no ticket"))?;
    ticket_id(name).ok_or_else(|| Fail::invalid(format!("{name:?} is not a ticket name")))
}

fn priority_param(value: &Value) -> Result<Option<i64>, Fail> {
    match value.as_str() {
        None => Ok(None),
        Some(token) => crate::config::priority(token)
            .map(|priority| Some(i64::from(priority)))
            .ok_or_else(|| Fail::invalid(format!("priority {token:?} is not P0 to P3"))),
    }
}

fn text_param(params: &Value, key: &str) -> Option<String> {
    params[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// Free lanes for one more attempt in `project`, counting the machine too.
fn free_lanes(daemon: &Daemon, project: &Project, loaded: &Loaded) -> Result<i64, Fail> {
    let own = project.read(attempts::lanes_held)?;
    let mut free = i64::from(loaded.config.max_lanes) - own;
    if let Some(machine) = daemon.machine.max_lanes {
        let projects: Vec<_> = daemon
            .projects
            .lock()
            .expect("projects lock")
            .values()
            .cloned()
            .collect();
        let mut held = 0;
        for other in projects {
            held += other.read(attempts::lanes_held)?;
        }
        free = free.min(i64::from(machine) - held);
    }
    Ok(free)
}

/// Take a lane for an attempt returning from the queue or from waiting.
pub fn take_lane(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    attempt: i64,
) -> Result<bool, Fail> {
    let _admission = daemon.admission.lock().expect("admission lock");
    if free_lanes(daemon, project, loaded)? <= 0 {
        return Ok(false);
    }
    project.tx(|tx| attempts::set_lane(tx, attempt, true))?;
    Ok(true)
}

/// Whether the scheduler would admit a ready ticket given a free lane: its
/// workflow exists, and a read-only workflow runs once, then only by
/// `yard attempt start`.
fn admissible(
    conn: &rusqlite::Connection,
    loaded: &Loaded,
    ticket: &tickets::Ticket,
) -> Result<bool, Fail> {
    let Ok(workflow) = loaded.config.workflow(&ticket.workflow) else {
        return Ok(false);
    };
    Ok(!(workflow.read_only && attempts::latest_for(conn, ticket.id)?.is_some()))
}

/// The scheduler's admissions: ready tickets in priority order.
pub fn scheduled(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
) -> Result<(), Fail> {
    let admitted = {
        let _admission = daemon.admission.lock().expect("admission lock");
        let mut admitted = Vec::new();
        let mut free = free_lanes(daemon, project, loaded)?;
        for ticket in project.read(tickets::ready)? {
            if free <= 0 {
                break;
            }
            if !project.read(|conn| admissible(conn, loaded, &ticket))? {
                continue;
            }
            admitted.push(project.tx(|tx| admit(tx, loaded, &ticket))?);
            free -= 1;
        }
        admitted
    };
    for execution in admitted {
        spawn(daemon, project, "implementation", execution);
    }
    Ok(())
}

/// Create the attempt, freeze its workflow and implementer, and record its
/// first execution, in one transaction.
fn admit(
    tx: &rusqlite::Connection,
    loaded: &Loaded,
    ticket: &tickets::Ticket,
) -> Result<i64, Fail> {
    let workflow = loaded.config.workflow(&ticket.workflow)?;
    let agent = &loaded.config.agents[&workflow.implementer];
    let mut implementer = serde_json::to_value(agent).expect("agent serializes");
    implementer["name"] = json!(workflow.implementer);
    let attempt = attempts::insert(tx, ticket.id, &ticket.workflow, &implementer, &loaded.head)?;
    let row = attempts::get(tx, attempt)?;
    super::audit_attempt(
        tx,
        "attempt.admitted",
        &row,
        None,
        json!({ "base": loaded.head, "workflow": ticket.workflow, "implementer": implementer }),
    )?;
    super::supervise::start_implementation(tx, &row, "first")
}

pub async fn status(daemon: &Daemon, project: &Project) -> Result<Value, Fail> {
    let loaded = load(daemon, project).await.ok();
    let no_lane = match &loaded {
        Some(loaded) => free_lanes(daemon, project, loaded)? <= 0,
        None => false,
    };
    let now = store::now_ms();
    project.read(|conn| {
        let tickets: Vec<Value> = tickets::list(conn)?
            .iter()
            .filter(|ticket| ticket.state == "open")
            .map(|ticket| {
                let mut value = ticket.to_json();
                let edges = tickets::dependencies(conn, ticket.id)?;
                let mut unfinished = Vec::new();
                for &on in &edges {
                    if tickets::get(conn, on)?.state != "done" {
                        unfinished.push(ticket_name(on));
                    }
                }
                value["depends_on"] =
                    json!(edges.iter().map(|on| ticket_name(*on)).collect::<Vec<_>>());
                value["waiting_on"] = json!(unfinished);
                let no_lane = match &loaded {
                    Some(loaded) if no_lane && tickets::blocker(conn, ticket.id)?.is_none() => {
                        admissible(conn, loaded, ticket)?
                    }
                    _ => false,
                };
                value["no_lane"] = json!(no_lane);
                Ok(value)
            })
            .collect::<Result<_, Fail>>()?;
        let attempts: Vec<Value> = attempts::live(conn)?
            .iter()
            .map(|attempt| {
                let mut value = attempt.to_json();
                let spent =
                    attempt.work_ms + attempt.lane_since.map(|since| now - since).unwrap_or(0);
                value["work_ms"] = json!(spent.max(0));
                value
            })
            .collect();
        let attention: Vec<Value> = attempts::open_attention(conn)?
            .iter()
            .map(attempts::Attention::to_json)
            .collect();
        let queue: Vec<Value> = checks::queue(conn)?
            .iter()
            .map(|approval| {
                let ticket = attempts::get(conn, approval.attempt)?.ticket;
                let edges: Vec<String> = tickets::dependencies(conn, ticket)?
                    .into_iter()
                    .map(ticket_name)
                    .collect();
                Ok(
                    json!({ "approval": approval.id, "attempt": approval.attempt,
                           "ticket": ticket_name(ticket),
                           "depends_on": edges,
                           "head": approval.head, "actor": approval.actor }),
                )
            })
            .collect::<Result<_, Fail>>()?;
        let activity = daemon.activity.lock().expect("activity lock");
        let running: Vec<Value> = executions::running(conn)?
            .iter()
            .map(|execution| {
                let mut value = execution.to_json();
                value["quiet_ms"] = json!(
                    activity
                        .get(&(project.key.clone(), execution.id))
                        .map(|last| last.elapsed().as_millis() as u64)
                );
                value
            })
            .collect();
        Ok(json!({
            "seq": store::last_seq(conn)?,
            "tickets": tickets,
            "attempts": attempts,
            "attention": attention,
            "queue": queue,
            "running": running,
        }))
    })
}

pub async fn doctor(daemon: &Daemon, project: &Project) -> Result<Value, Fail> {
    let pinfold = daemon
        .pinfold
        .version()
        .await
        .unwrap_or_else(|error| format!("not found: {error}"));
    let loaded = load(daemon, project).await;
    let connections: Vec<Value> = crate::harness::CONNECTIONS
        .iter()
        .map(|connection| {
            json!({
                "connection": connection.name,
                "origin": daemon.machine.origin(connection),
                "credential": daemon.machine.vars.contains_key(connection.key_var),
            })
        })
        .collect();
    // Each configured login by name, its pin and whether its token is set.
    // Yard never reads the token, and pinfold exposes no lapse.
    let mut logins = Vec::new();
    if let Ok(loaded) = &loaded {
        let mut seen = std::collections::BTreeSet::new();
        for agent in loaded
            .config
            .agents
            .values()
            .filter(|agent| agent.login == Some(true))
        {
            let Some(harness) = crate::harness::get(&agent.harness) else {
                continue;
            };
            let Some(login) = harness.login() else {
                continue;
            };
            if !seen.insert(login.name) {
                continue;
            }
            logins.push(json!({
                "login": login.name,
                "version": harness.version(),
                "credential": login.key_var.map(|key| daemon.machine.vars.contains_key(key)),
            }));
        }
    }
    let missing: Vec<String> = crate::daemon::missing_projects(daemon)?
        .into_iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    Ok(json!({
        "pinfold": pinfold,
        "service": crate::daemon::service_status().await,
        "configuration": match &loaded {
            Ok(loaded) => json!({ "target": loaded.branch, "head": loaded.head }),
            Err(fail) => json!({ "error": fail.message }),
        },
        "image": daemon.images.lock().expect("images lock").get(&project.key).map(|(head, id)| json!({ "head": head, "id": id })),
        "connections": connections,
        "logins": logins,
        "missing_projects": missing,
    }))
}

/// Import the checkout's head when it descends from canonical's; `None`
/// when it does not.
async fn import(
    daemon: &Daemon,
    project: &Project,
    branch: &str,
    ours: Option<&str>,
    theirs: &str,
) -> Result<Option<Value>, Fail> {
    let canonical = project.canonical_dir();
    let imported = match ours {
        None => true,
        Some(ours) => {
            daemon
                .git
                .is_ancestor(&canonical, ours, theirs, None)
                .await?
        }
    };
    if !imported {
        return Ok(None);
    }
    let loaded = super::read_config(daemon, &canonical, branch, theirs).await?;
    let orphaned = project.read(|conn| {
        Ok(tickets::list(conn)?
            .into_iter()
            .filter(|ticket| {
                ticket.state == "open" && !loaded.config.workflows.contains_key(&ticket.workflow)
            })
            .map(|ticket| ticket_name(ticket.id))
            .collect::<Vec<_>>())
    })?;
    if !orphaned.is_empty() {
        return Err(Fail::refused(format!(
            "the incoming configuration removes a workflow open tickets name: {}",
            orphaned.join(", ")
        ))
        .with(json!({ "tickets": orphaned })));
    }
    let target = crate::git::target_ref(branch);
    let moved = daemon
        .git
        .update_ref(&canonical, &target, theirs, ours, None)
        .await?;
    if !moved {
        return Err(Fail::refused(
            "canonical moved during the sync; run it again",
        ));
    }
    project.tx(|tx| {
        store::audit(
            tx,
            "sync.imported",
            store::Target::default(),
            None,
            json!({ "old": ours, "new": theirs }),
        )
    })?;
    Ok(Some(
        json!({ "sync": "imported", "old": ours, "head": theirs }),
    ))
}

pub async fn sync(daemon: &Daemon, project: &Project) -> Result<Value, Fail> {
    let _canonical = project.canonical.lock().await;
    let canonical = project.canonical_dir();
    let (branch, ours) = daemon.git.target_head(&canonical).await?;
    let target = crate::git::target_ref(&branch);
    let theirs = daemon
        .git
        .rev_parse(&project.root, &target)
        .await?
        .ok_or_else(|| Fail::refused(format!("the checkout has no branch {branch}")))?;
    if ours.as_deref() == Some(theirs.as_str()) {
        return Ok(json!({ "sync": "current", "head": theirs }));
    }
    // The objects only: no ref names them until the import moves the target.
    daemon.git.fetch(&canonical, &project.root, &target).await?;
    if let Some(result) = import(daemon, project, &branch, ours.as_deref(), &theirs).await? {
        return Ok(result);
    }
    let ours = ours.expect("a consumed canonical has a head");
    if !daemon
        .git
        .is_ancestor(&canonical, &theirs, &ours, None)
        .await?
    {
        return Err(Fail::refused(format!(
            "the checkout's {branch} at {theirs} and canonical at {ours} have diverged"
        ))
        .with(json!({ "checkout": theirs, "canonical": ours })));
    }
    daemon.git.fetch(&project.root, &canonical, &target).await?;
    daemon
        .git
        .fast_forward_checkout(&project.root, &branch, &theirs, &ours)
        .await?;
    project.tx(|tx| {
        store::audit(
            tx,
            "sync.consumed",
            store::Target::default(),
            None,
            json!({ "old": theirs, "new": ours }),
        )
    })?;
    Ok(json!({ "sync": "consumed", "old": theirs, "head": ours }))
}

pub async fn ticket_new(daemon: &Daemon, project: &Project, params: &Value) -> Result<Value, Fail> {
    let loaded = load(daemon, project).await?;
    let workflow = params["workflow"].as_str().unwrap_or("default");
    loaded.config.workflow(workflow).map_err(Fail::invalid)?;
    let priority = priority_param(&params["priority"])?.unwrap_or(2);
    let mut depends_on = Vec::new();
    for name in params["depends_on"].as_array().into_iter().flatten() {
        let name = name.as_str().unwrap_or_default();
        depends_on.push(
            ticket_id(name).ok_or_else(|| Fail::invalid(format!("{name:?} is not a ticket")))?,
        );
    }
    let id = project.tx(|tx| {
        tickets::create(
            tx,
            &tickets::NewTicket {
                title: params["title"].as_str().unwrap_or_default(),
                body: params["body"].as_str().unwrap_or_default(),
                priority,
                workflow,
                depends_on: &depends_on,
                parked: params["parked"].as_bool().unwrap_or(false),
                origin: "operator",
            },
            None,
        )
    })?;
    project.read(|conn| Ok(tickets::get(conn, id)?.to_json()))
}

pub fn ticket_show(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    project.read(|conn| {
        let ticket = tickets::get(conn, id)?;
        let mut value = ticket.to_json();
        value["depends_on"] = json!(
            tickets::dependencies(conn, id)?
                .into_iter()
                .map(ticket_name)
                .collect::<Vec<_>>()
        );
        value["attempt"] = attempts::latest_for(conn, id)?
            .map(|attempt| attempt.to_json())
            .unwrap_or(Value::Null);
        Ok(value)
    })
}

pub fn ticket_list(project: &Project) -> Result<Value, Fail> {
    project.read(|conn| {
        Ok(json!(
            tickets::list(conn)?
                .iter()
                .map(tickets::Ticket::to_json)
                .collect::<Vec<_>>()
        ))
    })
}

/// Commands refused while a landing intent names the ticket.
fn refuse_during_intent(conn: &rusqlite::Connection, ticket: i64) -> Result<(), Fail> {
    for intent in executions::open_intents(conn)? {
        let attempt = attempts::get(conn, intent.attempt)?;
        if attempt.ticket == ticket {
            return Err(Fail::refused(format!(
                "{} has a landing intent (execution {}) that is not yet resolved",
                ticket_name(ticket),
                intent.id
            ))
            .with(json!({ "intent": intent.id })));
        }
    }
    Ok(())
}

pub async fn ticket_edit(
    daemon: &Daemon,
    project: &Project,
    params: &Value,
) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let revision = params["revision"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("an edit names the revision it was written against"))?;
    let workflow = text_param(params, "workflow");
    if let Some(workflow) = &workflow {
        load(daemon, project)
            .await?
            .config
            .workflow(workflow)
            .map_err(Fail::invalid)?;
    }
    let priority = priority_param(&params["priority"])?;
    let stops = project.tx(|tx| {
        let stops = edit(
            tx,
            id,
            revision,
            params["title"].as_str(),
            params["body"].as_str(),
            priority,
            workflow.as_deref(),
        )?;
        if let Some(attempt) = attempts::live_for(tx, id)? {
            for item in attempts::open_for_attempt(tx, attempt.id)? {
                if item.kind == "proposal" && item.reason == "edit" {
                    attempts::resolve(tx, &item, "superseded", None)?;
                }
            }
        }
        Ok(stops)
    })?;
    stop_executions(daemon, project, &stops);
    project.read(|conn| Ok(tickets::get(conn, id)?.to_json()))
}

/// One ticket edit: bumps the revision, which supersedes every check and
/// approval item that read the old one, and steers the ticket's live attempt:
/// the implementer runs next. Returns the gate and review executions to stop.
fn edit(
    tx: &rusqlite::Connection,
    id: i64,
    revision: i64,
    title: Option<&str>,
    body: Option<&str>,
    priority: Option<i64>,
    workflow: Option<&str>,
) -> Result<Vec<i64>, Fail> {
    let ticket = tickets::get(tx, id)?;
    if ticket.revision != revision {
        return Err(Fail::stale(
            format!("{} is at revision {}", ticket_name(id), ticket.revision),
            json!(revision),
            json!(ticket.revision),
        ));
    }
    refuse_during_intent(tx, id)?;
    if title.is_some_and(|title| title.trim().is_empty()) {
        return Err(Fail::invalid("a ticket needs a title"));
    }
    tx.execute(
        "UPDATE ticket SET title = COALESCE(?2, title), body = COALESCE(?3, body),
            priority = COALESCE(?4, priority), workflow = COALESCE(?5, workflow),
            revision = revision + 1
         WHERE id = ?1",
        rusqlite::params![id, title, body, priority, workflow],
    )?;
    store::audit(
        tx,
        "ticket.edited",
        store::Target {
            ticket: Some(id),
            ..Default::default()
        },
        None,
        json!({ "revision": revision + 1, "title": title, "body_bytes": body.map(str::len),
                "priority": priority, "workflow": workflow }),
    )?;
    steer(tx, id)
}

/// Make the live attempt run its implementer next. The pending edit is derived
/// from the executions, so a running implementer finishes first and nothing is stored.
fn steer(tx: &rusqlite::Connection, id: i64) -> Result<Vec<i64>, Fail> {
    let Some(attempt) = attempts::live_for(tx, id)? else {
        return Ok(Vec::new());
    };
    let running: Vec<_> = executions::for_attempt(tx, attempt.id)?
        .into_iter()
        .filter(|row| row.status == "running")
        .collect();
    if running.iter().any(|row| row.kind == "implementation") {
        return Ok(Vec::new());
    }
    if let Some(approval) = checks::active_for(tx, attempt.id)? {
        checks::set_approval_state(tx, &approval, "withdrawn", id, Some("edited"))?;
    }
    for item in attempts::open_for_attempt(tx, attempt.id)? {
        let resolution = match item.kind.as_str() {
            "approval" => "superseded",
            "stopped" | "red" => "edit",
            _ => continue,
        };
        attempts::resolve(tx, &item, resolution, None)?;
        if item.kind != "stopped" {
            continue;
        }
        match item.reason.as_str() {
            "timeout" => attempts::renew_clock(tx, attempt.id)?,
            "limit" => {
                tx.execute(
                    "UPDATE attempt SET extra_rounds = extra_rounds + 1 WHERE id = ?1",
                    [attempt.id],
                )?;
            }
            _ => {}
        }
    }
    // An edit replaces a pending repair or retry of the old candidate.
    attempts::set_next(tx, attempt.id, None)?;
    Ok(running
        .iter()
        .filter(|row| row.parent.is_none() && matches!(row.kind.as_str(), "gate" | "review"))
        .map(|row| row.id)
        .collect())
}

fn stop_executions(daemon: &Daemon, project: &Project, executions: &[i64]) {
    let stops = daemon.stops.lock().expect("stops lock");
    for execution in executions {
        if let Some(stop) = stops.get(&(project.key.clone(), *execution)) {
            stop.notify_one();
        }
    }
}

pub fn ticket_park(project: &Project, params: &Value, parked: bool) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    project.tx(|tx| {
        tickets::get(tx, id)?;
        tx.execute(
            "UPDATE ticket SET parked = ?2 WHERE id = ?1",
            rusqlite::params![id, parked],
        )?;
        store::audit(
            tx,
            if parked {
                "ticket.parked"
            } else {
                "ticket.unparked"
            },
            store::Target {
                ticket: Some(id),
                ..Default::default()
            },
            None,
            json!({}),
        )
    })?;
    project.read(|conn| Ok(tickets::get(conn, id)?.to_json()))
}

pub fn ticket_depend(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let on = params["on"]
        .as_str()
        .and_then(ticket_id)
        .ok_or_else(|| Fail::invalid("depend names no ticket to depend on"))?;
    project.tx(|tx| link(tx, id, on))?;
    ticket_show(project, params)
}

fn link(tx: &rusqlite::Connection, id: i64, on: i64) -> Result<(), Fail> {
    tickets::get(tx, id)?;
    tickets::get(tx, on)?;
    if tickets::would_cycle(tx, id, on)? {
        return Err(Fail::refused(format!(
            "{} depending on {} closes a cycle",
            ticket_name(id),
            ticket_name(on)
        )));
    }
    tx.execute(
        "INSERT OR IGNORE INTO dependency (ticket, depends_on) VALUES (?1, ?2)",
        [id, on],
    )?;
    store::audit(
        tx,
        "ticket.linked",
        store::Target {
            ticket: Some(id),
            ..Default::default()
        },
        None,
        json!({ "depends_on": ticket_name(on) }),
    )?;
    Ok(())
}

pub fn ticket_close(project: &Project, params: &Value, state: &str) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let reason = params["reason"].as_str().unwrap_or_default();
    if state == "done" && reason.trim().is_empty() {
        return Err(Fail::invalid("closing a ticket by hand needs a reason"));
    }
    project.tx(|tx| {
        let ticket = tickets::get(tx, id)?;
        if ticket.state != "open" {
            return Err(Fail::refused(format!(
                "{} is {}",
                ticket_name(id),
                ticket.state
            )));
        }
        refuse_during_intent(tx, id)?;
        if let Some(attempt) = attempts::live_for(tx, id)? {
            return Err(Fail::refused(format!(
                "{} has a live attempt; abandon it first",
                ticket_name(id)
            ))
            .with(json!({ "attempt": attempt.id })));
        }
        tx.execute(
            "UPDATE ticket SET state = ?2, closed_at = ?3, close_reason = ?4 WHERE id = ?1",
            rusqlite::params![id, state, store::now(), reason],
        )?;
        store::audit(
            tx,
            if state == "done" {
                "ticket.done"
            } else {
                "ticket.abandoned"
            },
            store::Target {
                ticket: Some(id),
                ..Default::default()
            },
            Some(reason),
            json!({ "by": "operator" }),
        )
    })?;
    project.read(|conn| Ok(tickets::get(conn, id)?.to_json()))
}

/// `yard attempt start`: the scheduler's admission for one named ticket, or
/// the `start` of its live attempt's stopped or red item.
pub async fn attempt_start(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    params: &Value,
) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let loaded = load(daemon, project).await?;
    let live = project.read(|conn| attempts::live_for(conn, id))?;
    let items = project.read(|conn| {
        let mut items = attempts::open_attention(conn)?;
        items.retain(|item| {
            item.ticket == Some(id) && (item.kind == "stopped" || item.kind == "red")
        });
        Ok(items)
    })?;
    if let Some(expected) = params["attention"].as_i64()
        && !items.iter().any(|item| item.id == expected)
    {
        return Err(Fail::stale(
            format!("attention {expected} is not open on {}", ticket_name(id)),
            json!(expected),
            json!(items.iter().map(|item| item.id).collect::<Vec<_>>()),
        ));
    }
    if let Some(item) = items.first() {
        return answer_start(daemon, project, item).await;
    }
    if live.is_some() {
        return Err(Fail::refused(format!(
            "{} has a live attempt with nothing to start",
            ticket_name(id)
        )));
    }
    let execution = {
        let _admission = daemon.admission.lock().expect("admission lock");
        let ready = project.read(tickets::ready)?;
        let Some(ticket) = ready.into_iter().find(|ticket| ticket.id == id) else {
            let blocker = project.read(|conn| tickets::blocker(conn, id))?;
            return Err(Fail::refused(format!(
                "{} is not ready: {}",
                ticket_name(id),
                blocker.unwrap_or_default()
            )));
        };
        if free_lanes(daemon, project, &loaded)? <= 0 {
            return Err(Fail::refused("no lane is free").with(json!({ "reason": "capacity" })));
        }
        project.tx(|tx| admit(tx, &loaded, &ticket))?
    };
    spawn(daemon, project, "implementation", execution);
    project.read(|conn| {
        Ok(attempts::live_for(conn, id)?
            .map(|attempt| attempt.to_json())
            .unwrap_or(Value::Null))
    })
}

/// `start` on an item: try again from here.
async fn answer_start(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    item: &attempts::Attention,
) -> Result<Value, Fail> {
    if !item.exits().contains(&"start") {
        return Err(Fail::refused(format!(
            "a stopped:{} item exits only by edit or abandon",
            item.reason
        )));
    }
    let execution = item
        .execution
        .map(|id| project.read(|conn| executions::get(conn, id)))
        .transpose()?;
    if let Some(landing) = execution.as_ref().filter(|row| row.kind == "landing") {
        if landing.intent_state.as_deref() == Some("open") {
            // Read canonical against the restart table again.
            let decided = super::reconcile::decide_intent(daemon, project, landing).await?;
            if !decided {
                return Err(Fail::refused(
                    "canonical still decides nothing about the landing intent",
                ));
            }
            project.tx(|tx| attempts::resolve(tx, item, "start", None))?;
            return Ok(json!({ "started": item.id, "intent": "decided" }));
        }
        // A landing that could not run re-queues under its approval.
        project.tx(|tx| attempts::resolve(tx, item, "start", None))?;
        return Ok(json!({ "started": item.id, "requeued": true }));
    }
    project.tx(|tx| {
        attempts::resolve(tx, item, "start", None)?;
        let Some(attempt) = item.attempt else {
            return Ok(());
        };
        let rerun = execution
            .as_ref()
            .is_some_and(|row| matches!(row.kind.as_str(), "gate" | "review"));
        if !rerun {
            let reason = match item.reason.as_str() {
                "unchanged" => "restart",
                "landing" => "repair",
                _ => "retry",
            };
            let detail = item.payload["detail"].as_str().unwrap_or_default();
            attempts::set_next(
                tx,
                attempt,
                Some(&json!({ "reason": reason, "detail": detail })),
            )?;
        }
        Ok(())
    })?;
    Ok(json!({ "started": item.id }))
}

pub fn attempt_stop(daemon: &Daemon, project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let running = project.read(|conn| {
        let attempt = attempts::live_for(conn, id)?
            .ok_or_else(|| Fail::refused(format!("{} has no live attempt", ticket_name(id))))?;
        Ok(executions::for_attempt(conn, attempt.id)?
            .into_iter()
            .find(|row| row.status == "running" && row.kind == "implementation"))
    })?;
    let Some(running) = running else {
        return Err(Fail::refused(format!(
            "{} has no running implementer execution",
            ticket_name(id)
        )));
    };
    if let Some(stop) = daemon
        .stops
        .lock()
        .expect("stops lock")
        .get(&(project.key.clone(), running.id))
    {
        stop.notify_one();
    }
    project.tx(|tx| {
        store::audit(
            tx,
            "attempt.stopped",
            store::Target {
                ticket: Some(id),
                attempt: Some(running.attempt),
                execution: Some(running.id),
                ..Default::default()
            },
            None,
            json!({}),
        )
    })?;
    Ok(json!({ "stopping": running.id }))
}

/// The open approval item on a ticket's live attempt, checked against the
/// head and, when it has one, the proof digest the operator named. With
/// `over_limit`, a `stopped:limit` item answers as well.
fn approval_item(
    tx: &rusqlite::Connection,
    id: i64,
    head: &str,
    proof_digest: Option<&str>,
    over_limit: bool,
) -> Result<(attempts::Attempt, attempts::Attention), Fail> {
    let attempt = attempts::live_for(tx, id)?
        .ok_or_else(|| Fail::refused(format!("{} has no live attempt", ticket_name(id))))?;
    let item = attempts::open_for_attempt(tx, attempt.id)?
        .into_iter()
        .find(|item| {
            item.kind == "approval"
                || (over_limit && item.kind == "stopped" && item.reason == "limit")
        });
    let current = attempt.head.clone().unwrap_or_default();
    if current != head || item.is_none() {
        return Err(Fail::stale(
            format!("{} has no approval item on {head}", ticket_name(id)),
            json!(head),
            json!(item.as_ref().map(|_| current)),
        ));
    }
    // A named proof that is not the candidate's is stale even when the
    // candidate has no snapshot, so a delayed answer for a proof that has
    // since been removed does not approve the empty candidate. A candidate
    // with a real snapshot cannot be answered by head alone.
    let current_proof = attempt.proof.clone().unwrap_or_default();
    match proof_digest {
        Some(named) if named != current_proof => {
            return Err(Fail::stale(
                format!("{} has no approval item on proof {named}", ticket_name(id)),
                json!(named),
                json!(current_proof),
            ));
        }
        None if proof::required(&current_proof) => {
            return Err(Fail::stale(
                format!("{} has no approval item without its proof", ticket_name(id)),
                json!(""),
                json!(current_proof),
            ));
        }
        _ => {}
    }
    Ok((attempt, item.expect("checked above")))
}

/// What an approval over a limit item names: every candidate gate's passing
/// check, then the blocking review checks it overrides. A gate without a
/// passing check at the head refuses it.
fn limit_checks(
    tx: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    loaded: &Loaded,
) -> Result<Vec<i64>, Fail> {
    let (base, head) = attempt.candidate().unwrap_or_default();
    let input = checks::Input {
        attempt: attempt.id,
        base: base.to_string(),
        head: head.to_string(),
        proof: attempt.proof.clone().unwrap_or_default(),
        ticket_revision: ticket.revision,
        digest: loaded.gate_digest.clone(),
    };
    let mut named = Vec::new();
    for gate in loaded
        .config
        .gates
        .iter()
        .filter(|gate| gate.stage == Stage::Candidate)
    {
        match checks::current(tx, "gate", &gate.name, &input)? {
            Some(check) if check.verdict == "pass" => named.push(check.id),
            _ => {
                return Err(Fail::refused(format!(
                    "gate {} has no passing check on {head}",
                    gate.name
                )));
            }
        }
    }
    let review = checks::Input {
        digest: loaded.review_digest(&attempt.workflow)?,
        ..input
    };
    for seat in &loaded.config.workflow(&attempt.workflow)?.review {
        if let Some(check) = checks::current(tx, "review", seat, &review)?
            && check.verdict != "pass"
        {
            named.push(check.id);
        }
    }
    Ok(named)
}

pub async fn attempt_approve(
    daemon: &Daemon,
    project: &Project,
    params: &Value,
) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let head = params["head"]
        .as_str()
        .ok_or_else(|| Fail::invalid("approve names the head it binds"))?;
    let proof_digest = params["proof"].as_str();
    let text = text_param(params, "text");
    let loaded = load(daemon, project).await?;
    let approval = project.tx(|tx| {
        let (attempt, item) = approval_item(tx, id, head, proof_digest, true)?;
        let ticket = tickets::get(tx, id)?;
        if attempts::edit_pending(tx, attempt.id)? {
            return Err(Fail::stale(
                format!(
                    "{} has an edit the implementer has not read",
                    ticket_name(id)
                ),
                json!(ticket.revision),
                json!("pending"),
            ));
        }
        let review_digest = loaded.review_digest(&attempt.workflow)?;
        if item.kind == "stopped"
            && attempt.rounds < i64::from(loaded.config.review.max_rounds) + attempt.extra_rounds
        {
            return Err(Fail::refused(format!(
                "{} is not at its review round limit",
                ticket_name(id)
            )));
        }
        // Before the staleness read, so a gate that changed names itself.
        let overrode = item.kind == "stopped";
        let checks: Vec<i64> = if overrode {
            limit_checks(tx, &attempt, &ticket, &loaded)?
        } else {
            item.payload["checks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_i64)
                .collect()
        };
        if let Some((key, recorded, current)) =
            super::stale_part(&item.payload, &attempt, &ticket, &loaded)?
        {
            return Err(Fail::stale(
                format!("the item's {key} is no longer current"),
                recorded,
                current,
            ));
        }
        let proof = attempt.proof.clone().unwrap_or_default();
        let approval = checks::approve(
            tx,
            checks::Approve {
                attempt: attempt.id,
                ticket: id,
                base: &attempt.base,
                head,
                proof: &proof,
                ticket_revision: ticket.revision,
                gate_digest: &loaded.gate_digest,
                review_digest: &review_digest,
                checks: &checks,
                actor: "operator",
                text: text.as_deref(),
                overrode,
            },
        )?;
        attempts::resolve(tx, &item, "approve", text.as_deref())?;
        attempts::set_lane(tx, attempt.id, false)?;
        Ok(approval)
    })?;
    Ok(json!({ "approval": approval, "head": head }))
}

pub fn attempt_reject(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let head = params["head"]
        .as_str()
        .ok_or_else(|| Fail::invalid("reject names the head it answers"))?;
    let proof_digest = params["proof"].as_str();
    let text = params["text"].as_str().unwrap_or_default();
    if text.trim().is_empty() {
        return Err(Fail::invalid("a reject carries notes"));
    }
    project.tx(|tx| {
        let (attempt, item) = approval_item(tx, id, head, proof_digest, false)?;
        refuse_during_intent(tx, id)?;
        attempts::resolve(tx, &item, "reject", Some(text))?;
        attempts::set_next(
            tx,
            attempt.id,
            Some(&json!({ "reason": "repair", "detail": text, "rejected": head })),
        )
    })?;
    Ok(json!({ "rejected": head }))
}

pub fn attempt_abandon(daemon: &Daemon, project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let reason = params["reason"].as_str().unwrap_or_default();
    let running = project.tx(|tx| {
        refuse_during_intent(tx, id)?;
        let attempt = attempts::live_for(tx, id)?
            .ok_or_else(|| Fail::refused(format!("{} has no live attempt", ticket_name(id))))?;
        for item in attempts::open_for_attempt(tx, attempt.id)? {
            attempts::resolve(tx, &item, "abandon", None)?;
        }
        if let Some(approval) = checks::active_for(tx, attempt.id)? {
            checks::set_approval_state(tx, &approval, "withdrawn", id, Some("abandoned"))?;
        }
        attempts::end(tx, attempt.id, "abandoned")?;
        super::audit_attempt(tx, "attempt.abandoned", &attempt, Some(reason), json!({}))?;
        Ok(executions::for_attempt(tx, attempt.id)?
            .into_iter()
            .filter(|row| row.status == "running")
            .map(|row| row.id)
            .collect::<Vec<_>>())
    })?;
    stop_executions(daemon, project, &running);
    Ok(json!({ "abandoned": ticket_name(id) }))
}

pub fn attempt_show(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    project.read(|conn| {
        let ticket = tickets::get(conn, id)?;
        let attempt = attempts::latest_for(conn, id)?
            .ok_or_else(|| Fail::not_found(format!("{} has no attempt", ticket_name(id))))?;
        let executions: Vec<Value> = executions::for_attempt(conn, attempt.id)?
            .iter()
            .map(executions::Execution::to_json)
            .collect();
        let mut checks_json = Vec::new();
        for check in checks::for_attempt(conn, attempt.id)? {
            let mut value = check.to_json();
            value["findings"] = json!(checks::findings(conn, check.id)?);
            checks_json.push(value);
        }
        let attention: Vec<Value> = attempts::open_for_attempt(conn, attempt.id)?
            .iter()
            .map(attempts::Attention::to_json)
            .collect();
        Ok(json!({
            "ticket": ticket.to_json(),
            "attempt": attempt.to_json(),
            "executions": executions,
            "checks": checks_json,
            "approval": checks::active_for(conn, attempt.id)?.map(|approval| json!({
                "approval": approval.id, "head": approval.head, "actor": approval.actor,
            })),
            "attention": attention,
        }))
    })
}

pub async fn attempt_diff(
    daemon: &Daemon,
    project: &Project,
    params: &Value,
) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let attempt = project
        .read(|conn| attempts::latest_for(conn, id))?
        .ok_or_else(|| Fail::not_found(format!("{} has no attempt", ticket_name(id))))?;
    let (base, head) = attempt
        .candidate()
        .ok_or_else(|| Fail::not_found(format!("{} has no candidate", ticket_name(id))))?;
    Ok(json!(
        daemon
            .git
            .diff(&project.canonical_dir(), base, head)
            .await?
    ))
}

pub fn attempt_tail(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    let attempt = project
        .read(|conn| attempts::live_for(conn, id))?
        .ok_or_else(|| Fail::not_found(format!("{} has no live attempt", ticket_name(id))))?;
    let execution = project
        .read(|conn| executions::for_attempt(conn, attempt.id))?
        .into_iter()
        .rfind(|row| row.kind == "implementation" || row.kind == "review")
        .ok_or_else(|| Fail::not_found("no worker has run"))?;
    let path = super::supervise::transcript(project, attempt.id, execution.id);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| Fail::not_found(format!("{}: {error}", path.display())))?;
    Ok(json!(text))
}

/// Accept or reject a proposal. Acceptance runs an ordinary command against
/// current state.
pub async fn proposal_answer(
    daemon: &Daemon,
    project: &Project,
    params: &Value,
    accept: bool,
) -> Result<Value, Fail> {
    let id = params["attention"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("the call names no proposal"))?;
    let text = text_param(params, "text");
    let loaded = load(daemon, project).await?;
    let stops = std::cell::RefCell::new(Vec::new());
    let minted = project.tx(|tx| {
        let item = attempts::attention(tx, id)?;
        if item.kind != "proposal" || item.state != "open" {
            return Err(Fail::stale(
                format!("proposal {id} is not open"),
                json!(id),
                json!(item.state),
            ));
        }
        if !accept {
            attempts::resolve(tx, &item, "reject", text.as_deref())?;
            return Ok(None);
        }
        let proposer = item.ticket.expect("a proposal names its ticket");
        let payload = &item.payload;
        let field = |key: &str| payload[key].as_str();
        let minted = match item.reason.as_str() {
            "ticket" => {
                let workflow = field("workflow").unwrap_or("default");
                loaded.config.workflow(workflow).map_err(Fail::invalid)?;
                let mut depends_on = Vec::new();
                for reference in payload["depends_on"].as_array().into_iter().flatten() {
                    depends_on.push(resolve_reference(
                        tx,
                        item.execution,
                        reference.as_str().unwrap_or_default(),
                    )?);
                }
                let ticket = tickets::create(
                    tx,
                    &tickets::NewTicket {
                        title: field("title").unwrap_or_default(),
                        body: field("body").unwrap_or_default(),
                        priority: priority_param(&payload["priority"])?.unwrap_or(2),
                        workflow,
                        depends_on: &depends_on,
                        parked: payload["parked"].as_bool().unwrap_or(false),
                        origin: "proposal",
                    },
                    None,
                )?;
                let planning = item
                    .attempt
                    .map(|attempt| attempts::get(tx, attempt))
                    .transpose()?
                    .is_some_and(|attempt| {
                        loaded
                            .config
                            .workflow(&attempt.workflow)
                            .is_ok_and(|workflow| workflow.read_only)
                    });
                if planning {
                    link(tx, proposer, ticket)?;
                }
                Some(ticket)
            }
            "edit" => {
                let revision = payload["revision"]
                    .as_i64()
                    .ok_or_else(|| Fail::invalid("an edit proposal names no revision"))?;
                let stopped = edit(
                    tx,
                    proposer,
                    revision,
                    field("title"),
                    field("body"),
                    None,
                    None,
                )?;
                stops.borrow_mut().extend(stopped);
                None
            }
            "link" => {
                let from =
                    resolve_reference(tx, item.execution, field("ticket").unwrap_or_default())?;
                for reference in payload["depends_on"].as_array().into_iter().flatten() {
                    let on = resolve_reference(
                        tx,
                        item.execution,
                        reference.as_str().unwrap_or_default(),
                    )?;
                    link(tx, from, on)?;
                }
                None
            }
            other => return Err(Fail::invalid(format!("proposal kind {other:?}"))),
        };
        let resolution = match minted {
            Some(ticket) => format!("accept:{}", ticket_name(ticket)),
            None => "accept".to_string(),
        };
        attempts::resolve(tx, &item, &resolution, text.as_deref())?;
        Ok(minted)
    })?;
    stop_executions(daemon, project, &stops.into_inner());
    Ok(json!({ "proposal": id, "accepted": accept, "ticket": minted.map(ticket_name) }))
}

/// A ticket name, or a key a sibling proposal of the same execution gave a
/// ticket that acceptance already minted.
fn resolve_reference(
    tx: &rusqlite::Connection,
    execution: Option<i64>,
    reference: &str,
) -> Result<i64, Fail> {
    if let Some(id) = reference
        .strip_prefix("Y-")
        .and_then(|_| ticket_id(reference))
    {
        tickets::get(tx, id)?;
        return Ok(id);
    }
    let minted: Option<String> = tx
        .query_row(
            "SELECT resolution FROM attention WHERE kind = 'proposal' AND execution IS ?1
               AND json_extract(payload, '$.key') = ?2 AND resolution LIKE 'accept:%'",
            rusqlite::params![execution, reference],
            |row| row.get(0),
        )
        .ok();
    minted
        .as_deref()
        .and_then(|resolution| resolution.strip_prefix("accept:"))
        .and_then(ticket_id)
        .ok_or_else(|| {
            Fail::refused(format!(
                "{reference:?} names no ticket and no accepted sibling proposal"
            ))
        })
}
