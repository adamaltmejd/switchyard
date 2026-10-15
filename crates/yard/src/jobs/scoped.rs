//! Operator-created executions have frozen inputs and no candidate authority.

use super::{capacity, supervise};
use crate::api::Fail;
use crate::r#box::{BoxSpec, EnvValue, Mount};
use crate::daemon::{Daemon, Project};
use crate::harness::{Launch, Stage};
use crate::plan::{Access, Graph, NodeSpec, Output, Runner};
use crate::store::{self, Target, attempts, executions};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    project: String,
    target: String,
    input: String,
    node: NodeSpec,
    attempt: Option<i64>,
    timeout_seconds: u64,
}

pub async fn start(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    params: Value,
) -> Result<Value, Fail> {
    let request: Request =
        serde_json::from_value(params).map_err(|error| Fail::invalid(error.to_string()))?;
    if request.project != project.root.to_string_lossy()
        || !(1..=86400).contains(&request.timeout_seconds)
        || request.input.len() != 40
    {
        return Err(Fail::invalid("invalid_scope"));
    }
    let _canonical = project.canonical.lock().await;
    let loaded = super::load_target(daemon, project, &request.target).await?;
    let commit = daemon
        .git
        .rev_parse(
            &project.canonical_dir(),
            &format!("{}^{{commit}}", request.input),
        )
        .await?;
    if commit.as_deref() != Some(&request.input) {
        return Err(Fail::refused("invalid_input").with(json!({"input": request.input})));
    }
    let tree = daemon
        .git
        .root_tree(&project.canonical_dir(), &request.input)
        .await?;
    let plan = loaded
        .config
        .resolve_graph(&Graph {
            inputs: BTreeMap::new(),
            nodes: BTreeMap::from([("standalone".into(), request.node)]),
        })
        .map_err(|error| Fail::invalid(error.to_string()))?;
    let node = &plan.nodes["standalone"];
    if !node.inputs.is_empty()
        || !node.requires.is_empty()
        || !node.routes.is_empty()
        || node.operator_failure
        || node.outcomes != ["complete"]
    {
        return Err(Fail::refused("standalone_authority"));
    }
    if let Runner::Agent {
        skills, references, ..
    } = &node.runner
    {
        for (field, paths) in [("skills", skills), ("references", references)] {
            if !paths.is_empty() {
                return Err(Fail::refused("standalone_selected_inputs").with(json!({
                    "reason": "standalone_selected_inputs",
                    "input": format!("node.runner.{field}"), "selected": paths
                })));
            }
        }
    }
    let (kind, gate, agent, version) = match &node.runner {
        Runner::Command { gate }
            if node.outputs == BTreeMap::from([("result".into(), Output::Verdict)]) =>
        {
            let gate = loaded
                .config
                .gates
                .iter()
                .find(|entry| entry.name == *gate)
                .expect("resolved gate");
            if gate.runs_in != crate::config::RunsIn::Box || !gate.env.is_empty() {
                return Err(Fail::refused("standalone_box"));
            }
            ("command", json!(gate), Value::Null, None)
        }
        Runner::Agent { agent, .. }
            if node.access == Access::ReadOnly
                && node.outputs == BTreeMap::from([("result".into(), Output::Report)]) =>
        {
            let settings = &loaded.config.agents[agent];
            let harness = crate::harness::get(&settings.harness).expect("validated harness");
            (
                "report",
                Value::Null,
                json!(settings),
                Some(harness.version().to_string()),
            )
        }
        _ => return Err(Fail::refused("standalone_authority")),
    };
    let scope = json!({"project": project.root, "target": request.target,
        "input": {"commit": request.input, "tree": tree},
        "policy": {"node": node, "gate": gate, "agent": agent, "harness_version": version,
            "definitions": loaded.admission_inputs, "egress": loaded.config.egress,
            "yard_runner": super::admit::runner_identity()?},
        "timeout_seconds": request.timeout_seconds,
        "deadline_ms": store::now_ms() + (request.timeout_seconds * 1000) as i64});
    let id = project.tx(|tx| {
        if let Some(id) = request.attempt {
            let attempt = attempts::get(tx, id)?;
            if attempt.target_branch()? != request.target {
                return Err(Fail::refused("scope_mismatch")
                    .with(json!({"attempt": id, "target": request.target})));
            }
        }
        let settings = (!agent.is_null()).then_some(("standalone", &agent));
        executions::start(
            tx,
            executions::Start {
                attempt: request.attempt,
                scope: Some(&scope),
                kind,
                base: Some(&request.input),
                head: Some(&request.input),
                agent: settings,
                harness_version: version.as_deref(),
                ..Default::default()
            },
        )
    })?;
    super::spawn(daemon, project, kind, id);
    project.read(|conn| Ok(executions::get(conn, id)?.to_json()))
}

