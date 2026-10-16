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

fn target_param(params: &Value) -> Result<Option<&str>, Fail> {
    match &params["target"] {
        Value::Null => Ok(None),
        Value::String(target) => Ok(Some(target)),
        _ => Err(Fail::invalid("target must be a local branch name")),
    }
}

/// Free lanes for one more attempt in a project.
fn free_lanes(conn: &rusqlite::Connection, loaded: &Loaded) -> Result<i64, Fail> {
    Ok(i64::from(loaded.config.max_lanes) - attempts::lanes_held(conn)?)
}

/// Lanes a ready ticket may take: free lanes less those owed to live
/// attempts of this project waiting to reacquire one, on one connection so
/// both are one reading.
fn spare_lanes(
    conn: &rusqlite::Connection,
    project: &Project,
    loaded: &Loaded,
) -> Result<i64, Fail> {
    let mut waiting = 0;
    for attempt in attempts::live(conn)? {
        let configs = project.loaded.lock().expect("loaded lock");
        let selected = match &attempt.admission {
            Some(input) => {
                let (definition, staging, image) = super::source_commits(&input["inputs"])?;
                configs.sources.get(&(
                    definition.to_string(),
                    staging.to_string(),
                    image.to_string(),
                ))
            }
            None => configs.targets.get(attempt.target_branch()?),
        }
        .map(Arc::as_ref)
        .ok_or_else(|| Fail::refused("the attempt configuration is not loaded"))?;
        if !attempt.lane
            && super::runnable(conn, &super::resolved(selected, &attempt)?, &attempt)?.is_some()
        {
            waiting += 1;
        }
    }
    Ok(free_lanes(conn, loaded)? - waiting)
}

/// Take a lane for an attempt returning from the queue or from waiting.
pub fn take_lane(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    attempt: i64,
) -> Result<bool, Fail> {
    let _admission = daemon.admission.lock().expect("admission lock");
    project.tx(|tx| take_lane_in(tx, loaded, attempt))
}

pub(super) fn take_lane_in(
    tx: &rusqlite::Connection,
    loaded: &Loaded,
    attempt: i64,
) -> Result<bool, Fail> {
    if attempts::get(tx, attempt)?.lane {
        return Ok(true);
    }
    if free_lanes(tx, loaded)? <= 0 {
        return Ok(false);
    }
    attempts::set_lane(tx, attempt, true)?;
    Ok(true)
}

fn refuse_pending_continuation(conn: &rusqlite::Connection, attempt: i64) -> Result<(), Fail> {
    let attempt = attempts::get(conn, attempt)?;
    if attempt.admission.as_ref().is_some_and(|admission| {
        admission["runtime"] == "writer-graph"
            || crate::plan::Plan::admitted(admission).is_ok_and(|plan| plan.handoff())
    }) && let Some(next) = attempt.next
    {
        return Err(Fail::refused("pending_continuation").with(json!({
            "reason": "pending_continuation", "identity": "pending_continuation",
            "attempt": attempt.id, "choice": attempt.choice, "current": next
        })));
    }
    Ok(())
}

/// Whether the scheduler would admit a ready ticket given a free lane: its
/// workflow exists, and a read-only workflow runs once, then only by
/// Operator `attempt.start`.
pub fn admissible(
    conn: &rusqlite::Connection,
    loaded: &Loaded,
    ticket: &tickets::Ticket,
) -> Result<bool, Fail> {
    let Ok(workflow) = loaded.config.workflow(&ticket.workflow) else {
        return Ok(false);
    };
    Ok(!(workflow.read_only && attempts::latest_for(conn, ticket.id)?.is_some()))
}

pub async fn ready_configs(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
) -> Result<Vec<(tickets::Ticket, Arc<Loaded>)>, Fail> {
    let mut selected = Vec::new();
    for ticket in project.read(tickets::ready)? {
        let branch = ticket.target_branch.as_deref().unwrap_or(&loaded.branch);
        if let Ok(config) = super::load_target(daemon, project, branch).await {
            selected.push((ticket, config));
        }
    }
    Ok(selected)
}

/// The scheduler's admissions: ready tickets in priority order.
pub async fn scheduled(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
) -> Result<(), Fail> {
    let mut selected = Vec::new();
    let free = project.read(|conn| spare_lanes(conn, project, loaded))?;
    for (ticket, config) in ready_configs(daemon, project, loaded).await? {
        if selected.len() as i64 >= free {
            break;
        }
        if !project.read(|conn| admissible(conn, &config, &ticket))? {
            continue;
        }
        let plan = config.config.workflow_plan(&ticket.workflow)?;
        let inputs = match config
            .inputs(&daemon.git, &project.canonical_dir(), &plan)
            .await
        {
            Ok(inputs) => inputs,
            // Like an unavailable ticket target, refused selected inputs
            // leave this ticket unadmitted. Explicit start returns the refusal.
            Err(fail) if fail.code == "refused" => continue,
            Err(fail) => return Err(fail),
        };
        selected.push((ticket, config, inputs));
    }
    let admitted = {
        let _admission = daemon.admission.lock().expect("admission lock");
        project.tx(|tx| {
            let mut admitted = Vec::new();
            let mut free = spare_lanes(tx, project, loaded)?;
            for (ticket, selected, inputs) in &selected {
                if free <= 0 {
                    break;
                }
                if tickets::get(tx, ticket.id)?.revision != ticket.revision
                    || tickets::blocker(tx, ticket.id)?.is_some()
                    || !admissible(tx, selected, ticket)?
                {
                    continue;
                }
                admitted.push(admit(tx, selected, ticket, None, inputs.clone())?);
                free -= 1;
            }
            Ok(admitted)
        })?
    };
    for execution in admitted {
        spawn(daemon, project, "implementation", execution);
    }
    Ok(())
}

/// Record admission evidence, workflow name, implementer and first execution
/// in one transaction.
fn admit(
    tx: &rusqlite::Connection,
    loaded: &Loaded,
    ticket: &tickets::Ticket,
    selection: Option<&crate::plan::Request>,
    mut inputs: Value,
) -> Result<i64, Fail> {
    let default = crate::plan::Request {
        _project: String::new(),
        target: None,
        preset: None,
        workflow: Some(ticket.workflow.clone()),
        graph: None,
        overrides: Default::default(),
    };
    let request = selection.unwrap_or(&default);
    let plan = loaded
        .config
        .resolve_plan(request)
        .map_err(|invalid| Fail::invalid("invalid plan").with(json!(invalid)))?;
    let name = loaded
        .config
        .runnable_workflow(&plan, request.workflow.as_deref())
        .ok_or_else(|| {
            Fail::refused("plan shape has no runtime support")
                .with(json!({"input": "plan", "shape": plan.unsupported_shape(),
                "structurally_valid": true, "runnable": false}))
        })?;
    let workflow = loaded.config.workflow(&name)?;
    let agent = &loaded.config.agents[&workflow.implementer];
    let mut implementer = serde_json::to_value(agent).expect("agent serializes");
    implementer["name"] = json!(workflow.implementer);
    let mut agents = std::collections::BTreeMap::new();
    let mut seats = std::collections::BTreeMap::new();
    for name in workflow.review.iter().filter(|_| !workflow.read_only) {
        seats.insert(name, &loaded.config.seats[name]);
    }
    let classifier_profiles = classifier_profiles(&loaded.config, &plan);
    for runner in plan
        .nodes
        .values()
        .map(|node| &node.runner)
        .chain(
            plan.classifiers
                .values()
                .map(|classifier| &classifier.runner),
        )
        .chain(classifier_profiles.values().map(|profile| &profile.runner))
    {
        if let crate::plan::Runner::Agent { agent, .. }
        | crate::plan::Runner::Packaged { agent, .. } = runner
        {
            let settings = &loaded.config.agents[agent];
            let harness = crate::harness::get(&settings.harness)
                .ok_or_else(|| Fail::invalid("selected agent has no harness"))?;
            agents.insert(agent, json!({
                "settings": settings,
                "runner": {"name": settings.harness, "version": harness.version()},
                "login_name": settings.login.and_then(|_| harness.login().map(|login| login.name)),
            }));
        }
    }
    let ordinary = loaded.config.workflow_plan(&name)?;
    let skipped = skipped_nodes(&plan, &ordinary);
    let unreviewed = !plan.nodes.values().any(|node| {
        matches!(node.runner, crate::plan::Runner::Agent { .. })
            && node.outputs.get("result") == Some(&crate::plan::Output::Verdict)
    });
    if plan.packaged().is_some() {
        inputs["package"] = crate::pi::package_identity();
    }
    let writer_profiles = writer_profiles(&loaded.config, &plan, request, None);
    let mut admission = json!({
        "writer_profiles": writer_profiles,
        "status": "recorded",
        "runtime": plan.runtime(),
        "revision": 1,
        "ticket_revision": ticket.revision,
        "target": loaded.branch,
        "config_head": loaded.head,
        "workflow": workflow,
        "plan": plan,
        "skipped": skipped,
        "unreviewed": unreviewed,
        "selection": {"preset": request.preset, "workflow": request.workflow,
            "graph": request.graph, "overrides": request.overrides},
        "agents": agents,
        "classifier_profiles": classifier_profiles,
        "seats": seats,
        "review": if workflow.read_only { None } else { Some(&loaded.config.review) },
        "gates": if workflow.read_only { &[][..] } else { &loaded.config.gates[..] },
        "inputs": inputs,
        "yard_runner": {"version": env!("CARGO_PKG_VERSION"), "sha256": runner_identity()?},
    });
    if plan
        .nodes
        .values()
        .any(|node| matches!(node.runner, crate::plan::Runner::Packaged { .. }))
    {
        admission["package_launch"] = json!("effort");
        admission["package_checkpoint"] = json!("exclusive-v1");
    }
    if plan.packaged_cleanup().is_some() {
        admission["package_launch"] = json!("source-write-v1");
        admission["package_checkpoint"] = json!("exclusive-v1");
    }
    let attempt = attempts::insert(
        tx,
        ticket.id,
        &name,
        &implementer,
        &loaded.head,
        &loaded.branch,
        &admission,
    )?;
    let row = attempts::get(tx, attempt)?;
    super::audit_attempt(
        tx,
        "attempt.admitted",
        &row,
        None,
        json!({ "target": loaded.branch, "base": loaded.head, "workflow": name, "implementer": implementer }),
    )?;
    super::supervise::start_implementation(tx, &row, "first", None)
}

