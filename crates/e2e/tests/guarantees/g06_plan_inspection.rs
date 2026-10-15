//! G6/G2: typed plans resolve against an exact target without admission.
use e2e::*;
use serde_json::{Value, json};

fn call(operator: &mut Operator, project: &Project, tool: &str, mut args: Value) -> Value {
    args["project"] = json!(project.path);
    operator.call(tool, args)
}

/// G6/G2: admission, unlike inspection, must refuse unsupported shapes before
/// reserving work. Sabotage: bypass runnable_workflow at admission; the repeat
/// creates rows or reaches the model. Rewrite persisted selection on sync or
/// allow selection on a live attempt; inspection or active_plan differs. Drop targeted
/// overrides or ignore the one-off graph; the literal fixture plan differs.
/// The predecessor proves graph validation and frozen source recovery; this
/// block proves the selected admission boundary, not those mechanisms again.
#[test]
fn selected_admission_refuses_without_work_and_freezes_operator_inputs() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g6admit", move |request| {
        if request.opens() && request.last_user().contains("Chosen old.") {
            return Reply::Hold(held.clone(), Box::new(planned(request)));
        }
        planned(request)
    });
    machine.start();
    let fixture = config(
        r#"
[workflows.chosen]
access = "read-only"
review = "none"
instructions = "Chosen old."

[profiles.reporter]
access = "read-only"
outcomes = ["complete"]
[profiles.reporter.runner]
kind = "agent"
agent = "worker"
instructions = "Template old."
[profiles.reporter.outputs]
result = "report"

[presets.report.nodes.implementer]
profile = "reporter"
"#,
    );
    let project = Project::new(&machine, "p", &fixture);
    let mut operator = machine.operator();
    operator.initialize();
    let mut history = project.watch(0);
    call(
        &mut operator,
        &project,
        "yard_ticket_new",
        json!({"title": "Report", "workflow": "plan"}),
    );
    let planner = history.event("attempt.ended", &[("outcome", "planned")]);
    // Attempt completion precedes checked release. Snapshot only after its
    // cleanup also releases, so background settlement cannot change the rows.
    let cleanup = history.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner["attempt"]
    });
    history.find("planner cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let attempts = project.rows("SELECT * FROM attempt ORDER BY id");
    let executions = project.rows("SELECT * FROM execution ORDER BY id");
    let requests = machine.model.requests().len();
    let refused = call(
        &mut operator,
        &project,
        "yard_attempt_start",
        json!({
            "ticket": "Y-1", "preset": "report", "overrides": {"implementer": {
                "routes": {"complete": [{"to": "implementer", "repeat": 2}]}
            }}
        }),
    );
    assert_eq!(refused["error"]["code"], "refused");
    assert_eq!(
        refused["error"]["data"],
        json!({"input": "plan", "shape": "bounded-repeat",
        "structurally_valid": true, "runnable": false})
    );
    assert_eq!(project.rows("SELECT * FROM attempt ORDER BY id"), attempts);
    assert_eq!(
        project.rows("SELECT * FROM execution ORDER BY id"),
        executions
    );
    assert_eq!(machine.model.requests().len(), requests);

    let expected = json!({"inputs": {}, "nodes": {"implementer": {
        "runner": {"kind": "agent", "agent": "worker", "instructions": "Chosen old."},
        "access": "read-only", "inputs": {}, "outputs": {"result": "report"},
        "outcomes": ["complete"], "routes": {}, "requires": []
    }}});
    let admitted = call(
        &mut operator,
        &project,
        "yard_attempt_start",
        json!({
            "ticket": "Y-1", "preset": "report", "overrides": {"implementer": {
                "runner": {"kind": "agent", "agent": "worker", "instructions": "Chosen old."}
            }}
        }),
    );
    assert_eq!(admitted["result"]["admission"]["plan"], expected);
    assert_eq!(admitted["result"]["admission"]["revision"], 1);
    assert_eq!(
        admitted["result"]["admission"]["selection"]["preset"],
        "report"
    );
    let source = project.canonical_head();
    assert_eq!(
        admitted["result"]["admission"]["inputs"]["source_commit"],
        source
    );
    hold.wait_held();
    let active = call(
        &mut operator,
        &project,
        "yard_attempt_start",
        json!({"ticket": "Y-1", "graph": expected}),
    );
    assert_eq!(active["error"]["code"], "refused");
    assert_eq!(
        active["error"]["data"],
        json!({"input": "plan", "reason": "active_plan"})
    );
    project.write(
        ".yard/config.toml",
        &fixture
            .replace("Chosen old.", "Chosen new.")
            .replace("Template old.", "Chosen new."),
    );
    project.git(&["add", ".yard/config.toml"]);
    project.git(&["commit", "-qm", "New report template"]);
    project.json(&["sync"]);
    let inspection = call(
        &mut operator,
        &project,
        "yard_attempt_show",
        json!({"ticket": "Y-1"}),
    );
    assert_eq!(
        inspection["result"]["attempt"]["admission"],
        admitted["result"]["admission"]
    );
    hold.release();
    history.event("attempt.ended", &[("outcome", "planned")]);
}
