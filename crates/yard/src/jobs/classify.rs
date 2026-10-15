//! Optional continuation judgments. A route never changes its failed source.

use super::{Loaded, scoped, supervise};
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::plan::{Continuation, Plan, Runner};
use crate::store::{self, Target, attempts, checks, executions, tickets};
use rusqlite::{Connection, params};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

fn stale(expected: Value, current: Value) -> Fail {
    Fail::stale("classification_input", expected.clone(), current.clone()).with(
        json!({"reason": "classification_input", "identity": "classification",
            "expected": expected, "current": current}),
    )
}

/// The same reading guards launch, both publication transports and consumption.
pub fn current(conn: &Connection, row: &executions::Execution) -> Result<bool, Fail> {
    let attempt = attempts::get(conn, row.required_attempt()?)?;
    let Some(inputs) = &row.inputs else {
        return Ok(false);
    };
    let identity = &inputs["identity"];
    if attempt.state != "live"
        || attempt.next.is_some()
        || identity["attempt"] != attempt.id
        || identity["target"] != json!(attempt.target_branch)
        || identity["plan_revision"] != json!(attempt.plan_revision())
        || identity["ticket_revision"] != tickets::get(conn, attempt.ticket)?.revision
        || identity["choice"] != json!(attempt.choice)
        || identity["candidate"]
            != json!({"base": attempt.base, "head": attempt.head, "proof": attempt.proof})
    {
        return Ok(false);
    }
    let input: checks::Input = serde_json::from_value(inputs["check_input"].clone())
        .map_err(|_| Fail::refused("unbound_classification"))?;
    let evidence = &identity["failure"];
    let check = &evidence["check"];
    if let Some(original) = evidence["landing"]["execution"].as_i64() {
        let original = executions::get(conn, original)?;
        let child = executions::get(
            conn,
            check["execution"]
                .as_i64()
                .ok_or_else(|| Fail::refused("unbound_classification"))?,
        )?;
        if super::queue::failed_evidence(conn, &child, &original)?.as_ref() != Some(evidence)
            || checks::approval(
                conn,
                original
                    .approval
                    .ok_or_else(|| Fail::refused("unbound_classification"))?,
            )?
            .state
                != "active"
        {
            return Ok(false);
        }
    } else {
        let Some(selected) = checks::current(
            conn,
            check["kind"].as_str().unwrap_or_default(),
            check["name"].as_str().unwrap_or_default(),
            &input,
        )?
        else {
            return Ok(false);
        };
        if selected.id != check["check"] || checks::evidence(conn, &selected, &input)? != *evidence
        {
            return Ok(false);
        }
    }
    let plan = Plan::admitted(
        attempt
            .admission
            .as_ref()
            .ok_or_else(|| Fail::refused("unbound_classification"))?,
    )
    .map_err(|error| Fail::refused(error.to_string()))?;
    let Some(policy) = row
        .node
        .as_deref()
        .and_then(|name| name.strip_prefix("classifier:"))
        .and_then(|name| plan.classifiers.get(name))
    else {
        return Ok(false);
    };
    if identity["policy"] != json!(policy) {
        return Ok(false);
    }
    if plan.finding_selector().is_some() && check["kind"] == "review" {
        return Ok(crate::store::selections::remediation(conn, &attempt)? == inputs["feedback"]);
    }
    Ok(true)
}

pub fn binding(
    _conn: &Connection,
    attempt: &attempts::Attempt,
    start: &executions::Start<'_>,
) -> Result<Option<(String, Value)>, Fail> {
    let node = start
        .node
        .ok_or_else(|| Fail::refused("unbound_classification"))?;
    let inputs = start
        .feedback
        .ok_or_else(|| Fail::refused("unbound_classification"))?;
    if inputs["identity"]["attempt"] != attempt.id {
        return Err(Fail::refused("unbound_classification"));
    }
    let plan = Plan::admitted(
        attempt
            .admission
            .as_ref()
            .ok_or_else(|| Fail::refused("unbound_classification"))?,
    )
    .map_err(|error| Fail::refused(error.to_string()))?;
    if !plan.classifiers.contains_key(node) {
        return Err(Fail::refused("undeclared_classification"));
    }
    // Publication and consumption re-read the exact check; construction already
    // holds the transaction that selected it.
    Ok(Some((format!("classifier:{node}"), inputs.clone())))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    outcome: String,
    identity: Value,
    report: Report,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    summary: String,
    findings: Vec<String>,
}

