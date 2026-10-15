//! Admission, the execution state machine, the merge queue and
//! reconcile-on-start. `step` is the scheduler tick: it reads rows, decides
//! the next executions, records each before anything external runs, and
//! spawns it.

pub mod admit;
pub mod capacity;
pub mod classify;
pub mod cleanup;
pub mod proof;
pub mod queue;
pub mod reconcile;
pub mod report;
pub mod review;
pub mod scoped;
pub mod selection;
pub mod supervise;
pub mod writers;

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
    pub staging_commit: String,
    pub image_commit: String,
    pub gate_digest: String,
    pub admission_inputs: Value,
}

#[derive(Default)]
pub struct Configurations {
    pub targets: HashMap<String, Arc<Loaded>>,
    pub default: Option<String>,
    pub sources: HashMap<(String, String, String), Arc<Loaded>>,
}

/// Record at the launch boundary, from the execution's revision and prepared bytes.
/// Launch data must contain no credentials or fingerprints of credentials.
pub async fn verification(
    daemon: &Daemon,
    project: &Project,
    execution: i64,
    checkout: &std::path::Path,
    launch: Value,
    runtime: Value,
) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::for_execution(conn, &row))?;
    let tree = daemon.git.root_tree(checkout, "HEAD").await?;
    let base_tree = match &row.base {
        Some(base) => Some(daemon.git.root_tree(&project.canonical_dir(), base).await?),
        None => None,
    };
    let policy = attempt.admission.as_ref().and_then(|admission| {
        let plan = crate::plan::Plan::admitted(admission).ok()?;
        let node = classify::effective_node(&plan, &row).ok()?;
        Some(
            json!({"node": node, "config_blob": admission["inputs"]["config_blob"],
            "definition": executions::consumed_definition(
            admission, &node.runner, row.node.as_deref().unwrap_or_default(), row.kind == "review")
            .unwrap_or_else(|| json!({"status": "unknown"}))}),
        )
    });
    let image: Option<String> = project.read(|conn| {
        Ok(conn.query_row(
            "SELECT image_id FROM execution WHERE id = ?1",
            [execution],
            |row| row.get(0),
        )?)
    })?;
    let mut evidence = json!({
        "status": "recorded", "reuse": "disabled",
        "candidate": {"tree": tree, "base_tree": base_tree,
            "provenance": {"base": row.base, "head": row.head}, "proof": row.proof},
        "inputs": row.inputs.clone().unwrap_or_else(|| json!({"status": "unknown"})),
        "policy": policy.unwrap_or_else(|| json!({"status": "unknown"})),
        "launch": launch, "runtime": runtime,
        "image": image.map_or_else(|| json!({"status": "unknown"}),
            |id| json!({"status": "observed", "id": id})),
        "tools": {"yard": {"status": "observed", "sha256": admit::runner_identity()?},
            "executed": {"status": "unknown"}},
        "external_services": {"status": "unknown"},
    });
    let definition = &evidence["policy"]["definition"];
    let selected = definition["agent"]["settings"]["judgment_inputs"].clone();
    let read_only = evidence["policy"]["node"]["access"] == "read-only";
    if checks::supports_judgment_reference(&attempt, &row)
        && read_only
        && selected["configuration"]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty())
        && selected["model"]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty())
        && definition["agent"]["settings"]["login"].is_null()
        && evidence["runtime"]["egress"]["allow"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        evidence["runtime"]["supported"] = daemon
            .pinfold
            .runtime_identity()
            .await
            .unwrap_or_else(|| json!({"status": "unknown"}));
        evidence["runtime"]["environment"]["status"] = json!("declared");
        evidence["runtime"]["environment"]["definition"] = json!("yard-selected-judgment-v1");
        evidence["runtime"]["environment"]["configuration"] = selected["configuration"].clone();
        if evidence["runtime"]["supported"]["version"].is_string()
            && evidence["image"]["status"] == "observed"
            && evidence["policy"]["definition"].is_object()
            && row
                .inputs
                .as_ref()
                .and_then(Value::as_object)
                .is_some_and(|inputs| {
                    inputs.len() == 1
                        && inputs.values().all(|input| {
                            input["kind"] == "candidate"
                                && input["source"]["execution"].is_i64()
                                && ["base", "head", "proof"]
                                    .iter()
                                    .all(|key| input["value"][key].is_string())
                        })
                })
        {
            let mut inputs = row.inputs.clone().unwrap_or_default();
            for input in inputs.as_object_mut().expect("typed inputs").values_mut() {
                if input["kind"] == "candidate" {
                    let head = input["value"]["head"]
                        .as_str()
                        .ok_or_else(|| Fail::refused("unknown candidate identity"))?;
                    input["tree"] =
                        json!(daemon.git.root_tree(&project.canonical_dir(), head).await?);
                }
            }
            evidence["identity"] = json!({"declaration": selected,
                "candidate": {"tree": evidence["candidate"]["tree"], "base": row.base,
                    "base_tree": evidence["candidate"]["base_tree"], "proof": row.proof},
                "inputs": inputs, "definition": evidence["policy"]["definition"],
                "launch": evidence["launch"], "image": evidence["image"],
                "runtime": evidence["runtime"], "yard": evidence["tools"]["yard"],
                "ticket_revision": row.ticket_revision});
            evidence["reuse"] = json!("eligible");
            evidence["external_services"] =
                json!({"status": "declared", "model": selected["model"]});
            evidence["tools"]["executed"] = json!({"status": "declared", "source": "observed-image", "contract": "pinfold-box-v1"});
        }
    }
    let definition = &evidence["policy"]["definition"];
    let declaration = definition["gate"]["closed_inputs"].as_str();
    if row.kind == "gate"
        && declaration.is_some_and(|identity| !identity.trim().is_empty())
        && definition["gate"]["runs_in"] == "box"
        && evidence["runtime"]["environment"]["status"] == "declared"
        && evidence["runtime"]["supported"]["version"].is_string()
        && evidence["image"]["status"] == "observed"
        && row
            .inputs
            .as_ref()
            .and_then(Value::as_object)
            .is_some_and(|inputs| {
                !inputs.is_empty()
                    && inputs.values().all(|input| {
                        input["source"]["execution"].is_i64()
                            && input["source"]["status"] != "unknown"
                    })
            })
    {
        let mut definition = definition.clone();
        definition
            .as_object_mut()
            .expect("recorded definition")
            .remove("runner");
        definition["gate"]
            .as_object_mut()
            .expect("recorded gate")
            .remove("name");
        definition["gate"]
            .as_object_mut()
            .expect("recorded gate")
            .remove("stage");
        // The base commit is also the declared YARD_BASE source. Tree equality
        // cannot replace it. Per-launch handles remain outside semantic inputs.
        evidence["identity"] = json!({
            "declaration": declaration,
            "candidate": {"tree": evidence["candidate"]["tree"],
                "base": row.base, "base_tree": evidence["candidate"]["base_tree"], "proof": row.proof},
            "inputs": row.inputs, "definition": definition,
            "launch": evidence["launch"], "image": evidence["image"],
            "runtime": evidence["runtime"], "yard": evidence["tools"]["yard"]});
        evidence["reuse"] = json!("eligible");
        evidence["tools"]["executed"] = json!({"status": "declared",
            "source": "observed-image", "contract": "pinfold-box-v1"});
        evidence["external_services"] = json!({"status": "none"});
    }
    evidence["reference"] = project
        .read(|conn| checks::reusable(conn, &row, &evidence))?
        .unwrap_or(Value::Null);
    project.read(|conn| executions::verification(conn, execution, &evidence))
}

