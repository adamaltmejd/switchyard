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
