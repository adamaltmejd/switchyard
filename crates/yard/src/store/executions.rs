use super::{Target, audit, now};
use crate::api::Fail;
use crate::harness::{Usage, UsageFields};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct Execution {
    pub id: i64,
    pub attempt: Option<i64>,
    pub scope: Option<Value>,
    pub workspace: Option<String>,
    pub settings: Option<Value>,
    pub parent: Option<i64>,
    pub kind: String,
    pub reason: Option<String>,
    pub status: String,
    pub outcome: Option<String>,
    pub detail: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub proof: Option<String>,
    pub ticket_revision: Option<i64>,
    pub plan_revision: Option<i64>,
    pub node: Option<String>,
    pub inputs: Option<Value>,
    pub result: Option<Value>,
    pub report: Option<Value>,
    pub teardown: Option<bool>,
    pub result_eligible: bool,
    pub name: Option<String>,
    pub round: Option<i64>,
    pub handle: Option<String>,
    pub session_id: Option<String>,
    pub approval: Option<i64>,
    pub prepared: Option<Value>,
    pub verification: Option<Value>,
    pub intent_old: Option<String>,
    pub intent_ref: Option<String>,
    pub intent_merged: Option<String>,
    pub intent_state: Option<String>,
    pub progress: Option<String>,
    pub harness_version: Option<String>,
    pub resumed: Option<i64>,
    pub reserved: bool,
    pub lock_contract: bool,
    pub usage: Usage,
    pub harness: Option<String>,
    pub model: Option<String>,
}

const COLUMNS: &str = "id, attempt, parent, kind, reason, status, outcome, detail, base, head,
    proof, ticket_revision, name, round, handle, session_id, approval, intent_old, intent_merged,
    intent_state, progress, harness_version, resumed, reserved, lock_contract, intent_ref, plan_revision,
    tokens_in, tokens_out, cost, usage_fields, harness, model, node, inputs, result, teardown, report, prepared, verification, scope, workspace, settings";

pub const RESULT_ELIGIBLE: &str = "status = 'ended' AND reserved = 0 AND teardown = 1
    AND json_extract(result, '$.outputs.result') IS NOT NULL";

fn json_column(row: &Row, index: usize) -> rusqlite::Result<Option<Value>> {
    row.get::<_, Option<String>>(index)?
        .map(|text| {
            serde_json::from_str(&text).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    index,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .transpose()
}

fn row(row: &Row) -> rusqlite::Result<Execution> {
    Ok(Execution {
        id: row.get(0)?,
        attempt: row.get(1)?,
        parent: row.get(2)?,
        kind: row.get(3)?,
        reason: row.get(4)?,
        status: row.get(5)?,
        outcome: row.get(6)?,
        detail: row.get(7)?,
        base: row.get(8)?,
        head: row.get(9)?,
        proof: row.get(10)?,
        ticket_revision: row.get(11)?,
        name: row.get(12)?,
        round: row.get(13)?,
        handle: row.get(14)?,
        session_id: row.get(15)?,
        approval: row.get(16)?,
        intent_old: row.get(17)?,
        intent_merged: row.get(18)?,
        intent_state: row.get(19)?,
        progress: row.get(20)?,
        harness_version: row.get(21)?,
        resumed: row.get(22)?,
        reserved: row.get(23)?,
        lock_contract: row.get(24)?,
        intent_ref: row.get(25)?,
        plan_revision: row.get(26)?,
        usage: Usage {
            input: token_count(row, 27)?,
            output: token_count(row, 28)?,
            cost: row.get(29)?,
            fields: row
                .get::<_, Option<String>>(30)?
                .map(|raw| serde_json::from_str::<UsageFields>(&raw))
                .transpose()
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        30,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?
                .unwrap_or_default(),
        },
        harness: row.get(31)?,
        model: row.get(32)?,
        node: row.get(33)?,
        inputs: json_column(row, 34)?,
        result: json_column(row, 35)?,
        teardown: row.get(36)?,
        report: json_column(row, 37)?,
        prepared: json_column(row, 38)?,
        verification: json_column(row, 39)?,
        scope: json_column(row, 40)?,
        workspace: row.get(41)?,
        settings: json_column(row, 42)?,
        result_eligible: row.get(43)?,
    })
}

fn token_count(row: &Row, index: usize) -> rusqlite::Result<Option<u64>> {
    row.get::<_, Option<i64>>(index)?
        .map(u64::try_from)
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                index,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })
}

impl Execution {
    pub fn required_attempt(&self) -> Result<i64, Fail> {
        if self.scope.is_some() {
            return Err(Fail::refused("standalone_authority"));
        }
        self.attempt.ok_or_else(|| Fail::refused("missing_attempt"))
    }

    pub fn eligible_to_act(&self, conn: &Connection) -> Result<bool, Fail> {
        Ok(self.status == "running"
            && (self.scope.is_some()
                || self.kind == "cleanup"
                || super::attempts::get(conn, self.required_attempt()?)?.state == "live"))
    }

    pub fn to_json(&self) -> Value {
        json!({
            "execution": self.id,
            "attempt": self.attempt,
            "workspace": self.workspace,
            "settings": self.settings,
            "session_id": self.session_id,
            "scope": self.scope.clone().unwrap_or_else(|| json!({"status": "unknown"})),
            "parent": self.parent,
            "kind": self.kind,
            "reason": self.reason,
            "status": self.status,
            "reserved": self.reserved,
            "outcome": self.outcome,
            "detail": self.detail,
            "base": self.base,
            "head": self.head,
            "proof": self.proof,
            "ticket_revision": self.ticket_revision,
            "plan_revision": self.plan_revision,
            "binding": self.node.as_ref().map_or_else(|| json!({"status": "unknown"}), |node| json!({
                "status": "recorded", "node": node, "inputs": self.inputs,
            })),
            "result": self.result,
            "report": self.report,
            "teardown": self.teardown,
            "result_eligible": self.result_eligible,
            "name": self.name,
            "round": self.round,
            "usage": self.usage,
            "harness": self.harness,
            "harness_version": self.harness_version,
            "model": self.model,
            "resumed": self.resumed,
            "prepared": self.prepared,
            "verification": self.verification.clone().unwrap_or_else(|| json!({"status": "unknown"})),
            "intent": self.intent_state.as_ref().map(|state| json!({
                "state": state, "target_ref": self.intent_ref, "old": self.intent_old, "merged": self.intent_merged,
            })),
        })
    }
}

