//! Local comparisons of exact source publications and applied picker choices.
use super::{attempts, checks, executions, selections};
use crate::{api::Fail, plan::Plan};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Samples {
    rows: Vec<executions::Execution>,
    published: BTreeSet<String>,
    eligible: BTreeSet<String>,
    selected: BTreeSet<String>,
    current_selected: BTreeSet<String>,
    superseded: BTreeSet<String>,
    groups: BTreeSet<String>,
    publications: BTreeSet<i64>,
    eligible_publications: BTreeSet<i64>,
    pickers: BTreeSet<i64>,
    repairs: BTreeSet<i64>,
    landings: BTreeSet<i64>,
}

fn identity(row: &executions::Execution, role: &str) -> Value {
    json!({"role": role, "cohort": {"attempt": row.attempt, "ticket_revision": row.ticket_revision,
        "plan_revision": row.plan_revision}, "requested": {"harness": row.harness, "provider": row.provider,
        "model": row.model, "effort": row.effort}, "observed": {"status": "unknown"}})
}

fn source_key(source: &Value) -> Option<String> {
    source["finding"]
        .as_i64()
        .map(|id| format!("finding:{id}"))
        .or_else(|| {
            source["execution"]
                .as_i64()
                .map(|id| format!("candidate:{id}"))
        })
}

fn rate(selected: usize, eligible: usize) -> Value {
    if eligible == 0 {
        Value::Null
    } else {
        json!(selected as f64 / eligible as f64)
    }
}

