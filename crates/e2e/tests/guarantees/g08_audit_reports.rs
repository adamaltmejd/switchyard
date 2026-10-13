//! G8: an audit is an explicit execution-bound publication.

use e2e::*;
use serde_json::{Value, json};

/// G8 boundary and ownership: a blank report cannot
/// publish; the allowed report succeeds in the same execution, then a
/// duplicate cannot replace it. Inherited bearer calls lack the harness's
/// session and get 404. The real model holds after publication, so neither
/// completion nor eligibility can precede teardown. Expected content comes
/// from this fixture; frozen head comes from host git.
/// Sabotage: skip payload validation, overwrite report, accept another MCP
/// session, bind to mutable identities, or infer a result at publication.
/// Existing review scenarios publish verdicts, not reports.
#[test]
fn an_audit_publication_is_validated_bound_and_held_until_teardown() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g8-audit", move |request| {
        if request.opens() {
            return Reply::Tools(vec![bash(
                r#"for m in '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"yard_publish_report","arguments":{"outcome":"complete","report":{"summary":"rogue","findings":[]}}}}'; do curl -s -o /dev/null -w 'status=%{http_code} ' -X POST http://yard.mcp/mcp -H 'content-type: application/json' -H "authorization: Bearer $YARD_MCP_BEARER" -H 'mcp-session-id: wrong-session' -d "$m"; done"#,
            )]);
        }
        match request.tool_results().len() {
            1 => Reply::Tools(vec![tool(
                "yard_publish_report",
                json!({
                    "outcome": "complete", "report": {"summary": " ", "findings": []}
                }),
            )]),
            2 => Reply::Tools(vec![publish_report()]),
            3 => Reply::Tools(vec![publish_report()]),
            _ => Reply::Hold(held.clone(), Box::new(Reply::Text("done".into()))),
        }
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let head = project.canonical_head();
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Audit", "--workflow", "plan"]);
    hold.wait_held();
    let show = project.json(&["attempt", "show", "Y-1"]);
    let rows = project.rows("SELECT id, node, plan_revision, ticket_revision, inputs, base, head, proof, report, result, status FROM execution WHERE kind = 'implementation'");
    let row = &rows[0];
    let report: Value = serde_json::from_str(row["report"].as_str().unwrap()).unwrap();
    assert_eq!(
        report["report"],
        json!({"summary": "Fixture audit", "findings": []})
    );
    assert_eq!(report["outcome"], "complete");
    assert_eq!(report["node"], "implementer");
    assert_eq!(report["base"], head);
    // G8 promises agreement with the frozen execution binding, not current config.
    for key in [
        "node",
        "plan_revision",
        "ticket_revision",
        "base",
        "head",
        "proof",
    ] {
        assert_eq!(report[key], row[key]);
    }
    assert_eq!(report["execution"], row["id"]);
    // G7: the publication event names the same durable execution and its
    // attempt/ticket. Sabotage: omit or misbind report.published's target.
    assert_eq!(
        project
            .rows("SELECT ticket, attempt, execution FROM audit WHERE event = 'report.published'"),
        vec![json!({"ticket": 1, "attempt": 1, "execution": row["id"]})]
    );
    assert_eq!(
        report["inputs"],
        serde_json::from_str::<Value>(row["inputs"].as_str().unwrap()).unwrap()
    );
    assert_eq!(row["status"], "running");
    assert!(row["result"].is_null());
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE state = 'ended'")
            .is_empty()
    );
    assert!(!show["executions"][0]["result_eligible"].as_bool().unwrap());
    let requests = machine.model.requests();
    let results = requests.last().unwrap().tool_results();
    for token in [
        "status=404 status=404",
        "invalid_report",
        "duplicate_report",
    ] {
        assert!(
            results.iter().any(|(_, text)| text.contains(token)),
            "missing {token}: {results:?}"
        );
    }
    hold.release();
    watch.event("attempt.ended", &[("outcome", "planned")]);
    watch.until("audit reservation released", |event| {
        event["event"] == "execution.released" && event["execution"] == row["id"]
    });
    let show = project.json(&["attempt", "show", "Y-1"]);
    let execution = &show["executions"][0];
    assert_eq!(
        execution["result"]["outputs"]["result"],
        json!({"kind": "report", "value": report})
    );
    assert_eq!(execution["result"]["outcome"], "complete");
    assert_eq!(
        execution["result"]["infrastructure"]["outcome"],
        "completed"
    );
    assert_eq!(execution["teardown"], true);
    assert_eq!(execution["result_eligible"], true);
}