fn skipped_nodes(plan: &crate::plan::Plan, ordinary: &crate::plan::Plan) -> Vec<String> {
    use crate::plan::Runner;
    ordinary
        .nodes
        .iter()
        .filter(|(name, expected)| {
            plan.nodes.get(*name).is_none_or(|node| {
                expected.outputs.get("result") == Some(&crate::plan::Output::Verdict)
                    && (node.outputs != expected.outputs
                        || !matches!(
                            (&node.runner, &expected.runner),
                            (Runner::Agent { .. }, Runner::Agent { .. })
                                | (Runner::Command { .. }, Runner::Command { .. })
                        ))
            })
        })
        .map(|(name, _)| name.clone())
        .collect()
}

fn writer_profiles(
    config: &crate::config::Config,
    plan: &crate::plan::Plan,
    request: &crate::plan::Request,
    previous: Option<&Value>,
) -> Value {
    json!(plan.nodes.iter()
        .filter(|(_, node)| node.access == crate::plan::Access::WriteAllowed
            && matches!(node.runner, crate::plan::Runner::Agent { .. } | crate::plan::Runner::Packaged { .. }))
        .map(|(name, node)| {
            let selected = request.overrides.get(name)
                .or_else(|| request.graph.as_ref().and_then(|graph| graph.nodes.get(name)))
                .or_else(|| request.preset.as_ref().and_then(|preset| config.presets.get(preset))
                    .and_then(|graph| graph.nodes.get(name)))
                .and_then(|spec| spec.profile.as_deref());
            let mut definition = json!({"name": selected, "runner": node.runner, "access": node.access,
                "outputs": node.outputs, "outcomes": node.outcomes});
            if selected.is_none() && let Some(old) = previous.and_then(|p| p.get(name))
                && ["runner", "access", "outputs", "outcomes"].iter().all(|key| old[key] == definition[key]) {
                definition["name"] = old["name"].clone();
            }
            (name, definition)
        }).collect::<std::collections::BTreeMap<_, _>>())
}

fn classifier_profiles<'a>(
    config: &'a crate::config::Config,
    plan: &'a crate::plan::Plan,
) -> std::collections::BTreeMap<&'a str, &'a crate::plan::Profile> {
    plan.classifiers
        .values()
        .flat_map(|policy| policy.outcomes.values())
        .filter_map(|action| match action {
            crate::plan::Continuation::Fixer { profile } => {
                Some((profile.as_str(), &config.profiles[profile]))
            }
            _ => None,
        })
        .collect()
}

/// The admission daemon's tool implementation identity, read once per process.
pub(super) fn runner_identity() -> Result<&'static str, Fail> {
    use sha2::{Digest, Sha256};
    static ID: std::sync::OnceLock<Result<String, String>> = std::sync::OnceLock::new();
    ID.get_or_init(|| {
        let path = std::env::current_exe().map_err(|error| error.to_string())?;
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        Ok(crate::config::hex(&Sha256::digest(bytes)))
    })
    .as_deref()
    .map_err(|error| Fail::invalid(error.clone()))
}

pub async fn status(daemon: &Daemon, project: &Project) -> Result<Value, Fail> {
    let loaded = load(daemon, project).await.ok();
    for attempt in project.read(attempts::live)? {
        super::load_attempt(daemon, project, &attempt).await?;
    }
    let selected = match &loaded {
        Some(loaded) => ready_configs(daemon, project, loaded).await?,
        None => Vec::new(),
    };
    let no_lane = match &loaded {
        Some(loaded) => project.read(|conn| spare_lanes(conn, project, loaded))? <= 0,
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
                    Some(_) if no_lane && tickets::blocker(conn, ticket.id)?.is_none() => selected
                        .iter()
                        .find(|(t, _)| t.id == ticket.id)
                        .map(|(_, config)| admissible(conn, config, ticket))
                        .transpose()?
                        .unwrap_or(false),
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
                let attempt = attempts::get(conn, approval.attempt)?;
                let ticket = attempt.ticket;
                let edges: Vec<String> = tickets::dependencies(conn, ticket)?
                    .into_iter()
                    .map(ticket_name)
                    .collect();
                Ok(
                    json!({ "approval": approval.id, "attempt": approval.attempt,
                           "ticket": ticket_name(ticket), "target": attempt.target_branch,
                           "depends_on": edges,
                           "head": approval.head, "actor": approval.actor, "choice": approval.choice }),
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
        "image": daemon.images.lock().expect("images lock").get(&project.key).map(|(branch, head, id)| json!({ "target": branch, "head": head, "id": id })),
        "connections": connections,
        "logins": logins,
        "missing_projects": missing,
    }))
}

/// Import a validated fast-forward under its target lock.
async fn import(
    daemon: &Daemon,
    project: &Project,
    held: &tokio::sync::MutexGuard<'_, ()>,
    publication: Option<&tokio::sync::MutexGuard<'_, ()>>,
    branch: &str,
    ours: Option<&str>,
    theirs: &str,
) -> Result<Value, Fail> {
    let canonical = project.canonical_dir();
    let target = crate::git::target_ref(branch);
    let moved = daemon
        .git
        .update_ref(&canonical, &target, theirs, ours, None)
        .await?;
    if !moved {
        let acquired;
        let published = match publication {
            Some(published) => published,
            None => {
                acquired = project.canonical.lock().await;
                &acquired
            }
        };
        super::publish_default(daemon, project, published, branch, held).await?;
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
            json!({ "branch": branch, "old": ours, "new": theirs }),
        )
    })?;
    Ok(json!({ "sync": "imported", "old": ours, "head": theirs }))
}

fn refuse_default_switch(project: &Project, default: &str, branch: &str) -> Result<(), Fail> {
    if branch == default {
        return Ok(());
    }
    project.read(|conn| {
        let live = attempts::live(conn)?;
        let intents = executions::open_intents(conn)?;
        if !intents.is_empty() {
            return Err(
                Fail::refused("a landing intent prevents changing the default").with(
                    json!({ "reason": "live_target", "default": default, "branch": branch,
                    "attempts": live.iter().map(|a| a.id).collect::<Vec<_>>(),
                    "intents": intents.iter().map(|e| e.id).collect::<Vec<_>>() }),
                ),
            );
        }
        Ok(())
    })
}

