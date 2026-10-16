//! Review: one execution per seat, each in its own box on a fresh read-only
//! checkout of the head. A seat publishes once; the publication is the
//! decision.

use super::proof;
use super::supervise::{self, Worker};
use crate::api::Fail;
use crate::daemon::{Daemon, Project};
use crate::store::{attempts, checks, executions, ticket_name, tickets};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

pub async fn run(daemon: &Arc<Daemon>, project: &Arc<Project>, execution: i64) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    let attempt = project.read(|conn| attempts::for_execution(conn, &row))?;
    let loaded = super::load_attempt(daemon, project, &attempt).await?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let seat_name = row.name.clone().unwrap_or_default();
    let plan = match &attempt.admission {
        Some(admission) => crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?,
        None => loaded.config.workflow_plan(&attempt.workflow)?,
    };
    let node = match &row.node {
        Some(node) => node.clone(),
        None => {
            let index = loaded
                .config
                .workflow(&attempt.workflow)?
                .review
                .iter()
                .position(|name| name == &seat_name)
                .ok_or_else(|| {
                    Fail::invalid(format!("seat {seat_name} is no longer configured"))
                })?;
            format!("review-{index}")
        }
    };
    let Some(crate::plan::Runner::Agent {
        agent: selected,
        instructions,
        ..
    }) = plan.nodes.get(&node).map(|node| &node.runner)
    else {
        return Err(Fail::refused("review has no admitted agent runner"));
    };
    let finding_selector = plan.finding_selector().is_some() && node == "selector";
    let selector = (plan.selector().is_some() || finding_selector) && node == "selector";
    let kind = if selector {
        crate::mcp::Kind::Selector
    } else {
        crate::mcp::Kind::Review
    };
    let agent = &loaded.config.agents[selected];
    let connection = match &agent.provider {
        Some(name) => Some(
            crate::harness::connection(name)
                .ok_or_else(|| Fail::invalid(format!("connection {name:?} is unknown")))?,
        ),
        None => None,
    };
    let harness = crate::harness::get(&agent.harness)
        .ok_or_else(|| Fail::invalid(format!("harness {:?} is unknown", agent.harness)))?;
    let head = row.head.clone().unwrap_or_else(|| attempt.base.clone());
    let base = row.base.clone().unwrap_or_default();
    // The seat reads the snapshot the candidate was judged on, never the
    // implementer's live directory.
    let proof = row
        .proof
        .as_deref()
        .filter(|digest| !digest.is_empty())
        .map(|digest| proof::snapshot_path(project, attempt.id, digest));

    let dir = project
        .attempt_dir(attempt.id)
        .join("reviews")
        .join(execution.to_string());
    let checkout = dir.join("checkout");
    let state = dir.join("state");
    let input = dir.join("input");
    let _ = std::fs::remove_dir_all(&dir);
    daemon
        .git
        .clone_detached(&project.canonical_dir(), &checkout, &head)
        .await?;
    let env = crate::agent_env::AgentEnv::load(
        &daemon.git,
        &project.canonical_dir(),
        if attempt.admission.is_some() {
            &loaded.staging_commit
        } else {
            &base
        },
        &checkout,
    )
    .await
    .map_err(Fail::refused)?;
    let stage = crate::harness::Stage {
        state: &state,
        input: &input,
        connection,
        env: &env,
    };
    harness.stage(&stage).map_err(|error| error.to_string())?;
    super::supervise::stage_selected(
        daemon,
        project,
        &attempt,
        &node,
        &plan.nodes[&node].runner,
        &input,
    )
    .await?;

    let stat = daemon
        .git
        .diff_stat(&project.canonical_dir(), &base, &head)
        .await?;
    let prompt = format!(
        "You are the {seat_name} reviewer for ticket {}: {}\n\n{}\n\n{}\n\n\
         The candidate is checked out read-only in /workspace at {head}; its base is {base}.\n\
         The candidate's proof snapshot is at /yard/proof, read-only.\n\
         Changed files:\n{stat}\n\
         Read the change (`git diff {base} {head}`), then publish your review exactly once with \
         yard_publish_review. Give every finding a priority from P0 (worst) to P3, and a file and \
         line where you can. Publish an empty findings list if you find nothing.\n",
        ticket_name(ticket.id),
        ticket.title,
        ticket.body,
        instructions
    );
    let prompt = if finding_selector {
        format!(
            "You are the read-only finding selector for ticket {}: {}\n\n{}\n\n{}\n\nFrozen inputs:\n{}\n\nCall yard_publish_selection once with groups. Each group has a nonblank reason and sources: exact source objects from the declared review inputs' findings. Preserve every contributor when combining duplicates. Select by public source identity only. This output grants no repair, acceptance, approval or landing authority.\n",
            ticket_name(ticket.id),
            ticket.title,
            ticket.body,
            instructions,
            serde_json::to_string_pretty(&row.inputs).expect("inputs serialize")
        )
    } else if selector {
        format!(
            "You are the read-only selector for ticket {}: {}\n\n{}\n\n{}\n\nFrozen inputs:\n{}\n\nEach candidate is read-only at /yard/candidates/<hex input name>/source with its proof at /yard/candidates/<hex input name>/proof. Compare only the declared candidate results and bound reviews. Call yard_publish_selection exactly once with candidate naming one candidate input. This selects an existing reference and grants no write, approval or landing authority.\n",
            ticket_name(ticket.id),
            ticket.title,
            ticket.body,
            instructions,
            serde_json::to_string_pretty(&row.inputs).expect("inputs serialize")
        )
    } else {
        prompt
    };
    let prompt = format!(
        "{}{}",
        super::supervise::selected_prompt(&plan.nodes[&node].runner),
        prompt
    );
    let argv = harness
        .argv(&crate::harness::Launch {
            provider: agent.provider.as_deref(),
            model: &agent.model,
            effort: agent.effort.as_deref(),
            resume: None,
            prompt: &prompt,
            env: &env,
        })
        .map_err(Fail::invalid)?;
    let image = supervise::image(daemon, project, &loaded, execution).await?;
    // Resolve the route, including a login's token, before any bearer is
    // issued, so a missing credential never leaves a live grant.
    let model = harness
        .route(&stage, &daemon.machine)
        .map_err(Fail::refused)?;
    let candidate_inputs = if selector && !finding_selector {
        supervise::candidate_mounts(daemon, project, &row, false).await?
    } else {
        Vec::new()
    };
    let bearer = daemon.grants.issue(crate::mcp::Grant {
        project: project.clone(),
        execution,
        kind,
    });
    let mut secrets = vec![(crate::harness::BEARER_VAR.to_string(), bearer)];
    secrets.extend(model.secret.clone());
    let mut spec = supervise::worker_spec(
        daemon,
        project,
        &Worker {
            execution,
            image: &image,
            workspace: &checkout,
            read_only: true,
            proof: proof.as_deref(),
            harness,
            stage,
            model: &model,
            egress: &[],
        },
    );
    spec.mounts.extend(candidate_inputs);
    let live = supervise::up_worker(daemon, project, execution, &spec, &secrets).await?;
    let timeout = Duration::from_secs(loaded.config.review.timeout_minutes * 60);
    let run = async {
        let launch = json!({
            "argv_digest": crate::config::digest(&argv.iter().map(|arg| arg.as_bytes()).collect::<Vec<_>>()),
            "trusted_bytes": env.evidence(&agent.harness),
        });
        super::verification(
            daemon, project, execution, &checkout, launch, supervise::runtime_evidence(&spec),
        ).await?;
        if project.read(|conn| Ok(crate::store::executions::get(conn, execution)?.verification
            .is_some_and(|v| !v["reference"].is_null())))? { return Ok(None); }
        supervise::run_harness(
            daemon,
            project,
            execution,
            kind,
            harness,
            &argv,
            &state,
            &supervise::transcript(project, &project.read(|conn| executions::get(conn, execution))?),
            timeout,
            tokio::time::Instant::now() + timeout,
        )
        .await.map(Some)
    }
    .await;
    daemon.grants.revoke(project, execution);
    live.down(supervise::DOWN_TIMEOUT).await?;
    let _ = std::fs::remove_dir_all(&dir);
    let Some(run) = run? else {
        return project.tx(|tx| {
            executions::end(
                tx,
                execution,
                executions::End {
                    outcome: "reused",
                    infrastructure: Some("reused"),
                    ticket: Some(ticket.id),
                    ..Default::default()
                },
            )
        });
    };

    // Review verdicts survive a failed runner; selector results require success.
    // Eligibility also needs checked registration and ownership proofs.
    project.tx(|tx| {
        if attempts::get(tx, attempt.id)?.state != "live" {
            return executions::end(
                tx,
                execution,
                executions::End {
                    outcome: "abandoned",
                    ticket: Some(ticket.id),
                    ..Default::default()
                },
            );
        }
        let published = checks::for_execution(tx, execution)?;
        let selection = executions::get(tx, execution)?.report.filter(|_| selector);
        let has_publication = published.is_some() || selection.is_some();
        // An edit that landed while the seat ran superseded it: no error, no round.
        let moved = executions::plan_superseded(tx, &attempts::get(tx, attempt.id)?, &row)?
            || match row.ticket_revision {
                Some(read) => attempts::superseded_by_edit(tx, attempt.id, ticket.id, read)?,
                None => false,
            };
        let infrastructure = if run.timed_out {
            "timeout"
        } else if run.stopped {
            "stopped"
        } else if matches!(run.terminal, Some(crate::harness::Event::Failed { .. }))
            || run.exit_code.is_some_and(|code| code != 0)
        {
            "failed"
        } else if run.terminal.is_none() {
            "unknown"
        } else {
            "completed"
        };
        let failure = match (has_publication, &run.registered, &run.terminal) {
            (true, _, _) if selector && !run.stopped && !moved && infrastructure != "completed" => {
                Some(match &run.terminal {
                    Some(crate::harness::Event::Failed { message, .. }) => message.clone(),
                    _ => format!("the selector ended with infrastructure outcome {infrastructure}"),
                })
            }
            (true, _, _) => None,
            // An edit stopped the seat: it judged nothing and is not an error.
            (false, _, _) if run.stopped || moved => None,
            // A refused registration ends the run before a terminal frame, so
            // name its reason rather than the generic no-publication error.
            (false, Some(crate::harness::Registration::Refused(reason)), _) => Some(reason.clone()),
            (false, _, _) if run.timed_out => {
                Some("the seat ran past its timeout without publishing".to_string())
            }
            (false, _, Some(crate::harness::Event::Failed { message, .. })) => {
                Some(message.clone())
            }
            (false, _, _) => Some("the seat ended without publishing".to_string()),
        };
        let outcome = match &failure {
            Some(_) => "error",
            None if (run.stopped || moved) && (!has_publication || selector) => "stopped",
            None if selector && selection.is_some() && infrastructure == "completed" => "selected",
            None => published
                .as_ref()
                .map_or("error", |check| check.verdict.as_str()),
        };
        executions::end(
            tx,
            execution,
            executions::End {
                outcome,
                infrastructure: Some(infrastructure),
                output: selection
                    .filter(|_| infrastructure == "completed" && !moved)
                    .map(|value| json!({"kind": if finding_selector { "finding-selection" } else { "selection" }, "value": value})),
                detail: failure.as_deref(),
                exit_code: run.exit_code,
                ticket: Some(ticket.id),
                ..Default::default()
            },
        )?;
        if let Some(detail) = failure {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "red",
                    reason: "review",
                    ticket: Some(ticket.id),
                    attempt: Some(attempt.id),
                    execution: Some(execution),
                    payload: json!({ "seat": seat_name, "detail": detail }),
                    text: Some(&detail),
                },
            )?;
        }
        Ok(())
    })
}