/// G8: successful terminal text cannot supply an audit or route. The second
/// ticket in the same project publishes explicitly and completes, controlling
/// the failure. Sabotage: synthesize a report from successful exit, or treat
/// missing publication as a completed report outcome.
#[test]
fn a_successful_exit_without_an_audit_report_is_an_infrastructure_error() {
    let machine = Machine::new("g8-audit-missing", |request| {
        if request.prompt().contains("Missing audit") {
            Reply::Text("The audit passed.".into())
        } else {
            planned(request)
        }
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Missing audit",
        "--workflow",
        "plan",
    ]);
    let attention = watch.attention();
    assert_eq!(attention["data"]["payload"]["detail"], "missing_report");
    let execution = project.json(&["attempt", "show", "Y-1"])["executions"][0].clone();
    assert!(execution["report"].is_null());
    assert_eq!(execution["result"]["outputs"], json!({}));
    assert!(execution["result"]["outcome"].is_null());
    assert_eq!(execution["result"]["infrastructure"]["outcome"], "failed");
    assert_eq!(execution["result_eligible"], false);
    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Published audit",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    assert_eq!(
        project.json(&["attempt", "show", "Y-2"])["executions"][0]["report"]["report"],
        json!({"summary": "Fixture audit", "findings": []})
    );
}

/// G8/G2: an edit supersedes an audit held before publication. Its late
/// report is refused, but the missing old report is not an infrastructure
/// error: the same attempt reruns and publishes on the edited revision.
/// The fixture supplies revisions, body and report; CLI and store observe
/// the old empty result and the new execution-bound publication.
/// Sabotage: remove the edited exemption from missing_report, or accept the
/// superseded publication. Missing-report and held-publication scenarios
/// do not edit a ticket between registration and publication.
#[test]
fn a_superseded_audit_reruns_without_a_missing_report_error() {
    superseded_audit(false);
}

/// G2/G9: revise admitted read-only instructions before held publication.
/// Ticket edits change ticket identity; this keeps it fixed. The replacement
/// receives the fixture profile and revision diff, and publishes successfully.
/// Sabotage: accept the superseded report, infer missing_report after supersession,
/// or load a running worker from the latest plan.
#[test]
fn a_plan_revision_hands_off_a_selected_read_only_implementer() {
    superseded_audit(true);
}