pub fn publish(conn: &Connection, execution: i64, arguments: &Value) -> Result<Value, Fail> {
    let row = executions::get(conn, execution)?;
    let publication: Publication = serde_json::from_value(arguments.clone())
        .map_err(|_| Fail::invalid("invalid_classification"))?;
    if row.status != "running" || row.reason.as_deref() != Some("classification") {
        return Err(Fail::refused("execution_ended"));
    }
    if row.report.is_some() {
        return Err(Fail::refused("duplicate_report"));
    }
    let text = |value: &str| !value.trim().is_empty() && value.len() <= 8000;
    if arguments.to_string().len() > 262144
        || !text(&publication.report.summary)
        || publication.report.findings.len() > 200
        || !publication.report.findings.iter().all(|value| text(value))
    {
        return Err(Fail::invalid("invalid_classification"));
    }
    let attempt = attempts::get(conn, row.required_attempt()?)?;
    if !current(conn, &row)?
        || row.inputs.as_ref().map(|inputs| &inputs["identity"]) != Some(&publication.identity)
    {
        return Err(stale(
            publication.identity,
            json!({"attempt": attempt.to_json(), "inputs": row.inputs}),
        ));
    }
    let policy = &row.inputs.as_ref().expect("bound")["identity"]["policy"];
    if policy["outcomes"].get(&publication.outcome).is_none() {
        return Err(Fail::invalid("undeclared_outcome"));
    }
    conn.execute(
        "UPDATE execution SET report = ?2 WHERE id = ?1 AND report IS NULL",
        params![row.id, arguments.to_string()],
    )?;
    store::audit(
        conn,
        "classification.published",
        Target {
            attempt: row.attempt,
            execution: Some(row.id),
            ticket: Some(attempt.ticket),
            ..Default::default()
        },
        None,
        json!({"outcome": publication.outcome, "identity": publication.identity}),
    )?;
    Ok(json!({"published": row.id}))
}

pub async fn run(daemon: &Arc<Daemon>, project: &Arc<Project>, execution: i64) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    if !project.read(|conn| current(conn, &row))? {
        return Err(stale(json!(row.inputs), json!({"execution": execution})));
    }
    let scope = &row
        .prepared
        .as_ref()
        .ok_or_else(|| Fail::refused("unbound_classification"))?["classification"];
    let timeout = std::time::Duration::from_secs(
        scope["timeout_seconds"]
            .as_u64()
            .ok_or_else(|| Fail::refused("unbound_classification"))?,
    );
    let stop = daemon
        .stops
        .lock()
        .expect("stops lock")
        .entry((project.key.clone(), execution))
        .or_insert_with(|| Arc::new(tokio::sync::Notify::new()))
        .clone();
    supervise::notify_if_edited(project, execution, &stop);
    let deadline = tokio::time::Instant::now() + timeout;
    let result = tokio::select! {
        biased;
        _ = stop.notified() => Err(Fail::new("stopped", "operator stopped execution")),
        _ = tokio::time::sleep_until(deadline) => Err(Fail::new("timeout", "classification execution lifetime expired")),
        result = scoped::work(daemon, project, &row, scope, deadline) => result,
    };
    if result.is_err() {
        daemon.grants.revoke(project, execution);
        // A cancelled launch may not have returned its handle yet. Teardown
        // inherits the existing execution lock before checked release.
        let _ = daemon
            .pinfold
            .down(&supervise::box_name(project, execution))
            .await;
    }
    result
}