pub async fn sync(daemon: &Daemon, project: &Project, params: &Value) -> Result<Value, Fail> {
    let named = match &params["branch"] {
        Value::Null => None,
        Value::String(branch) => Some(branch.as_str()),
        _ => return Err(Fail::invalid("sync branch must be a local branch name")),
    };
    let select = match &params["default"] {
        Value::Null => false,
        Value::Bool(select) => *select,
        _ => return Err(Fail::invalid("sync default must be a boolean")),
    };
    if select && named.is_none() {
        return Err(Fail::invalid("selecting a default requires a named branch"));
    }
    if select {
        // A held landing child owns canonical. Its intent must refuse now.
        let default = daemon.git.head_branch(&project.canonical_dir()).await?;
        refuse_default_switch(project, &default, named.expect("selection names a branch"))?;
    }
    let canonical = project.canonical_dir();
    let initial_default = daemon.git.head_branch(&canonical).await?;
    let mut selected_branch = named.unwrap_or(&initial_default).to_string();
    let refuse_intent = |branch: &str, target: &str| {
        project.read(|conn| {
            if let Some(intent) = executions::open_intents(conn)?
                .into_iter()
                .find(|row| row.intent_ref.as_deref() == Some(target))
            {
                return Err(
                    Fail::refused("a landing intent prevents sync of its target")
                        .with(json!({ "reason": "intent", "branch": branch, "intent": intent.id })),
                );
            }
            Ok(())
        })
    };
    let mut target_lock = project.target_lock(&crate::git::target_ref(&selected_branch));
    let (held, publication, default) = loop {
        let target = crate::git::target_ref(&selected_branch);
        if named.is_some() {
            refuse_intent(&selected_branch, &target)?;
        }
        let held = target_lock.lock().await;
        let publication = project.canonical.lock().await;
        let default = daemon.git.head_branch(&canonical).await?;
        if named.is_none() && selected_branch != default {
            drop(publication);
            drop(held);
            selected_branch = default;
            target_lock = project.target_lock(&crate::git::target_ref(&selected_branch));
            continue;
        }
        refuse_intent(&selected_branch, &target)?;
        project.read(|conn| {
            if let Some(owner) = executions::query(
                conn,
                "kind = 'landing' AND reserved = 1 AND intent_ref = ?1",
                [&target],
            )?
            .first()
            {
                return Err(Fail::refused("a landing child still owns this target")
                    .with(json!({ "reason": "ownership", "branch": selected_branch, "execution": owner.id })));
            }
            Ok(())
        })?;
        break (held, publication, default);
    };
    let branch = selected_branch.as_str();
    let target = crate::git::target_ref(branch);
    let mut selection = Some(publication);
    if !select {
        drop(selection.take());
    }
    if !daemon.git.valid_branch(&canonical, branch).await? {
        return Err(
            Fail::invalid("sync requires a local branch name").with(json!({ "branch": branch }))
        );
    }
    let ours = daemon.git.rev_parse(&canonical, &target).await?;
    let checkout_head = daemon.git.rev_parse(&project.root, &target).await?;
    // Selection can use history already imported into canonical without
    // creating or switching a checkout branch.
    let selection_only = select && checkout_head.is_none() && ours.is_some();
    let theirs = checkout_head
        .clone()
        .or_else(|| if selection_only { ours.clone() } else { None })
        .ok_or_else(|| {
            Fail::refused(format!("the checkout has no branch {branch}"))
                .with(json!({ "reason": "missing_branch", "branch": branch }))
        })?;
    if select {
        refuse_default_switch(project, &default, branch)?;
    }
    // The objects only: no ref names them until the import moves the target.
    if checkout_head.is_some() {
        daemon.git.fetch(&canonical, &project.root, &target).await?;
    }
    let importing = match &ours {
        None => true,
        Some(ours) => {
            daemon
                .git
                .is_ancestor(&canonical, ours, &theirs, None)
                .await?
        }
    };
    if !importing
        && !daemon
            .git
            .is_ancestor(&canonical, &theirs, ours.as_deref().unwrap(), None)
            .await?
    {
        return Err(Fail::refused(format!(
            "the checkout's {branch} at {theirs} and canonical at {} have diverged",
            ours.as_deref().unwrap()
        ))
        .with(json!({ "checkout": theirs, "canonical": ours })));
    }
    let incoming = if importing {
        &theirs
    } else {
        ours.as_deref().unwrap()
    };
    let loaded = if select || (importing && ours.as_deref() != Some(theirs.as_str())) {
        Some(
            super::read_config(
                daemon,
                &canonical,
                branch,
                Some(if select { branch } else { &default }),
                incoming,
            )
            .await?,
        )
    } else {
        None
    };
    if let Some(loaded) = &loaded {
        let orphaned = project.read(|conn| {
            let mut orphaned = Vec::new();
            for ticket in tickets::list(conn)?
                .into_iter()
                .filter(|t| t.state == "open")
            {
                let future = ticket.target_branch.as_deref().unwrap_or(if select {
                    branch
                } else {
                    &default
                });
                let removed =
                    future == branch && !loaded.config.workflows.contains_key(&ticket.workflow);
                let live_removed = attempts::live_for(conn, ticket.id)?.is_some_and(|attempt| {
                    attempt.target_branch.as_deref() == Some(branch)
                        && !loaded.config.workflows.contains_key(&attempt.workflow)
                });
                if removed || live_removed {
                    orphaned.push(ticket_name(ticket.id));
                }
            }
            Ok(orphaned)
        })?;
        if !orphaned.is_empty() {
            return Err(Fail::refused(format!(
                "the incoming configuration removes a workflow open tickets name: {}",
                orphaned.join(", ")
            ))
            .with(json!({ "tickets": orphaned })));
        }
    }
    let mut result = if selection_only || (select && !importing) {
        json!({ "sync": "selected", "head": incoming })
    } else if ours.as_deref() == Some(theirs.as_str()) {
        json!({ "sync": "current", "head": theirs })
    } else if importing {
        if !select {
            selection = Some(project.canonical.lock().await);
        }
        let current_default = daemon.git.head_branch(&canonical).await?;
        if !select && current_default != default {
            super::read_config(daemon, &canonical, branch, Some(&current_default), &theirs).await?;
        }
        if branch == current_default {
            project.loaded.lock().expect("loaded lock").default = None;
        }
        if !select {
            drop(selection.take());
        }
        import(
            daemon,
            project,
            &held,
            selection.as_ref(),
            branch,
            ours.as_deref(),
            &theirs,
        )
        .await?
    } else {
        let ours = ours.as_deref().expect("a consumed canonical has a head");
        let checked_out = daemon.git.current_branch(&project.root).await?;
        if checked_out.as_deref() != Some(branch) {
            return Err(Fail::refused("consume requires the selected branch checked out")
            .with(json!({ "reason": "checkout_branch", "branch": branch, "checked_out": checked_out,
                "checkout": theirs, "canonical": ours })));
        }
        if !daemon.git.clean(&project.root).await? {
            return Err(Fail::refused("consume requires a clean checkout")
                .with(json!({ "reason": "dirty_checkout", "branch": branch,
                "checkout": theirs, "canonical": ours })));
        }
        daemon.git.fetch(&project.root, &canonical, &target).await?;
        if !daemon
            .git
            .fast_forward_checkout(&project.root, branch, &theirs, ours)
            .await?
        {
            return Err(
                Fail::refused("the checkout changed before consuming canonical")
                    .with(json!({ "reason": "checkout_changed", "branch": branch,
                    "checkout": theirs, "canonical": ours })),
            );
        }
        project.tx(|tx| {
            store::audit(
                tx,
                "sync.consumed",
                store::Target::default(),
                None,
                json!({ "branch": branch, "old": theirs, "new": ours }),
            )
        })?;
        json!({ "sync": "consumed", "old": theirs, "head": ours })
    };
    if select && branch != default {
        project.loaded.lock().expect("loaded lock").default = None;
        daemon.git.select_default(&canonical, branch).await?;
        project.tx(|tx| {
            store::audit(
                tx,
                "sync.default",
                store::Target::default(),
                None,
                json!({ "old": default, "new": branch }),
            )
        })?;
    }
    result["branch"] = json!(branch);
    result["default"] = json!(if select { branch } else { &default });
    let published = match selection {
        Some(held) => held,
        None => project.canonical.lock().await,
    };
    if (importing && ours.as_deref() != Some(theirs.as_str())) || (select && branch != default) {
        super::publish_default(daemon, project, &published, branch, &held).await?;
    } else {
        // A checkout-only consume must not heal an uncertain default mutation.
        super::refresh_default(daemon, project, &published).await?;
    }
    Ok(result)
}