fn superseded_audit(revise: bool) {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new(
        if revise {
            "g8-audit-revise"
        } else {
            "g8-audit-edit"
        },
        move |request| {
            if request.prompt().contains("Revised audit input") {
                return planned(request);
            }
            if request.opens() {
                return Reply::Hold(held.clone(), Box::new(Reply::Tools(vec![publish_report()])));
            }
            Reply::Text("done".into())
        },
    );
    machine.start();
    // A fresh harness conversation makes the new revision's model input
    // independent of the old refused publication.
    let project = Project::new(
        &machine,
        "p",
        &config("[profiles.revised]\nrunner = {kind = \"agent\", agent = \"worker\", instructions = \"Revised audit input\"}\naccess = \"read-only\"\noutputs = {result = \"report\"}\noutcomes = [\"complete\"]\n").replace(
            "[workflows.default]\n",
            "[workflows.default]\nmax_session_executions = 1\n",
        ),
    );
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Audit edit",
        "--workflow",
        "plan",
    ]);
    hold.wait_held();
    let frozen = project
        .rows("SELECT id, plan_revision, inputs FROM execution WHERE kind = 'implementation'");
    if revise {
        let mut operator = machine.operator();
        operator.initialize();
        let adjusted = operator.call(
            "yard_attempt_revise",
            json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 1,
            "overrides": {"implementer": {"profile": "revised"}}}),
        );
        assert_eq!(adjusted["result"]["plan_revision"], 2, "{adjusted}");
        assert_eq!(
            project.rows(
                "SELECT id, plan_revision, inputs FROM execution WHERE kind = 'implementation'"
            ),
            frozen
        );
    } else {
        project.json(&[
            "ticket",
            "edit",
            "Y-1",
            "--revision",
            "1",
            "--body",
            "Revised audit input",
        ]);
    }
    hold.release();
    watch.event("attempt.ended", &[("outcome", "planned")]);
    assert_eq!(project.rows("SELECT id, plan_revision, inputs FROM execution WHERE kind = 'implementation' ORDER BY id LIMIT 1"), frozen);
    if revise {
        assert_eq!(
            project.rows(
                "SELECT plan_revision FROM execution WHERE kind = 'implementation' ORDER BY id"
            ),
            vec![json!({"plan_revision": 1}), json!({"plan_revision": 2})]
        );
        let prompt = machine
            .model
            .requests()
            .into_iter()
            .rfind(|request| request.opens())
            .unwrap()
            .last_user();
        // G9: the model's delivered Git diff carries the fixture change and
        // its attached operator label. Fresh initial instructions also carry
        // the marker, so they cannot establish this handoff.
        // Diff-only sabotage: omit the plan diff but keep its operator heading.
        // Actor-only sabotage: use "Plan revision:" with the full diff intact.
        let mut lines = prompt.lines().peekable();
        let mut annotation = "";
        let mut revision_annotation = None;
        while let Some(line) = lines.next() {
            if !line.starts_with("diff --git ") {
                if !line.trim().is_empty() {
                    annotation = line.trim();
                }
                continue;
            }
            let mut in_hunk = false;
            let mut added_marker = false;
            while let Some(line) = lines.peek() {
                if line.starts_with("@@ ") {
                    in_hunk = true;
                } else if !(line.starts_with("index ")
                    || line.starts_with("--- ")
                    || line.starts_with("+++ ")
                    || (in_hunk && line.starts_with([' ', '+', '-', '\\'])))
                {
                    break;
                }
                added_marker |= in_hunk
                    && line.starts_with('+')
                    && !line.starts_with("+++")
                    && line[1..].contains("Revised audit input");
                lines.next();
            }
            if added_marker {
                revision_annotation = Some(annotation);
                break;
            }
            // Another diff cannot inherit this section's label.
            annotation = "";
        }
        let annotation = revision_annotation
            .unwrap_or_else(|| panic!("missing revised added content in Git diff: {prompt}"));
        assert!(
            annotation
                .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                .any(|word| word.eq_ignore_ascii_case("operator")),
            "missing operator label attached to revised Git diff: {prompt}"
        );
    }
    let rows = project.rows("SELECT id, ticket_revision, report, result FROM execution WHERE kind = 'implementation' ORDER BY id");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0]["ticket_revision"], 1);
    assert!(rows[0]["report"].is_null());
    let old: Value = serde_json::from_str(rows[0]["result"].as_str().unwrap()).unwrap();
    assert_eq!(old["outputs"], json!({}));
    assert!(old["outcome"].is_null());
    assert_eq!(rows[1]["ticket_revision"], if revise { 1 } else { 2 });
    let publication: Value = serde_json::from_str(rows[1]["report"].as_str().unwrap()).unwrap();
    assert_eq!(publication["execution"], rows[1]["id"]);
    assert_eq!(publication["ticket_revision"], if revise { 1 } else { 2 });
    assert_eq!(
        publication["report"],
        json!({"summary": "Fixture audit", "findings": []})
    );
    assert!(
        project
            .rows("SELECT id FROM attention WHERE kind = 'stopped' AND reason = 'failed'")
            .is_empty()
    );
    assert!(machine.model.requests().iter().any(|request| {
        request
            .tool_results()
            .iter()
            .any(|(_, text)| text.contains("superseded_report"))
    }));
}

