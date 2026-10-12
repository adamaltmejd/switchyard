//! The merge queue: approved candidates in approval order, landed one at a
//! time per target on their own full gate run.

use super::{Loaded, load_target, spawn};
use crate::api::Fail;
use crate::config::Approve;
use crate::daemon::{Daemon, Project};
use crate::git::Merge;
use crate::store::{self, attempts, checks, executions, ticket_name, tickets};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

pub fn landing_dir(project: &Project, landing: i64) -> PathBuf {
    project.local().join("landings").join(landing.to_string())
}

/// Start the next landing for each free target.
pub async fn next(daemon: &Arc<Daemon>, project: &Arc<Project>) -> Result<(), Fail> {
    let mut busy = project.read(|conn| {
        let mut targets = std::collections::HashSet::new();
        for row in executions::query(
            conn,
            "((kind = 'landing' OR parent IN (SELECT id FROM execution WHERE kind = 'landing'))
             AND (status = 'running' OR reserved = 1)) OR intent_state = 'open'",
            [],
        )? {
            let target = match row.intent_ref {
                Some(target) => target,
                None => crate::git::target_ref(attempts::get(conn, row.attempt)?.target_branch()?),
            };
            targets.insert(target);
        }
        Ok(targets)
    })?;
    for approval in project.read(checks::queue)? {
        let attempt = project.read(|conn| attempts::get(conn, approval.attempt))?;
        let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
        let blocked = project.read(|conn| {
            Ok(attempts::open_attention(conn)?
                .iter()
                .any(|item| item.ticket == Some(ticket.id) && item.kind == "red"))
        })?;
        if blocked {
            continue;
        }
        if busy.contains(&crate::git::target_ref(attempt.target_branch()?)) {
            continue;
        }
        let execution = project.tx(|tx| {
            if checks::approval(tx, approval.id)?.state != "active"
                || attempts::superseded_by_edit(
                    tx,
                    attempt.id,
                    ticket.id,
                    approval.ticket_revision,
                )?
            {
                return Ok(None);
            }
            let previous = executions::query(
                tx,
                "kind = 'landing' AND approval = ?1 ORDER BY id DESC LIMIT 1",
                [approval.id],
            )?
            .into_iter()
            .next()
            .filter(|row| matches!(row.outcome.as_deref(), Some("error" | "interrupted")));
            let id = executions::start(
                tx,
                executions::Start {
                    attempt: attempt.id,
                    kind: "landing",
                    reason: Some("queue"),
                    base: Some(&approval.base),
                    head: Some(&approval.head),
                    proof: Some(&approval.proof),
                    ticket_revision: Some(approval.ticket_revision),
                    approval: Some(approval.id),
                    ticket: Some(ticket.id),
                    ..Default::default()
                },
            )?;
            if let Some(previous) = previous {
                tx.execute(
                    "UPDATE execution SET resumed = ?2 WHERE id = ?1",
                    rusqlite::params![id, previous.id],
                )?;
            }
            Ok(Some(id))
        })?;
        if let Some(execution) = execution {
            busy.insert(crate::git::target_ref(attempt.target_branch()?));
            spawn(daemon, project, "landing", execution);
        }
    }
    Ok(())
}

/// Follow existing execution provenance back to the one prepared input.
fn history(
    conn: &rusqlite::Connection,
    row: &executions::Execution,
) -> Result<Vec<executions::Execution>, Fail> {
    let mut rows = Vec::new();
    let mut previous = row.resumed;
    let mut boundary = row.id;
    while let Some(id) = previous {
        let old = executions::get(conn, id)?;
        if id >= boundary || old.kind != "landing" || old.approval != row.approval {
            return Err(Fail::refused("invalid landing continuation provenance"));
        }
        boundary = id;
        previous = old.resumed;
        rows.push(old);
    }
    Ok(rows)
}