/// The bounded handoff judges immutable publications before cleanup and after repair.
async fn cleanup_audits(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
    plan: &crate::plan::Plan,
) -> Result<bool, Fail> {
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    for (name, node) in plan.cleanup_audits() {
        let ready = project.tx(|tx| {
            let now = attempts::get(tx, attempt.id)?;
            if now.state != "live"
                || now.plan_revision() != attempt.plan_revision()
                || now.candidate() != attempt.candidate()
                || now.proof != attempt.proof
                || tickets::get(tx, ticket.id)?.revision != ticket.revision
            {
                return Ok((false, None));
            }
            let start = executions::Start {
                attempt: Some(now.id),
                kind: "implementation",
                node: Some(name),
                ticket_revision: Some(ticket.revision),
                ..Default::default()
            };
            let Some((_, inputs)) = executions::binding(tx, &now, &start)? else {
                return Ok((false, None));
            };
            let rows = executions::attempt_work(tx, now.id)?;
            let publication = rows
                .iter()
                .rev()
                .find(|row| {
                    row.result_eligible
                        && row.result.as_ref().is_some_and(|result| {
                            result["outputs"]["result"]["kind"] == "candidate"
                        })
                })
                .map(|row| row.id)
                .unwrap_or(0);
            let previous = rows.iter().rev().find(|row| {
                row.node.as_deref() == Some(name)
                    && row.plan_revision == now.plan_revision()
                    && row.ticket_revision == Some(ticket.revision)
            });
            if let Some(previous) = previous
                && previous.id > publication
                && previous.inputs.as_ref() == Some(&inputs)
            {
                if previous.result_eligible || checks::referenced_pass(tx, previous)? {
                    return Ok((true, None));
                }
                let retry: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM attention WHERE execution = ?1
                     AND state = 'resolved' AND resolution = 'start')",
                    [previous.id],
                    |row| row.get(0),
                )?;
                if !retry {
                    return Ok((false, None));
                }
            }
            if rows
                .iter()
                .any(|row| row.status == "running" || row.reserved)
            {
                return Ok((false, None));
            }
            let candidate = &inputs["candidate"]["value"];
            let agent = match &node.runner {
                crate::plan::Runner::Agent { agent, .. }
                | crate::plan::Runner::Packaged { agent, .. } => agent,
                _ => return Err(Fail::refused("invalid cleanup audit")),
            };
            let settings = now.writer_settings(name)?;
            let id = executions::start(
                tx,
                executions::Start {
                    base: candidate["base"].as_str(),
                    head: candidate["head"].as_str(),
                    proof: candidate["proof"].as_str(),
                    name: Some(name),
                    reason: Some("candidate"),
                    ticket: Some(ticket.id),
                    agent: Some((agent, &settings)),
                    harness_version: crate::harness::get(&loaded.config.agents[agent].harness)
                        .map(|h| h.version()),
                    ..start
                },
            )?;
            Ok((false, Some(id)))
        })?;
        if let Some(id) = ready.1 {
            spawn(daemon, project, "implementation", id);
        }
        if !ready.0 {
            return Ok(false);
        }
    }
    Ok(true)
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
    pub async fn inputs(
        &self,
        git: &crate::git::Git,
        canonical: &std::path::Path,
        plan: &crate::plan::Plan,
    ) -> Result<Value, Fail> {
        let mut inputs = self.admission_inputs.clone();
        for (name, node) in &plan.nodes {
            let Some((skills, references)) = node.runner.selected_paths() else {
                continue;
            };
            if skills.is_empty() && references.is_empty() {
                continue;
            }
            let mut sources = Vec::new();
            let mut total = 0;
            let paths: Vec<_> = skills
                .iter()
                .chain(references)
                .map(String::as_str)
                .collect();
            let entries = git
                .ls_tree(canonical, &self.staging_commit, &paths, false)
                .await?;
            for path in skills.iter().chain(references) {
                let entry = entries
                    .iter()
                    .find(|entry| entry.path == *path && entry.regular)
                    .ok_or_else(|| {
                        Fail::refused("invalid_input")
                            .with(json!({"path": path, "reason": "selected_regular_file"}))
                    })?;
                total += entry.size;
                if entry.size
                    > if skills.contains(path) {
                        32768
                    } else {
                        1024 * 1024
                    }
                    || total > 8 * 1024 * 1024
                {
                    return Err(Fail::refused("invalid_input")
                        .with(json!({"path": path, "reason": "selected_input_bound"})));
                }
                sources
                    .push(json!({"path": entry.path, "blob": entry.oid, "executable": entry.exec}));
            }
            if matches!(node.runner, crate::plan::Runner::Packaged { .. }) {
                if plan.packaged().is_some() {
                    inputs["trusted_staged_sources"] = json!(sources);
                } else {
                    if inputs.get("packaged_nodes").is_none() {
                        inputs["packaged_nodes"] = json!({});
                    }
                    inputs["packaged_nodes"][name] = json!({
                        "trusted_staged_sources": sources,
                        "package": crate::pi::package_identity(),
                    });
                }
            } else {
                if inputs.get("selected_agent_sources").is_none() {
                    inputs["selected_agent_sources"] = json!({});
                }
                inputs["selected_agent_sources"][name] = json!(sources);
            }
        }
        Ok(inputs)
    }

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
    read_sources(daemon, repo, branch, default, commit, commit, commit).await
}