pub fn retry(conn: &Connection, row: &executions::Execution) -> Result<i64, Fail> {
    if row.status != "ended" || row.reserved || row.teardown != Some(true) || !current(conn, row)? {
        return Err(Fail::refused("classification_input"));
    }
    if executions::attempt_work(conn, row.required_attempt()?)?
        .iter()
        .any(|row| row.status == "running" || row.reserved)
    {
        return Err(Fail::refused("workspace_owned"));
    }
    let name = row
        .node
        .as_deref()
        .and_then(|node| node.strip_prefix("classifier:"))
        .ok_or_else(|| Fail::refused("unbound_classification"))?;
    let id = executions::start(
        conn,
        executions::Start {
            attempt: row.attempt,
            kind: "review",
            node: Some(name),
            reason: Some("classification"),
            base: row.base.as_deref(),
            head: row.head.as_deref(),
            proof: row.proof.as_deref(),
            ticket_revision: row.ticket_revision,
            feedback: row.inputs.as_ref(),
            agent: row.settings.as_ref().map(|settings| {
                (
                    row.prepared
                        .as_ref()
                        .and_then(|prepared| {
                            prepared["classification"]["policy"]["node"]["runner"]["agent"].as_str()
                        })
                        .unwrap_or_default(),
                    settings,
                )
            }),
            harness_version: row.harness_version.as_deref(),
            ..Default::default()
        },
    )?;
    conn.execute(
        "UPDATE execution SET prepared = ?2 WHERE id = ?1",
        params![id, row.prepared.as_ref().map(Value::to_string)],
    )?;
    Ok(id)
}