fn passing_child(
    child: &executions::Execution,
    row: &executions::Execution,
    target: &str,
    merged: &str,
) -> bool {
    child.kind == "gate"
        && child.outcome.as_deref() == Some("pass")
        && child.result_eligible
        && child.lock_contract
        && child.base.as_deref() == Some(target)
        && child.head.as_deref() == Some(merged)
        && child.proof == row.proof
        && child.ticket_revision == row.ticket_revision
        && child.plan_revision == row.plan_revision
        && child.name.as_ref().is_some_and(|name| child.node.as_deref() == Some(&format!("landing-gate-{name}")))
        && child.inputs.as_ref().is_some_and(|inputs| {
            inputs["merged"]["source"] == json!({"external": "merged", "execution": child.parent})
                && inputs["merged"]["value"] == json!({"base": target, "head": merged, "proof": row.proof.as_deref().unwrap_or_default()})
        })
        && child.result.as_ref().is_some_and(|result| {
            result["infrastructure"]["outcome"] == "completed"
                && result["judgment"]["verdict"] == "pass"
                && result["outputs"]["result"]["value"]["verdict"] == "pass"
        })
}

fn retire_preparation(project: &Project, execution: i64, reason: &str) -> Result<(), Fail> {
    project.tx(|tx| {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: "retired",
                detail: Some(reason),
                ..Default::default()
            },
        )
    })
}