pub fn get(conn: &Connection, id: i64) -> Result<Execution, Fail> {
    conn.query_row(
        &format!("SELECT {COLUMNS}, COALESCE(({RESULT_ELIGIBLE}), 0) FROM execution WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(|| Fail::not_found(format!("execution {id} does not exist")))
}

pub fn running(conn: &Connection) -> Result<Vec<Execution>, Fail> {
    query(conn, "status = 'running' ORDER BY id", [])
}

pub fn for_attempt(conn: &Connection, attempt: i64) -> Result<Vec<Execution>, Fail> {
    query(conn, "attempt = ?1 ORDER BY id", [attempt])
}

/// Optional standalone attachment carries no attempt lifecycle authority.
pub fn attempt_work(conn: &Connection, attempt: i64) -> Result<Vec<Execution>, Fail> {
    query(
        conn,
        "attempt = ?1 AND scope IS NULL ORDER BY id",
        [attempt],
    )
}

/// The attempt's session to resume: the most recent implementation before
/// `current` that reported a session id, and how many executions have run on
/// that session. A run that began fresh (`resumed` is NULL) is a boundary: an
/// older session is not resumed across it. The caller checks the session
/// owner's version, so a harness version change also ends continuity.
pub fn session(
    conn: &Connection,
    attempt: i64,
    current: i64,
) -> Result<Option<(Execution, i64)>, Fail> {
    let mut owner: Option<Execution> = None;
    for execution in query(
        conn,
        "attempt = ?1 AND kind = 'implementation' AND id < ?2
            AND workspace IS (SELECT workspace FROM execution WHERE id = ?2) ORDER BY id DESC",
        params![attempt, current],
    )? {
        if execution.session_id.is_some() {
            owner = Some(execution);
            break;
        }
        // A run that reported no session but began fresh broke the chain:
        // an older session is not resumed across it.
        if execution.resumed.is_none() {
            break;
        }
    }
    let Some(owner) = owner else {
        return Ok(None);
    };
    let active = get(conn, current)?;
    if owner.reserved
        || owner.status != "ended"
        || owner.teardown != Some(true)
        || owner.settings != active.settings
        || owner.settings.is_none()
    {
        return Ok(None);
    }
    let mut count = 0;
    let mut next = Some(owner.id);
    while let Some(id) = next {
        count += 1;
        next = conn.query_row("SELECT resumed FROM execution WHERE id = ?1", [id], |row| {
            row.get(0)
        })?;
    }
    Ok(Some((owner, count)))
}

/// Bind the checkpoint while this execution holds the existing reservation.
/// Only checked interrupted ownership can supply a continuation. Guest bytes
/// never determine either the identity or the owner.
pub fn durable_checkpoint(conn: &Connection, current: i64, identity: Value) -> Result<Value, Fail> {
    let row = get(conn, current)?;
    if !row.reserved || row.status != "running" || row.prepared.is_some() {
        return Err(Fail::refused(format!(
            "checkpoint_owned: execution {current}"
        )));
    }
    let previous = query(
        conn,
        "attempt = ?1 AND kind = 'implementation' AND id < ?2 ORDER BY id DESC LIMIT 1",
        params![row.attempt, current],
    )?
    .pop();
    let previous = previous.filter(|previous| {
        previous.status == "ended"
            && previous.outcome.as_deref() == Some("interrupted")
            && !previous.reserved
            && previous.teardown == Some(true)
            && previous.report.is_none()
            && previous
                .prepared
                .as_ref()
                .is_some_and(|prepared| prepared["durable"]["identity"] == identity)
    });
    let checkpoint = previous
        .as_ref()
        .and_then(|previous| previous.prepared.as_ref()?["durable"]["checkpoint"].as_i64())
        .unwrap_or(current);
    if let Some(owner) = query(
        conn,
        "reserved = 1 AND id != ?1 AND json_extract(prepared, '$.durable.checkpoint') = ?2 LIMIT 1",
        params![current, checkpoint],
    )?
    .pop()
    {
        return Err(Fail::refused(format!(
            "checkpoint_owned: checkpoint execution {checkpoint} owned by execution {}",
            owner.id
        )));
    }
    let prepared = json!({"durable": {"checkpoint": checkpoint,
        "predecessor": previous.as_ref().map(|previous| previous.id), "identity": identity}});
    conn.execute(
        "UPDATE execution SET prepared = ?2, resumed = ?3 WHERE id = ?1",
        params![
            current,
            prepared.to_string(),
            previous.map(|previous| previous.id)
        ],
    )?;
    Ok(prepared["durable"].clone())
}

/// Landings whose intent is still open.
pub fn open_intents(conn: &Connection) -> Result<Vec<Execution>, Fail> {
    query(
        conn,
        "kind = 'landing' AND intent_state = 'open' ORDER BY id",
        [],
    )
}

pub fn query<P: rusqlite::Params>(
    conn: &Connection,
    filter: &str,
    params: P,
) -> Result<Vec<Execution>, Fail> {
    super::all(
        conn,
        &format!(
            "SELECT {COLUMNS}, COALESCE(({RESULT_ELIGIBLE}), 0) FROM execution WHERE {filter}"
        ),
        params,
        row,
    )
}

/// What an execution is started on.
#[derive(Default)]
pub struct Start<'a> {
    pub attempt: Option<i64>,
    pub scope: Option<&'a Value>,
    pub node: Option<&'a str>,
    pub workspace: Option<&'a str>,
    pub parent: Option<i64>,
    pub kind: &'a str,
    pub reason: Option<&'a str>,
    pub base: Option<&'a str>,
    pub head: Option<&'a str>,
    pub proof: Option<&'a str>,
    pub ticket_revision: Option<i64>,
    pub digest: Option<&'a str>,
    pub name: Option<&'a str>,
    pub round: Option<i64>,
    pub approval: Option<i64>,
    pub ticket: Option<i64>,
    /// Worker settings: agent name and its `{harness, provider login, model,
    /// effort}`.
    pub agent: Option<(&'a str, &'a Value)>,
    /// The pinfold pin of the worker's harness, recorded with the intent.
    pub harness_version: Option<&'a str>,
    pub feedback: Option<&'a Value>,
}

/// The intent: the row exists before any effect.
pub fn start(tx: &Connection, start: Start) -> Result<i64, Fail> {
    let attempt = start
        .attempt
        .map(|id| super::attempts::get(tx, id))
        .transpose()?;
    if start.scope.is_some() {
        if !matches!(start.kind, "command" | "report")
            || start.approval.is_some()
            || start.parent.is_some()
            || start.proof.is_some()
            || start.ticket_revision.is_some()
            || start.digest.is_some()
            || start.ticket.is_some()
        {
            return Err(Fail::refused("standalone_authority"));
        }
    } else if attempt.is_none() || matches!(start.kind, "command" | "report") {
        return Err(Fail::refused("missing_attempt"));
    }
    let digest = start
        .digest
        .map(|digest| {
            check_digest(
                attempt
                    .as_ref()
                    .ok_or_else(|| Fail::refused("missing_attempt"))?,
                start.kind,
                start.name.unwrap_or_default(),
                digest,
            )
        })
        .transpose()?;
    if let Some(workspace) = start.workspace
        && let Some(owner) = query(tx, "attempt = ?1 AND kind = 'implementation'
            AND (workspace = ?2 OR workspace IS NULL) AND (status = 'running' OR reserved = 1) LIMIT 1",
            params![start.attempt, workspace])?.pop() {
            return Err(Fail::refused("workspace_owned").with(json!({"reason": "workspace_owned",
                "workspace": workspace, "execution": owner.id})));
    }
    let settings = start.agent.map(|(_, settings)| settings);
    let field = |key: &str| settings.and_then(|value| value[key].as_str().map(str::to_string));
    tx.execute(
        "INSERT INTO execution (attempt, parent, kind, reason, status, base, head, proof, ticket_revision,
            digest, name, round, approval, agent, harness, harness_version, provider, model, effort, started_at)
         VALUES (?1, ?2, ?3, ?4, 'running', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            start.attempt,
            start.parent,
            start.kind,
            start.reason,
            start.base,
            start.head,
            start.proof,
            start.ticket_revision,
            digest,
            start.name,
            start.round,
            start.approval,
            start.agent.map(|(name, _)| name),
            field("harness"),
            start.harness_version,
            field("provider"),
            field("model"),
            field("effort"),
            now()
        ],
    )?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE execution SET workspace = ?2, settings = ?3 WHERE id = ?1",
        params![id, start.workspace, settings.map(Value::to_string)],
    )?;
    let revision = attempt.as_ref().and_then(|attempt| attempt.plan_revision());
    let binding = if start.scope.is_some() {
        None
    } else {
        attempt
            .as_ref()
            .map(|attempt| binding(tx, attempt, &start))
            .transpose()?
            .flatten()
    };
    if attempt
        .as_ref()
        .is_some_and(|attempt| attempt.admission.is_some())
        && matches!(start.kind, "implementation" | "gate" | "review")
        && binding.is_none()
    {
        return Err(Fail::refused("node input has no eligible producer"));
    }
    tx.execute(
        "UPDATE execution SET plan_revision = ?2, node = ?3, inputs = ?4, teardown = ?5 WHERE id = ?1",
        params![id, revision, binding.as_ref().map(|(node, _)| node),
            binding.as_ref().map(|(_, inputs)| inputs.to_string()).or_else(|| start.feedback.map(|feedback| json!({"feedback": feedback}).to_string())), binding.as_ref().map(|_| false)],
    )?;
    if let Some(scope) = start.scope {
        tx.execute("UPDATE execution SET scope = ?2, node = 'standalone', inputs = ?3, teardown = 0, plan_revision = NULL WHERE id = ?1",
            params![id, scope.to_string(), scope["input"].to_string()])?;
    }
    audit(
        tx,
        "execution.started",
        Target {
            ticket: start.ticket,
            attempt: start.attempt,
            execution: Some(id),
            attention: None,
        },
        None,
        json!({ "kind": start.kind, "reason": start.reason, "name": start.name,
                "head": start.head, "proof": start.proof, "parent": start.parent }),
    )?;
    Ok(id)
}