pub async fn ticket_new(daemon: &Daemon, project: &Project, params: &Value) -> Result<Value, Fail> {
    let _canonical = project.canonical.lock().await;
    let loaded = load(daemon, project).await?;
    let target = target_param(params)?;
    if let Some(target) = target {
        daemon
            .git
            .valid_branch(&project.canonical_dir(), target)
            .await?
            .then_some(())
            .ok_or_else(|| Fail::invalid("target requires a local branch name"))?;
    }
    let loaded = if let Some(target) = target {
        // Missing branches may be filed; admission refuses them without an attempt.
        if daemon
            .git
            .rev_parse(&project.canonical_dir(), &crate::git::target_ref(target))
            .await?
            .is_some()
        {
            super::load_target(daemon, project, target).await?
        } else {
            loaded
        }
    } else {
        loaded
    };
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
                target_branch: target,
            },
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
pub(super) fn refuse_during_intent(conn: &rusqlite::Connection, ticket: i64) -> Result<(), Fail> {
    for intent in executions::open_intents(conn)? {
        let attempt = attempts::get(conn, intent.required_attempt()?)?;
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
    let target = target_param(params)?;
    let clear_target = match &params["default_target"] {
        Value::Null => false,
        Value::Bool(clear) => *clear,
        _ => return Err(Fail::invalid("default_target must be a boolean")),
    };
    let target_edit = target.is_some() || clear_target;
    if target.is_some() && clear_target {
        return Err(Fail::invalid("target and default_target are exclusive"));
    }
    if target_edit {
        project.read(|conn| target_editable(conn, id, revision))?;
    }
    if let Some(target) = target
        && !daemon
            .git
            .valid_branch(&project.canonical_dir(), target)
            .await?
    {
        return Err(Fail::invalid("target requires a local branch name"));
    }
    let workflow = text_param(params, "workflow");
    let _canonical = if workflow.is_some() {
        project.read(|conn| refuse_during_intent(conn, id))?;
        Some(project.canonical.lock().await)
    } else {
        None
    };
    if let Some(workflow) = &workflow {
        let ticket = project.read(|conn| tickets::get(conn, id))?;
        let branch = if clear_target {
            None
        } else {
            target.or(ticket.target_branch.as_deref())
        };
        let loaded = if let Some(branch) = branch {
            super::load_target(daemon, project, branch).await?
        } else {
            load(daemon, project).await?
        };
        loaded.config.workflow(workflow).map_err(Fail::invalid)?;
    }
    let priority = priority_param(&params["priority"])?;
    let stops = project.tx(|tx| {
        if target_edit {
            target_editable(tx, id, revision)?;
        }
        if target_edit {
            tx.execute(
                "UPDATE ticket SET target_branch = ?2 WHERE id = ?1",
                rusqlite::params![id, target],
            )?;
        }
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

fn target_editable(conn: &rusqlite::Connection, id: i64, revision: i64) -> Result<(), Fail> {
    let ticket = tickets::get(conn, id)?;
    if ticket.revision != revision {
        return Err(Fail::stale(
            "the ticket revision changed",
            json!(revision),
            json!(ticket.revision),
        ));
    }
    refuse_during_intent(conn, id)?;
    if let Some(attempt) = attempts::live_for(conn, id)? {
        return Err(Fail::refused("a live attempt freezes its target").with(json!({ "reason": "live_target", "attempt": attempt.id, "target": attempt.target_branch })));
    }
    if ticket.state != "open" {
        return Err(Fail::refused("only an open ticket can change target"));
    }
    Ok(())
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
                "priority": priority, "workflow": workflow, "target": tickets::get(tx, id)?.target_branch }),
    )?;
    steer(tx, id)
}

/// Make the live attempt run its implementer next. The pending edit is derived
/// from the executions, so a running implementer finishes first and nothing is stored.
fn steer(tx: &rusqlite::Connection, id: i64) -> Result<Vec<i64>, Fail> {
    let Some(attempt) = attempts::live_for(tx, id)? else {
        return Ok(Vec::new());
    };
    if attempt
        .admission
        .as_ref()
        .and_then(|admission| crate::plan::Plan::admitted(admission).ok())
        .is_some_and(|plan| plan.writers())
    {
        return super::writers::steer(tx, &attempt);
    }
    let running: Vec<_> = executions::attempt_work(tx, attempt.id)?
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
            "approval" | "candidate" => "superseded",
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

/// Trusted adjustment uses the admitted source, never the current template.
pub async fn attempt_revise(
    daemon: &Daemon,
    project: &Project,
    params: &Value,
) -> Result<Value, Fail> {
    crate::mcp::strict(
        params,
        &[
            "project",
            "ticket",
            "plan_revision",
            "preset",
            "workflow",
            "graph",
            "overrides",
            "definition_commit",
            "staging_commit",
            "image_commit",
        ],
    )?;
    let id = ticket_param(params)?;
    let expected = params["plan_revision"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("adjustment names plan_revision"))?;
    let attempt = project.read(|conn| {
        let attempt =
            attempts::live_for(conn, id)?.ok_or_else(|| Fail::refused("no live attempt"))?;
        attempts::require_plan(conn, attempt.id, Some(expected))?;
        Ok(attempt)
    })?;
    let frozen = super::load_attempt(daemon, project, &attempt).await?;
    let adopting = ["definition_commit", "staging_commit", "image_commit"]
        .iter()
        .any(|key| params.get(*key).is_some());
    let source = |key: &str, previous: &str| -> Result<String, Fail> {
        match params.get(key) {
            None => Ok(previous.to_string()),
            Some(value) => value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| Fail::invalid(format!("{key} requires an exact commit"))),
        }
    };
    let loaded = if adopting {
        let mut loaded = super::read_sources(
            daemon,
            &project.canonical_dir(),
            attempt.target_branch()?,
            None,
            &source("definition_commit", &frozen.head)?,
            &source("staging_commit", &frozen.staging_commit)?,
            &source("image_commit", &frozen.image_commit)?,
        )
        .await?;
        if params.get("definition_commit").is_none() {
            loaded.config = frozen.config.clone();
        }
        Arc::new(loaded)
    } else {
        frozen.clone()
    };
    let mut evidence = attempt
        .admission
        .clone()
        .ok_or_else(|| Fail::refused("unrecorded plan"))?;
    let old =
        crate::plan::Plan::admitted(&evidence).map_err(|error| Fail::refused(error.to_string()))?;
    let packaged = old.packaged().is_some();
    let mut selection = if params.get("definition_commit").is_some() {
        evidence["selection"].clone()
    } else {
        json!({"project": project.root})
    };
    selection["project"] = json!(project.root);
    if ["preset", "workflow", "graph"]
        .iter()
        .any(|key| params.get(*key).is_some())
    {
        selection = json!({"project": project.root});
    }
    for key in ["preset", "workflow", "graph", "overrides"] {
        if let Some(value) = params.get(key) {
            selection[key] = value.clone();
        }
    }
    if params.get("definition_commit").is_none()
        && !["preset", "workflow", "graph"]
            .iter()
            .any(|key| params.get(*key).is_some())
    {
        selection["graph"] = serde_json::to_value(&old).expect("plan serializes");
    }
    let request: crate::plan::Request = serde_json::from_value(selection.clone())
        .map_err(|error| Fail::invalid(error.to_string()))?;
    let plan = loaded
        .config
        .resolve_plan(&request)
        .map_err(|invalid| Fail::invalid("invalid plan").with(json!(invalid)))?;
    if plan.runtime() != old.runtime()
        || loaded
            .config
            .runnable_workflow(&plan, Some(&attempt.workflow))
            .as_deref()
            != Some(&attempt.workflow)
    {
        return Err(Fail::refused("plan shape has no runtime support")
            .with(json!({"reason": "unsupported_revision", "shape": plan.unsupported_shape()})));
    }
    let mut implementer = plan.nodes["implementer"].clone();
    let mut old_implementer = old.nodes["implementer"].clone();
    implementer.routes.remove("complete");
    old_implementer.routes.remove("complete");
    // Guidance may change; access, agent and publication grants remain frozen.
    if let (
        crate::plan::Runner::Agent {
            instructions,
            skills,
            references,
            ..
        },
        crate::plan::Runner::Agent {
            instructions: previous,
            skills: old_skills,
            references: old_references,
            ..
        },
    ) = (&mut implementer.runner, &old_implementer.runner)
    {
        *instructions = previous.clone();
        *skills = old_skills.clone();
        *references = old_references.clone();
    }
    if let (
        crate::plan::Runner::Packaged { instructions, .. },
        crate::plan::Runner::Packaged {
            instructions: previous,
            ..
        },
    ) = (&mut implementer.runner, &old_implementer.runner)
    {
        *instructions = previous.clone();
    }
    if (plan.writers() || plan.handoff())
        && (plan
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.access == crate::plan::Access::WriteAllowed
                    && matches!(
                        node.runner,
                        crate::plan::Runner::Agent { .. } | crate::plan::Runner::Packaged { .. }
                    )
            })
            .any(|(name, node)| {
                old.nodes.get(name).is_none_or(|previous| {
                    let mut node = node.clone();
                    let mut previous = previous.clone();
                    node.routes.clear();
                    previous.routes.clear();
                    if let (
                        crate::plan::Runner::Agent {
                            instructions,
                            skills,
                            references,
                            ..
                        },
                        crate::plan::Runner::Agent {
                            instructions: old,
                            skills: old_skills,
                            references: old_references,
                            ..
                        },
                    ) = (&mut node.runner, &previous.runner)
                    {
                        *instructions = old.clone();
                        *skills = old_skills.clone();
                        *references = old_references.clone();
                    }
                    if let (
                        crate::plan::Runner::Packaged {
                            instructions,
                            skill,
                            references,
                            files,
                            ..
                        },
                        crate::plan::Runner::Packaged {
                            instructions: old,
                            skill: old_skill,
                            references: old_references,
                            files: old_files,
                            ..
                        },
                    ) = (&mut node.runner, &previous.runner)
                    {
                        *instructions = old.clone();
                        *skill = old_skill.clone();
                        *references = old_references.clone();
                        *files = old_files.clone();
                    }
                    node != previous
                })
            })
            || old.nodes.iter().any(|(name, node)| {
                node.access == crate::plan::Access::WriteAllowed
                    && matches!(
                        node.runner,
                        crate::plan::Runner::Agent { .. } | crate::plan::Runner::Packaged { .. }
                    )
                    && !plan.nodes.contains_key(name)
            }))
    {
        return Err(Fail::refused("adjustment changes frozen writer grants")
            .with(json!({"reason": "unsupported_revision"})));
    }
    if implementer != old_implementer {
        return Err(Fail::refused("adjustment changes frozen grants")
            .with(json!({"reason": "unsupported_revision"})));
    }
    if adopting {
        for (name, node) in &old.nodes {
            if name != "implementer"
                && (!(old.writers() || old.handoff())
                    || node.access != crate::plan::Access::WriteAllowed
                    || !matches!(
                        node.runner,
                        crate::plan::Runner::Agent { .. } | crate::plan::Runner::Packaged { .. }
                    ))
            {
                continue;
            }
            let (crate::plan::Runner::Agent { agent, .. }
            | crate::plan::Runner::Packaged { agent, .. }) = &node.runner
            else {
                return Err(Fail::refused("unsupported implementer"));
            };
            let mut settings = loaded.config.agents[agent].clone();
            let previous = &frozen.config.agents[agent];
            settings.model = previous.model.clone();
            settings.effort = previous.effort.clone();
            if settings != *previous || loaded.config.egress != frozen.config.egress {
                return Err(Fail::refused(
                    "source adoption changes frozen implementation harness, connection or grants",
                )
                .with(json!({"reason": "unsupported_revision"})));
            }
        }
        let workflow = loaded.config.workflow(&attempt.workflow)?;
        evidence["workflow"] = json!(workflow);
        evidence["gates"] = json!(if workflow.read_only {
            &[][..]
        } else {
            &loaded.config.gates[..]
        });
        evidence["review"] = if workflow.read_only {
            Value::Null
        } else {
            json!(loaded.config.review)
        };
        evidence["seats"] = json!(
            workflow
                .review
                .iter()
                .filter(|_| !workflow.read_only)
                .map(|name| (name, &loaded.config.seats[name]))
                .collect::<std::collections::BTreeMap<_, _>>()
        );
        evidence["agents"] = json!({});
        evidence["inputs"] = loaded
            .inputs(&daemon.git, &project.canonical_dir(), &plan)
            .await?;
        if packaged {
            evidence["inputs"]["package"] =
                attempt.admission.as_ref().expect("recorded plan")["inputs"]["package"].clone();
        }
        evidence["config_head"] = json!(loaded.head);
    }
    if !adopting && !packaged {
        evidence["inputs"] = loaded
            .inputs(&daemon.git, &project.canonical_dir(), &plan)
            .await?;
    }
    // A first selected package uses today's launch contract. Retained frozen
    // packages keep their historical capabilities, including an absent record.
    if !old
        .nodes
        .values()
        .any(|node| matches!(node.runner, crate::plan::Runner::Packaged { .. }))
        && plan
            .nodes
            .values()
            .any(|node| matches!(node.runner, crate::plan::Runner::Packaged { .. }))
        && evidence.get("package_launch").is_none()
    {
        evidence["package_launch"] = json!("effort");
        evidence["package_checkpoint"] = json!("exclusive-v1");
    }
    let recorded = attempt.admission.as_ref().expect("recorded plan");
    for (name, node) in &plan.nodes {
        let crate::plan::Runner::Packaged { agent, .. } = &node.runner else {
            continue;
        };
        if old
            .nodes
            .get(name)
            .is_some_and(|node| matches!(node.runner, crate::plan::Runner::Packaged { .. }))
        {
            let identity = crate::pi::package_inputs(recorded, name)["package"].clone();
            if plan.packaged().is_some() {
                evidence["inputs"]["package"] = identity;
            } else {
                evidence["inputs"]["packaged_nodes"][name]["package"] = identity;
            }
        }
        let settings = &loaded.config.agents[agent];
        let previous = &recorded["agents"][agent]["settings"]["effort"];
        if adopting
            && !crate::pi::package_effort_capable(&evidence, name)
            && settings.effort.is_some()
            && json!(settings.effort) != *previous
        {
            return Err(
                Fail::refused("recorded package cannot control requested effort")
                    .with(json!({"reason": "unsupported_effort", "identity": "effort",
                    "expected": settings.effort, "current": previous})),
            );
        }
    }
    let ordinary = loaded.config.workflow_plan(&attempt.workflow)?;
    let skipped = skipped_nodes(&plan, &ordinary);
    evidence["revision"] = json!(
        expected
            .checked_add(1)
            .ok_or_else(|| Fail::invalid("plan revision overflow"))?
    );
    evidence["plan"] = json!(plan);
    let classifier_profiles = classifier_profiles(&loaded.config, &plan);
    evidence["classifier_profiles"] = json!(classifier_profiles);
    for runner in plan
        .nodes
        .values()
        .map(|node| &node.runner)
        .chain(
            plan.classifiers
                .values()
                .map(|classifier| &classifier.runner),
        )
        .chain(classifier_profiles.values().map(|profile| &profile.runner))
    {
        if let crate::plan::Runner::Agent { agent, .. }
        | crate::plan::Runner::Packaged { agent, .. } = runner
        {
            let settings = &loaded.config.agents[agent];
            let harness = crate::harness::get(&settings.harness)
                .ok_or_else(|| Fail::invalid("selected agent has no harness"))?;
            evidence["agents"][agent] = json!({
                "settings": settings,
                "runner": {"name": settings.harness, "version": harness.version()},
                "login_name": settings.login.and_then(|_| harness.login().map(|login| login.name)),
            });
        }
    }
    evidence["writer_profiles"] = writer_profiles(
        &loaded.config,
        &plan,
        &request,
        Some(&evidence["writer_profiles"]),
    );
    evidence["selection"] = selection;
    evidence["skipped"] = json!(skipped);
    evidence["unreviewed"] = json!(!plan.nodes.values().any(|node| {
        matches!(node.runner, crate::plan::Runner::Agent { .. })
            && node.outputs.get("result") == Some(&crate::plan::Output::Verdict)
    }));
    let (result, stops) = project.tx(|tx| {
        let current = attempts::live_for(tx, id)?
            .ok_or_else(|| Fail::refused("no live attempt"))?;
        if current.id != attempt.id {
            return Err(candidate_stale("attempt", json!(attempt.id), json!(current.id)));
        }
        attempts::require_plan(tx, current.id, Some(expected))?;
        let executions = executions::attempt_work(tx, current.id)?;
        let open = attempts::open_for_attempt(tx, current.id)?;
        if adopting && executions.iter().any(|row| row.status == "running" && matches!(row.kind.as_str(), "gate" | "review"))
        {
            return Err(Fail::refused("source adoption requires implementation or quiescent candidate work")
                .with(json!({"reason": "active_plan", "attempt": current.id})));
        }
        let implementing = executions.iter().any(|row| row.kind == "implementation" && row.status == "running");
        let mut stopped_owner = None;
        if adopting && packaged && !implementing {
            for row in &executions {
                if row.kind == "implementation" && row.status == "ended"
                    && row.outcome.as_deref() == Some("unchanged") && row.reserved
                    && row.lock_contract && row.node.as_deref() == Some("implementer")
                    && open.iter().any(|item| item.kind == "stopped" && item.reason == "unchanged" && item.execution == Some(row.id))
                    && open.iter().any(|item| item.kind == "red" && item.reason == "ownership" && item.execution == Some(row.id))
                    && attempts::for_execution(tx, row)?.admission.as_ref()
                        .is_some_and(|admission| admission["runtime"] == "packaged-audit")
                {
                    stopped_owner = Some(row.id);
                }
            }
        }
        if (packaged && !implementing && stopped_owner.is_none())
            || (current.candidate().is_none() && !implementing && stopped_owner.is_none())
            || (current.next.is_some() && !implementing)
            || open.iter().any(|item| !matches!(item.kind.as_str(), "approval" | "candidate" | "red")
                && !(stopped_owner.is_some() && item.kind == "stopped" && item.reason == "unchanged" && item.execution == stopped_owner)
                || (item.kind == "red" && item.reason != "ownership"))
        {
            return Err(Fail::refused("plan adjustment requires implementation or candidate work")
                .with(json!({"reason": "active_plan", "attempt": current.id})));
        }
        refuse_during_intent(tx, id)?;
        let landing_requirements = |plan: &crate::plan::Plan, evidence: &Value| {
            plan.nodes.iter().filter(|(_, node)| matches!(node.runner, crate::plan::Runner::Command { .. })
                && node.inputs.values().any(|input| input.node.is_none() && input.output == "merged"))
                .map(|(name, node)| {
                    json!({"name": name, "node": node,
                        "definition": executions::consumed_definition(evidence, &node.runner, name, false)})
                }).collect::<Vec<_>>()
        };
        if landing_requirements(&plan, &evidence)
            != landing_requirements(&old, current.admission.as_ref().expect("recorded plan"))
            && let Some(intent) = executions.iter().find(|row| row.intent_state.is_some())
        {
            return Err(Fail::refused("landing requirements already have an intent")
                .with(json!({"intent": intent.id})));
        }
        // A revision during implementation reaches its successor as an operator diff.
        if let Some(row) = executions.iter().rev().find(|row| row.kind == "implementation" && row.status == "running") {
            evidence["handoff_from"] = json!(row.id);
        }
        attempts::append_plan(tx, current.id, expected, &evidence)?;
        if let Some(approval) = checks::active_for(tx, current.id)? {
            checks::set_approval_state(tx, &approval, "withdrawn", id, None)?;
        }
        for item in open { if item.reason != "ownership" && item.kind != "stopped" { attempts::resolve(tx, &item, "superseded", None)?; } }
        let revised = attempts::get(tx, current.id)?;
        let stops = executions.iter().filter(|row| row.status == "running" && row.parent.is_none() && matches!(row.kind.as_str(), "gate" | "review"))
            .filter_map(|row| match executions::plan_superseded(tx, &revised, row) { Ok(false) => None, _ => Some(row.id) }).collect::<Vec<_>>();
        if attempts::edit_pending(tx, current.id)? { attempts::set_next(tx, current.id, None)?; }
        super::audit_attempt(tx, "plan.revised", &current, None,
            json!({"expected": expected, "plan_revision": expected + 1, "selection": evidence["selection"]}))?;
        Ok((attempts::get(tx, current.id)?.to_json(), stops))
    })?;
    stop_executions(daemon, project, &stops);
    Ok(result)
}

