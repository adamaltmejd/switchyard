//! G11: Only the operator's sync changes `.yard` and canonical from outside.

use e2e::*;
use serde_json::json;

fn unreviewed() -> String {
    config("").replace("review = [\"correctness\"]", "review = \"none\"")
}

const BASE_GATE: &str =
    "\n[gates.base]\ncommand = \"true\"\nstage = \"candidate\"\nruns_in = \"host\"\n";
const EXTRA_GATE: &str =
    "\n[gates.extra]\ncommand = \"true\"\nstage = \"candidate\"\nruns_in = \"host\"\n";

/// A worker commit under `.yard` comes back with the reason and no gate
/// runs on it: the project's candidate gate runs only on the repaired head.
/// The same change through `yard sync` is in force for the next execution.
/// The change adds a second candidate gate: refused from the worker, it runs
/// on the candidate once the operator syncs it.
///
/// Then a checkout and canonical that each hold a commit the other lacks:
/// both directions refuse naming both heads; a fast-forward passes.
/// Canonical moves by the candidate's landing, the checkout by an operator
/// commit.
///
/// Sabotage: make `supervise::implement` skip its `.yard` check; the
/// worker's commit becomes the candidate and the gate runs on it. Make
/// `admit::sync` import whenever the heads differ; the operator's commit
/// replaces the landing in canonical.
#[test]
fn only_the_operators_sync_changes_yard_and_canonical() {
    let gated = format!("{}{BASE_GATE}", unreviewed());
    let configured = format!("{gated}{EXTRA_GATE}");
    let change = configured.replace('\'', "'\\''");
    let machine = Machine::new("g11-yard", move |request| {
        if request.opens() && request.last_user().contains(".yard") && request.turn() > 0 {
            return Reply::Tools(vec![bash(
                "cd /workspace && git reset -q --hard HEAD~1 && printf 'feature\\n' > feature.txt \
                 && git add -A && git commit -q -m 'Add feature' && echo committed",
            )]);
        }
        act(
            request,
            vec![bash(&format!(
                "cd /workspace && printf '%s' '{change}' > .yard/config.toml && printf 'feature\\n' > feature.txt \
                 && git add -A && git commit -q -m 'Add feature and a gate' && git rev-parse HEAD"
            ))],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &gated);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = watch.attention();
    assert_eq!(first["data"]["kind"], "approval", "{first}");
    let head = first["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        project.rows(
            "SELECT reason, outcome FROM execution WHERE kind = 'implementation' ORDER BY id"
        ),
        vec![
            json!({ "reason": "first", "outcome": "refused" }),
            json!({ "reason": "repair", "outcome": "candidate" }),
        ]
    );
    // The refused head, as git in the box printed it.
    let refused = machine
        .model
        .requests()
        .iter()
        .find_map(|request| request.tool_results().first().cloned())
        .expect("the first execution's commit")
        .1
        .trim()
        .to_string();
    assert_ne!(refused, head);
    assert_eq!(
        project.rows("SELECT name, head, outcome FROM execution WHERE kind = 'gate'"),
        vec![json!({ "name": "base", "head": head, "outcome": "pass" })]
    );

    project.reconfigure(&configured);
    let second = watch.attention();
    assert_eq!(second["data"]["kind"], "approval", "{second}");
    assert_eq!(second["data"]["payload"]["head"], head.as_str());
    // The synced gate is in force for the candidate.
    assert_eq!(
        project.rows("SELECT head, outcome FROM execution WHERE kind = 'gate' AND name = 'extra'"),
        vec![json!({ "head": head, "outcome": "pass" })]
    );

    project.json(&["attempt", "approve", "Y-1", "--head", &head]);
    watch.event("landing.recorded", &[]);
    let landed = project.canonical_head();

    project.write("notes.md", "notes\n");
    project.git(&["add", "notes.md"]);
    project.git(&["commit", "--quiet", "-m", "Operator notes"]);
    let local = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let refused = project.refused(&["sync"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(
        refused["data"],
        json!({ "checkout": local, "canonical": landed })
    );
    assert_eq!(project.canonical_head(), landed);
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), local);

    project.git(&["reset", "--quiet", "--hard", "HEAD~1"]);
    assert_eq!(project.json(&["sync"])["sync"], "consumed");
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), landed);
    project.git(&["cherry-pick", &local]);
    assert_eq!(project.json(&["sync"])["sync"], "imported");
    assert_eq!(
        project.canonical_head(),
        project.git(&["rev-parse", "HEAD"]).trim()
    );
}