/// The definitions this runner consumes, from recorded source evidence.
/// Commit labels are provenance; blobs, modes and the whole image tree are inputs.
pub fn consumed_definition(
    admission: &Value,
    runner: &crate::plan::Runner,
    reviewer: bool,
) -> Option<Value> {
    let inputs = &admission["inputs"];
    let image = inputs.get("build_context_tree")?.as_str()?;
    let dockerfile = inputs.get("dockerfile_blob")?.as_str()?;
    let yard = admission.get("yard_runner")?;
    yard.get("sha256")?.as_str()?;
    let mut definition = json!({"runner": runner, "yard_runner": yard,
        "build_context_tree": image, "dockerfile_blob": dockerfile});
    match runner {
        crate::plan::Runner::Command { gate } => {
            definition["gate"] = admission["gates"]
                .as_array()?
                .iter()
                .find(|definition| definition["name"] == *gate)?
                .clone();
            if definition["gate"]["runs_in"] == "host" {
                let definition = definition.as_object_mut()?;
                definition.remove("build_context_tree");
                definition.remove("dockerfile_blob");
            }
        }
        crate::plan::Runner::Agent { agent, .. } | crate::plan::Runner::Packaged { agent, .. } => {
            let settings = admission["agents"].get(agent)?;
            let harness = settings["settings"]["harness"].as_str()?;
            settings["runner"]["version"].as_str()?;
            let staged = inputs.get("trusted_staged_sources")?.as_array()?;
            let packaged = matches!(runner, crate::plan::Runner::Packaged { .. });
            let sources: Vec<_> = staged
                .iter()
                .filter(|source| {
                    let path = source["path"].as_str().unwrap_or_default();
                    packaged || crate::agent_env::consumed_path(harness, path)
                })
                .collect();
            definition["agent"] = settings.clone();
            definition["trusted_staged_sources"] = json!(sources);
            if reviewer {
                definition["review"] = admission.get("review")?.clone();
            }
            if packaged {
                definition["package"] = inputs.get("package")?.clone();
                if let Some(contract) = admission.get("package_launch") {
                    definition["package_launch"] = contract.clone();
                }
                if let Some(contract) = admission.get("package_checkpoint") {
                    definition["package_checkpoint"] = contract.clone();
                }
            }
        }
    }
    Some(definition)
}