/// Why an approval no longer holds. A superseded one's checks count for
/// nothing; one the current policy would not give keeps them.
pub enum Lapse {
    Superseded(&'static str),
    /// The reason, and the protected paths the candidate touches.
    Policy(String, Vec<String>),
}

impl Lapse {
    fn reason(&self) -> &str {
        match self {
            Lapse::Superseded(reason) => reason,
            Lapse::Policy(reason, _) => reason,
        }
    }
}

/// Whether an approval still holds against current rows and policy.
async fn holds(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    policy: &Loaded,
    approval: &checks::Approval,
    attempt: &attempts::Attempt,
    ticket: &tickets::Ticket,
) -> Result<Option<Lapse>, Fail> {
    let superseded = if approval.state != "active" {
        Some("the approval was withdrawn")
    } else if attempt.state != "live" {
        Some("the attempt ended")
    } else if attempt.target_branch.as_deref() != Some(loaded.branch.as_str()) {
        Some("the target changed")
    } else if attempt.candidate() != Some((approval.base.as_str(), approval.head.as_str()))
        || attempt.proof.clone().unwrap_or_default() != approval.proof
    {
        Some("the candidate changed")
    } else if ticket.revision != approval.ticket_revision {
        Some("the ticket changed")
    } else if approval.plan_revision != attempt.plan_revision() {
        Some("the plan revision changed")
    } else if approval.gate_digest != loaded.gate_digest {
        Some("the gate configuration changed")
    } else if approval.review_digest != loaded.review_digest(&attempt.workflow)? {
        Some("the review configuration changed")
    } else {
        None
    };
    if let Some(reason) = superseded {
        return Ok(Some(Lapse::Superseded(reason)));
    }
    if let Some(decision) = &approval.accepted_skipped
        && (approval.actor != "operator"
            || decision["identity"] != super::skipped_identity(attempt, ticket))
    {
        return Ok(Some(Lapse::Superseded("the skipped selection changed")));
    }
    if project
        .read(|conn| {
            super::selected_checks_current(conn, attempt, ticket, loaded, &approval.checks)
        })?
        .is_some()
    {
        return Ok(Some(Lapse::Superseded("the selected result changed")));
    }
    if approval.actor == "auto" {
        let protected =
            super::protected_paths(daemon, project, policy, &approval.base, &approval.head).await?;
        if !protected.is_empty() {
            let reason = format!(
                "the candidate touches protected paths: {}",
                protected.join(", ")
            );
            return Ok(Some(Lapse::Policy(reason, protected)));
        }
        if policy.config.approve != Approve::Auto {
            return Ok(Some(Lapse::Policy(
                "approve is now manual".into(),
                protected,
            )));
        }
    }
    Ok(None)
}

/// Take a lapsed approval out of the queue. One the policy would no longer
/// give goes to the operator on the checks it already has.
fn lapsed(
    tx: &rusqlite::Connection,
    loaded: &Loaded,
    approval: &checks::Approval,
    attempt: &attempts::Attempt,
    lapse: &Lapse,
) -> Result<(), Fail> {
    let current = checks::approval(tx, approval.id)?;
    if current.state == "active" {
        checks::set_approval_state(
            tx,
            &current,
            "withdrawn",
            attempt.ticket,
            Some(lapse.reason()),
        )?;
    }
    let Lapse::Policy(_, protected) = lapse else {
        return Ok(());
    };
    let plan = match &attempt.admission {
        Some(admission) => crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?,
        None => loaded.config.workflow_plan(&attempt.workflow)?,
    };
    let payload = json!({
        "target": attempt.target_branch, "base": approval.base, "head": approval.head, "proof": approval.proof,
        "revision": approval.ticket_revision, "plan_revision": attempt.plan_revision(),
        "gate_digest": approval.gate_digest, "review_digest": approval.review_digest,
        "checks": approval.checks, "protected": protected,
        "accepted": approval.checks.iter().map(|check| checks::accepted_evidence(tx, attempt.id, *check))
            .collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect::<Vec<_>>(),
        "unreviewed": !plan.nodes.keys().any(|name| name.starts_with("review-")),
        "skipped": attempt.admission.as_ref().map(|admission| &admission["skipped"]),
    });
    // Only for the candidate as it is now: the lapse was read before this
    // transaction, and the attempt may have ended or moved on since.
    let now = attempts::get(tx, attempt.id)?;
    if now.state == "live"
        && super::stale_part(&payload, &now, &tickets::get(tx, now.ticket)?, loaded)?.is_none()
    {
        super::raise_approval(tx, &now, &payload)?;
    }
    Ok(())
}

/// The approval, its attempt and ticket, and the policy, as they are now.
struct Current {
    loaded: Arc<Loaded>,
    approval: checks::Approval,
    attempt: attempts::Attempt,
    ticket: tickets::Ticket,
}

/// Re-read the approval against current rows and the current policy, then
/// run `then` in a transaction that sees the same rows. A lapse, or a row
/// that changed after the read, runs nothing. The caller holds the canonical
/// target lock, so no sync changes the policy between the read and `then`.
async fn admit<T>(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    _target: &tokio::sync::MutexGuard<'_, ()>,
    approval: i64,
    execution: i64,
    then: impl FnOnce(&rusqlite::Transaction, &Current) -> Result<T, Fail>,
) -> Result<Result<T, (Current, Lapse)>, Fail> {
    let (approval, ticket, attempt) = project.read(|conn| {
        let row = executions::get(conn, execution)?;
        let attempt = attempts::get(conn, row.attempt)?;
        Ok((
            checks::approval(conn, approval)?,
            tickets::get(conn, attempt.ticket)?,
            attempt,
        ))
    })?;
    let policy = load_target(daemon, project, attempt.target_branch()?).await?;
    let loaded = super::load_attempt(daemon, project, &attempt).await?;
    let lapse = holds(
        daemon, project, &loaded, &policy, &approval, &attempt, &ticket,
    )
    .await?;
    let now = Current {
        loaded,
        approval,
        attempt,
        ticket,
    };
    // Keep the existing reservation and host lock while publication is absent.
    // Register before reading so validated publication cannot lose its wakeup.
    let mut announced = false;
    let _publication = loop {
        let changed = project.events.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let publication = project.canonical.lock().await;
        let row = project.read(|conn| executions::get(conn, execution))?;
        if row.status != "running" {
            return Err(Fail::refused("execution no longer eligible to act"));
        }
        // An ended attempt lapses its approval; it is not an effect failure.
        if project.read(|conn| Ok(attempts::get(conn, row.attempt)?.state != "live"))? {
            return Ok(Err((now, Lapse::Superseded("the attempt ended"))));
        }
        if project
            .loaded
            .lock()
            .expect("loaded lock")
            .default
            .is_some()
        {
            break publication;
        }
        if !announced {
            project.tx(|tx| {
                store::audit(
                    tx,
                    "execution.waiting",
                    store::Target {
                        ticket: Some(now.ticket.id),
                        attempt: Some(row.attempt),
                        execution: Some(row.id),
                        ..Default::default()
                    },
                    None,
                    json!({ "reason": "capacity" }),
                )
            })?;
            announced = true;
        }
        drop(publication);
        changed.await;
    };
    let admitted = project.tx(|tx| {
        let unchanged = checks::approval(tx, now.approval.id)?.state == now.approval.state
            && !attempts::superseded_by_edit(
                tx,
                now.attempt.id,
                now.ticket.id,
                now.approval.ticket_revision,
            )?
            && attempts::get(tx, now.attempt.id)?.state == now.attempt.state;
        match lapse {
            Some(lapse) => Ok(Err(lapse)),
            None if !unchanged => Ok(Err(Lapse::Superseded("the approval changed"))),
            None => then(tx, &now).map(Ok),
        }
    })?;
    Ok(admitted.map_err(|lapse| (now, lapse)))
}

pub async fn land(
    daemon: &Arc<Daemon>,
    project: &Arc<Project>,
    execution: i64,
) -> Result<(), Fail> {
    let row = project.read(|conn| executions::get(conn, execution))?;
    let approval = project
        .read(|conn| checks::approval(conn, row.approval.expect("a landing binds an approval")))?;
    let attempt = project.read(|conn| attempts::get(conn, row.attempt))?;
    let ticket = project.read(|conn| tickets::get(conn, attempt.ticket))?;
    let canonical = project.canonical_dir();
    let head = approval.head.clone();
    let target_lock = project.target_lock(&crate::git::target_ref(attempt.target_branch()?));

    let predecessors = project.read(|conn| history(conn, &row))?;
    let mut retained = Vec::new();
    let prepared = super::capacity::effect(daemon, project, execution, async {
        let held = target_lock.lock().await;
        let loaded = match admit(daemon, project, &held, approval.id, execution, |_, now| {
            Ok(now.loaded.clone())
        })
        .await?
        {
            Ok(loaded) => loaded,
            Err((now, lapse)) => {
                withdraw(
                    project,
                    &now.loaded,
                    execution,
                    &now.approval,
                    &now.attempt,
                    &lapse,
                )?;
                return Ok(None);
            }
        };
        let target_ref = crate::git::target_ref(&loaded.branch);
        let target = daemon
            .git
            .rev_parse(&canonical, &target_ref)
            .await?
            .ok_or_else(|| Fail::refused("canonical has no target"))?;
        if let Some(original) = predecessors.last().filter(|old| old.prepared.is_some()) {
            let prepared = original.prepared.as_ref().expect("prepared input");
            let old = prepared["old"].as_str().ok_or_else(|| Fail::refused("invalid prepared target"))?;
            let merged = prepared["merged"].as_str().ok_or_else(|| Fail::refused("invalid prepared head"))?;
            // Never consume evidence until every predecessor owner is proved gone.
            let boxes = daemon.pinfold.list(&format!("dev.yard.project={}", project.key)).await?;
            for parent in &predecessors {
                let children = project.read(|conn| executions::query(conn, "parent = ?1 ORDER BY id", [parent.id]))?;
                for owner in std::iter::once(parent).chain(children.iter()) {
                    if owner.status != "ended" || owner.reserved || !owner.lock_contract
                        || boxes.contains(&super::supervise::box_name(project, owner.id))
                        || !super::supervise::lock_free(&super::capacity::dir(project, owner))?
                    {
                        return Err(Fail::refused("landing continuation ownership is unknown"));
                    }
                }
                retained.extend(children);
            }
            let valid = target == old && prepared["target_ref"] == target_ref
                && daemon.git.rev_parse(&canonical, &format!("refs/yard/landings/{}", original.id)).await?.as_deref() == Some(merged)
                && predecessors.iter().all(|old| old.attempt == row.attempt && old.base == row.base
                    && old.head == row.head && old.proof == row.proof && old.ticket_revision == row.ticket_revision
                    && old.plan_revision == row.plan_revision)
                && retained.iter().all(|child| child.outcome.as_deref() != Some("pass")
                    || passing_child(child, &row, old, merged));
            if !valid {
                retire_preparation(project, execution, "prepared_input_changed")?;
                return Ok(None);
            }
            project.tx(|tx| store::audit(tx, "landing.continued", store::Target {
                attempt: Some(row.attempt), execution: Some(execution), ..Default::default()
            }, None, json!({"original": original.id, "prepared": prepared,
                "children": retained.iter().filter(|child| passing_child(child, &row, old, merged)).map(|child| child.id).collect::<Vec<_>>()})))?;
            return Ok(Some((loaded, target_ref, target, merged.to_string())));
        }
        if row.resumed.is_some() {
            retire_preparation(project, execution, "prepared_input_absent")?;
            return Ok(None);
        }
        let merged = if daemon
            .git
            .is_ancestor(&canonical, &target, &head, None)
            .await?
        {
            head.clone()
        } else {
            match daemon.git.merge_tree(&canonical, &target, &head).await? {
                Merge::Conflict { paths } => {
                    returned(
                        daemon,
                        project,
                        &loaded,
                        execution,
                        &target,
                        "conflict",
                        &format!(
                            "The candidate does not merge cleanly. Conflicting paths:\n{}",
                            paths.join("\n")
                        ),
                    )
                    .await?;
                    return Ok(None);
                }
                Merge::Clean { tree } => {
                    let message = format!("Merge {}: {}", ticket_name(ticket.id), ticket.title);
                    daemon
                        .git
                        .commit_tree(&canonical, &tree, &[&target, &head], &message)
                        .await?
                }
            }
        };
        // A crash after pinning must not lose which merge was prepared.
        project.tx(|tx| executions::prepare(tx, execution, &json!({
            "target_ref": target_ref, "old": target, "merged": merged,
        })))?;
        daemon
            .git
            .update_ref(
                &canonical,
                &format!("refs/yard/landings/{execution}"),
                &merged,
                None,
                None,
            )
            .await?;
        Ok(Some((loaded, target_ref, target, merged)))
    })
    .await?;
    let Some((loaded, target_ref, target, merged)) = prepared else {
        return Ok(());
    };

    let plan = match &attempt.admission {
        Some(admission) => crate::plan::Plan::admitted(admission)
            .map_err(|error| Fail::refused(error.to_string()))?,
        None => loaded.config.workflow_plan(&attempt.workflow)?,
    };
    let mut selected = Vec::new();
    let mut current = "implementer";
    while let Some(route) = plan.nodes[current]
        .routes
        .get("complete")
        .and_then(|routes| routes.first())
    {
        current = &route.to;
        let node = &plan.nodes[current];
        if !node
            .inputs
            .values()
            .any(|input| input.node.is_none() && input.output == "merged")
        {
            continue;
        }
        let crate::plan::Runner::Command { gate: name } = &node.runner else {
            return Err(Fail::refused("unsupported admitted landing runner"));
        };
        let gate = loaded
            .config
            .gates
            .iter()
            .find(|gate| &gate.name == name)
            .ok_or_else(|| Fail::refused("admitted landing gate absent"))?;
        if let Some(child) = retained.iter().rev().find(|child| {
            child.name.as_deref() == Some(&gate.name)
                && passing_child(child, &row, &target, &merged)
        }) {
            let admitted = super::capacity::effect(daemon, project, execution, async {
                let held = target_lock.lock().await;
                admit(daemon, project, &held, approval.id, execution, |tx, _| {
                    if !passing_child(&executions::get(tx, child.id)?, &row, &target, &merged) {
                        return Err(Fail::refused("prepared child evidence changed"));
                    }
                    Ok(())
                })
                .await
            })
            .await?;
            if let Err((now, lapse)) = admitted {
                return withdraw(
                    project,
                    &now.loaded,
                    execution,
                    &now.approval,
                    &now.attempt,
                    &lapse,
                );
            }
            selected.push(child.id);
            continue;
        }
        let mut pending_child = None;
        let admitted = super::capacity::effect(daemon, project, execution, async {
            let held = target_lock.lock().await;
            let admitted = admit(daemon, project, &held, approval.id, execution, |tx, now| {
                executions::start(
                    tx,
                    executions::Start {
                        attempt: attempt.id,
                        parent: Some(execution),
                        kind: "gate",
                        reason: Some("landing"),
                        base: Some(&target),
                        head: Some(&merged),
                        proof: Some(&now.approval.proof),
                        ticket_revision: Some(now.ticket.revision),
                        digest: Some(&now.loaded.gate_digest),
                        name: Some(&gate.name),
                        ticket: Some(ticket.id),
                        ..Default::default()
                    },
                )
            })
            .await?;
            if let Ok(child) = &admitted {
                pending_child = Some(*child);
            }
            Ok(admitted)
        })
        .await;
        let admitted = match admitted {
            Ok(admitted) => admitted,
            Err(fail) => {
                // Admission committed, but the parent's checked release can
                // refuse before the child controller is ever started.
                if let Some(child) = pending_child {
                    project.tx(|tx| {
                        if executions::get(tx, child)?.status == "running" {
                            executions::end(
                                tx,
                                child,
                                executions::End {
                                    outcome: "interrupted",
                                    detail: Some(&fail.message),
                                    ticket: Some(ticket.id),
                                    ..Default::default()
                                },
                            )?;
                        }
                        Ok(())
                    })?;
                }
                return Err(fail);
            }
        };
        let child = match admitted {
            Ok(child) => child,
            Err((now, lapse)) => {
                return withdraw(
                    project,
                    &now.loaded,
                    execution,
                    &now.approval,
                    &now.attempt,
                    &lapse,
                );
            }
        };
        let child_result = super::capacity::effect(
            daemon,
            project,
            child,
            super::supervise::gate(daemon, project, child),
        )
        .await;
        if let Err(fail) = child_result {
            project.tx(|tx| {
                if executions::get(tx, child)?.status == "running" {
                    executions::end(
                        tx,
                        child,
                        executions::End {
                            outcome: if attempts::get(tx, attempt.id)?.state == "live" {
                                "error"
                            } else {
                                "abandoned"
                            },
                            detail: Some(&fail.message),
                            ticket: Some(ticket.id),
                            ..Default::default()
                        },
                    )?;
                }
                Ok(())
            })?;
            return Err(fail);
        }
        let result = project.read(|conn| executions::get(conn, child))?;
        match result.outcome.as_deref() {
            Some("pass") if result.node.is_none() || result.result_eligible => {
                selected.push(child);
            }
            Some("abandoned") => {
                return project.tx(|tx| {
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: "withdrawn",
                            ticket: Some(ticket.id),
                            ..Default::default()
                        },
                    )
                });
            }
            Some("fail") => {
                let detail = format!(
                    "The gate {} failed on the merged ref {merged}:\n{}",
                    gate.name,
                    result.detail.unwrap_or_default()
                );
                return super::capacity::effect(
                    daemon,
                    project,
                    execution,
                    returned(daemon, project, &loaded, execution, &target, "red", &detail),
                )
                .await;
            }
            _ => {
                let detail = format!(
                    "The gate {} could not run: {}",
                    gate.name,
                    result.detail.unwrap_or_default()
                );
                return project.tx(|tx| {
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: "error",
                            detail: Some(&detail),
                            ticket: Some(ticket.id),
                            ..Default::default()
                        },
                    )?;
                    // An edit withdrew this approval and returned the attempt already.
                    if tickets::get(tx, ticket.id)?.revision != approval.ticket_revision {
                        return Ok(());
                    }
                    attempts::raise(
                        tx,
                        attempts::Raise {
                            kind: "red",
                            reason: "landing",
                            ticket: Some(ticket.id),
                            attempt: None,
                            execution: Some(execution),
                            payload: json!({ "detail": detail, "gate": gate.name }),
                            text: Some(&detail),
                        },
                    )?;
                    Ok(())
                });
            }
        }
    }

    // Keep target serialization through the effect's ownership proof, but
    // never hold the target while waiting for an execution slot.
    let mut held = None;
    let result = super::capacity::effect(daemon, project, execution, async {
        let held = held.insert(target_lock.lock().await);
        let original = predecessors.last().map_or(execution, |row| row.id);
        if daemon
            .git
            .rev_parse(&canonical, &format!("refs/yard/landings/{original}"))
            .await?
            .as_deref()
            != Some(&merged)
        {
            return retire_preparation(project, execution, "prepared_ref_changed");
        }
        let admitted = admit(daemon, project, held, approval.id, execution, |tx, now| {
            for child in &selected {
                let child = executions::get(tx, *child)?;
                if child.node.is_some() && !passing_child(&child, &row, &target, &merged) {
                    return Err(Fail::refused("prepared child evidence changed"));
                }
            }
            tx.execute(
                "UPDATE execution SET inputs = ?2 WHERE id = ?1",
                rusqlite::params![execution, json!({"children": selected}).to_string()],
            )?;
            executions::set_intent(
                tx,
                execution,
                &target_ref,
                &target,
                &merged,
                now.ticket.id,
                now.attempt.id,
            )?;
            let mut configs = project.loaded.lock().expect("loaded lock");
            if configs.default.as_deref() == Some(now.loaded.branch.as_str()) {
                configs.default = None;
            }
            Ok(())
        })
        .await?;
        if let Err((now, lapse)) = admitted {
            return withdraw(
                project,
                &now.loaded,
                execution,
                &now.approval,
                &now.attempt,
                &lapse,
            );
        }
        let moved = daemon
            .git
            .update_ref(&canonical, &target_ref, &merged, Some(&target), None)
            .await;
        match moved {
            Ok(false) => {
                let published = project.canonical.lock().await;
                let _ =
                    super::publish_default(daemon, project, &published, &loaded.branch, held).await;
                // The target moved under the landing: retired silently, re-queued.
                return project.tx(|tx| {
                    executions::resolve_intent(tx, execution)?;
                    executions::end(
                        tx,
                        execution,
                        executions::End {
                            outcome: "retired",
                            ticket: Some(ticket.id),
                            ..Default::default()
                        },
                    )
                });
            }
            Ok(true) => {}
            Err(_) => {
                // Command success is never read as a landing: decide from canonical.
                return Ok(());
            }
        }
        if !daemon
            .git
            .is_ancestor(&canonical, &head, &target_ref, None)
            .await?
        {
            return Err(Fail::new(
                "internal",
                format!("canonical does not contain {head} after the landing"),
            ));
        }
        let published = project.canonical.lock().await;
        let _ = super::publish_default(daemon, project, &published, &loaded.branch, held).await;
        project.tx(|tx| record_landing(tx, execution))
    })
    .await;
    let row = project.read(|conn| executions::get(conn, execution))?;
    if row.reserved {
        let _published = project.canonical.lock().await;
        let mut configs = project.loaded.lock().expect("loaded lock");
        if configs.default.as_deref() == Some(loaded.branch.as_str()) {
            configs.default = None;
        }
    }
    drop(held);
    if row.intent_state.as_deref() == Some("open") {
        super::reconcile::decide_intent(daemon, project, &row).await?;
    }
    result
}