pub fn show(project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = params["execution"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("execution required"))?;
    project.read(|conn| Ok(executions::get(conn, id)?.to_json()))
}

pub fn stop(daemon: &Daemon, project: &Project, params: &Value) -> Result<Value, Fail> {
    let id = params["execution"]
        .as_i64()
        .ok_or_else(|| Fail::invalid("execution required"))?;
    let stop = daemon
        .stops
        .lock()
        .expect("stops lock")
        .get(&(project.key.clone(), id))
        .cloned();
    project.tx(|tx| {
        let row = executions::get(tx, id)?;
        if row.scope.is_none() || row.status != "running" {
            return Err(Fail::refused("execution_ended").with(json!({"execution": id})));
        }
        if !row.reserved {
            executions::end(
                tx,
                id,
                executions::End {
                    outcome: "stopped",
                    ..Default::default()
                },
            )?;
        } else if stop.is_none() {
            return Err(Fail::refused("ownership").with(json!({"execution": id})));
        }
        store::audit(
            tx,
            "execution.stopped",
            Target {
                execution: Some(id),
                attempt: row.attempt,
                ..Default::default()
            },
            None,
            json!({}),
        )?;
        Ok(())
    })?;
    if let Some(stop) = stop {
        stop.notify_one();
    }
    daemon.wake.notify_one();
    Ok(json!({"stopping": id}))
}

/// Cancellation joins normal box teardown, then uses the shared owner probe.
/// A restart never replays this execution or resets its admitted lifetime.
pub async fn execute(daemon: &Arc<Daemon>, project: &Arc<Project>, id: i64) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, id))?;
    let scope = row
        .scope
        .as_ref()
        .ok_or_else(|| Fail::refused("missing_scope"))?;
    let remaining = (scope["deadline_ms"]
        .as_i64()
        .ok_or_else(|| Fail::refused("missing_scope"))?
        - store::now_ms())
    .max(0) as u64;
    let stop = daemon
        .stops
        .lock()
        .expect("stops lock")
        .get(&(project.key.clone(), id))
        .cloned()
        .expect("spawn stop handle");
    let deadline = tokio::time::Instant::now() + Duration::from_millis(remaining);
    let acquired = tokio::select! {
        biased;
        _ = stop.notified() => Err(Fail::new("stopped", "operator stopped execution")),
        _ = tokio::time::sleep_until(deadline) => Err(Fail::new("timeout", "original execution lifetime expired")),
        result = capacity::acquire(daemon, project, id) => result,
    };
    let result = match acquired {
        Ok(lock) => {
            let result = capacity::HOST_LOCK.scope(lock.as_raw_fd(), async {
                let result = tokio::select! {
                    biased;
                    _ = stop.notified() => Err(Fail::new("stopped", "operator stopped execution")),
                    _ = tokio::time::sleep_until(deadline) => Err(Fail::new("timeout", "original execution lifetime expired")),
                    result = work(daemon, project, &row, deadline) => result,
                };
                if result.is_err() {
                    daemon.grants.revoke(project, id);
                    // Cancellation can drop a launch before it returns a handle.
                    // Its teardown child must inherit this same execution lock.
                    let _ = daemon.pinfold.down(&supervise::box_name(project, id)).await;
                }
                result
            }).await;
            drop(lock);
            result
        }
        Err(fail) => Err(fail),
    };
    daemon
        .activity
        .lock()
        .expect("activity lock")
        .remove(&(project.key.clone(), id));
    daemon.grants.revoke(project, id);
    if let Err(fail) = result {
        project.tx(|tx| {
            let row = executions::get(tx, id)?;
            if row.status == "running" {
                let outcome = match fail.code {
                    "stopped" => "stopped",
                    "timeout" => "timeout",
                    _ => "error",
                };
                executions::end(
                    tx,
                    id,
                    executions::End {
                        outcome,
                        detail: Some(&fail.message),
                        ..Default::default()
                    },
                )?;
            }
            Ok(())
        })?;
    }
    let row = project.read(|conn| executions::get(conn, id))?;
    capacity::release(daemon, project, &row).await?;
    Ok(())
}