/// Whole gate/panel identities bind decisions. A check binds its own runner.
pub fn check_digest(
    attempt: &super::attempts::Attempt,
    kind: &str,
    name: &str,
    baseline: &str,
) -> Result<String, Fail> {
    if !matches!(kind, "gate" | "review") {
        return Ok(baseline.into());
    }
    let Some(admission) = &attempt.admission else {
        return Ok(baseline.into());
    };
    let plan =
        crate::plan::Plan::admitted(admission).map_err(|error| Fail::refused(error.to_string()))?;
    let runner = if kind == "gate" {
        crate::plan::Runner::Command { gate: name.into() }
    } else {
        let index = admission["workflow"]["review"]
            .as_array()
            .and_then(|seats| seats.iter().position(|seat| seat.as_str() == Some(name)))
            .ok_or_else(|| Fail::refused("seat absent from admitted panel"))?;
        let Some(node) = plan.nodes.get(&format!("review-{index}")) else {
            return Ok(baseline.into());
        };
        node.runner.clone()
    };
    let Some(definition) = consumed_definition(admission, &runner, kind == "review") else {
        // Unrecorded source proof stays unknown; its historical digest stays intact.
        if kind != "review" {
            return Ok(baseline.into());
        }
        let crate::plan::Runner::Agent {
            agent,
            instructions,
        } = &runner
        else {
            return Err(Fail::refused("review has no admitted agent runner"));
        };
        if admission["seats"][name]["agent"] == *agent
            && admission["seats"][name]["instructions"] == *instructions
        {
            return Ok(baseline.into());
        }
        let settings = &admission["agents"][agent]["settings"];
        if settings.is_null() {
            return Err(Fail::refused("selected reviewer has no admitted settings"));
        }
        let policy = serde_json::to_vec(&json!({"runner": runner, "settings": settings}))
            .expect("review policy serializes");
        return Ok(crate::config::digest(&[
            b"selected-review",
            baseline.as_bytes(),
            &policy,
        ]));
    };
    let definition = serde_json::to_vec(&definition).expect("consumer definition serializes");
    Ok(crate::config::digest(&[b"consumer", &definition]))
}