async fn read_sources(
    daemon: &Daemon,
    repo: &std::path::Path,
    branch: &str,
    default: Option<&str>,
    commit: &str,
    staging_commit: &str,
    image_commit: &str,
) -> Result<Loaded, Fail> {
    for source in [commit, staging_commit, image_commit] {
        if daemon.git.rev_parse(repo, source).await?.as_deref() != Some(source) {
            return Err(Fail::refused("invalid committed source").with(json!({"source": source})));
        }
    }
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
        .show(repo, image_commit, crate::config::DOCKERFILE_PATH)
        .await?
        .ok_or_else(|| Fail::invalid(format!("{image_commit} has no .yard/Dockerfile")))?;
    let tree = daemon.git.root_tree(repo, image_commit).await?;
    let mut objects = std::collections::BTreeMap::new();
    for (source, path) in [
        (commit, crate::config::CONFIG_PATH),
        (image_commit, crate::config::DOCKERFILE_PATH),
    ] {
        let entries = daemon.git.ls_tree(repo, source, &[path], false).await?;
        let entry = entries
            .iter()
            .find(|entry| entry.path == path && entry.regular)
            .ok_or_else(|| Fail::refused("invalid committed input").with(json!({"path": path})))?;
        objects.insert(path, entry.oid.clone());
    }
    let staged: Vec<Value> = daemon
        .git
        .ls_tree(repo, staging_commit, &crate::agent_env::PATHS, true)
        .await?
        .iter()
        .filter(|entry| crate::agent_env::staged_input(entry))
        .map(|entry| json!({"path": entry.path, "blob": entry.oid, "executable": entry.exec}))
        .collect();
    Ok(Loaded {
        admission_inputs: json!({
            "source_commit": commit,
            "definition_commit": commit,
            "staging_commit": staging_commit,
            "image_commit": image_commit,
            "build_context_tree": tree,
            "config_blob": objects[crate::config::CONFIG_PATH],
            "dockerfile_blob": objects[crate::config::DOCKERFILE_PATH],
            "trusted_staged_sources": staged,
        }),
        gate_digest: config.gate_digest(dockerfile.as_bytes()),
        config,
        branch: branch.to_string(),
        head: commit.to_string(),
        staging_commit: staging_commit.to_string(),
        image_commit: image_commit.to_string(),
    })
}