/// Record a landing once: close the ticket, end the attempt, resolve the
/// intent.
pub fn record_landing(tx: &rusqlite::Connection, execution: i64) -> Result<(), Fail> {
    let row = executions::get(tx, execution)?;
    // Recovery and the live controller may both finish external probes.
    // Only the transaction that still sees the open intent records it.
    if row.intent_state.as_deref() != Some("open") {
        return Ok(());
    }
    let attempt = attempts::get(tx, row.attempt)?;
    let approval = checks::approval(tx, row.approval.expect("a landing binds an approval"))?;
    executions::resolve_intent(tx, execution)?;
    if row.status == "running" {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: "landed",
                ticket: Some(attempt.ticket),
                ..Default::default()
            },
        )?;
    } else {
        tx.execute(
            "UPDATE execution SET outcome = 'landed' WHERE id = ?1",
            [execution],
        )?;
    }
    checks::set_approval_state(tx, &approval, "landed", attempt.ticket, None)?;
    attempts::end(tx, attempt.id, "landed")?;
    tx.execute(
        "UPDATE ticket SET state = 'done', closed_at = ?2, close_reason = 'landed' WHERE id = ?1",
        rusqlite::params![attempt.ticket, store::now()],
    )?;
    store::audit(
        tx,
        "landing.recorded",
        store::Target {
            ticket: Some(attempt.ticket),
            attempt: Some(attempt.id),
            execution: Some(execution),
            attention: None,
        },
        None,
        json!({ "target": attempt.target_branch, "old": row.intent_old, "merged": row.intent_merged, "head": row.head }),
    )?;
    store::audit(
        tx,
        "ticket.done",
        store::Target {
            ticket: Some(attempt.ticket),
            ..Default::default()
        },
        None,
        json!({ "by": "landing" }),
    )?;
    super::audit_attempt(
        tx,
        "attempt.ended",
        &attempt,
        None,
        json!({ "outcome": "landed" }),
    )?;
    Ok(())
}