/// Merge, cleanup and old history have no graph node.
pub fn binding(
    conn: &Connection,
    attempt: &super::attempts::Attempt,
    start: &Start,
) -> Result<Option<(String, Value)>, Fail> {
    let Some(admission) = &attempt.admission else {
        return Ok(None);
    };
    let node = if let Some(node) = start.node {
        node.to_string()
    } else {
        match start.kind {
            "implementation" => "implementer".to_string(),
            "gate" => format!(
                "{}-gate-{}",
                if start.parent.is_some() {
                    "landing"
                } else {
                    "candidate"
                },
                start.name.unwrap_or_default()
            ),
            "review" => {
                let seats = admission["workflow"]["review"]
                    .as_array()
                    .ok_or_else(|| Fail::refused("invalid admitted panel"))?;
                let index = seats
                    .iter()
                    .position(|seat| seat.as_str() == start.name)
                    .ok_or_else(|| Fail::refused("seat absent from admitted panel"))?;
                format!("review-{index}")
            }
            _ => return Ok(None),
        }
    };
    let plan =
        crate::plan::Plan::admitted(admission).map_err(|error| Fail::refused(error.to_string()))?;
    if plan.writers() {
        let declared = plan
            .nodes
            .get(&node)
            .ok_or_else(|| Fail::refused("undeclared_workspace"))?;
        let mut inputs = serde_json::Map::new();
        for (name, reference) in &declared.inputs {
            let Some(source) = reference.node.as_deref() else {
                if reference.output != "merged" || start.parent.is_none() {
                    return Ok(None);
                }
                inputs.insert(name.clone(), json!({"kind": "candidate",
                    "source": {"external": "merged", "execution": start.parent},
                    "value": {"base": start.base, "head": start.head, "proof": start.proof.unwrap_or_default()}}));
                continue;
            };
            let Some(producer) = query(
                conn,
                "attempt = ?1 AND node = ?2 AND plan_revision = ?3
                AND ticket_revision = ?4 ORDER BY id DESC LIMIT 1",
                params![
                    attempt.id,
                    source,
                    attempt.plan_revision(),
                    start.ticket_revision
                ],
            )?
            .pop() else {
                return Ok(None);
            };
            if !producer.result_eligible {
                return Ok(None);
            }
            if reference.kind == crate::plan::Output::Verdict
                && binding(
                    conn,
                    attempt,
                    &Start {
                        kind: "review",
                        node: Some(source),
                        base: producer.base.as_deref(),
                        head: producer.head.as_deref(),
                        proof: producer.proof.as_deref(),
                        ticket_revision: producer.ticket_revision,
                        ..Default::default()
                    },
                )?
                .map(|(_, inputs)| inputs)
                    != producer.inputs
            {
                return Ok(None);
            }
            if reference.kind == crate::plan::Output::Selection
                && attempt.choice != Some(producer.id)
            {
                return Ok(None);
            }
            let output =
                &producer.result.as_ref().expect("eligible result")["outputs"][&reference.output];
            if output["kind"] != serde_json::to_value(&reference.kind).expect("output kind") {
                return Ok(None);
            }
            let mut value = if reference.kind == crate::plan::Output::Selection {
                output["value"]["candidate"]["value"].clone()
            } else {
                output["value"].clone()
            };
            if declared.outputs.get("result") == Some(&crate::plan::Output::Selection)
                && reference.kind == crate::plan::Output::Verdict
            {
                let Some(check) = super::checks::for_execution(conn, producer.id)? else {
                    return Ok(None);
                };
                value["review"] = json!({"check": check.to_json(),
                    "findings": super::checks::findings(conn, check.id)?,
                    "node": plan.nodes[source], "settings": producer.settings});
            }
            inputs.insert(
                name.clone(),
                json!({"kind": reference.kind, "source": {
                "execution": producer.id, "node": source, "output": reference.output,
                "plan_revision": producer.plan_revision}, "value": value}),
            );
        }
        if start.kind == "implementation"
            && let Some(feedback) = start.feedback
        {
            inputs.insert("feedback".into(), feedback.clone());
        }
        return Ok(Some((node, Value::Object(inputs))));
    }
    let declared = serde_json::to_value(
        &plan
            .nodes
            .get(&node)
            .ok_or_else(|| Fail::refused("node absent from admitted plan"))?
            .inputs,
    )
    .expect("inputs serialize");
    let declared = declared
        .as_object()
        .ok_or_else(|| Fail::refused("node absent from admitted plan"))?;
    let mut inputs = serde_json::Map::new();
    for (name, reference) in declared {
        if reference["output"] == "feedback" {
            if let Some(feedback) = start.feedback {
                inputs.insert(name.clone(), feedback.clone());
            }
            continue;
        }
        if let Some(source) = reference["node"]
            .as_str()
            .filter(|node| *node != "implementer")
        {
            if plan.concurrent() {
                let (kind, supplier) = match &plan.nodes[source].runner {
                    crate::plan::Runner::Command { gate } => ("gate", gate.as_str()),
                    crate::plan::Runner::Agent { .. } => {
                        let index = source
                            .strip_prefix("review-")
                            .and_then(|index| index.parse::<usize>().ok())
                            .ok_or_else(|| Fail::refused("invalid result supplier"))?;
                        (
                            "review",
                            admission["workflow"]["review"][index]
                                .as_str()
                                .ok_or_else(|| Fail::refused("invalid result supplier"))?,
                        )
                    }
                    _ => return Err(Fail::refused("invalid result supplier")),
                };
                let input = super::checks::Input {
                    attempt: attempt.id,
                    base: start.base.unwrap_or_default().into(),
                    head: start.head.unwrap_or_default().into(),
                    proof: start.proof.unwrap_or_default().into(),
                    ticket_revision: start.ticket_revision.unwrap_or_default(),
                    plan_revision: attempt.plan_revision(),
                    digest: start.digest.unwrap_or_default().into(),
                };
                let Some(check) = super::checks::current(conn, kind, supplier, &input)? else {
                    return Ok(None);
                };
                let producer = get(conn, check.execution)?;
                if producer.node.as_deref() != Some(source)
                    || producer.plan_revision != attempt.plan_revision()
                    || !producer.result_eligible
                {
                    return Ok(None);
                }
                let output = &producer
                    .result
                    .as_ref()
                    .ok_or_else(|| Fail::refused("missing supplier result"))?["outputs"]
                    [reference["output"].as_str().unwrap_or_default()];
                if output["kind"] != reference["kind"] {
                    return Ok(None);
                }
                inputs.insert(
                    name.clone(),
                    json!({"kind": reference["kind"],
                    "source": {"execution": producer.id, "node": source,
                        "output": reference["output"], "plan_revision": producer.plan_revision},
                    "value": output["value"]}),
                );
                continue;
            }
            let index = source
                .strip_prefix("review-")
                .and_then(|index| index.parse::<usize>().ok())
                .ok_or_else(|| Fail::refused("invalid panel node"))?;
            let seat = admission["workflow"]["review"][index]
                .as_str()
                .ok_or_else(|| Fail::refused("invalid panel seat"))?;
            let input = super::checks::Input {
                attempt: attempt.id,
                base: start.base.unwrap_or_default().into(),
                head: start.head.unwrap_or_default().into(),
                proof: start.proof.unwrap_or_default().into(),
                ticket_revision: start.ticket_revision.unwrap_or_default(),
                plan_revision: attempt.plan_revision(),
                digest: start.digest.unwrap_or_default().into(),
            };
            let Some(check) = super::checks::current(conn, "review", seat, &input)? else {
                return Ok(None);
            };
            let producer = get(conn, check.execution)?.node.map(|_| check.execution);
            let (supplier, mut value) = match producer {
                Some(producer) => {
                    let result = get(conn, producer)?
                        .result
                        .ok_or_else(|| Fail::refused("missing panel result"))?;
                    (
                        json!({"execution": producer, "node": source, "output": reference["output"]}),
                        result["outputs"]["result"]["value"].clone(),
                    )
                }
                None => (json!({"status": "unknown"}), json!({})),
            };
            // Old checks can precede node/result binding. Preserve their
            // supported panel continuation without inventing a producer result.
            if value.get("blocked").is_none() {
                let mut blocked = Vec::new();
                let mut previous = Some(source);
                while let Some(node) = previous {
                    let index = node
                        .strip_prefix("review-")
                        .and_then(|index| index.parse::<usize>().ok())
                        .ok_or_else(|| Fail::refused("invalid historical panel node"))?;
                    let seat = admission["workflow"]["review"][index]
                        .as_str()
                        .ok_or_else(|| Fail::refused("invalid historical panel seat"))?;
                    let check = super::checks::current(conn, "review", seat, &input)?
                        .ok_or_else(|| Fail::refused("panel input has no eligible check"))?;
                    if node == source && producer.is_none() {
                        if get(conn, check.execution)?.node.is_some() {
                            return Err(Fail::refused("panel input has no eligible result"));
                        }
                        value = json!({"publication": check.id, "verdict": check.verdict});
                    }
                    if check.verdict == "fail" {
                        blocked.push(json!(check.id));
                    }
                    previous = plan.nodes[node]
                        .inputs
                        .get("panel")
                        .and_then(|input| input.node.as_deref());
                }
                blocked.reverse();
                value["blocked"] = json!(blocked);
            }
            inputs.insert(
                name.clone(),
                json!({"kind": reference["kind"],
                "source": supplier, "value": value}),
            );
            continue;
        }
        let value = json!({"base": start.base, "head": start.head, "proof": start.proof.unwrap_or_default()});
        let source = if reference["node"] == "implementer" {
            let producer = conn.query_row(
                &format!("SELECT id FROM execution WHERE attempt = ?1 AND node = 'implementer'
                 AND {RESULT_ELIGIBLE} AND json_extract(result, '$.outputs.result.value.base') = ?2
                 AND json_extract(result, '$.outputs.result.value.head') = ?3
                 AND json_extract(result, '$.outputs.result.value.proof') = ?4 ORDER BY id DESC LIMIT 1"),
                params![attempt.id, start.base, start.head, start.proof.unwrap_or_default()],
                |row| row.get::<_, i64>(0),
            ).optional()?;
            // An upgraded live attempt can still carry a pre-binding candidate.
            if producer.is_none() && !conn.query_row(
                "SELECT COALESCE((SELECT node IS NULL FROM execution WHERE attempt = ?1
                 AND kind = 'implementation' AND outcome = 'candidate' ORDER BY id DESC LIMIT 1), 0)",
                [attempt.id], |row| row.get::<_, bool>(0),
            )? {
                return Err(Fail::refused("candidate has no eligible implementer result"));
            }
            producer.map_or_else(|| json!({"status": "unknown"}), |execution| json!({"execution": execution, "node": "implementer", "output": "result"}))
        } else {
            json!({"external": "merged", "execution": start.parent})
        };
        inputs.insert(
            name.clone(),
            json!({"kind": reference["kind"], "source": source, "value": value}),
        );
    }
    Ok(Some((node, Value::Object(inputs))))
}