async fn work(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    row: &executions::Execution,
    deadline: tokio::time::Instant,
) -> Result<(), Fail> {
    let scope = row.scope.as_ref().expect("standalone scope");
    let policy = &scope["policy"];
    let node: crate::plan::Node = serde_json::from_value(policy["node"].clone())
        .map_err(|error| Fail::refused(error.to_string()))?;
    let definitions = &policy["definitions"];
    let loaded = super::read_config(
        daemon,
        &project.canonical_dir(),
        scope["target"].as_str().expect("scope target"),
        None,
        definitions["definition_commit"]
            .as_str()
            .expect("definition commit"),
    )
    .await?;
    let dir = capacity::dir(project, row);
    let checkout = dir.join("checkout");
    daemon
        .git
        .clone_detached(
            &project.canonical_dir(),
            &checkout,
            row.head.as_deref().expect("scope input"),
        )
        .await?;
    let image = supervise::image(daemon, project, &loaded, row.id).await?;
    let (outcome, infrastructure, code, oom_kills, detail, output) = match &node.runner {
        Runner::Command { .. } => {
            let spec = BoxSpec {
                name: supervise::box_name(project, row.id),
                labels: supervise::labels(project),
                harness: None,
                image,
                mounts: vec![Mount {
                    host: checkout,
                    guest: "/workspace".into(),
                    readonly: node.access == Access::ReadOnly,
                }],
                env: BTreeMap::from([
                    ("HOME".into(), EnvValue::Value("/tmp".into())),
                    (
                        "PATH".into(),
                        EnvValue::Value("/usr/local/bin:/usr/bin:/bin".into()),
                    ),
                    (
                        "YARD_BASE".into(),
                        EnvValue::Value(row.base.clone().expect("scope input")),
                    ),
                ]),
                egress: None,
                memory: daemon.machine.box_memory.clone(),
            };
            let live = supervise::up_worker(daemon, project, row.id, &spec, &[]).await?;
            let argv = [
                "sh".into(),
                "-c".into(),
                policy["gate"]["command"].as_str().expect("command").into(),
            ];
            let result = daemon
                .pinfold
                .exec(
                    &live.name,
                    Some("/workspace"),
                    &argv,
                    deadline.saturating_duration_since(tokio::time::Instant::now()),
                )
                .await;
            let oom_kills = daemon.pinfold.oom_kills(&live.name).await;
            live.down(supervise::DOWN_TIMEOUT).await?;
            let result = result.map_err(|error| Fail::new("command", error.to_string()))?;
            let outcome = if result.code == 0 {
                "complete"
            } else {
                "failed"
            };
            (
                outcome,
                "completed",
                Some(result.code),
                oom_kills,
                format!("{}{}", result.stdout, result.stderr),
                Some(
                    json!({"kind": "verdict", "value": {"verdict": if result.code == 0 {"pass"} else {"fail"}, "exit_code": result.code}}),
                ),
            )
        }
        Runner::Agent { instructions, .. } => {
            let agent: crate::config::Agent = serde_json::from_value(policy["agent"].clone())
                .map_err(|error| Fail::refused(error.to_string()))?;
            let harness = crate::harness::get(&agent.harness).expect("admitted harness");
            if policy["harness_version"] != harness.version() {
                return Err(Fail::refused("runner_unavailable"));
            }
            let env = crate::agent_env::AgentEnv::load(
                &daemon.git,
                &project.canonical_dir(),
                &loaded.staging_commit,
                &checkout,
            )
            .await
            .map_err(Fail::refused)?;
            let state = dir.join("state");
            let input = dir.join("input");
            let stage = Stage {
                state: &state,
                input: &input,
                env: &env,
                connection: agent
                    .provider
                    .as_deref()
                    .and_then(crate::harness::connection),
            };
            harness.stage(&stage).map_err(|error| error.to_string())?;
            let model = harness
                .route(&stage, &daemon.machine)
                .map_err(Fail::refused)?;
            let kind = crate::mcp::Kind::StandaloneReport;
            let bearer = daemon.grants.issue(crate::mcp::Grant {
                project: project.clone(),
                execution: row.id,
                kind,
            });
            let mut secrets = vec![(crate::harness::BEARER_VAR.into(), bearer)];
            secrets.extend(model.secret.clone());
            let egress = supervise::egress(daemon, project, &loaded).await?;
            let spec = supervise::worker_spec(
                daemon,
                project,
                &supervise::Worker {
                    execution: row.id,
                    image: &image,
                    workspace: &checkout,
                    read_only: true,
                    proof: None,
                    harness,
                    stage,
                    model: &model,
                    egress: &egress,
                },
            );
            let prompt = format!(
                "{instructions}\nPublish a read-only report with yard_publish_report. Frozen scope: {scope}"
            );
            let argv = harness
                .argv(&Launch {
                    provider: agent.provider.as_deref(),
                    model: &agent.model,
                    effort: agent.effort.as_deref(),
                    resume: None,
                    prompt: &prompt,
                    env: &env,
                })
                .map_err(Fail::invalid)?;
            let live = supervise::up_worker(daemon, project, row.id, &spec, &secrets).await?;
            let run = supervise::run_harness(
                daemon,
                project,
                row.id,
                kind,
                harness,
                &argv,
                &state,
                &dir.join("transcript.jsonl"),
                Duration::from_secs(scope["timeout_seconds"].as_u64().expect("timeout")),
                deadline,
            )
            .await;
            daemon.grants.revoke(project, row.id);
            let oom_kills = daemon.pinfold.oom_kills(&live.name).await;
            live.down(supervise::DOWN_TIMEOUT).await?;
            let run = run?;
            if run.stopped {
                return Err(Fail::new("stopped", "operator stopped execution"));
            }
            if run.timed_out {
                return Err(Fail::new("timeout", "original execution lifetime expired"));
            }
            if run.exit_code != Some(0)
                || !matches!(run.terminal, Some(crate::harness::Event::Finished { .. }))
                || !matches!(
                    run.registered,
                    Some(crate::harness::Registration::Registered(_))
                )
            {
                (
                    "error",
                    "error",
                    run.exit_code,
                    oom_kills,
                    "report_run_failed".into(),
                    None,
                )
            } else if let Some(report) = project.read(|conn| executions::get(conn, row.id))?.report
            {
                (
                    "complete",
                    "completed",
                    run.exit_code,
                    oom_kills,
                    String::new(),
                    Some(json!({"kind": "report", "value": report})),
                )
            } else {
                (
                    "error",
                    "error",
                    run.exit_code,
                    oom_kills,
                    "missing_report".into(),
                    None,
                )
            }
        }
        Runner::Packaged { .. } => return Err(Fail::refused("standalone_authority")),
    };
    project.tx(|tx| {
        executions::end(
            tx,
            row.id,
            executions::End {
                outcome,
                exit_code: code,
                detail: Some(&detail),
                infrastructure: Some(infrastructure),
                oom_kills,
                output,
                ..Default::default()
            },
        )
    })
}
