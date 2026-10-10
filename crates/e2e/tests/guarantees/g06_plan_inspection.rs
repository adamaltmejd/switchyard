//! G6/G2: typed plans resolve against an exact target without admission.
use e2e::*;
use serde_json::{Value, json};

fn call(operator: &mut Operator, project: &Project, tool: &str, mut args: Value) -> Value {
    args["project"] = json!(project.path);
    operator.call(tool, args)
}

/// References, joins and repeat bounds cross the real operator/daemon
/// boundary after profile expansion and targeted overrides. Expectations
/// are the fixture's declared graph, named inputs and host git identity.
/// No older G6 block exercises typed output refs or graph eligibility.
/// Sabotage: skip output-reference validation; the missing-output override
/// resolves. Ignore types, outcomes, joins or repeat limits; their malformed
/// overrides resolve. Treat any valid graph as runnable; the bounded repeat
/// passes runnable validation. Read the default config for a named target;
/// feature/a's different agent instructions disappear. Admit on inspection;
/// the externally read store contains an attempt or execution.
/// Filter landing checks by configured stage in workflow_plan, or bind
/// their input to implementer.result: the literal full-chain fixture differs.
/// Native boxed check inputs are observed after a target advance forces a
/// distinct merged ref. No existing block compares the inspected full gate
/// set with the native candidate and landing executions.
/// Skip the native second candidate-stage gate at landing, or check out the
/// candidate in its box: the instances or boxed git identities differ.
#[test]
fn typed_plans_inspect_without_admission_and_refuse_unsupported_runtime() {
    let machine = Machine::new("g6plans", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Feature")],
        )
    });
    machine.start();
    let fixture = config(
        r#"
[gates.check]
command = "test -f feature.txt && git rev-parse HEAD"
stage = "candidate"

[gates.final]
command = "test -f target.txt && git rev-parse HEAD"
stage = "landing"

[profiles.choose]
access = "read-only"
outcomes = ["selected"]
[profiles.choose.runner]
kind = "agent"
agent = "worker"
instructions = "Choose on main."
[profiles.choose.outputs]
choice = "selection"

[presets.selection.nodes.choose]
profile = "choose"
[presets.selection.nodes.choose.routes]
selected = [{ to = "choose", repeat = 2 }]
"#,
    )
    .replace("review = [\"correctness\"]", "review = \"none\"");
    let project = Project::new(&machine, "p", &fixture);
    project.git(&["checkout", "-qb", "feature/a"]);
    project.write(
        ".yard/config.toml",
        &fixture.replace("Choose on main.", "Choose on feature/a."),
    );
    project.git(&["add", ".yard/config.toml"]);
    project.git(&["commit", "-qm", "Target-specific profile"]);
    let feature_head = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/a"]);
    let mut operator = machine.operator();
    operator.initialize();
    let audit_before = project.rows("SELECT * FROM audit ORDER BY seq");
    let expected = json!({"inputs": {}, "nodes": {"choose": {
        "runner": {"kind": "agent", "agent": "worker", "instructions": "Choose on feature/a."},
        "access": "read-only", "inputs": {}, "outputs": {"choice": "selection"},
        "outcomes": ["selected"], "routes": {"selected": [{"to": "choose", "repeat": 2}]},
        "requires": []
    }}});
    let resolved = call(
        &mut operator,
        &project,
        "yard_plan_resolve",
        json!({"target": "feature/a", "preset": "selection"}),
    );
    assert_eq!(resolved["result"]["plan"], expected);
    assert_eq!(resolved["result"]["config_head"], feature_head);
    assert_eq!(resolved["result"]["target"], "feature/a");
    assert_eq!(resolved["result"]["structurally_valid"], true);
    assert_eq!(resolved["result"]["runnable"], false);
    let refused = call(
        &mut operator,
        &project,
        "yard_plan_validate",
        json!({"target": "feature/a", "preset": "selection"}),
    );
    assert_eq!(refused["error"]["code"], "refused");
    assert_eq!(
        refused["error"]["data"],
        json!({"shape": "bounded-repeat", "structurally_valid": true, "runnable": false})
    );

    // Every refused override has its corrected version in this project.
    let cases = [
        (
            json!({"inputs": {"choice": {"node": "absent", "output": "choice", "kind": "selection"}}}),
            "nodes.choose.inputs.choice.node",
            "unresolved_reference",
        ),
        (
            json!({"inputs": {"choice": {"node": "choose", "output": "absent", "kind": "selection"}}}),
            "nodes.choose.inputs.choice.output",
            "unresolved_reference",
        ),
        (
            json!({"inputs": {"choice": {"node": "choose", "output": "choice", "kind": "candidate"}}}),
            "nodes.choose.inputs.choice.kind",
            "output_type_mismatch",
        ),
        (
            json!({"routes": {"absent": [{"to": "choose", "repeat": 2}]}}),
            "nodes.choose.routes.absent",
            "undeclared_outcome",
        ),
        (
            json!({"routes": {"selected": [{"to": "choose"}]}}),
            "nodes.choose.routes.selected.0.repeat",
            "unbounded_repeat",
        ),
        (
            json!({"routes": {"selected": [{"to": "choose", "repeat": 0}]}}),
            "nodes.choose.routes.selected.0.repeat",
            "unbounded_repeat",
        ),
        (
            json!({"requires": ["absent"]}),
            "nodes.choose.requires.absent",
            "unresolved_or_duplicate_result",
        ),
    ];
    let corrected = json!({
        "inputs": {"choice": {"node": "choose", "output": "choice", "kind": "selection"}},
        "requires": ["choice"], "routes": {"selected": [{"to": "choose", "repeat": 2}]}
    });
    let mut corrected_expected = expected.clone();
    corrected_expected["nodes"]["choose"]["inputs"] = corrected["inputs"].clone();
    corrected_expected["nodes"]["choose"]["requires"] = corrected["requires"].clone();
    for (patch, input, reason) in cases {
        let answer = call(
            &mut operator,
            &project,
            "yard_plan_resolve",
            json!({"target": "feature/a", "preset": "selection", "overrides": {"choose": patch}}),
        );
        assert_eq!(answer["error"]["code"], "invalid", "{answer}");
        assert_eq!(
            answer["error"]["data"],
            json!({"input": input, "reason": reason})
        );
        let control = call(
            &mut operator,
            &project,
            "yard_plan_resolve",
            json!({"target": "feature/a", "preset": "selection", "overrides": {"choose": corrected}}),
        );
        assert_eq!(control["result"]["plan"], corrected_expected);
    }
    // A complete one-off graph and a preset with overrides share the model.
    let one_off = call(
        &mut operator,
        &project,
        "yard_plan_resolve",
        json!({"target": "feature/a", "graph": corrected_expected}),
    );
    assert_eq!(one_off["result"]["plan"], corrected_expected);

    let legacy = call(
        &mut operator,
        &project,
        "yard_plan_validate",
        json!({"target": "main"}),
    );
    let legacy_expected = json!({"inputs": {"merged": "candidate"}, "nodes": {
        "implementer": {"runner": {"kind": "agent", "agent": "worker", "instructions": ""},
            "access": "write-allowed", "inputs": {}, "outputs": {"result": "candidate"},
            "outcomes": ["complete"], "routes": {"complete": [{"to": "candidate-gate-check", "repeat": null}]}, "requires": []},
        "candidate-gate-check": {"runner": {"kind": "command", "gate": "check"}, "access": "write-allowed",
            "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"],
            "routes": {"complete": [{"to": "landing-gate-check", "repeat": null}]}, "requires": ["candidate"]},
        "landing-gate-check": {"runner": {"kind": "command", "gate": "check"}, "access": "write-allowed",
            "inputs": {"merged": {"node": null, "output": "merged", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"],
            "routes": {"complete": [{"to": "landing-gate-final", "repeat": null}]}, "requires": ["merged"]},
        "landing-gate-final": {"runner": {"kind": "command", "gate": "final"}, "access": "write-allowed",
            "inputs": {"merged": {"node": null, "output": "merged", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"], "routes": {}, "requires": ["merged"]}
    }});
    assert_eq!(legacy["result"]["plan"], legacy_expected);
    assert_eq!(legacy["result"]["runnable"], true);
    assert_eq!(legacy["result"]["shape"], "legacy-linear");
    assert!(project.rows("SELECT id FROM attempt").is_empty());
    assert!(project.rows("SELECT id FROM execution").is_empty());
    assert!(project.rows("SELECT id FROM ticket").is_empty());
    assert_eq!(
        project.rows("SELECT * FROM audit ORDER BY seq"),
        audit_before
    );

    // Runnable means the existing workflow can run: real Pi and gate boxes,
    // through the scripted host model. Inspection itself has admitted none.
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Feature"]);
    let approval = history.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    let candidate = git(
        &project.path.join(".yard/local/attempts/1/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    // Advance only target history, preserving gate and approval policy. The
    // landing ref must then differ from implementer.result.
    project.git(&["checkout", "-q", "main"]);
    project.write("target.txt", "target advance\n");
    project.git(&["add", "target.txt"]);
    project.git(&["commit", "-qm", "Advance target before landing"]);
    project.json(&["sync", "main"]);
    let approved = call(
        &mut operator,
        &project,
        "yard_attempt_approve",
        json!({
            "ticket": "Y-1", "target": "main", "head": candidate,
            "proof": approval["data"]["payload"]["proof"]
        }),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    history.event("landing.recorded", &[]);
    let merged = project.canonical_head();
    assert_ne!(candidate, merged);
    // These three instances come from the fixture's declared phase controls,
    // not from walking the resolver's graph. Box git output establishes the
    // actual checkout identity as well as the externally observed row head.
    let checks = project
        .rows("SELECT name, reason, head, detail FROM execution WHERE kind = 'gate' ORDER BY id");
    assert_eq!(
        checks
            .iter()
            .map(|row| json!({"name": row["name"], "reason": row["reason"], "head": row["head"]}))
            .collect::<Vec<_>>(),
        vec![
            json!({"name": "check", "reason": "candidate", "head": candidate}),
            json!({"name": "check", "reason": "landing", "head": merged}),
            json!({"name": "final", "reason": "landing", "head": merged}),
        ]
    );
    for (row, head) in checks.iter().zip([&candidate, &merged, &merged]) {
        assert!(row["detail"].as_str().unwrap().contains(head), "{row}");
    }
}