/// Reuse an actual producer, never relabel its revision. Complete routing
/// selects successors; every other node policy and its typed inputs bind evidence.
pub fn qualifies(
    conn: &Connection,
    attempt: &super::attempts::Attempt,
    execution: &Execution,
    digest: &str,
) -> Result<bool, Fail> {
    let Some(node) = execution.node.as_deref() else {
        return Ok(execution.plan_revision == attempt.plan_revision());
    };
    let Some(revision) = execution.plan_revision else {
        return Ok(false);
    };
    let Some(original) = super::attempts::revision(conn, attempt.id, revision)? else {
        return Ok(false);
    };
    let Some(current) = &attempt.admission else {
        return Ok(false);
    };
    let original_evidence = original.clone();
    let current_evidence = current;
    let original =
        crate::plan::Plan::admitted(&original).map_err(|error| Fail::refused(error.to_string()))?;
    let current =
        crate::plan::Plan::admitted(current).map_err(|error| Fail::refused(error.to_string()))?;
    if current.concurrent()
        && execution.kind != "implementation"
        && execution.plan_revision != attempt.plan_revision()
    {
        return Ok(false);
    }
    let Some(mut old) = original.nodes.get(node).cloned() else {
        return Ok(false);
    };
    let Some(mut new) = current.nodes.get(node).cloned() else {
        return Ok(false);
    };
    old.routes.remove("complete");
    new.routes.remove("complete");
    if old != new {
        return Ok(false);
    }
    let reviewer = execution.kind == "review";
    let old_definition = consumed_definition(&original_evidence, &old.runner, reviewer);
    let new_definition = consumed_definition(current_evidence, &new.runner, reviewer);
    if old_definition != new_definition
        || (old_definition.is_none() && execution.plan_revision != attempt.plan_revision())
    {
        return Ok(false);
    }
    let expected = binding(
        conn,
        attempt,
        &Start {
            attempt: Some(attempt.id),
            kind: &execution.kind,
            node: execution.node.as_deref(),
            name: execution.name.as_deref(),
            parent: execution.parent,
            base: execution.base.as_deref(),
            head: execution.head.as_deref(),
            proof: execution.proof.as_deref(),
            ticket_revision: execution.ticket_revision,
            digest: Some(digest),
            feedback: execution
                .inputs
                .as_ref()
                .and_then(|inputs| inputs.get("feedback")),
            ..Default::default()
        },
    )?;
    Ok(expected.is_some_and(|(expected_node, inputs)| {
        expected_node == node && execution.inputs.as_ref() == Some(&inputs)
    }))
}