/// Operator `attempt.start`: the scheduler's admission for one named ticket, or
/// the `start` of its live attempt's stopped or red item.
pub async fn attempt_start(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    params: &Value,
) -> Result<Value, Fail> {
    crate::mcp::strict(
        params,
        &[
            "project",
            "ticket",
            "attention",
            "preset",
            "workflow",
            "graph",
            "overrides",
            "workspace",
        ],
    )?;
    let selected = ["preset", "workflow", "graph", "overrides"]
        .iter()
        .any(|key| params.get(*key).is_some());
    let id = ticket_param(params)?;
    // A pending writer dispatch needs no configuration read to refuse a rival start.
    project.read(|conn| {
        if let Some(attempt) = attempts::live_for(conn, id)? {
            refuse_pending_continuation(conn, attempt.id)?;
        }
        Ok(())
    })?;
    let canonical = project.canonical.lock().await;
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
    if selected
        && (project.read(|conn| attempts::live_for(conn, id))?.is_some() || !items.is_empty())
    {
        return Err(Fail::refused("plan selection requires a new attempt")
            .with(json!({"input": "plan", "reason": "active_plan"})));
    }
    let item = if let Some(expected) = params["attention"].as_i64() {
        items.iter().find(|item| item.id == expected)
    } else {
        let mut recovery = None;
        for item in &items {
            if let Some(execution) = item.execution
                && project.read(|conn| {
                    Ok(executions::get(conn, execution)?.intent_state.as_deref() == Some("open"))
                })?
            {
                recovery = Some(item);
                break;
            }
        }
        recovery.or_else(|| items.first())
    };
    if let Some(item) = item
        && let Some(execution) = item.execution
        && project.read(|conn| {
            Ok(executions::get(conn, execution)?.intent_state.as_deref() == Some("open"))
        })?
    {
        // Intent repair proves ownership before restoring an unavailable input.
        drop(canonical);
        return answer_start(daemon, project, item).await;
    }
    let default = super::refresh_default(daemon, project, &canonical).await?;
    let live = project.read(|conn| attempts::live_for(conn, id))?;
    let ticket = project.read(|conn| tickets::get(conn, id))?;
    if let Some(workspace) = params["workspace"].as_str() {
        if selected || params.get("attention").is_some() {
            return Err(Fail::invalid(
                "workspace continuation cannot select a plan or attention",
            ));
        }
        let attempt = live
            .as_ref()
            .ok_or_else(|| Fail::refused("undeclared_workspace"))?;
        let plan = crate::plan::Plan::admitted(
            attempt
                .admission
                .as_ref()
                .ok_or_else(|| Fail::refused("undeclared_workspace"))?,
        )
        .map_err(|error| Fail::refused(error.to_string()))?;
        if !(plan.handoff() && workspace == "implementer")
            && (!plan.writers()
                || plan.nodes.get(workspace).is_none_or(|node| {
                    node.access != crate::plan::Access::WriteAllowed
                        || node.outputs.get("result") != Some(&crate::plan::Output::Candidate)
                }))
        {
            return Err(Fail::refused("undeclared_workspace")
                .with(json!({"reason": "undeclared_workspace", "workspace": workspace})));
        }
        let execution = {
            let _admission = daemon.admission.lock().expect("admission lock");
            project.tx(|tx| {
                attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
                refuse_pending_continuation(tx, attempt.id)?;
                let open = attempts::open_for_attempt(tx, attempt.id)?;
                if let Some(item) = open.iter().find(|item| {
                    (item.kind == "stopped" && item.reason == "timeout")
                        || (item.kind == "proposal" && item.reason == "edit")
                }) {
                    return Err(Fail::refused(
                        "continuation requires the pending attention to be answered",
                    )
                    .with(json!({"reason": item.reason, "attention": item.id})));
                }
                let node = if plan.handoff() {
                    super::supervise::implementation_node(tx, attempt)?
                } else {
                    workspace.to_owned()
                };
                let execution = super::supervise::start_writer(tx, attempt, &node, "retry", None)?;
                if !take_lane_in(tx, &default, attempt.id)? {
                    return Err(
                        Fail::refused("no lane is free").with(json!({"reason": "capacity"}))
                    );
                }
                for item in attempts::open_for_attempt(tx, attempt.id)? {
                    if (item.kind == "stopped" && item.reason == "candidates")
                        || (matches!(item.kind.as_str(), "stopped" | "red")
                            && item.exits().contains(&"start")
                            && item
                                .execution
                                .and_then(|id| executions::get(tx, id).ok())
                                .is_some_and(|row| row.workspace.as_deref() == Some(workspace)))
                    {
                        attempts::resolve(tx, &item, "start", None)?;
                    }
                }
                Ok(execution)
            })?
        };
        spawn(daemon, project, "implementation", execution);
        return Ok(json!({"execution": execution, "workspace": workspace}));
    }
    if let Some(attempt) = &live
        && attempt
            .admission
            .as_ref()
            .and_then(|a| crate::plan::Plan::admitted(a).ok())
            .is_some_and(|p| p.writers())
        && (params["attention"].as_i64().is_none()
            || item.is_none_or(|item| item.execution.is_none()))
    {
        return Err(Fail::refused("undeclared_workspace")
            .with(json!({"reason": "undeclared_workspace",
            "attempt": attempt.id})));
    }
    if let Some(attempt) = &live
        && let Some(owner) = project.read(|conn| {
            Ok(executions::attempt_work(conn, attempt.id)?
                .into_iter()
                .find(|row| {
                    row.reserved
                        && row
                            .prepared
                            .as_ref()
                            .is_some_and(|prepared| !prepared["durable"].is_null())
                }))
        })?
    {
        return Err(Fail::refused("checkpoint_owned: previous execution still owns its checkpoint")
            .with(json!({"reason": "checkpoint_owned", "execution": owner.id,
                "checkpoint": owner.prepared.as_ref().expect("durable claim")["durable"]["checkpoint"]})));
    }
    let branch = live
        .as_ref()
        .map(attempts::Attempt::target_branch)
        .transpose()?
        .or(ticket.target_branch.as_deref())
        .unwrap_or(&default.branch);
    let loaded = super::load_target(daemon, project, branch).await?;
    if let Some(attempt) = &live
        && attempt.admission.is_none()
        && loaded.config.workflow(&attempt.workflow)?.read_only
    {
        return Err(Fail::refused(
            "unbound_report: abandon the legacy read-only attempt, then start a new admitted attempt",
        ).with(json!({"reason": "unbound_report", "attempt": attempt.id})));
    }
    if let Some(item) = item {
        drop(canonical);
        return answer_start(daemon, project, item).await;
    }
    if live.is_some() {
        return Err(Fail::refused(format!(
            "{} has a live attempt with nothing to start",
            ticket_name(id)
        )));
    }
    for attempt in project.read(attempts::live)? {
        super::load_attempt(daemon, project, &attempt).await?;
    }
    let ticket = project.read(|conn| {
        if let Some(blocker) = tickets::blocker(conn, id)? {
            return Err(Fail::refused(format!(
                "{} is not ready: {blocker}",
                ticket_name(id)
            )));
        }
        if spare_lanes(conn, project, &default)? <= 0 {
            return Err(Fail::refused("no lane is free").with(json!({"reason": "capacity"})));
        }
        tickets::get(conn, id)
    })?;
    let revision = ticket.revision;
    let selection = if selected {
        let mut fields = serde_json::Map::new();
        fields.insert("project".into(), params["project"].clone());
        for key in ["preset", "workflow", "graph", "overrides"] {
            if let Some(value) = params.get(key) {
                fields.insert(key.into(), value.clone());
            }
        }
        if !fields.contains_key("preset")
            && !fields.contains_key("graph")
            && !fields.contains_key("workflow")
        {
            fields.insert("workflow".into(), json!(ticket.workflow));
        }
        Some(
            serde_json::from_value::<crate::plan::Request>(Value::Object(fields))
                .map_err(|error| Fail::invalid(error.to_string()))?,
        )
    } else {
        None
    };
    let plan = match &selection {
        Some(request) => loaded
            .config
            .resolve_plan(request)
            .map_err(|invalid| Fail::invalid("invalid plan").with(json!(invalid)))?,
        None => loaded.config.workflow_plan(&ticket.workflow)?,
    };
    let inputs = loaded
        .inputs(&daemon.git, &project.canonical_dir(), &plan)
        .await?;
    let execution = {
        let _admission = daemon.admission.lock().expect("admission lock");
        project.tx(|tx| {
            let Some(ticket) = tickets::ready(tx)?.into_iter().find(|t| t.id == id) else {
                let blocker = tickets::blocker(tx, id)?;
                return Err(Fail::refused(format!(
                    "{} is not ready: {}",
                    ticket_name(id),
                    blocker.unwrap_or_default()
                )));
            };
            if ticket.target_branch.as_deref().unwrap_or(&default.branch) != loaded.branch {
                return Err(Fail::stale(
                    "the ticket target changed",
                    json!(loaded.branch),
                    json!(ticket.target_branch),
                ));
            }
            if ticket.revision != revision {
                return Err(Fail::stale(
                    "the ticket changed before admission",
                    json!(revision),
                    json!(ticket.revision),
                ));
            }
            if spare_lanes(tx, project, &default)? <= 0 {
                return Err(Fail::refused("no lane is free").with(json!({ "reason": "capacity" })));
            }
            admit(tx, &loaded, &ticket, selection.as_ref(), inputs)
        })?
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
    if let Some(row) = execution
        .as_ref()
        .filter(|row| row.reason.as_deref() == Some("classification"))
    {
        let successor = project.tx(|tx| {
            let row = executions::get(tx, row.id)?;
            let successor = super::classify::retry(tx, &row)?;
            attempts::resolve(tx, item, "start", None)?;
            Ok(successor)
        })?;
        spawn(daemon, project, "review", successor);
        return Ok(json!({"started": item.id, "execution": successor}));
    }
    let cleanup_audit = if let Some(row) = execution.as_ref().filter(|row| row.attempt.is_some()) {
        let attempt = project.read(|conn| attempts::get(conn, row.required_attempt()?))?;
        attempt
            .admission
            .as_ref()
            .and_then(|admission| crate::plan::Plan::admitted(admission).ok())
            .is_some_and(|plan| {
                plan.cleanup_audits()
                    .any(|(name, _)| row.node.as_ref() == Some(name))
            })
    } else {
        false
    };
    if let Some(previous) = execution
        .as_ref()
        .filter(|row| row.kind == "implementation" && !cleanup_audit)
    {
        let attempt = project.read(|conn| attempts::get(conn, previous.required_attempt()?))?;
        if attempt
            .admission
            .as_ref()
            .and_then(|admission| crate::plan::Plan::admitted(admission).ok())
            .is_some_and(|plan| plan.writers())
        {
            let workspace = previous
                .workspace
                .as_deref()
                .ok_or_else(|| Fail::refused("undeclared_workspace"))?;
            let default = super::load(daemon, project).await?;
            let successor =
                {
                    let _admission = daemon.admission.lock().expect("admission lock");
                    project.tx(|tx| {
                        attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
                        refuse_pending_continuation(tx, attempt.id)?;
                        if let Some(blocker) = attempts::open_for_attempt(tx, attempt.id)?
                            .iter()
                            .find(|open| {
                                (open.kind == "stopped" && open.reason == "timeout")
                                    || (open.kind == "proposal" && open.reason == "edit")
                            })
                        {
                            return Err(Fail::refused(
                                "continuation requires the pending attention to be answered",
                            )
                            .with(json!({"reason": blocker.reason, "attention": blocker.id})));
                        }
                        let successor = super::supervise::start_writer(
                            tx,
                            &attempt,
                            workspace,
                            if item.reason == "unchanged" {
                                "restart"
                            } else {
                                "retry"
                            },
                            None,
                        )?;
                        if !take_lane_in(tx, &default, attempt.id)? {
                            return Err(Fail::refused("no lane is free")
                                .with(json!({"reason": "capacity"})));
                        }
                        attempts::resolve(tx, item, "start", None)?;
                        Ok(successor)
                    })?
                };
            spawn(daemon, project, "implementation", successor);
            return Ok(json!({"started": item.id, "execution": successor, "workspace": workspace}));
        }
    }
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
        let rerun = cleanup_audit || execution
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
                Some(&json!({ "reason": reason, "detail": detail, "workspace": execution.as_ref().and_then(|row| row.workspace.as_deref()) })),
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
        let writers: Vec<_> = executions::attempt_work(conn, attempt.id)?.into_iter()
            .filter(|row| row.status == "running" && row.kind == "implementation"
                && params["execution"].as_i64().is_none_or(|id| row.id == id)).collect();
        if writers.len() > 1 { return Err(Fail::refused("ambiguous_execution").with(json!({
            "reason": "ambiguous_execution", "executions": writers.iter().map(|row| row.id).collect::<Vec<_>>()}))); }
        Ok(writers.into_iter().next())
    })?;
    let Some(running) = running else {
        return Err(Fail::refused(format!(
            "{} has no running implementer execution",
            ticket_name(id)
        )));
    };
    let stop = daemon
        .stops
        .lock()
        .expect("stops lock")
        .get(&(project.key.clone(), running.id))
        .cloned();
    let stopped_waiter = project.tx(|tx| {
        let row = executions::get(tx, running.id)?;
        if row.status != "running" {
            return Err(Fail::refused("the implementer execution already ended"));
        }
        if row.reserved {
            if stop.is_none() {
                let attention = attempts::open_attention(tx)?
                    .into_iter()
                    .find(|item| item.execution == Some(row.id) && item.reason == "ownership");
                return Err(Fail::refused("the retained execution has no current stop handle")
                    .with(json!({
                        "reason": "ownership", "ticket": ticket_name(id),
                        "attempt": row.attempt, "execution": row.id,
                        "attention": attention.map(|item| item.id),
                    })));
            }
            return Ok(false);
        }
        executions::end(tx, row.id, executions::End {
            outcome: "unchanged", ticket: Some(id), ..Default::default()
        })?;
        attempts::raise(tx, attempts::Raise {
            kind: "stopped", reason: "unchanged", ticket: Some(id),
            attempt: row.attempt, execution: Some(row.id),
            payload: json!({ "detail": "the operator stopped the execution before external work" }),
            text: None,
        })?;
        Ok(true)
    })?;
    if let Some(stop) = stop
        && !stopped_waiter
    {
        stop.notify_one();
    }
    project.tx(|tx| {
        store::audit(
            tx,
            "attempt.stopped",
            store::Target {
                ticket: Some(id),
                attempt: running.attempt,
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
    target: &str,
    proof_digest: Option<&str>,
    over_limit: bool,
) -> Result<(attempts::Attempt, attempts::Attention), Fail> {
    let attempt = attempts::live_for(tx, id)?
        .ok_or_else(|| Fail::refused(format!("{} has no live attempt", ticket_name(id))))?;
    if attempt.target_branch.as_deref() != Some(target) {
        return Err(Fail::stale(
            "the approval target changed",
            json!(target),
            json!(attempt.target_branch),
        ));
    }
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

/// A limit approval names selected passing or accepted candidate gates and
/// the blocking review checks it overrides. Eligibility and identity still
/// apply to every named check.
fn limit_checks(
    tx: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    loaded: &Loaded,
) -> Result<Vec<i64>, Fail> {
    let (base, head) = attempt.candidate().unwrap_or_default();
    let input = checks::Input {
        plan_revision: attempt.plan_revision(),
        attempt: attempt.id,
        base: base.to_string(),
        head: head.to_string(),
        proof: attempt.proof.clone().unwrap_or_default(),
        ticket_revision: ticket.revision,
        digest: loaded.gate_digest.clone(),
    };
    let mut named = Vec::new();
    let plan = match &attempt.admission {
        Some(admission) => crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?,
        None => loaded.config.workflow_plan(&attempt.workflow)?,
    };
    for gate in loaded.config.gates.iter().filter(|gate| {
        gate.stage == Stage::Candidate
            && plan
                .nodes
                .contains_key(&format!("candidate-gate-{}", gate.name))
    }) {
        if let Some(check) = checks::current(tx, "gate", &gate.name, &input)? {
            named.push(check.id);
        }
    }
    let review = checks::Input {
        digest: loaded.review_digest(&attempt.workflow)?,
        ..input
    };
    let choice = crate::store::selections::current(tx, attempt)?;
    let producer = choice
        .as_ref()
        .and_then(|row| row.result.as_ref())
        .and_then(|result| {
            result["outputs"]["result"]["value"]["candidate"]["source"]["node"].as_str()
        });
    for (_, seat) in loaded
        .config
        .workflow(&attempt.workflow)?
        .review
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            plan.nodes
                .get(&format!("review-{index}"))
                .is_some_and(|node| {
                    matches!(node.runner, crate::plan::Runner::Agent { .. })
                        && node.outputs.get("result") == Some(&crate::plan::Output::Verdict)
                        && (producer.is_none()
                            || node
                                .inputs
                                .get("candidate")
                                .is_some_and(|input| input.node.as_deref() == producer))
                })
        })
    {
        if let Some(check) = checks::current(tx, "review", seat, &review)?
            && (producer.is_some() || check.verdict != "pass")
        {
            named.push(check.id);
        }
    }
    Ok(named)
}

fn candidate_stale(identity: &str, expected: Value, current: Value) -> Fail {
    Fail::stale(
        format!("candidate {identity} changed"),
        expected.clone(),
        current.clone(),
    )
    .with(json!({"identity": identity, "expected": expected, "current": current}))
}

pub async fn candidate_answer(
    daemon: &Daemon,
    project: &Project,
    params: &Value,
    accept: bool,
) -> Result<Value, Fail> {
    let ticket = ticket_param(params)?;
    let attention = params["attention"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("a candidate decision names attention"))?;
    let named_check = params["check"].as_i64();
    let named_plan = params["plan_revision"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("a candidate decision names its plan revision"))?;
    let text = params["text"]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| Fail::invalid("a candidate decision carries nonempty text"))?;
    let item = project.read(|conn| attempts::attention(conn, attention))?;
    let attempt = project
        .read(|conn| attempts::live_for(conn, ticket))?
        .ok_or_else(|| {
            if item.kind == "candidate" && item.reason == "landing" {
                candidate_stale("attempt", json!(item.attempt), Value::Null)
            } else {
                Fail::refused("no live attempt")
            }
        })?;
    let loaded = super::load_attempt(daemon, project, &attempt).await?;
    let omission = item.payload["evidence"]["nodes"].is_array();
    if omission && !accept {
        return Err(Fail::refused(
            "landing omissions have no failed result to repair",
        ));
    }
    if !omission && named_check.is_none() {
        return Err(Fail::invalid("a candidate decision names its check"));
    }
    let landing = if item.kind == "candidate" && item.reason == "landing" {
        Some(item.payload["evidence"]["landing"].clone())
    } else {
        None
    };
    let target_lock = project.target_lock(&crate::git::target_ref(attempt.target_branch()?));
    let _held = if landing.is_some() {
        Some(target_lock.lock().await)
    } else {
        None
    };
    if let Some(identity) = &landing {
        for (key, expected) in [
            ("landing", &identity["execution"]),
            ("child", &item.payload["evidence"]["check"]["execution"]),
            ("base", &identity["base"]),
            ("revision", &identity["revision"]),
            ("old", &identity["prepared"]["old"]),
            ("merged", &identity["prepared"]["merged"]),
            ("node", &item.payload["evidence"]["node"]),
        ] {
            if omission && matches!(key, "child" | "node") {
                continue;
            }
            if &params[key] != expected {
                return Err(candidate_stale(key, params[key].clone(), expected.clone()));
            }
        }
        let canonical = project.canonical_dir();
        let target = identity["prepared"]["target_ref"]
            .as_str()
            .unwrap_or_default();
        for (key, reference, expected) in [
            (
                "old",
                target.to_string(),
                identity["prepared"]["old"].clone(),
            ),
            (
                "merged",
                format!("refs/yard/landings/{}", identity["execution"]),
                identity["prepared"]["merged"].clone(),
            ),
        ] {
            let current = json!(daemon.git.rev_parse(&canonical, &reference).await?);
            if current != expected {
                return Err(candidate_stale(key, expected, current));
            }
        }
        let parent = if omission {
            item.execution
                .ok_or_else(|| Fail::refused("omission has no landing"))?
        } else {
            project
                .read(|conn| {
                    executions::get(
                        conn,
                        item.payload["evidence"]["check"]["execution"]
                            .as_i64()
                            .unwrap_or_default(),
                    )
                })?
                .parent
                .ok_or_else(|| Fail::refused("landing child has no parent"))?
        };
        if omission && params["accept_skipped"] != item.payload["evidence"]["nodes"] {
            return Err(candidate_stale(
                "skipped",
                params["accept_skipped"].clone(),
                item.payload["evidence"]["nodes"].clone(),
            ));
        }
        super::queue::prove_predecessors(daemon, project, parent).await?;
    }
    project.tx(|tx| {
        let item = attempts::attention(tx, attention)?;
        if item.kind != "candidate"
            || item.attempt != Some(attempt.id)
            || item.ticket != Some(ticket)
        {
            return Err(candidate_stale(
                "attention",
                json!(attention),
                json!(item.to_json()),
            ));
        }
        for key in ["target", "head", "proof"] {
            if key == "proof"
                && params[key].is_null()
                && !proof::required(item.payload[key].as_str().unwrap_or_default())
            {
                continue;
            }
            let named = if key == "proof" {
                params[key].as_str().unwrap_or("")
            } else {
                params[key]
                    .as_str()
                    .ok_or_else(|| Fail::invalid(format!("a candidate decision names {key}")))?
            };
            if item.payload[key] != named {
                return Err(candidate_stale(
                    key,
                    json!(named),
                    item.payload[key].clone(),
                ));
            }
        }
        let now = attempts::get(tx, attempt.id)?;
        let ticket = tickets::get(tx, ticket)?;
        if landing.is_some() && now.state != "live" {
            return Err(candidate_stale(
                "attempt",
                json!(attempt.id),
                json!(attempts::live_for(tx, ticket.id)?.map(|current| current.id)),
            ));
        }
        if let Some((key, expected, current)) =
            super::stale_part(&item.payload, &now, &ticket, &loaded)?
        {
            return Err(candidate_stale(key, expected, current));
        }
        if item.state != "open" {
            return Err(candidate_stale(
                "attention",
                json!("open"),
                json!(item.state),
            ));
        }
        if now.plan_revision() != Some(named_plan) {
            return Err(candidate_stale(
                "plan_revision",
                json!(named_plan),
                json!(now.plan_revision()),
            ));
        }
        if !omission && item.payload["evidence"]["check"]["check"] != json!(named_check) {
            return Err(candidate_stale(
                "selected_result",
                json!(named_check),
                item.payload["evidence"]["check"]["check"].clone(),
            ));
        }
        let named = &item.payload["evidence"];
        let check = &named["check"];
        let input = checks::Input {
            attempt: now.id,
            base: now.base.clone(),
            head: now.head.clone().unwrap_or_default(),
            proof: now.proof.clone().unwrap_or_default(),
            ticket_revision: ticket.revision,
            plan_revision: now.plan_revision(),
            digest: if check["kind"] == "review" {
                loaded.review_digest(&now.workflow)?
            } else {
                loaded.gate_digest.clone()
            },
        };
        let current = if let Some(identity) = &landing {
            let original = executions::get(tx, identity["execution"].as_i64().unwrap_or_default())?;
            let approval = checks::approval(tx, original.approval.unwrap_or_default())?;
            if approval.state != "active" {
                return Err(candidate_stale(
                    "approval",
                    json!("active"),
                    json!(approval.state),
                ));
            }
            if omission {
                super::queue::skipped_evidence(tx, &original)?.unwrap_or(Value::Null)
            } else {
                let child = executions::get(tx, check["execution"].as_i64().unwrap_or_default())?;
                super::queue::failed_evidence(tx, &child, &original)?.unwrap_or(Value::Null)
            }
        } else {
            checks::current(
                tx,
                check["kind"].as_str().unwrap_or_default(),
                check["name"].as_str().unwrap_or_default(),
                &input,
            )?
            .map(|check| checks::evidence(tx, &check, &input))
            .transpose()?
            .unwrap_or(Value::Null)
        };
        if &current != named {
            return Err(candidate_stale("selected_result", named.clone(), current));
        }
        attempts::resolve(
            tx,
            &item,
            if accept { "accept" } else { "repair" },
            Some(text),
        )?;
        Ok(json!({"attention": attention, "accepted": accept, "evidence": named}))
    })
}

fn skipped_decision(
    params: &Value,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
) -> Result<Option<Value>, Fail> {
    let nodes = match &params["accept_skipped"] {
        Value::Null => return Ok(None),
        Value::Array(nodes) if nodes.is_empty() => return Ok(None),
        Value::Array(nodes) => nodes,
        _ => {
            return Err(Fail::invalid(
                "accept_skipped names candidate omitted nodes",
            ));
        }
    };
    let text = params["text"]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| Fail::invalid("skipped acceptance carries nonempty text"))?;
    let attention = params["attention"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("skipped acceptance names approval attention"))?;
    let identity = super::skipped_identity(attempt, ticket);
    for key in [
        "target",
        "base",
        "head",
        "proof",
        "revision",
        "plan_revision",
    ] {
        let named = if key == "proof" && params[key].is_null() {
            if proof::required(identity[key].as_str().unwrap_or_default()) {
                json!("")
            } else {
                identity[key].clone()
            }
        } else {
            let valid = if matches!(key, "revision" | "plan_revision") {
                params[key].as_i64().is_some()
            } else {
                params[key].as_str().is_some()
            };
            if !valid {
                return Err(Fail::invalid(format!("skipped acceptance names {key}")));
            }
            params[key].clone()
        };
        if named != identity[key] {
            return Err(candidate_stale(key, named, identity[key].clone()));
        }
    }
    let mut distinct = std::collections::HashSet::new();
    for node in nodes {
        let name = node
            .as_str()
            .ok_or_else(|| Fail::invalid("skipped node is a name"))?;
        if !distinct.insert(name) {
            return Err(Fail::invalid("skipped nodes must be distinct"));
        }
        if !identity["skipped"]
            .as_array()
            .is_some_and(|skipped| skipped.contains(node))
        {
            return Err(candidate_stale(
                "skipped",
                json!(nodes),
                identity["skipped"].clone(),
            ));
        }
    }
    Ok(Some(
        json!({"identity": identity, "attention": attention, "nodes": nodes, "actor": "operator", "text": text}),
    ))
}

