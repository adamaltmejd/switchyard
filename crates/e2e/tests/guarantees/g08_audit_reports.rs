//! G8: an audit is an explicit execution-bound publication.

use e2e::*;
use serde_json::{Value, json};

/// G8 boundary and ownership: a blank report cannot
/// publish; the allowed report succeeds in the same execution, then a
/// duplicate cannot replace it. The real model holds after publication, so neither
/// completion nor eligibility can precede teardown. Expected content comes
/// from this fixture; frozen head comes from host git.
/// Sabotage: skip payload validation, overwrite report, bind to mutable identities, or infer a result at publication.
/// Existing review scenarios publish verdicts, not reports.
#[test]
fn an_audit_publication_is_validated_bound_and_held_until_teardown() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g8-audit", move |request| {
        match request.tool_results().len() {
            0 => Reply::Tools(vec![tool(
                "yard_publish_report",
                json!({
                    "outcome": "complete", "report": {"summary": " ", "findings": []}
                }),
            )]),
            1 => Reply::Tools(vec![publish_report()]),
            2 => Reply::Tools(vec![publish_report()]),
            _ => Reply::Hold(held.clone(), Box::new(Reply::Text("done".into()))),
        }
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let head = project.canonical_head();
    let mut watch = project.watch(0);
    project.ok("ticket.new", json!({"title": "Audit", "workflow": "plan"}));
    hold.wait_held();
    let show = project.ok("attempt.show", json!({"ticket": "Y-1"}));
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
    for token in ["invalid_report", "duplicate_report"] {
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
    let show = project.ok("attempt.show", json!({"ticket": "Y-1"}));
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