/// `yard_publish_review`: findings, the seat's check and its event in one
/// transaction. Once per execution.
pub fn publish(grant: &crate::mcp::Grant, arguments: &Value) -> Result<Value, Fail> {
    let object = crate::mcp::strict(arguments, &["findings"])?;
    let findings: Vec<checks::Finding> = match object.get("findings") {
        Some(Value::Array(items)) => {
            let mut findings = Vec::new();
            for item in items {
                crate::mcp::strict(item, &["priority", "file", "line", "category", "body"])?;
                let finding: checks::Finding = serde_json::from_value(item.clone())
                    .map_err(|error| Fail::invalid(format!("finding: {error}")))?;
                if crate::config::priority(&finding.priority).is_none() {
                    return Err(Fail::invalid(format!(
                        "priority {:?} is not P0 to P3",
                        finding.priority
                    )));
                }
                if finding.line.is_some_and(|line| line < 1) {
                    return Err(Fail::invalid("line starts at 1"));
                }
                findings.push(finding);
            }
            findings
        }
        _ => return Err(Fail::invalid("findings is a list")),
    };
    let project = &grant.project;
    let attempt = project.read(|conn| {
        let row = executions::get(conn, grant.execution)?;
        attempts::get(conn, row.required_attempt()?)
    })?;
    let blocking = if let Some(admission) = &attempt.admission {
        admission["review"]["blocking"]
            .as_u64()
            .ok_or_else(|| Fail::refused("invalid admitted review"))? as u8
    } else {
        project
            .loaded
            .lock()
            .expect("loaded lock")
            .targets
            .get(attempt.target_branch()?)
            .ok_or_else(|| Fail::refused("the configuration is not loaded"))?
            .config
            .review
            .blocking
    };
    let check = project.tx(|tx| {
        if checks::for_execution(tx, grant.execution)?.is_some() {
            return Err(Fail::refused("this seat has already published"));
        }
        let row = executions::get(tx, grant.execution)?;
        let attempt = attempts::get(tx, row.required_attempt()?)?;
        if attempt.state != "live" {
            return Err(Fail::refused("the attempt has ended"));
        }
        if executions::plan_superseded(tx, &attempt, &row)?
            || row.ticket_revision.is_some_and(|read| {
                attempts::superseded_by_edit(tx, attempt.id, attempt.ticket, read).unwrap_or(true)
            })
        {
            return Err(Fail::refused(
                "the ticket was edited; this candidate is superseded and nothing is recorded",
            ));
        }
        let blocked = findings.iter().any(|finding| {
            crate::config::priority(&finding.priority).is_some_and(|priority| priority <= blocking)
        });
        let check = checks::record(
            tx,
            checks::Record {
                execution: grant.execution,
                kind: "review",
                verdict: if blocked { "fail" } else { "pass" },
                ticket: attempt.ticket,
            },
        )?;
        for finding in &findings {
            checks::add_finding(tx, check, finding)?;
        }
        Ok(check)
    })?;
    Ok(json!({ "published": check, "findings": findings.len() }))
}