fn source_commits(inputs: &Value) -> Result<(&str, &str, &str), Fail> {
    let separate = ["definition_commit", "staging_commit", "image_commit"]
        .iter()
        .any(|key| inputs.get(key).is_some());
    let source = |key: &str| {
        inputs[if separate { key } else { "source_commit" }]
            .as_str()
            .ok_or_else(|| Fail::refused("missing admitted source"))
    };
    Ok((
        source("definition_commit")?,
        source("staging_commit")?,
        source("image_commit")?,
    ))
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
    let (head, staging_commit, image_commit) = source_commits(&admission["inputs"])?;
    let separate = admission["inputs"].get("definition_commit").is_some();
    let key = (
        head.to_string(),
        staging_commit.to_string(),
        image_commit.to_string(),
    );
    let cached = project
        .loaded
        .lock()
        .expect("loaded lock")
        .sources
        .get(&key)
        .cloned();
    let loaded = match cached {
        Some(loaded) => loaded,
        None => {
            let loaded = Arc::new(
                read_sources(
                    daemon,
                    &project.canonical_dir(),
                    attempt.target_branch()?,
                    None,
                    head,
                    staging_commit,
                    image_commit,
                )
                .await?,
            );
            project
                .loaded
                .lock()
                .expect("loaded lock")
                .sources
                .insert(key, loaded.clone());
            loaded
        }
    };

    let admitted =
        crate::plan::Plan::admitted(admission).map_err(|error| Fail::refused(error.to_string()))?;
    let expected = if matches!(
        admission["runtime"].as_str(),
        Some("ordinary-linear" | "packaged-audit" | "read-only-graph" | "writer-graph")
    ) {
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
    let mut recorded_inputs = admission["inputs"].clone();
    if admitted
        .nodes
        .values()
        .any(|node| matches!(node.runner, crate::plan::Runner::Packaged { .. }))
    {
        if admission.get("package_checkpoint").is_some()
            && admission["package_checkpoint"] != "exclusive-v1"
        {
            return Err(Fail::refused("invalid admitted checkpoint contract"));
        }
        if admission.get("package_launch").is_some()
            && admission["package_launch"] != "effort"
            && !(admitted.packaged_cleanup().is_some()
                && admission["package_launch"] == "source-write-v1")
        {
            return Err(Fail::refused("invalid admitted package launch contract"));
        }
        for (name, node) in &admitted.nodes {
            if matches!(node.runner, crate::plan::Runner::Packaged { .. })
                && !crate::pi::valid_package_identity(
                    &crate::pi::package_inputs(admission, name)["package"],
                )
            {
                return Err(Fail::refused("invalid admitted package identity"));
            }
        }
        if admitted.packaged().is_some() {
            recorded_inputs
                .as_object_mut()
                .expect("admitted inputs")
                .remove("package");
        }
    }
    let mut expected_inputs = loaded
        .inputs(&daemon.git, &project.canonical_dir(), &admitted)
        .await?;
    for (name, node) in &admitted.nodes {
        if matches!(node.runner, crate::plan::Runner::Packaged { .. })
            && admitted.packaged().is_none()
        {
            expected_inputs["packaged_nodes"][name]["package"] =
                crate::pi::package_inputs(admission, name)["package"].clone();
        }
    }
    if !separate {
        for key in ["definition_commit", "staging_commit", "image_commit"] {
            expected_inputs
                .as_object_mut()
                .expect("source inputs")
                .remove(key);
        }
    }
    if expected_inputs != recorded_inputs
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
    if matches!(kind.as_str(), "implementation" | "command" | "report") {
        daemon.stops.lock().expect("stops lock").insert(
            (project.key.clone(), execution),
            Arc::new(tokio::sync::Notify::new()),
        );
    }
    tokio::spawn(async move {
        let result = if matches!(kind.as_str(), "command" | "report") {
            scoped::execute(&daemon, &project, execution).await
        } else if kind == "landing" {
            // Keep native landing storage out of this large dispatch frame (Y-73).
            Box::pin(queue::land(&daemon, &project, execution)).await
        } else {
            capacity::effect(&daemon, &project, execution, async {
                match kind.as_str() {
                    "implementation" => supervise::implement(&daemon, &project, execution).await,
                    "gate" => supervise::gate(&daemon, &project, execution).await,
                    "review" => {
                        let row = project.read(|conn| executions::get(conn, execution))?;
                        if row.reason.as_deref() == Some("classification") {
                            classify::run(&daemon, &project, execution).await
                        } else {
                            review::run(&daemon, &project, execution).await
                        }
                    }
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
                    if row.scope.is_some() {
                        return executions::end(
                            tx,
                            execution,
                            executions::End {
                                outcome: "error",
                                detail: Some(&fail.message),
                                ..Default::default()
                            },
                        );
                    }
                    let ended = row.kind != "cleanup"
                        && attempts::get(tx, row.required_attempt()?)?
                            .outcome
                            .as_deref()
                            == Some("abandoned");
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: if ended { "abandoned" } else { "error" },
                            detail: Some(&fail.message),
                            ..Default::default()
                        },
                    )?;
                    let attempt = attempts::get(tx, row.required_attempt()?)?;
                    // A gate or seat that read a pre-edit candidate records nothing.
                    let edited = row.parent.is_none()
                        && matches!(row.kind.as_str(), "gate" | "review")
                        && (executions::plan_superseded(tx, &attempt, &row)?
                            || row.ticket_revision.is_some_and(|revision| {
                                attempts::superseded_by_edit(
                                    tx,
                                    attempt.id,
                                    attempt.ticket,
                                    revision,
                                )
                                .unwrap_or(false)
                            }));
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
                                attempt: (row.kind != "landing").then_some(row.attempt).flatten(),
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
    let current = attempts::get(conn, attempt.id)?;
    if current.state != "live" || current.plan_revision() != attempt.plan_revision() {
        return Ok(None);
    }
    if attempt.admission.is_none() && loaded.config.workflow(&attempt.workflow)?.read_only {
        return Ok(None);
    }
    let executions = executions::attempt_work(conn, attempt.id)?;
    let edited = attempts::edit_pending(conn, attempt.id)?;
    let concurrent = attempt
        .admission
        .as_ref()
        .and_then(|admission| crate::plan::Plan::admitted(admission).ok())
        .filter(|plan| plan.concurrent());
    if executions.iter().any(|row| {
        (row.status == "running" || row.reserved)
            && !concurrent.as_ref().is_some_and(|plan| {
                !edited
                    && attempt.next.is_none()
                    && row.parent.is_none()
                    && row.plan_revision == attempt.plan_revision()
                    && row
                        .node
                        .as_ref()
                        .and_then(|node| plan.nodes.get(node))
                        .is_some_and(|node| node.access == crate::plan::Access::ReadOnly)
            })
    }) {
        return Ok(None);
    }
    let open = attempts::open_for_attempt(conn, attempt.id)?;
    // A pending edit outranks an edit proposal made from the pre-edit text.
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
    if let Some(admission) = &attempt.admission {
        let plan = crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?;
        if plan.writers() && !writers::advance(daemon, project, loaded, attempt, &plan).await? {
            return Ok(());
        }
    }
    let Some(superseded) = project.read(|conn| runnable(conn, loaded, attempt))? else {
        return Ok(());
    };
    if let Some(item) = superseded {
        project.tx(|tx| {
            attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
            attempts::resolve(tx, &item, "superseded", None)
        })?;
    }
    let executions = project.read(|conn| executions::attempt_work(conn, attempt.id))?;
    let edited = project.read(|conn| attempts::edit_pending(conn, attempt.id))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    if let Some(next) = &attempt.next {
        let canonical = if !attempt.lane {
            Some(project.canonical.lock().await)
        } else {
            None
        };
        let default = if let Some(canonical) = &canonical {
            let Ok(default) = refresh_default(daemon, project, canonical).await else {
                return Ok(());
            };
            Some(default)
        } else {
            None
        };
        let execution = {
            let _admission = daemon.admission.lock().expect("admission lock");
            project.tx(|tx| {
                let now = attempts::get(tx, attempt.id)?;
                if now.state != "live"
                    || now.plan_revision() != attempt.plan_revision()
                    || now.choice != attempt.choice
                    || now.candidate() != attempt.candidate()
                    || now.proof != attempt.proof
                    || now.lane != attempt.lane
                    || now.next.as_ref() != Some(next)
                    // A deliberate restart consumes observed edits and handoffs.
                    || tickets::get(tx, ticket.id)?.revision != ticket.revision
                    || attempts::edit_pending(tx, now.id)? != edited
                    || (now.choice.is_some()
                        && crate::store::selections::current(tx, &now)?.is_none())
                    || runnable(tx, loaded, &now)?.is_none()
                {
                    return Ok(None);
                }
                if let Some(default) = &default
                    && !admit::take_lane_in(tx, default, now.id)?
                {
                    return Ok(None);
                }
                let reason = next["reason"].as_str().unwrap_or("repair");
                supervise::start_implementation(tx, &now, reason, next.get("feedback")).map(Some)
            })?
        };
        if let Some(execution) = execution {
            spawn(daemon, project, "implementation", execution);
        }
        return Ok(());
    }
    if !attempt.lane {
        let canonical = project.canonical.lock().await;
        let Ok(default) = refresh_default(daemon, project, &canonical).await else {
            return Ok(());
        };
        if !admit::take_lane(daemon, project, &default, attempt.id)? {
            return Ok(());
        }
    }
    // A pending edit is derived: the ticket is newer than the last implementer's read.
    if let Some(admission) = &attempt.admission {
        let plan = crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?;
        if plan.handoff() {
            let (next, complete) = project.read(|conn| {
                for name in ["implementer", "cleanup", "repair"] {
                    let row = executions.iter().rev().find(|row| {
                        row.node.as_deref() == Some(name)
                            && row.ticket_revision == Some(ticket.revision)
                            && row.plan_revision == attempt.plan_revision()
                    });
                    if row.is_none() {
                        return Ok((Some(name), false));
                    }
                    let row = row.expect("present");
                    if !row.result_eligible {
                        return Ok((None, false));
                    }
                    let bound = executions::binding(
                        conn,
                        attempt,
                        &executions::Start {
                            kind: "implementation",
                            node: Some(name),
                            ticket_revision: Some(ticket.revision),
                            feedback: row
                                .inputs
                                .as_ref()
                                .and_then(|inputs| inputs.get("feedback")),
                            ..Default::default()
                        },
                    )?;
                    if bound.map(|(_, inputs)| inputs) != row.inputs {
                        return Ok((Some(name), false));
                    }
                }
                Ok((None, true))
            })?;
            if let Some(node) = next {
                if node == "cleanup"
                    && !cleanup_audits(daemon, project, loaded, attempt, &plan).await?
                {
                    return Ok(());
                }
                let execution = project.tx(|tx| {
                    attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
                    supervise::start_writer(
                        tx,
                        attempt,
                        node,
                        if edited { "edit" } else { "first" },
                        None,
                    )
                })?;
                spawn(daemon, project, "implementation", execution);
                return Ok(());
            }
            if !complete {
                return Ok(());
            }
            if !cleanup_audits(daemon, project, loaded, attempt, &plan).await? {
                return Ok(());
            }
        }
    }
    if edited {
        let execution = project.tx(|tx| {
            attempts::require_plan(tx, attempt.id, attempt.plan_revision())?;
            supervise::start_implementation(tx, attempt, "edit", None)
        })?;
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
        attempt: Some(attempt.id),
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
    let mut selected = Vec::new();
    let mut accepted = Vec::new();
    let mut blocked = Vec::new();
    let mut chosen_blocked = Vec::new();
    let mut chosen_failures = Vec::new();
    let concurrent = plan.concurrent();
    let active = executions
        .iter()
        .any(|row| row.status == "running" || row.reserved);
    let mut unfinished = false;
    let mut completed = std::collections::BTreeSet::from(["implementer"]);
    if plan.handoff() {
        completed.extend(["cleanup", "repair"]);
    }
    let reviewed = project.read(|conn| candidate_reviewed(conn, attempt, &plan))?;
    if let Some(choice) = project.read(|conn| crate::store::selections::current(conn, attempt))? {
        let value =
            &choice.result.as_ref().expect("choice")["outputs"]["result"]["value"]["candidate"];
        let producer = value["source"]["node"].as_str().expect("candidate source");
        for (name, node) in &plan.nodes {
            if matches!(node.runner, crate::plan::Runner::Agent { .. }) {
                completed.insert(name);
            }
            if !matches!(node.runner, crate::plan::Runner::Agent { .. })
                || node.outputs.get("result") != Some(&crate::plan::Output::Verdict)
                || node
                    .inputs
                    .get("candidate")
                    .is_none_or(|input| input.node.as_deref() != Some(producer))
            {
                continue;
            }
            let row = executions
                .iter()
                .rev()
                .find(|row| row.node.as_deref() == Some(name) && row.result_eligible)
                .ok_or_else(|| Fail::refused("selected candidate has no review"))?;
            let check = project
                .read(|conn| checks::for_execution(conn, row.id))?
                .ok_or_else(|| Fail::refused("selected review has no publication"))?;
            let input = checks::Input {
                digest: review_digest.clone(),
                ..gate_input.clone()
            };
            if project
                .read(|conn| checks::current(conn, "review", &check.name, &input))?
                .map(|check| check.id)
                != Some(check.id)
            {
                return Ok(());
            }
            let evidence = project.read(|conn| checks::evidence(conn, &check, &input))?;
            let decision = project.read(|conn| checks::decision(conn, attempt.id, &evidence))?;
            if check.verdict != "pass" && node.operator_failure && decision.is_none() {
                chosen_failures.push((check.clone(), evidence.clone()));
            }
            if check.verdict != "pass" && decision.as_deref() != Some("accept") {
                chosen_blocked.push(check.clone());
            }
            selected.push(check.id);
            if decision.as_deref() == Some("accept") {
                accepted.push(evidence);
            }
        }
    }
    let routed = attempt
        .admission
        .as_ref()
        .is_some_and(|admission| admission["plan"]["inputs"].get("feedback").is_some());
    // Only the candidate/review portion consumes routes here. The queue owns
    // the external merged input and all effects beyond this boundary.
    for current in plan
        .complete_order()
        .ok_or_else(|| Fail::refused("unsupported admitted complete routes"))?
        .into_iter()
        .filter(|node| {
            *node != "implementer"
                && !(plan.handoff()
                    && (matches!(*node, "cleanup" | "repair")
                        || node.starts_with("cleanup-audit-")))
        })
    {
        let node = &plan.nodes[current];
        if plan.selector().is_some() && matches!(node.runner, crate::plan::Runner::Agent { .. }) {
            continue;
        }
        if node
            .inputs
            .values()
            .any(|input| input.node.is_none() && input.output == "merged")
        {
            continue;
        }
        if plan.nodes.iter().any(|(source, predecessor)| {
            predecessor
                .routes
                .get("complete")
                .is_some_and(|routes| routes.iter().any(|route| route.to == current))
                && !completed.contains(source.as_str())
        }) {
            unfinished = true;
            continue;
        }
        if current == "selector" && plan.finding_selector().is_some() {
            let previous = executions
                .iter()
                .rev()
                .find(|row| row.node.as_deref() == Some(current));
            if let Some(row) = previous {
                if project
                    .read(|conn| crate::store::selections::finding_current(conn, attempt, row))?
                {
                    completed.insert(current);
                    continue;
                }
                if row.status == "running" || row.reserved {
                    unfinished = true;
                    continue;
                }
            }
            let crate::plan::Runner::Agent { agent, .. } = &node.runner else {
                return Err(Fail::refused("invalid selector"));
            };
            let settings = serde_json::to_value(&loaded.config.agents[agent]).expect("settings");
            let execution = project.tx(|tx| {
                let now = attempts::get(tx, attempt.id)?;
                if now.state != "live"
                    || now.plan_revision() != attempt.plan_revision()
                    || now.candidate() != attempt.candidate()
                    || now.proof != attempt.proof
                    || now.next.is_some()
                    || attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)?
                    || executions::attempt_work(tx, attempt.id)?
                        .iter()
                        .rev()
                        .find(|row| row.node.as_deref() == Some(current))
                        .map(|row| row.id)
                        != previous.map(|row| row.id)
                    || crate::store::selections::inputs(tx, &now)?.is_none()
                {
                    return Ok(None);
                }
                executions::start(
                    tx,
                    executions::Start {
                        kind: "review",
                        node: Some(current),
                        name: Some(current),
                        reason: Some("first"),
                        agent: Some((agent, &settings)),
                        harness_version: crate::harness::get(&loaded.config.agents[agent].harness)
                            .map(|h| h.version()),
                        ..shared
                    },
                )
                .map(Some)
            })?;
            if let Some(execution) = execution {
                spawn(daemon, project, "review", execution);
            }
            unfinished = true;
            continue;
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
            crate::plan::Runner::Packaged { .. } => {
                return Err(Fail::refused("unsupported admitted review node"));
            }
        };
        let input = checks::Input {
            digest: digest.to_string(),
            ..gate_input.clone()
        };
        let check = project.read(|conn| checks::current(conn, kind, name, &input))?;
        let Some(check) = check else {
            if executions.iter().any(|row| {
                row.node.as_deref() == Some(current) && (row.status == "running" || row.reserved)
            }) {
                unfinished = true;
                continue;
            }
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
                let now = attempts::get(tx, attempt.id)?;
                if now.state != "live"
                    || now.plan_revision() != attempt.plan_revision()
                    || now.choice != attempt.choice
                    || now.candidate() != attempt.candidate()
                    || now.proof != attempt.proof
                    || now.next.is_some()
                    || attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)?
                    || (now.choice.is_some()
                        && crate::store::selections::current(tx, &now)?.is_none())
                {
                    return Ok(None);
                }
                let start = executions::Start {
                    kind,
                    name: Some(name),
                    digest: Some(digest),
                    reason: Some(if kind == "gate" { "candidate" } else { "first" }),
                    round: (kind == "review").then_some(attempt.rounds + 1),
                    agent: agent.zip(settings.as_ref()),
                    harness_version,
                    ..shared
                };
                if now.admission.is_some() {
                    let Some((_, inputs)) = executions::binding(tx, &now, &start)? else {
                        return Ok(None);
                    };
                    // The gate's checkout and columns must name the same candidate
                    // as its immutable selection input, before any row is created.
                    if plan.selector().is_some()
                        && (inputs["candidate"]["value"]["base"] != json!(start.base)
                            || inputs["candidate"]["value"]["head"] != json!(start.head)
                            || inputs["candidate"]["value"]["proof"]
                                != start.proof.unwrap_or_default())
                    {
                        return Ok(None);
                    }
                }
                executions::start(tx, start).map(Some)
            })?;
            if let Some(execution) = execution {
                spawn(daemon, project, kind, execution);
            }
            if !concurrent {
                return Ok(());
            }
            unfinished = true;
            continue;
        };
        let result = project.read(|conn| executions::get(conn, check.execution))?;
        // Intermediate seats publish their own verdict; the final result also
        // binds the earlier blocks it carries. Acceptance never rewrites it.
        let outcome = if routed {
            result
                .result
                .as_ref()
                .and_then(|result| result["outcome"].as_str())
                .ok_or_else(|| Fail::refused("eligible judgment has no outcome"))?
        } else if kind == "gate" && check.verdict != "pass" {
            "failed"
        } else if node.outcomes.iter().any(|outcome| outcome == "blocked")
            && (check.verdict != "pass" || !blocked.is_empty())
        {
            "blocked"
        } else {
            "complete"
        };
        let evidence = project.read(|conn| checks::evidence(conn, &check, &input))?;
        let decision = project.read(|conn| checks::decision(conn, attempt.id, &evidence))?;
        let failed = check.verdict != "pass" || (outcome == "blocked" && !blocked.is_empty());
        if plan.finding_selector().is_some() && kind == "review" {
            // Selection directs repair after the exact join. Required verdicts
            // and their operator decisions remain independent.
            if !node.advisory {
                selected.push(check.id);
                let carried = result
                    .result
                    .as_ref()
                    .and_then(|result| result["outputs"]["result"]["value"]["blocked"].as_array());
                let carries = |id| carried.is_some_and(|ids| ids.contains(&json!(id)));
                let inherited =
                    outcome == "blocked" && chosen_blocked.iter().any(|check| carries(check.id));
                if node.operator_failure
                    && decision.is_none()
                    && (check.verdict != "pass" || inherited)
                {
                    chosen_failures.push((check.clone(), evidence.clone()));
                }
                if decision.as_deref() == Some("accept") {
                    accepted.push(evidence);
                    if outcome == "blocked" {
                        chosen_blocked.retain(|check| !carries(check.id));
                    }
                } else if check.verdict != "pass" {
                    chosen_blocked.push(check.clone());
                }
            }
            completed.insert(current);
            continue;
        }
        if failed
            && decision.is_none()
            && classify::route(daemon, project, loaded, attempt, &check, &input, &blocked)?
        {
            return Ok(());
        }
        if failed && node.operator_failure && decision.is_none() {
            raise_candidate_failure(project, loaded, attempt, &check, &input, &evidence)?;
            return Ok(());
        }
        selected.push(check.id);
        if decision.as_deref() == Some("accept") {
            accepted.push(evidence);
            if outcome == "blocked" {
                blocked.clear();
            }
        } else if kind == "review" && check.verdict != "pass" {
            blocked.push(check.clone());
        }
        if outcome == "complete"
            || (kind == "gate" && decision.as_deref() == Some("accept"))
            || (kind == "review" && blocked.is_empty())
        {
            completed.insert(current);
            continue;
        }
        // A repair is a writer. Every reader must release before it starts.
        if active || unfinished {
            return Ok(());
        }
        let route = node
            .routes
            .get(outcome)
            .and_then(|routes| routes.first())
            .ok_or_else(|| Fail::refused("judgment has no admitted outcome route"))?;
        if route.to != plan.candidate_node() || route.repeat.is_none() {
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
                // The failed check belongs to the observed choice, not whichever
                // producer a concurrent operator selection would dispatch now.
                let now = attempts::get(tx, attempt.id)?;
                if now.state != "live"
                    || now.plan_revision() != attempt.plan_revision()
                    || now.choice != attempt.choice
                    || now.candidate() != attempt.candidate()
                    || now.proof != attempt.proof
                    || now.next.is_some()
                    || attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)?
                    || (now.choice.is_some()
                        && crate::store::selections::current(tx, &now)?.is_none())
                    || checks::current(tx, kind, name, &input)?
                        .filter(|current| current.verdict != "pass")
                        .map(|current| (current.id, current.execution))
                        != Some((check.id, check.execution))
                    || checks::decision(tx, attempt.id, &checks::evidence(tx, &check, &input)?)?
                        != decision
                {
                    return Ok(None);
                }
                supervise::start_implementation(tx, &now, "repair", Some(&feedback)).map(Some)
            })?
        };
        if let Some(execution) = execution {
            spawn(daemon, project, "implementation", execution);
        }
        return Ok(());
    }
    if unfinished || active {
        return Ok(());
    }
    if !chosen_blocked.is_empty() && plan.finding_selector().is_some() {
        for name in plan.complete_order().unwrap_or_default().into_iter().rev() {
            if !plan.classifiers.contains_key(name) {
                continue;
            }
            let Some(index) = name
                .strip_prefix("review-")
                .and_then(|index| index.parse::<usize>().ok())
            else {
                continue;
            };
            let input = checks::Input {
                digest: review_digest.clone(),
                ..gate_input.clone()
            };
            if let Some(check) = project
                .read(|conn| checks::current(conn, "review", &workflow.review[index], &input))?
                && project.read(|conn| {
                    let result = executions::get(conn, check.execution)?;
                    let carried = result.result.as_ref().and_then(|result| {
                        result["outputs"]["result"]["value"]["blocked"].as_array()
                    });
                    Ok(chosen_blocked.iter().any(|blocked| blocked.id == check.id)
                        || carried.is_some_and(|ids| {
                            chosen_blocked
                                .iter()
                                .any(|blocked| ids.contains(&json!(blocked.id)))
                        }))
                })?
                && classify::route(
                    daemon,
                    project,
                    loaded,
                    attempt,
                    &check,
                    &input,
                    &chosen_blocked,
                )?
            {
                return Ok(());
            }
        }
    }
    if let Some((check, evidence)) = chosen_failures.first() {
        let input = checks::Input {
            digest: review_digest.clone(),
            ..gate_input
        };
        return raise_candidate_failure(project, loaded, attempt, check, &input, evidence);
    }
    if let Some(trigger) = chosen_blocked.last() {
        review::blocked(
            project,
            loaded,
            attempt,
            &ticket,
            &chosen_blocked,
            loaded.config.review.max_rounds,
            trigger,
        )?
        .inspect(|id| spawn(daemon, project, "implementation", *id));
        return Ok(());
    }

    let target_lock = project.target_lock(&crate::git::target_ref(attempt.target_branch()?));
    let _target = target_lock.lock().await;
    let policy = load_target(daemon, project, attempt.target_branch()?).await?;
    let protected = protected_paths(daemon, project, &policy, base, head).await?;
    let payload = json!({
        "target": attempt.target_branch, "choice": attempt.choice, "base": base, "head": head, "proof": attempt.proof.clone().unwrap_or_default(),
        "revision": ticket.revision, "plan_revision": attempt.plan_revision(),
        "gate_digest": loaded.gate_digest, "review_digest": review_digest,
        "checks": selected, "accepted": accepted, "protected": protected,
        "unreviewed": !reviewed,
        "skipped": attempt.admission.as_ref().map(|admission| &admission["skipped"]),
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
                    checks: &selected,
                    actor: "auto",
                    accepted_skipped: None,
                    text: None,
                    overrode: !accepted.is_empty(),
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
        ("choice", json!(attempt.choice)),
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

pub fn landing_skipped(attempt: &attempts::Attempt) -> Vec<Value> {
    attempt
        .admission
        .as_ref()
        .and_then(|a| a["skipped"].as_array())
        .into_iter()
        .flatten()
        .filter(|node| {
            node.as_str()
                .is_some_and(|n| n.starts_with("landing-gate-"))
        })
        .cloned()
        .collect()
}

pub fn skipped_identity(attempt: &attempts::Attempt, ticket: &tickets::Ticket) -> Value {
    let skipped: Vec<_> = attempt
        .admission
        .as_ref()
        .and_then(|admission| admission["skipped"].as_array())
        .into_iter()
        .flatten()
        .filter(|node| {
            node.as_str().is_some_and(|name| {
                name.starts_with("candidate-gate-") || name.starts_with("review-")
            })
        })
        .cloned()
        .collect();
    json!({
        "attempt": attempt.id, "target": attempt.target_branch, "base": attempt.base,
        "head": attempt.head, "proof": attempt.proof.clone().unwrap_or_default(),
        "revision": ticket.revision, "plan_revision": attempt.plan_revision(),
        "skipped": skipped
    })
}

fn raise_candidate_failure(
    project: &Project,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
    check: &checks::Check,
    input: &checks::Input,
    evidence: &Value,
) -> Result<(), Fail> {
    project.tx(|tx| {
        let now = attempts::get(tx, attempt.id)?;
        let ticket = tickets::get(tx, attempt.ticket)?;
        let mut payload = json!({
            "actor": "operator", "target": attempt.target_branch, "choice": attempt.choice,
            "base": input.base, "head": input.head, "proof": input.proof,
            "revision": input.ticket_revision, "plan_revision": attempt.plan_revision(),
            "gate_digest": loaded.gate_digest, "review_digest": loaded.review_digest(&attempt.workflow)?,
            "evidence": evidence,
        });
        let classification = classify::attention(tx, &now, evidence)?;
        if !classification.is_null() {
            payload["classification"] = classification;
        }
        if now.state != "live"
            || stale_part(&payload, &now, &ticket, loaded)?.is_some()
            || checks::current(tx, &check.kind, &check.name, input)?.map(|check| check.id)
                != Some(check.id)
        {
            return Ok(());
        }
        attempts::raise(
            tx,
            attempts::Raise {
                kind: "candidate",
                reason: if check.kind == "gate" { "failed" } else { "blocked" },
                ticket: Some(ticket.id),
                attempt: Some(attempt.id),
                execution: Some(check.execution),
                payload,
                text: None,
            },
        )
        .map(drop)
    })
}

fn candidate_reviewed(
    conn: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    plan: &crate::plan::Plan,
) -> Result<bool, Fail> {
    let choice = crate::store::selections::current(conn, attempt)?;
    let producer = choice
        .as_ref()
        .and_then(|row| row.result.as_ref())
        .and_then(|result| {
            result["outputs"]["result"]["value"]["candidate"]["source"]["node"].as_str()
        });
    Ok(plan.nodes.values().any(|node| {
        matches!(node.runner, crate::plan::Runner::Agent { .. })
            && node.outputs.get("result") == Some(&crate::plan::Output::Verdict)
            && !node.advisory
            && (plan.selector().is_none()
                || producer.is_some_and(|producer| {
                    node.inputs
                        .get("candidate")
                        .is_some_and(|input| input.node.as_deref() == Some(producer))
                }))
    }))
}

/// Re-read each selected identity at approval and every queue boundary.
pub fn selected_checks_current(
    conn: &rusqlite::Connection,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    loaded: &Loaded,
    named: &[i64],
) -> Result<Option<(Value, Value)>, Fail> {
    if attempt.choice.is_some() && crate::store::selections::current(conn, attempt)?.is_none() {
        return Ok(Some((json!(attempt.choice), Value::Null)));
    }
    let input = checks::Input {
        attempt: attempt.id,
        base: attempt.base.clone(),
        head: attempt.head.clone().unwrap_or_default(),
        proof: attempt.proof.clone().unwrap_or_default(),
        ticket_revision: ticket.revision,
        plan_revision: attempt.plan_revision(),
        digest: loaded.gate_digest.clone(),
    };
    let checks = checks::for_attempt(conn, attempt.id)?;
    for id in named {
        let Some(check) = checks.iter().find(|check| check.id == *id) else {
            return Ok(Some((json!(id), Value::Null)));
        };
        let input = checks::Input {
            digest: if check.kind == "review" {
                loaded.review_digest(&attempt.workflow)?
            } else {
                loaded.gate_digest.clone()
            },
            ..input.clone()
        };
        let current = checks::current(conn, &check.kind, &check.name, &input)?;
        if current.as_ref().map(|check| check.id) != Some(*id) {
            return Ok(Some((json!(id), json!(current.map(|check| check.id)))));
        }
        if let Some(expected) = checks::accepted_evidence(conn, attempt.id, *id)? {
            let current = checks::evidence(conn, check, &input)?;
            if expected != current {
                return Ok(Some((expected, current)));
            }
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
        "execution.start" => scoped::start(daemon, &project, params).await,
        "execution.show" => scoped::show(&project, &params),
        "execution.stop" => scoped::stop(daemon, &project, &params),
        "attempt.start" => admit::attempt_start(daemon, &project, &params).await,
        "attempt.revise" => admit::attempt_revise(daemon, &project, &params).await,
        "attempt.select" => selection::operator(&project, &params),
        "attempt.stop" => admit::attempt_stop(daemon, &project, &params),
        "attempt.approve" => admit::attempt_approve(daemon, &project, &params).await,
        "attempt.accept" => admit::candidate_answer(daemon, &project, &params, true).await,
        "attempt.repair" => admit::candidate_answer(daemon, &project, &params, false).await,
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
        let attempt = attempts::get(tx, execution.required_attempt()?)?;
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
        plan.runtime()
    } else {
        plan.unsupported_shape()
    };
    if method == "plan.validate" && workflow.is_none() {
        return Err(Fail::refused("plan shape has no runtime support")
            .with(json!({"shape": shape, "structurally_valid": true, "runnable": false})));
    }
    let selected_inputs = if plan.nodes.values().any(|node| {
        node.runner
            .selected_paths()
            .is_some_and(|(skills, references)| !skills.is_empty() || !references.is_empty())
    }) {
        let mut inputs = loaded
            .inputs(&daemon.git, &project.canonical_dir(), &plan)
            .await?;
        if plan.packaged().is_some() {
            inputs["package"] = crate::pi::package_identity();
        }
        Some(inputs)
    } else {
        None
    };
    Ok(
        json!({"target": branch, "config_head": loaded.head, "plan": plan,
        "structurally_valid": true, "runnable": workflow.is_some(), "shape": shape,
        "workflow": workflow,
        "bindings": {"agents": loaded.config.agents, "gates": loaded.config.gates,
            "selected_inputs": selected_inputs,
            "review": loaded.config.review,
            "workflow_settings": workflow.as_ref().and_then(|name| loaded.config.workflows.get(name))}}),
    )
}