/// Returns true when classification owns continuation, including attention.
#[allow(clippy::too_many_arguments)]
pub fn route(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
    attempt: &attempts::Attempt,
    check: &checks::Check,
    input: &checks::Input,
    blocked: &[checks::Check],
) -> Result<bool, Fail> {
    let Some(admission) = &attempt.admission else {
        return Ok(false);
    };
    let plan = Plan::admitted(admission).map_err(|error| Fail::refused(error.to_string()))?;
    let source = project.read(|conn| executions::get(conn, check.execution))?;
    let Some(name) = source.node.as_deref() else {
        return Ok(false);
    };
    let Some(policy) = plan.classifiers.get(name) else {
        return Ok(false);
    };
    let classifier_node = format!("classifier:{name}");
    let mut spawn = None;
    let mut attention = false;
    let handled = project.tx(|tx| {
        let now = attempts::get(tx, attempt.id)?;
        if now.plan_revision() != attempt.plan_revision() || now.candidate() != attempt.candidate()
            || now.proof != attempt.proof || now.choice != attempt.choice || now.next.is_some()
            || now.state != "live" || tickets::get(tx, now.ticket)?.revision != input.ticket_revision {
            return Ok(true);
        }
        let evidence = checks::evidence(tx, check, input)?;
        if checks::current(tx, &check.kind, &check.name, input)?.map(|check| check.id) != Some(check.id) {
            return Ok(true);
        }
        if checks::decision(tx, now.id, &evidence)?.is_some() { return Ok(false); }
        let feedback = if check.kind == "review" && plan.finding_selector().is_some() {
            crate::store::selections::remediation(tx, &now)?
        } else {
            let mut findings = Vec::new();
            let mut blocks = blocked.to_vec();
            if check.kind == "review" && check.verdict == "fail" && !blocks.iter().any(|item| item.id == check.id) {
                blocks.push(check.clone());
            }
            for blocked in &blocks {
                for finding in checks::findings(tx, blocked.id)? {
                    findings.push(json!({"seat": blocked.name, "finding": finding}));
                }
            }
            json!({"kind": "verdict", "source": {"external": "feedback", "execution": source.id,
                "node": name, "output": "result"}, "value": {"verdict": "fail", "gate": check.name,
                "detail": source.detail, "findings": findings}})
        };
        let inputs = json!({"identity": {"attempt": now.id, "target": now.target_branch,
            "candidate": {"base": now.base, "head": now.head, "proof": now.proof},
            "choice": now.choice, "ticket_revision": input.ticket_revision, "plan_revision": now.plan_revision(),
            "failure": evidence, "policy": policy}, "check_input": input, "feedback": feedback});
        let rows = executions::attempt_work(tx, now.id)?;
        if rows.iter().any(|row| row.status == "running" || row.reserved) { return Ok(true); }
        let previous = rows.iter().rev().find(|row| row.reason.as_deref() == Some("classification")
            && row.node.as_deref() == Some(&classifier_node) && row.inputs.as_ref() == Some(&inputs));
        if let Some(row) = previous {
            if !row.result_eligible {
                attempts::raise(tx, attempts::Raise {kind: "red", reason: "classification", ticket: Some(now.ticket), attempt: Some(now.id), execution: Some(row.id),
                    payload: json!({"identity": inputs["identity"], "detail": "missing_or_invalid_classification"}), text: None})?;
                return Ok(true);
            }
            if !current(tx, row)? { return Err(stale(inputs["identity"].clone(), json!(row.inputs))); }
            let published = row.report.as_ref().ok_or_else(|| Fail::refused("missing_classification"))?;
            let outcome = published["outcome"].as_str().ok_or_else(|| Fail::refused("missing_classification"))?;
            let action = policy.outcomes.get(outcome).ok_or_else(|| Fail::refused("undeclared_outcome"))?;
            let consumed: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM audit WHERE event = 'classification.routed' AND execution = ?1)", [row.id], |row| row.get(0))?;
            if consumed { return Ok(true); }
            if matches!(action, Continuation::Accept { .. }) && accept(tx, loaded, row.id)? {
                return Ok(true);
            }
            if matches!(action, Continuation::Operator | Continuation::Accept { .. }) {
                attention = true;
                return Ok(true);
            }
            let used: u32 = tx.query_row("SELECT COUNT(*) FROM audit a JOIN execution e ON e.id = a.execution
                WHERE a.event = 'classification.routed' AND e.attempt = ?1 AND e.node = ?2 AND e.plan_revision IS ?3",
                params![now.id, classifier_node, now.plan_revision()], |row| row.get(0))?;
            if used >= policy.repeat {
                attention = true;
                return Ok(true);
            }
            let (kind, execution) = if matches!(action, Continuation::Retry) {
                let settings = source.settings.as_ref();
                let agent = settings.and_then(|settings| settings["name"].as_str()).unwrap_or(&check.name);
                let id = executions::start(tx, executions::Start {attempt: Some(now.id), kind: &source.kind, node: Some(name),
                    name: source.name.as_deref(), reason: Some("retry"), base: source.base.as_deref(), head: source.head.as_deref(), proof: source.proof.as_deref(),
                    ticket_revision: source.ticket_revision, digest: Some(&input.digest), round: source.round,
                    agent: settings.map(|settings| (agent, settings)), harness_version: source.harness_version.as_deref(), ..Default::default()})?;
                (source.kind.as_str(), id)
            } else {
                let mut feedback = feedback;
                if let Continuation::Fixer { profile } = action {
                    feedback["classification"] = json!({"execution": row.id, "fixer": profile});
                }
                ("implementation", supervise::start_implementation(tx, &now, "repair", Some(&feedback))?)
            };
            store::audit(tx, "classification.routed", Target {ticket: Some(now.ticket), attempt: Some(now.id), execution: Some(row.id), ..Default::default()}, None,
                json!({"outcome": outcome, "execution": execution, "identity": inputs["identity"]}))?;
            spawn = Some((kind.to_owned(), execution));
            return Ok(true);
        }
        let id = launch(tx, loaded, &now, &source, name, policy, &inputs)?;
        spawn = Some(("review".into(), id));
        Ok(true)
    })?;
    if attention {
        let evidence = project.read(|conn| checks::evidence(conn, check, input))?;
        super::raise_candidate_failure(project, loaded, attempt, check, input, &evidence)?;
    }
    if let Some((kind, execution)) = spawn {
        super::spawn(daemon, project, &kind, execution);
    }
    Ok(handled)
}