/// The input rows set the scope. No result traversal creates a new sample.
pub fn statistics(conn: &Connection, rows: &[executions::Execution]) -> Result<Value, Fail> {
    let mut models: BTreeMap<String, Samples> = BTreeMap::new();
    let mut owners = BTreeMap::new();
    let mut cohorts: BTreeMap<String, Value> = BTreeMap::new();
    let mut cohort_rows: BTreeMap<String, Vec<&executions::Execution>> = BTreeMap::new();
    let mut picker_rows = Vec::new();
    for row in rows
        .iter()
        .filter(|row| row.scope.is_none() && row.node.is_some())
    {
        let Some(attempt_id) = row.attempt else {
            continue;
        };
        let Some(revision) = row.plan_revision else {
            continue;
        };
        let Some(admission) = attempts::revision(conn, attempt_id, revision)? else {
            continue;
        };
        let plan = Plan::admitted(&admission).map_err(|error| Fail::refused(error.to_string()))?;
        if !plan.writers() && plan.selector().is_none() && plan.finding_selector().is_none() {
            continue;
        }
        let Some(node) = row.node.as_ref().and_then(|node| plan.nodes.get(node)) else {
            continue;
        };
        let cohort = json!([attempt_id, row.ticket_revision, revision]).to_string();
        cohort_rows.entry(cohort.clone()).or_default().push(row);
        if let std::collections::btree_map::Entry::Vacant(entry) = cohorts.entry(cohort) {
            let attempt = attempts::get(conn, attempt_id)?;
            entry.insert(json!({"attempt": attempt_id, "ticket": super::ticket_name(attempt.ticket),
                "ticket_revision": row.ticket_revision, "plan_revision": revision, "admission": admission}));
        }
        if !matches!(node.runner, crate::plan::Runner::Agent { .. }) {
            continue;
        }
        let role = if row.node.as_deref() == Some("selector")
            && (plan.selector().is_some() || plan.finding_selector().is_some())
        {
            "picker"
        } else if node
            .outputs
            .values()
            .any(|output| *output == crate::plan::Output::Candidate)
        {
            "implementer"
        } else if node
            .outputs
            .values()
            .any(|output| *output == crate::plan::Output::Verdict)
        {
            "reviewer"
        } else {
            continue;
        };
        let model = identity(row, role).to_string();
        owners.insert(row.id, model.clone());
        let samples = models.entry(model).or_default();
        samples.rows.push(row.clone());
        // Reuse references are consumers, never additional publications.
        if row.result.as_ref().is_some_and(|result| {
            result["reference"].is_null() && result["outputs"]["result"]["kind"] == "candidate"
        }) {
            samples.published.insert(format!("candidate:{}", row.id));
            samples.publications.insert(row.id);
        }
        if role == "reviewer"
            && let Some(check) = checks::for_execution(conn, row.id)?
        {
            samples.publications.insert(row.id);
            for finding in checks::findings(conn, check.id)? {
                if let Some(key) = source_key(&finding.source) {
                    samples.published.insert(key);
                }
            }
        }
        if role == "picker" {
            if row.report.is_some() {
                samples.publications.insert(row.id);
            }
            picker_rows.push((row, plan.finding_selector().is_some()));
        }
    }
    let mut choices = Vec::new();
    for (picker, finding_picker) in picker_rows {
        let attempt = attempts::get(conn, picker.required_attempt()?)?;
        let output = picker
            .result
            .as_ref()
            .map(|result| &result["outputs"]["result"]);
        let kind = output.and_then(|output| output["kind"].as_str());
        let current = match kind {
            Some("selection") => {
                selections::current(conn, &attempt)?.is_some_and(|row| row.id == picker.id)
            }
            Some("finding-selection") => {
                let latest = executions::query(
                    conn,
                    "attempt = ?1 AND node = 'selector' ORDER BY id DESC LIMIT 1",
                    [attempt.id],
                )?
                .pop();
                latest.is_some_and(|row| row.id == picker.id)
                    && selections::finding_current(conn, &attempt, picker)?
            }
            _ => false,
        };
        let recorded = picker.choice_application.as_ref();
        let status = match recorded.and_then(|value| value["status"].as_str()) {
            Some("applied") if current => "applied",
            Some("applied") => "superseded",
            Some("unapplied") => "unapplied",
            _ => "unknown",
        };
        let mut eligible = BTreeMap::new();
        for input in picker
            .inputs
            .as_ref()
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|inputs| inputs.values())
        {
            if (input["kind"] == "verdict" || (input["kind"] == "candidate" && !finding_picker))
                && let Some(samples) = input["source"]["execution"]
                    .as_i64()
                    .and_then(|id| owners.get(&id))
                    .and_then(|model| models.get_mut(model))
            {
                samples
                    .eligible_publications
                    .insert(input["source"]["execution"].as_i64().expect("source"));
                samples.pickers.insert(picker.id);
            }
            if input["kind"] == "candidate" && !finding_picker {
                if let Some(key) = source_key(&input["source"]) {
                    eligible.insert(key, input["source"].clone());
                }
            } else if input["kind"] == "verdict" {
                for finding in input["value"]["review"]["findings"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    if let Some(key) = source_key(&finding["source"]) {
                        eligible.insert(key, finding["source"].clone());
                    }
                }
            }
        }
        let value = output
            .map(|output| &output["value"])
            .unwrap_or(&Value::Null);
        let mut selected = BTreeMap::new();
        let mut groups = Vec::new();
        if kind == Some("selection") {
            let source = &value["candidate"]["source"];
            if let Some(key) = source_key(source) {
                selected.insert(key, source.clone());
            }
        } else if kind == Some("finding-selection") {
            for group in value["groups"].as_array().into_iter().flatten() {
                let mut keys = BTreeSet::new();
                for source in group["sources"].as_array().into_iter().flatten() {
                    if let Some(key) = source_key(source) {
                        keys.insert(key.clone());
                        selected.insert(key, source.clone());
                    }
                }
                groups.push(keys);
            }
        }
        let repairs: Vec<_> = rows
            .iter()
            .filter(|row| {
                row.kind == "implementation"
                    && row.reason.as_deref() == Some("repair")
                    && row.inputs.as_ref().is_some_and(|inputs| {
                        inputs["feedback"]["source"]["execution"] == picker.id
                    })
            })
            .map(|row| row.id)
            .collect();
        let mut landings = Vec::new();
        for row in rows
            .iter()
            .filter(|row| row.kind == "landing" && row.outcome.as_deref() == Some("landed"))
        {
            if let Some(approval) = row.approval
                && checks::approval(conn, approval)?.choice == Some(picker.id)
            {
                landings.push(row.id);
            }
        }
        for (key, source) in &eligible {
            if let Some(samples) = source["execution"]
                .as_i64()
                .and_then(|id| owners.get(&id))
                .and_then(|model| models.get_mut(model))
            {
                samples.eligible.insert(key.clone());
                samples.pickers.insert(picker.id);
                if selected.contains_key(key) {
                    if matches!(status, "applied" | "superseded") {
                        samples.selected.insert(key.clone());
                    }
                    if status == "applied" {
                        samples.current_selected.insert(key.clone());
                    }
                    if status == "superseded" {
                        samples.superseded.insert(key.clone());
                    }
                    // Consumption and landing are separate historical facts, not correctness.
                    samples.repairs.extend(&repairs);
                    samples.landings.extend(&landings);
                    if matches!(status, "applied" | "superseded") {
                        for group in &groups {
                            if group.contains(key) {
                                samples
                                    .groups
                                    .insert(serde_json::to_string(group).expect("source keys"));
                            }
                        }
                    }
                }
            }
        }
        choices.push(json!({"execution": picker.id, "attempt": picker.attempt, "ticket_revision": picker.ticket_revision,
            "plan_revision": picker.plan_revision, "picker_identity": identity(picker, "picker"), "settings": picker.settings,
            "inputs": picker.inputs, "publication": picker.report, "result": picker.result,
            "application": {"status": status, "recorded": recorded}, "eligible_sources": eligible.values().collect::<Vec<_>>(),
            "selected_sources": selected.values().collect::<Vec<_>>(), "repair_consumers": repairs, "landings": landings}));
    }
    let models = models.into_iter().map(|(identity, samples)| {
        let mut outcomes = json!({"completed": 0, "failed": 0, "stopped": 0, "unknown": 0});
        for row in &samples.rows {
            let infrastructure = row.result.as_ref().and_then(|result| result["infrastructure"]["outcome"].as_str());
            let outcome = match infrastructure { Some("completed") => "completed", Some("failed" | "timeout" | "error") => "failed", Some("stopped") => "stopped", _ => "unknown" };
            outcomes[outcome] = json!(outcomes[outcome].as_u64().expect("count") + 1);
        }
        Ok(json!({"identity": serde_json::from_str::<Value>(&identity).expect("identity"),
            "executions": samples.rows.iter().map(|row| row.id).collect::<Vec<_>>(), "outcomes": outcomes,
            "published_count": samples.published.len(), "eligible_count": samples.eligible.len(),
            "publication_count": samples.publications.len(), "eligible_publication_count": samples.eligible_publications.len(),
            "selected_count": samples.selected.len(), "selected_rate": rate(samples.selected.len(), samples.eligible.len()),
            "current_selected_count": samples.current_selected.len(),
            "superseded_selected_count": samples.superseded.len(), "selected_group_count": samples.groups.len(),
            "published_sources": samples.published, "eligible_sources": samples.eligible, "selected_sources": samples.selected,
            "picker_uses": samples.pickers, "repair_consumers": samples.repairs, "landings": samples.landings,
            "measurements": executions::aggregate(&samples.rows)?}))
    }).collect::<Result<Vec<_>, Fail>>()?;
    for (key, cohort) in &mut cohorts {
        let members = &cohort_rows[key];
        let start = members.iter().map(|row| row.started_at.as_str()).min();
        let end = if members.iter().all(|row| row.ended_at.is_some()) {
            members
                .iter()
                .filter_map(|row| row.ended_at.as_deref())
                .max()
        } else {
            None
        };
        let elapsed: Option<f64> = conn.query_row(
            "SELECT (julianday(?2) - julianday(?1)) * 86400.0",
            params![start, end],
            |row| row.get(0),
        )?;
        cohort["executions"] = json!(members.iter().map(|row| row.id).collect::<Vec<_>>());
        cohort["timing"] = json!({"kind": "observed_graph_span_seconds", "started_at": start, "ended_at": end, "elapsed_seconds": elapsed});
    }
    Ok(
        json!({"meaning": "picker-selected usefulness is a proxy, not correctness", "models": models,
        "cohorts": cohorts.into_values().collect::<Vec<_>>(), "choices": choices}),
    )
}