fn decision_plan(
    conn: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    params: &Value,
) -> Result<(), Fail> {
    if params["choice"] != json!(attempt.choice) {
        return Err(candidate_stale(
            "choice",
            params["choice"].clone(),
            json!(attempt.choice),
        ));
    }
    if attempt.choice.is_some() && crate::store::selections::current(conn, attempt)?.is_none() {
        return Err(candidate_stale(
            "choice",
            json!(attempt.choice),
            Value::Null,
        ));
    }
    if let Some(named) = params.get("plan_revision").filter(|value| !value.is_null()) {
        let named = named
            .as_i64()
            .ok_or_else(|| Fail::invalid("decision names plan_revision"))?;
        attempts::require_plan(conn, attempt.id, Some(named))?;
    } else if attempt.plan_revision().is_some_and(|revision| revision > 1) {
        return Err(Fail::invalid(
            "a revised candidate decision names plan_revision",
        ));
    }
    Ok(())
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
    let attempt = project
        .read(|conn| attempts::live_for(conn, id))?
        .ok_or_else(|| Fail::refused("no live attempt"))?;
    let loaded = super::load_attempt(daemon, project, &attempt).await?;
    let approval = project.tx(|tx| {
        // Loading admitted policy may yield while the live attempt is replaced.
        let current = match attempts::live_for(tx, id)? {
            Some(current) if current.id == attempt.id => current,
            current => {
                return Err(candidate_stale(
                    "attempt",
                    json!(attempt.id),
                    json!(current.map(|attempt| attempt.id)),
                ));
            }
        };
        let ticket = tickets::get(tx, id)?;
        attempts::require_plan(tx, current.id, attempt.plan_revision())?;
        decision_plan(tx, &current, params)?;
        let accepted_skipped = skipped_decision(params, &current, &ticket)?;
        let (attempt, item) = approval_item(
            tx,
            id,
            head,
            params["target"]
                .as_str()
                .ok_or_else(|| Fail::invalid("a judgment names its target"))?,
            proof_digest,
            true,
        )?;
        if accepted_skipped.is_some() && item.kind != "approval" {
            return Err(Fail::refused(
                "skipped acceptance waits for all selected candidate checks",
            ));
        }
        if let Some(decision) = &accepted_skipped
            && decision["attention"] != item.id
        {
            return Err(candidate_stale(
                "attention",
                decision["attention"].clone(),
                json!(item.id),
            ));
        }
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
        if let Some((key, recorded, current)) =
            super::stale_part(&item.payload, &attempt, &ticket, &loaded)?
        {
            return Err(Fail::stale(
                format!("the item's {key} is no longer current"),
                recorded,
                current,
            ));
        }
        let limit = item.kind == "stopped";
        let overrode = limit
            || item.payload["accepted"]
                .as_array()
                .is_some_and(|accepted| !accepted.is_empty());
        let checks: Vec<i64> = if limit {
            limit_checks(tx, &attempt, &ticket, &loaded)?
        } else {
            item.payload["checks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_i64)
                .collect()
        };
        if let Some((expected, current)) =
            super::selected_checks_current(tx, &attempt, &ticket, &loaded, &checks)?
        {
            return Err(Fail::stale(
                "approval selected_result changed",
                expected,
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
                accepted_skipped: accepted_skipped.as_ref(),
                text: text.as_deref(),
                overrode,
            },
        )?;
        if !params["observe_landing_skipped"].is_null()
            && !params["observe_landing_skipped"].is_array()
        {
            return Err(Fail::invalid(
                "observe_landing_skipped names omitted landing nodes",
            ));
        }
        if let Some(nodes) = params["observe_landing_skipped"]
            .as_array()
            .filter(|nodes| !nodes.is_empty())
        {
            let skipped = super::landing_skipped(&attempt);
            let mut distinct = std::collections::HashSet::new();
            for node in nodes {
                let name = node
                    .as_str()
                    .ok_or_else(|| Fail::invalid("skipped node is a name"))?;
                if !distinct.insert(name) {
                    return Err(Fail::invalid("skipped nodes must be distinct"));
                }
                if !skipped.contains(node) {
                    return Err(candidate_stale("skipped", json!(nodes), json!(skipped)));
                }
            }
            if item.kind != "approval" {
                return Err(Fail::refused(
                    "landing omission observation waits for candidate checks",
                ));
            }
            let mut payload = item.payload.clone();
            payload["landing_skipped_request"] = json!(nodes);
            payload["approval"] = json!(approval);
            tx.execute(
                "UPDATE attention SET payload = ?2 WHERE id = ?1",
                rusqlite::params![item.id, payload.to_string()],
            )?;
        }
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
        let current =
            attempts::live_for(tx, id)?.ok_or_else(|| Fail::refused("no live attempt"))?;
        decision_plan(tx, &current, params)?;
        let (attempt, item) = approval_item(
            tx,
            id,
            head,
            params["target"]
                .as_str()
                .ok_or_else(|| Fail::invalid("a judgment names its target"))?,
            proof_digest,
            false,
        )?;
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
        super::audit_attempt(
            tx,
            "attempt.ended",
            &attempt,
            None,
            json!({ "outcome": "abandoned" }),
        )?;
        Ok(executions::attempt_work(tx, attempt.id)?
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
            .map(|row| execution_inspection(project, row))
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
            "plan_revisions": attempts::revisions(conn, attempt.id)?,
            "candidates": executions.iter().filter_map(|row| row["result"]["outputs"]["result"].as_object()
                .filter(|output| output.get("kind").and_then(Value::as_str) == Some("candidate"))
                .map(|output| output["value"].clone())).collect::<Vec<_>>(),
            "executions": executions,
            "checks": checks_json,
            "approval": checks::active_for(conn, attempt.id)?.map(|approval| json!({
                "approval": approval.id, "head": approval.head, "actor": approval.actor,
                "choice": approval.choice,
                "accepted_skipped": approval.accepted_skipped,
            })),
            "attention": attention,
        }))
    })
}