#[allow(clippy::too_many_arguments)]
fn launch(
    tx: &Connection,
    loaded: &Loaded,
    now: &attempts::Attempt,
    source: &executions::Execution,
    name: &str,
    policy: &crate::plan::Classifier,
    inputs: &Value,
) -> Result<i64, Fail> {
    let admission = now
        .admission
        .as_ref()
        .ok_or_else(|| Fail::refused("unbound_classification"))?;
    let node = policy.node();
    let (agent, settings, version) = match &policy.runner {
        Runner::Agent { agent, .. } => {
            let settings = admission["agents"][agent]["settings"].clone();
            let version = admission["agents"][agent]["runner"]["version"]
                .as_str()
                .map(str::to_owned);
            (Some(agent.as_str()), settings, version)
        }
        _ => (None, Value::Null, None),
    };
    let gate = match &policy.runner {
        Runner::Command { gate } => admission["gates"]
            .as_array()
            .and_then(|gates| gates.iter().find(|entry| entry["name"] == *gate))
            .cloned()
            .ok_or_else(|| Fail::refused("unbound_classification"))?,
        _ => Value::Null,
    };
    let id = executions::start(
        tx,
        executions::Start {
            attempt: Some(now.id),
            kind: "review",
            node: Some(name),
            reason: Some("classification"),
            base: source.base.as_deref(),
            head: source.head.as_deref(),
            proof: source.proof.as_deref(),
            ticket_revision: source.ticket_revision,
            agent: agent.map(|agent| (agent, &settings)),
            harness_version: version.as_deref(),
            feedback: Some(inputs),
            ..Default::default()
        },
    )?;
    let scope = json!({"target": now.target_branch, "timeout_seconds": loaded.config.review.timeout_minutes * 60,
            "policy": {"node": node, "gate": gate, "agent": settings, "harness_version": version,
                "definitions": admission["inputs"]}});
    tx.execute(
        "UPDATE execution SET prepared = ?2 WHERE id = ?1",
        params![id, json!({"classification": scope}).to_string()],
    )?;
    Ok(id)
}

pub async fn landing(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    loaded: &Loaded,
    source: &executions::Execution,
    original: &executions::Execution,
) -> Result<Option<(Continuation, i64)>, Fail> {
    let attempt = project.read(|conn| attempts::get(conn, source.required_attempt()?))?;
    let plan = Plan::admitted(
        attempt
            .admission
            .as_ref()
            .ok_or_else(|| Fail::refused("unbound_classification"))?,
    )
    .map_err(|error| Fail::refused(error.to_string()))?;
    let name = source
        .node
        .as_deref()
        .ok_or_else(|| Fail::refused("unbound_classification"))?;
    let Some(policy) = plan.classifiers.get(name) else {
        return Ok(None);
    };
    let inputs = project.read(|conn| {
        let evidence = super::queue::failed_evidence(conn, source, original)?.ok_or_else(|| Fail::refused("unbound_classification"))?;
        Ok(json!({"identity": {"attempt": attempt.id, "target": attempt.target_branch,
            "candidate": {"base": attempt.base, "head": attempt.head, "proof": attempt.proof},
            "choice": attempt.choice, "ticket_revision": source.ticket_revision, "plan_revision": attempt.plan_revision(),
            "failure": evidence, "policy": policy}, "check_input": checks::Input {
                attempt: attempt.id, base: source.base.clone().unwrap_or_default(), head: source.head.clone().unwrap_or_default(),
                proof: source.proof.clone().unwrap_or_default(), ticket_revision: source.ticket_revision.unwrap_or_default(),
                plan_revision: source.plan_revision, digest: loaded.gate_digest.clone()},
            "feedback": {"kind": "verdict", "source": {"external": "feedback", "execution": source.id,
                "node": name, "output": "result"}, "value": {"verdict": "fail", "detail": source.detail, "evidence": evidence}}}))
    })?;
    let (id, fresh) = project.tx(|tx| {
        let now = attempts::get(tx, attempt.id)?;
        if now.admission != attempt.admission
            || now.candidate() != attempt.candidate()
            || now.proof != attempt.proof
            || now.choice != attempt.choice
        {
            return Err(stale(inputs["identity"].clone(), now.to_json()));
        }
        let previous = executions::attempt_work(tx, now.id)?
            .into_iter()
            .rev()
            .find(|row| {
                row.reason.as_deref() == Some("classification")
                    && row.inputs.as_ref() == Some(&inputs)
            });
        if let Some(previous) = previous {
            if !previous.result_eligible {
                return Err(Fail::refused("missing_or_invalid_classification")
                    .with(json!({"classification": previous.id})));
            }
            return Ok((previous.id, false));
        }
        launch(tx, loaded, &now, source, name, policy, &inputs).map(|id| (id, true))
    })?;
    if fresh
        && let Err(fail) =
            super::capacity::effect(daemon, project, id, run(daemon, project, id)).await
    {
        project.tx(|tx| {
            if executions::get(tx, id)?.status == "running" {
                executions::end(
                    tx,
                    id,
                    executions::End {
                        outcome: "error",
                        infrastructure: Some("error"),
                        detail: Some(&fail.message),
                        ..Default::default()
                    },
                )?;
            }
            Ok(())
        })?;
        return Err(fail.with(json!({"classification": id})));
    }
    project.read(|conn| {
        let row = executions::get(conn, id)?;
        if !row.result_eligible || !current(conn, &row)? { return Err(Fail::refused("missing_or_invalid_classification").with(json!({"classification": id}))); }
        let action = row.report.as_ref().and_then(|report| report["outcome"].as_str())
            .and_then(|outcome| policy.outcomes.get(outcome)).ok_or_else(|| Fail::refused("undeclared_outcome"))?.clone();
        let used: u32 = conn.query_row("SELECT COUNT(*) FROM audit a JOIN execution e ON e.id = a.execution
            WHERE a.event = 'classification.routed' AND e.attempt = ?1 AND e.node = ?2 AND e.plan_revision IS ?3",
            params![attempt.id, row.node, attempt.plan_revision()], |row| row.get(0))?;
        let consumed: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM audit WHERE event = 'classification.routed' AND execution = ?1)", [id], |row| row.get(0))?;
        let repeats = matches!(action, Continuation::Original | Continuation::Fixer { .. } | Continuation::Retry);
        Ok(Some((if consumed || (repeats && used >= policy.repeat) { Continuation::Operator } else { action }, id)))
    })
}

