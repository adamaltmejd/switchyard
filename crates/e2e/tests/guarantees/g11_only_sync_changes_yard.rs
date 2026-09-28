//! G11: Only the operator's sync changes `.yard` and canonical from outside.

use e2e::*;
use serde_json::json;

fn unreviewed() -> String {
    config("").replace("review = [\"correctness\"]", "review = \"none\"")
}

const EXTRA_GATE: &str = "\n[gates.extra]\ncommand = \"true\"\nstage = \"candidate\"\n";

/// A worker commit under `.yard` comes back with the reason and no gate
/// runs; the same change through `yard sync` is in force for the next
/// execution. The change adds a candidate gate: refused from the worker, it
/// runs on the candidate once the operator syncs it.
///
/// Sabotage: make `supervise::implement` skip its `.yard` check; the
/// worker's commit becomes the candidate.
#[test]
fn a_worker_yard_change_is_refused_and_sync_carries_it() {
    let configured = format!("{}{EXTRA_GATE}", unreviewed());
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
                 && git add -A && git commit -q -m 'Add feature and a gate' && echo committed"
            ))],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed());
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = watch.attention();
    assert_eq!(first["data"]["kind"], "approval", "{first}");
    let head = first["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        project.rows("SELECT reason, outcome FROM execution ORDER BY id"),
        vec![
            json!({ "reason": "first", "outcome": "refused" }),
            json!({ "reason": "repair", "outcome": "candidate" }),
        ]
    );
    let openings: Vec<_> = machine
        .model
        .requests()
        .into_iter()
        .filter(ModelRequest::opens)
        .collect();
    assert!(openings[1].last_user().contains(".yard"));

    project.reconfigure(&configured);
    let second = watch.attention();
    assert_eq!(second["data"]["kind"], "approval", "{second}");
    assert_eq!(second["data"]["payload"]["head"], head.as_str());
    assert_eq!(
        project.rows("SELECT name, head, outcome FROM execution WHERE kind = 'gate'"),
        vec![json!({ "name": "extra", "head": head, "outcome": "pass" })]
    );
}

/// A checkout and canonical that each hold a commit the other lacks: both
/// directions refuse naming both heads; a fast-forward passes. Canonical
/// moves by a landing, the checkout by an operator commit.
///
/// Sabotage: make `admit::sync` import whenever the heads differ; the
/// operator's commit replaces the landing in canonical.
#[test]
fn diverged_checkout_and_canonical_refuse_both_ways() {
    let machine = Machine::new("g11-diverged", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &unreviewed().replace("approve = \"manual\"", "approve = \"auto\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
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