/// A round that blocked: a repair, or `stopped:limit` at `max_rounds`.
pub fn blocked(
    project: &Project,
    loaded: &super::Loaded,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
    blocked: &[checks::Check],
    max_rounds: u32,
    trigger: &checks::Check,
) -> Result<Option<i64>, Fail> {
    let mut findings = Vec::new();
    for check in blocked {
        for finding in project.read(|conn| checks::findings(conn, check.id))? {
            findings.push(json!({ "seat": check.name, "finding": finding }));
        }
    }
    let rounds = attempt.rounds + 1;
    let limit = i64::from(max_rounds) + attempt.extra_rounds;
    project.tx(|tx| {
        // Write only what the read saw: an edit since then owns the next step.
        let current = attempts::get(tx, attempt.id)?;
        if current.state != "live"
            || attempts::superseded_by_edit(tx, attempt.id, ticket.id, ticket.revision)?
            || current.plan_revision() != attempt.plan_revision()
            || current.rounds != attempt.rounds
            || current.choice != attempt.choice
            || current.candidate() != attempt.candidate()
            || current.proof != attempt.proof
        {
            return Ok(None);
        }
        let (base, head) = current.candidate().unwrap_or_default();
        let input = checks::Input {
            attempt: current.id,
            base: base.to_owned(), head: head.to_owned(),
            proof: current.proof.clone().unwrap_or_default(),
            ticket_revision: ticket.revision, plan_revision: current.plan_revision(),
            digest: loaded.review_digest(&current.workflow)?,
        };
        for check in blocked {
            if checks::current(tx, "review", &check.name, &input)?.map(|row| row.id) != Some(check.id)
                || checks::decision(tx, current.id, &checks::evidence(tx, check, &input)?)?.as_deref() == Some("accept") {
                return Ok(None);
            }
        }
        // Only review findings consume a picker. Other repair feedback keeps
        // the gate, landing or pre-publication failure that caused it.
        let selection = if current.admission.as_ref().map(crate::plan::Plan::admitted).transpose()
            .map_err(|error| Fail::refused(error.to_string()))?
            .is_some_and(|plan| plan.finding_selector().is_some()) {
            Some(crate::store::selections::remediation(tx, &current)?)
        } else { None };
        tx.execute(
            "UPDATE attempt SET rounds = ?2 WHERE id = ?1",
            rusqlite::params![attempt.id, rounds],
        )?;
        if rounds >= limit {
            let (base, head) = current.candidate().unwrap_or_default();
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "stopped",
                    reason: "limit",
                    ticket: Some(ticket.id),
                    attempt: Some(attempt.id),
                    execution: blocked.last().map(|check| check.execution),
                    payload: json!({
                        "rounds": rounds, "findings": findings,
                        "target": attempt.target_branch, "choice": attempt.choice, "base": base, "head": head,
                        "proof": current.proof.clone().unwrap_or_default(),
                        "revision": ticket.revision, "plan_revision": attempt.plan_revision(),
                        "gate_digest": loaded.gate_digest,
                        "review_digest": loaded.review_digest(&attempt.workflow)?,
                    }),
                    text: None,
                },
            )?;
            return Ok(None);
        }
        let feedback = if let Some(selection) = selection {
            selection
        } else {
            let source = executions::get(tx, trigger.execution)?;
            json!({"kind": "verdict", "source": {
                "external": "feedback", "execution": trigger.execution, "node": source.node, "output": "result"},
                "value": {"verdict": "fail", "findings": findings}})
        };
        supervise::start_implementation(tx, attempt, "repair", Some(&feedback)).map(Some)
    })
}