/// Explicit drilldowns share the authoritative execution usage metadata.
pub fn execution_inspection(project: &Project, row: &executions::Execution) -> Value {
    let mut value = row.to_json();
    let transcript = super::supervise::transcript(project, row);
    let proof = row
        .attempt
        .zip(row.proof.as_deref())
        .map(|(attempt, digest)| super::proof::snapshot_path(project, attempt, digest));
    let gate_log = row.attempt.map(|attempt| {
        let dir = row.parent.map_or_else(
            || project.attempt_dir(attempt),
            |parent| super::queue::landing_dir(project, parent),
        );
        dir.join("gates").join(format!("{}.log", row.id))
    });
    value["artifacts"] = json!({
        "transcript": if transcript.is_file() { "available" } else { "unavailable" },
        "proof": if proof.is_some_and(|path| path.is_dir()) { "available" } else { "unavailable" },
        "gate_log": if row.kind == "gate" && gate_log.is_some_and(|path| path.is_file()) { "available" } else { "unavailable" },
    });
    value
}

pub fn project_stats(project: &Project) -> Result<Value, Fail> {
    project.read(|conn| {
        let rows = executions::query(conn, "1 ORDER BY id", [])?;
        Ok(json!({"aggregate": executions::aggregate(&rows)?,
            "trials": store::trials::statistics(conn, &rows)?,
            "executions": rows.iter().map(|row| execution_inspection(project, row)).collect::<Vec<_>>()}))
    })
}