/// A live revision affects only changed policy or typed producer bindings.
pub fn plan_superseded(
    conn: &Connection,
    attempt: &super::attempts::Attempt,
    execution: &Execution,
) -> Result<bool, Fail> {
    if execution.plan_revision == attempt.plan_revision() {
        return Ok(false);
    }
    let digest: Option<String> = conn.query_row(
        "SELECT digest FROM execution WHERE id = ?1",
        [execution.id],
        |row| row.get(0),
    )?;
    Ok(!qualifies(
        conn,
        attempt,
        execution,
        digest.as_deref().unwrap_or_default(),
    )
    .unwrap_or(false))
}

/// Observational: no event.
pub fn set_handle(
    conn: &Connection,
    id: i64,
    handle: &str,
    image_id: Option<&str>,
) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET handle = ?2, image_id = COALESCE(?3, image_id) WHERE id = ?1",
        params![id, handle, image_id],
    )?;
    Ok(())
}

/// Append real launch inputs and any original-result reference once.
pub fn verification(conn: &Connection, id: i64, evidence: &Value) -> Result<(), Fail> {
    if conn.execute(
        "UPDATE execution SET verification = ?2 WHERE id = ?1 AND status = 'running' AND verification IS NULL",
        params![id, evidence.to_string()],
    )? != 1 {
        return Err(Fail::refused("execution already has verification evidence"));
    }
    Ok(())
}

/// Observational: a worker's session and usage.
pub fn set_worker(
    conn: &Connection,
    id: i64,
    session_id: Option<&str>,
    usage: Option<&Usage>,
) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET session_id = COALESCE(?2, session_id) WHERE id = ?1 AND status = 'running'",
        params![id, session_id],
    )?;
    if let Some(usage) = usage {
        conn.execute(
            "UPDATE execution SET tokens_in = ?2, tokens_out = ?3, cost = ?4,
                usage_fields = ?5 WHERE id = ?1 AND status = 'running'",
            params![
                id,
                usage
                    .input
                    .map(i64::try_from)
                    .transpose()
                    .map_err(|error| Fail::new("internal", error.to_string()))?,
                usage
                    .output
                    .map(i64::try_from)
                    .transpose()
                    .map_err(|error| Fail::new("internal", error.to_string()))?,
                usage.cost,
                serde_json::to_string(&usage.fields)
                    .map_err(|error| Fail::new("internal", error.to_string()))?
            ],
        )?;
    }
    Ok(())
}

pub fn set_progress(conn: &Connection, id: i64, note: &str) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET progress = ?2 WHERE id = ?1 AND status = 'running'",
        params![id, note],
    )?;
    Ok(())
}

/// Observational: the harness's registration proof, recorded once its reader
/// yields a complete one. `mcp` is `registered` or `refused`.
pub fn set_mcp(conn: &Connection, id: i64, mcp: &str) -> Result<(), Fail> {
    conn.execute(
        "UPDATE execution SET mcp = ?2 WHERE id = ?1 AND status = 'running'",
        params![id, mcp],
    )?;
    Ok(())
}

/// How an execution ended.
#[derive(Default)]
pub struct End<'a> {
    pub outcome: &'a str,
    pub infrastructure: Option<&'a str>,
    pub detail: Option<&'a str>,
    pub exit_cause: Option<&'a str>,
    pub exit_code: Option<i32>,
    pub oom_kills: Option<u64>,
    pub ticket: Option<i64>,
    /// A candidate or report admitted after the implementer's checked teardown.
    pub output: Option<Value>,
}

