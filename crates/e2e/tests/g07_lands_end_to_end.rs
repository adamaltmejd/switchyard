//! G7: A ticket lands end to end and leaves only rows.

use e2e::*;
use serde_json::json;

/// New ticket, worker commit, candidate gate, review pass, approval, green
/// landing: canonical moves and the ticket is done. Afterwards every
/// decision has one audit event naming its target and text, the execution
/// rows carry tokens, cost, model and start reason, no handle event exists,
/// the attempt directory and its boxes are gone, and another live attempt's
/// directory and canonical are untouched.
///
/// Sabotage: make cleanup skip `remove(&project.attempt_dir(..))`; the
/// landed attempt's directory survives and the test fails.
#[test]
fn a_ticket_lands_end_to_end_and_leaves_only_rows() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g7", move |request| {
        if request.has_tool("yard_publish_review") {
            return match request.turn() {
                0 => Reply::Tools(vec![publish(json!([
                    { "priority": "P3", "file": "feature.txt", "line": 1, "body": "a nit" }
                ]))]),
                _ => Reply::Text("published".into()),
            };
        }
        if request.prompt().contains("Y-2") {
            return Reply::Hold(held.clone(), Box::new(Reply::Text("never mind".into())));
        }
        match request.turn() {
            0 => Reply::Tools(vec![commit_file("feature.txt", "feature\n", "Add feature")]),
            _ => Reply::Text("done".into()),
        }
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n"),
    );
    let start = project.canonical_head();
    let mut watch = project.watch(0);

    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create feature.txt",
    ]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Hold",
        "--body",
        "The fixture holds this one",
    ]);
    hold.wait_held();

    let raised = watch.until("approval raised", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });
    assert_eq!(raised["ticket"], "Y-1");
    let head = raised["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();
    // The candidate is the worker's commit on top of the target.
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:feature.txt")]
        ),
        "feature\n"
    );
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{head}^")]).trim(),
        start
    );

    project.json(&[
        "attempt", "approve", "Y-1", "--head", &head, "--text", "ship it",
    ]);
    watch.event("landing.recorded", &[]);
    watch.until("cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });

    // Canonical is a fast-forward to the candidate, and the ticket is done.
    assert_eq!(project.canonical_head(), head);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");

    // Every decision has one audit event naming its target, with its text.
    let events = project.rows("SELECT event, ticket, text FROM audit ORDER BY seq");
    let count = |name: &str, ticket: i64| {
        events
            .iter()
            .filter(|event| event["event"] == name && event["ticket"] == ticket)
            .count()
    };
    for name in [
        "ticket.new",
        "attempt.admitted",
        "approval.given",
        "landing.intent",
        "landing.recorded",
        "ticket.done",
    ] {
        assert_eq!(count(name, 1), 1, "{name} for Y-1 in {events:?}");
    }
    let approval = events
        .iter()
        .find(|event| event["event"] == "approval.given")
        .unwrap();
    assert_eq!(approval["text"], "ship it");
    // Handles and usage are column writes, never events.
    assert!(
        events
            .iter()
            .all(|event| !event["event"].as_str().unwrap().contains("handle")),
        "a handle event: {events:?}"
    );

    // The worker rows carry the statistics.
    let workers = project.rows(
        "SELECT kind, reason, model, provider, tokens_in, tokens_out, cost FROM execution
         WHERE attempt = 1 AND kind IN ('implementation', 'review') ORDER BY id",
    );
    assert_eq!(workers.len(), 2, "{workers:?}");
    let implementation = &workers[0];
    assert_eq!(implementation["kind"], "implementation");
    assert_eq!(implementation["reason"], "first");
    assert_eq!(implementation["model"], "fake-model");
    assert_eq!(implementation["provider"], "openrouter");
    // Two model turns at the fixture's fixed usage.
    assert_eq!(implementation["tokens_in"], 2 * PROMPT_TOKENS as i64);
    assert_eq!(implementation["tokens_out"], 2 * COMPLETION_TOKENS as i64);
    assert!(implementation["cost"].as_f64().unwrap() > 0.0);

    // The landed attempt left rows only: no directory, no box.
    assert!(!project.path.join(".yard/local/attempts/1").exists());
    let handles = project.rows(
        "SELECT handle FROM execution WHERE attempt = 1 AND kind IN ('implementation', 'gate', 'review')",
    );
    assert!(!handles.is_empty());
    let boxes = machine.boxes("dev.yard.project");
    for handle in &handles {
        assert!(
            boxes
                .iter()
                .all(|listed| listed["name"] != handle["handle"]),
            "{handle} still has a box"
        );
    }
    // The other live attempt kept its clone and its worker.
    assert!(
        project
            .path
            .join(".yard/local/attempts/2/clone/.git")
            .is_dir()
    );
    assert!(hold.is_held());
    hold.release();
}