pub fn attempt_stats(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = ticket_param(params)?;
    project.read(|conn| {
        let attempt = if let Some(attempt) = params["attempt"].as_i64() {
            let attempt = attempts::get(conn, attempt)?;
            if attempt.ticket != id {
                return Err(Fail::invalid("attempt belongs to another ticket"));
            }
            attempt
        } else {
            attempts::latest_for(conn, id)?
                .ok_or_else(|| Fail::not_found(format!("{} has no attempt", ticket_name(id))))?
        };
        let rows = executions::for_attempt(conn, attempt.id)?;
        Ok(json!({"attempt": attempt.id, "aggregate": executions::aggregate(&rows)?,
            "trials": store::trials::statistics(conn, &rows)?,
            "executions": rows.iter().map(|row| execution_inspection(project, row)).collect::<Vec<_>>()}))
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
        .read(|conn| executions::attempt_work(conn, attempt.id))?
        .into_iter()
        .rfind(|row| row.kind == "implementation" || row.kind == "review")
        .ok_or_else(|| Fail::not_found("no worker has run"))?;
    let path = super::supervise::transcript(project, &execution);
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
    let needs_config = accept
        && project.read(|conn| {
            let item = attempts::attention(conn, id)?;
            Ok(item.kind == "proposal" && item.state == "open" && item.reason == "ticket")
        })?;
    let _canonical = if needs_config {
        Some(project.canonical.lock().await)
    } else {
        None
    };
    let loaded = if needs_config {
        Some(load(daemon, project).await?)
    } else {
        None
    };
    let proposing_config = if needs_config {
        let attempt = project.read(|conn| {
            attempts::attention(conn, id)?
                .attempt
                .map(|id| attempts::get(conn, id))
                .transpose()
        })?;
        match attempt {
            // Ended pre-target planners have open proposals but no branch identity.
            Some(attempt) if attempt.target_branch.is_none() && attempt.state != "live" => {
                loaded.clone()
            }
            Some(attempt) => Some(super::load_attempt(daemon, project, &attempt).await?),
            None => None,
        }
    } else {
        None
    };
    let mut stops = Vec::new();
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
                let loaded = loaded
                    .as_ref()
                    .expect("ticket acceptance reads configuration");
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
                        target_branch: None,
                    },
                )?;
                let planning = item
                    .attempt
                    .map(|attempt| attempts::get(tx, attempt))
                    .transpose()?
                    .is_some_and(|attempt| {
                        proposing_config
                            .as_ref()
                            .expect("a proposing attempt reads its target configuration")
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
                stops = edit(
                    tx,
                    proposer,
                    revision,
                    field("title"),
                    field("body"),
                    None,
                    None,
                )?;
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
    stop_executions(daemon, project, &stops);
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