/// End a landing whose approval lapsed.
fn withdraw(
    project: &Project,
    loaded: &Loaded,
    execution: i64,
    approval: &checks::Approval,
    attempt: &attempts::Attempt,
    lapse: &Lapse,
) -> Result<(), Fail> {
    project.tx(|tx| {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome: "withdrawn",
                detail: Some(lapse.reason()),
                ticket: Some(attempt.ticket),
                ..Default::default()
            },
        )?;
        lapsed(tx, loaded, approval, attempt, lapse)
    })
}

/// A red or conflicting landing is the candidate's: one repair with the
/// target made available, and a second consecutive red raises `red`.
async fn returned(
    daemon: &Daemon,
    project: &Project,
    loaded: &Loaded,
    execution: i64,
    target: &str,
    outcome: &str,
    detail: &str,
) -> Result<(), Fail> {
    let (attempt, approval) = project.read(|conn| {
        let row = executions::get(conn, execution)?;
        let approval = checks::approval(conn, row.approval.expect("a landing binds an approval"))?;
        Ok((attempts::get(conn, row.attempt)?, approval))
    })?;
    let ticket = attempt.ticket;
    let input = project.attempt_dir(attempt.id).join("input");
    std::fs::create_dir_all(&input).map_err(|error| error.to_string())?;
    let bundle = input.join("target.bundle");
    let _ = std::fs::remove_file(&bundle);
    if target != attempt.base {
        daemon
            .git
            .run(
                &project.canonical_dir(),
                &[
                    "bundle",
                    "create",
                    "--quiet",
                    bundle.to_str().unwrap_or_default(),
                    &crate::git::target_ref(&loaded.branch),
                    &format!("^{}", attempt.base),
                ],
            )
            .await?;
    }
    project.tx(|tx| {
        executions::end(
            tx,
            execution,
            executions::End {
                outcome,
                detail: Some(detail),
                ticket: Some(ticket),
                ..Default::default()
            },
        )?;
        checks::set_approval_state(tx, &approval, "retired", ticket, Some(outcome))?;
        // An edit since the landing began owns the next step.
        if attempts::superseded_by_edit(tx, attempt.id, ticket, approval.ticket_revision)? {
            return Ok(());
        }
        let reds = if outcome == "red" {
            attempt.landing_reds + 1
        } else {
            attempt.landing_reds
        };
        tx.execute(
            "UPDATE attempt SET target = ?2, landing_reds = ?3 WHERE id = ?1",
            rusqlite::params![attempt.id, target, reds],
        )?;
        if reds >= 2 {
            attempts::raise(
                tx,
                attempts::Raise {
                    kind: "red",
                    reason: "landing",
                    ticket: Some(ticket),
                    attempt: Some(attempt.id),
                    execution: Some(execution),
                    payload: json!({ "detail": detail, "target": target }),
                    text: Some(detail),
                },
            )?;
            return Ok(());
        }
        attempts::set_next(
            tx,
            attempt.id,
            Some(&json!({
                "reason": "repair", "detail": detail, "target": target, "branch": loaded.branch,
            })),
        )
    })
}