pub fn routed(conn: &Connection, classifier: i64, successor: i64) -> Result<(), Fail> {
    let row = executions::get(conn, classifier)?;
    let attempt = attempts::get(conn, row.required_attempt()?)?;
    store::audit(conn, "classification.routed", Target {ticket: Some(attempt.ticket), attempt: row.attempt,
        execution: Some(row.id), ..Default::default()}, None,
        json!({"outcome": row.report.as_ref().map(|report| &report["outcome"]), "execution": successor, "identity": row.inputs.as_ref().map(|inputs| &inputs["identity"])})).map(|_| ())
}

pub fn attention(
    conn: &Connection,
    attempt: &attempts::Attempt,
    evidence: &Value,
) -> Result<Value, Fail> {
    let Some(classifier) = executions::attempt_work(conn, attempt.id)?
        .into_iter()
        .rev()
        .find(|row| {
            row.reason.as_deref() == Some("classification")
                && row.result_eligible
                && row
                    .inputs
                    .as_ref()
                    .is_some_and(|inputs| inputs["identity"]["failure"] == *evidence)
        })
    else {
        return Ok(Value::Null);
    };
    let used: u32 = conn.query_row("SELECT COUNT(*) FROM audit a JOIN execution e ON e.id = a.execution
        WHERE a.event = 'classification.routed' AND e.attempt = ?1 AND e.node IS ?2 AND e.plan_revision IS ?3",
        params![attempt.id, classifier.node, attempt.plan_revision()], |row| row.get(0))?;
    let repeat =
        classifier.inputs.as_ref().expect("classifier")["identity"]["policy"]["repeat"].clone();
    Ok(
        json!({"execution": classifier.id, "result": classifier.result,
        "repeat": repeat, "used": used, "exhausted": repeat.as_u64().is_some_and(|repeat| u64::from(used) >= repeat)}),
    )
}