pub fn end(tx: &Connection, id: i64, end: End) -> Result<(), Fail> {
    let execution = get(tx, id)?;
    if execution.status != "running" {
        return Err(Fail::new(
            "internal",
            format!("execution {id} has already ended"),
        ));
    }
    if execution.scope.is_some() && end.infrastructure != Some("completed") {
        let mut usage = execution.usage.clone();
        for field in [
            &mut usage.fields.input,
            &mut usage.fields.output,
            &mut usage.fields.cost,
        ] {
            if field.completeness == crate::harness::Completeness::Complete {
                field.completeness = crate::harness::Completeness::Partial;
            }
        }
        set_worker(tx, id, None, Some(&usage))?;
    }
    tx.execute(
        "UPDATE execution SET status = 'ended', outcome = ?2, detail = ?3, exit_cause = ?4,
            exit_code = ?5, oom_kills = ?6, ended_at = ?7
         WHERE id = ?1",
        params![
            id,
            end.outcome,
            end.detail,
            end.exit_cause,
            end.exit_code,
            end.oom_kills.map(|count| count as i64),
            now()
        ],
    )?;
    if execution.scope.is_some() {
        let output = if end.infrastructure == Some("completed") {
            end.output.clone()
        } else {
            None
        };
        let result = json!({"outcome": output.as_ref().map(|_| end.outcome),
            "infrastructure": {"outcome": end.infrastructure.unwrap_or(end.outcome),
                "exit_cause": end.exit_cause, "exit_code": end.exit_code},
            "judgment": null, "outputs": output.map_or_else(|| json!({}), |output| json!({"result": output})),
            "scope": execution.scope});
        tx.execute(
            "UPDATE execution SET result = ?2 WHERE id = ?1",
            params![id, result.to_string()],
        )?;
    } else if execution.node.is_some() {
        let publication = super::checks::for_execution(tx, id)?;
        let registered: Option<String> =
            tx.query_row("SELECT mcp FROM execution WHERE id = ?1", [id], |row| {
                row.get(0)
            })?;
        let judgment = if end.outcome == "abandoned" {
            None
        } else {
            publication
                .as_ref()
                .filter(|_| registered.as_deref() == Some("registered"))
                .map(|check| json!({"publication": check.id, "verdict": check.verdict}))
                .or_else(|| {
                    (execution.kind == "gate" && matches!(end.outcome, "pass" | "fail"))
                        .then(|| json!({"verdict": end.outcome}))
                })
        };
        let infrastructure = end.infrastructure.unwrap_or(
            if matches!(end.outcome, "pass" | "fail" | "candidate") || end.output.is_some() {
                "completed"
            } else {
                end.outcome
            },
        );
        let mut output = end.output.or_else(|| {
            judgment
                .as_ref()
                .map(|judgment| json!({"kind": "verdict", "value": judgment}))
        });
        let attempt = super::attempts::for_execution(tx, &execution)?;
        let mut outcome = output
            .as_ref()
            .filter(|output| output["kind"] == "report")
            .and(execution.report.as_ref())
            .and_then(|report| report["outcome"].as_str())
            .unwrap_or("complete");
        if let Some(admission) = &attempt.admission {
            let plan = crate::plan::Plan::admitted(admission)
                .map_err(|error| Fail::refused(error.to_string()))?;
            let node = &plan.nodes[execution.node.as_deref().expect("bound node")];
            if execution.kind == "review"
                && let Some(output) = &mut output
                && output["kind"] == "verdict"
            {
                let mut blocked = execution
                    .inputs
                    .as_ref()
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|inputs| inputs.values())
                    .filter(|input| input["kind"] == "verdict")
                    .filter_map(|input| input["value"]["blocked"].as_array())
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>();
                blocked.sort_by_key(Value::as_i64);
                blocked.dedup();
                if let Some(check) = &publication
                    && check.verdict == "fail"
                {
                    blocked.push(json!(check.id));
                }
                if !blocked.is_empty() && node.outcomes.iter().any(|name| name == "blocked") {
                    outcome = "blocked";
                }
                output["value"]["blocked"] = json!(blocked);
            } else if execution.kind == "gate"
                && end.outcome == "fail"
                && node.outcomes.iter().any(|name| name == "failed")
            {
                outcome = "failed";
            }
        }
        let result = json!({
            "outcome": output.as_ref().map(|_| outcome),
            "infrastructure": {"outcome": infrastructure, "exit_cause": end.exit_cause, "exit_code": end.exit_code},
            "judgment": judgment,
            "reference": execution.verification.as_ref().and_then(|v| v.get("reference")),
            "verification": execution.verification.clone().unwrap_or_else(|| json!({"status": "unknown"})),
            "outputs": output.map_or_else(|| json!({}), |output| json!({"result": output})),
        });
        tx.execute(
            "UPDATE execution SET result = ?2 WHERE id = ?1",
            params![id, result.to_string()],
        )?;
        // A recovered publication can advance only after checked ownership.
        // Other failures keep their attention and require their existing exit.
        if execution.kind == "review"
            && end.outcome == "interrupted"
            && execution.lock_contract
            && registered.as_deref() == Some("registered")
            && publication.is_some()
            && get(tx, id)?.result_eligible
        {
            for item in super::attempts::open_for_attempt(tx, execution.required_attempt()?)? {
                if item.kind == "red" && item.reason == "ownership" && item.execution == Some(id) {
                    super::attempts::resolve(tx, &item, "reconciled", None)?;
                }
            }
        }
    }
    audit(
        tx,
        "execution.ended",
        Target {
            ticket: end.ticket,
            attempt: execution.attempt,
            execution: Some(id),
            attention: None,
        },
        end.detail,
        json!({ "kind": execution.kind, "outcome": end.outcome, "name": execution.name,
                "exit_cause": end.exit_cause }),
    )?;
    Ok(())
}

/// Preparation is immutable input evidence, never canonical mutation authority.
pub fn prepare(tx: &Connection, id: i64, prepared: &Value) -> Result<(), Fail> {
    if tx.execute(
        "UPDATE execution SET prepared = ?2 WHERE id = ?1 AND prepared IS NULL AND resumed IS NULL",
        params![id, prepared.to_string()],
    )? != 1
    {
        return Err(Fail::refused("landing already has a prepared identity"));
    }
    let row = get(tx, id)?;
    audit(
        tx,
        "landing.prepared",
        Target {
            attempt: row.attempt,
            execution: Some(id),
            ..Default::default()
        },
        None,
        prepared.clone(),
    )?;
    Ok(())
}

pub fn set_intent(
    tx: &Connection,
    id: i64,
    target_ref: &str,
    old: &str,
    merged: &str,
    ticket: i64,
    attempt: i64,
) -> Result<(), Fail> {
    tx.execute(
        "UPDATE execution SET intent_old = ?2, intent_merged = ?3, intent_state = 'open', intent_ref = ?4 WHERE id = ?1",
        params![id, old, merged, target_ref],
    )?;
    audit(
        tx,
        "landing.intent",
        Target {
            ticket: Some(ticket),
            attempt: Some(attempt),
            execution: Some(id),
            attention: None,
        },
        None,
        json!({ "target_ref": target_ref, "old": old, "merged": merged }),
    )?;
    Ok(())
}

pub fn resolve_intent(tx: &Connection, id: i64) -> Result<(), Fail> {
    tx.execute(
        "UPDATE execution SET intent_state = 'resolved' WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn reserve(tx: &Connection, id: i64, reserved: bool) -> Result<(), Fail> {
    let row = get(tx, id)?;
    if row.reserved == reserved {
        return Ok(());
    }
    tx.execute(
        "UPDATE execution SET reserved = ?2 WHERE id = ?1",
        params![id, reserved],
    )?;
    audit(
        tx,
        if reserved {
            "execution.reserved"
        } else {
            "execution.released"
        },
        Target {
            attempt: row.attempt,
            execution: Some(id),
            ..Default::default()
        },
        None,
        json!({}),
    )?;
    Ok(())
}