/// G8/G6: a real pre-target daemon stops a read-only planner before it
/// publishes. Upgrade must refuse continuation without inventing admission
/// or verification evidence. An edit must not schedule that unbound attempt.
/// Abandon then fresh start in the same project is the successful control.
/// Old binary and fixture supply historical rows; spec supplies unbound_report
/// and unknown evidence. This is quiescent upgrade, not a crash: the old
/// execution has ended before replacing the daemon, so G5 does not mask G8.
/// Sabotage: allow unbound read-only start, schedule its pending edit, backfill
/// admission/node/results, or require bindings for ended planner history.
/// Existing upgrade test covers ended proposals; none resumes a live planner.
#[test]
fn an_unbound_legacy_planner_requires_a_fresh_admitted_attempt() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g8-audit-legacy", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        Reply::Hold(
            held.clone(),
            Box::new(Reply::Text("old planner stopped".into())),
        )
    });
    machine.use_binary(Some(super::g15_plans_and_proposals::pre_target_binary()));
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Legacy audit",
        "--workflow",
        "plan",
    ]);
    if let Err(failure) =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hold.wait_held()))
    {
        // Capture the old fixture before Machine::drop removes it. Project
        // only identities, lifecycle and fixed error flags, never raw detail,
        // audit payloads, model bodies, headers or credentials.
        let evidence = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let requests = machine.model.requests();
            json!({
                "model_requests": requests.len(),
                "report_tool_requests": requests.iter().filter(|request| request.has_tool("yard_publish_report")).count(),
                "attempts": project.rows("SELECT id, ticket, state, outcome, cleaned FROM attempt ORDER BY id LIMIT 16"),
                "executions": project.rows("SELECT id, attempt, parent, kind, reason, status, outcome, ticket_revision, mcp, exit_cause, exit_code, handle IS NOT NULL AS has_handle, length(CAST(detail AS BLOB)) AS detail_bytes, instr(COALESCE(detail, ''), 'image') > 0 AS mentions_image, instr(COALESCE(detail, ''), 'image-missing') > 0 AS mentions_image_missing, instr(COALESCE(detail, ''), 'image not known') > 0 AS mentions_image_not_known, instr(COALESCE(detail, ''), 'box') > 0 AS mentions_box, instr(COALESCE(detail, ''), 'timed out') > 0 AS mentions_timeout, instr(COALESCE(detail, ''), 'ownership') > 0 AS mentions_ownership, instr(COALESCE(detail, ''), 'registration') > 0 AS mentions_registration FROM execution ORDER BY id LIMIT 16"),
                "attention": project.rows("SELECT id, ticket, attempt, execution, kind, reason, state FROM attention ORDER BY id LIMIT 16"),
                "history": project.rows("SELECT seq, event, ticket, attempt, execution, attention FROM audit ORDER BY seq DESC LIMIT 64")
            })
        }));
        match evidence {
            Ok(evidence) => eprintln!("legacy planner first-hold evidence: {evidence}"),
            Err(_) => eprintln!(
                "legacy planner first-hold evidence unavailable: read-only capture failed"
            ),
        }
        std::panic::resume_unwind(failure);
    }
    project.json(&["attempt", "stop", "Y-1"]);
    watch.find("old planner stopped", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "stopped"
    });
    assert!(
        project
            .rows("SELECT id FROM execution WHERE status = 'running'")
            .is_empty()
    );
    hold.release();
    machine.kill();
    machine.use_binary(None);
    machine.start();
    let before =
        project.rows("SELECT * FROM execution WHERE attempt = 1 AND kind = 'implementation'");
    let historical = project.json(&["attempt", "show", "Y-1"]);
    assert_eq!(
        historical["attempt"]["admission"],
        json!({"status": "unknown"})
    );
    assert_eq!(
        historical["executions"][0]["binding"],
        json!({"status": "unknown"})
    );
    assert!(historical["executions"][0]["result"].is_null());
    assert!(historical["executions"][0]["teardown"].is_null());
    assert_eq!(historical["executions"][0]["result_eligible"], false);
    let refused = project.refused(&["attempt", "start", "Y-1"]);
    assert_eq!(refused["code"], "refused");
    assert_eq!(
        refused["data"],
        json!({"reason": "unbound_report", "attempt": 1})
    );
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        "1",
        "--body",
        "Fresh audit input",
    ]);
    let refused = project.refused(&["attempt", "start", "Y-1"]);
    assert_eq!(refused["code"], "refused");
    assert_eq!(
        refused["data"],
        json!({"reason": "unbound_report", "attempt": 1})
    );
    project.json(&["attempt", "abandon", "Y-1"]);
    let mut watch = project.watch(0);
    project.json(&["attempt", "start", "Y-1"]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let fresh = project.json(&["attempt", "show", "Y-1"]);
    assert_eq!(fresh["executions"][0]["report"]["ticket_revision"], 2);
    assert_eq!(
        fresh["executions"][0]["report"]["report"],
        json!({"summary": "Fixture audit", "findings": []})
    );
    assert_eq!(
        project.rows("SELECT * FROM execution WHERE attempt = 1 AND kind = 'implementation'"),
        before
    );
    assert_eq!(
        project.rows("SELECT admission FROM attempt WHERE id = 1"),
        vec![json!({"admission": null})]
    );
}