/// Only text frozen in the operator's admitted outcome authorizes acceptance.
pub fn accept(conn: &Connection, loaded: &Loaded, classifier: i64) -> Result<bool, Fail> {
    let row = executions::get(conn, classifier)?;
    let inputs = row
        .inputs
        .as_ref()
        .ok_or_else(|| Fail::refused("unbound_classification"))?;
    let outcome = row
        .report
        .as_ref()
        .and_then(|report| report["outcome"].as_str())
        .ok_or_else(|| Fail::refused("missing_classification"))?;
    let action: Continuation =
        serde_json::from_value(inputs["identity"]["policy"]["outcomes"][outcome].clone())
            .map_err(|_| Fail::refused("undeclared_outcome"))?;
    let Continuation::Accept {
        authorization: Some(text),
    } = action
    else {
        return Ok(false);
    };
    if !row.result_eligible || !current(conn, &row)? {
        return Err(stale(inputs["identity"].clone(), Value::Null));
    }
    let attempt = attempts::get(conn, row.required_attempt()?)?;
    let evidence = &inputs["identity"]["failure"];
    let id = attempts::raise(
        conn,
        attempts::Raise {
            kind: "candidate",
            reason: if evidence.get("landing").is_some() {
                "landing"
            } else if evidence["check"]["kind"] == "gate" {
                "failed"
            } else {
                "blocked"
            },
            ticket: Some(attempt.ticket),
            attempt: Some(attempt.id),
            execution: evidence["check"]["execution"].as_i64(),
            payload: json!({"actor": "operator-plan", "target": attempt.target_branch, "choice": attempt.choice,
            "base": attempt.base, "head": attempt.head, "proof": attempt.proof.clone().unwrap_or_default(),
            "revision": inputs["identity"]["ticket_revision"], "plan_revision": attempt.plan_revision(),
            "gate_digest": loaded.gate_digest, "review_digest": loaded.review_digest(&attempt.workflow)?,
            "evidence": evidence, "classification": {"execution": classifier}}),
            text: None,
        },
    )?;
    attempts::resolve(conn, &attempts::attention(conn, id)?, "accept", Some(&text))?;
    Ok(true)
}

pub fn fixer(
    conn: &Connection,
    attempt: &attempts::Attempt,
    feedback: &Value,
) -> Result<Option<crate::plan::Node>, Fail> {
    let Some(profile) = feedback["classification"]["fixer"].as_str() else {
        return Ok(None);
    };
    let id = feedback["classification"]["execution"]
        .as_i64()
        .ok_or_else(|| Fail::refused("unbound_fixer"))?;
    let row = executions::get(conn, id)?;
    if !row.result_eligible
        || row.attempt != Some(attempt.id)
        || row.plan_revision != attempt.plan_revision()
    {
        return Err(Fail::refused("unbound_fixer"));
    }
    let report = row
        .report
        .as_ref()
        .ok_or_else(|| Fail::refused("unbound_fixer"))?;
    let action = &row
        .inputs
        .as_ref()
        .ok_or_else(|| Fail::refused("unbound_fixer"))?["identity"]["policy"]["outcomes"]
        [report["outcome"]
            .as_str()
            .ok_or_else(|| Fail::refused("unbound_fixer"))?];
    if action != &json!({"kind": "fixer", "profile": profile}) {
        return Err(Fail::refused("unbound_fixer"));
    }
    let definition: crate::plan::Profile = serde_json::from_value(
        attempt
            .admission
            .as_ref()
            .ok_or_else(|| Fail::refused("unbound_fixer"))?["classifier_profiles"][profile]
            .clone(),
    )
    .map_err(|_| Fail::refused("unbound_fixer"))?;
    Ok(Some(definition.node()))
}

pub fn effective_node(plan: &Plan, row: &executions::Execution) -> Result<crate::plan::Node, Fail> {
    let mut node = plan
        .nodes
        .get(row.node.as_deref().unwrap_or("implementer"))
        .ok_or_else(|| Fail::refused("undeclared_workspace"))?
        .clone();
    if let Some(fixer) = row
        .prepared
        .as_ref()
        .and_then(|prepared| prepared.get("fixer"))
    {
        let fixer: crate::plan::Node =
            serde_json::from_value(fixer.clone()).map_err(|_| Fail::refused("unbound_fixer"))?;
        node.runner = fixer.runner;
    }
    Ok(node)
}
