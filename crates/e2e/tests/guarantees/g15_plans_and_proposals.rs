//! G15: Plans and proposals resolve.

use e2e::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn propose(arguments: Value) -> ToolCall {
    tool("yard_propose", arguments)
}

/// The proposals raised so far, in order: (attention id, payload).
fn proposals(project: &Project) -> Vec<(i64, Value)> {
    project
        .rows("SELECT id, payload FROM attention WHERE kind = 'proposal' ORDER BY id")
        .into_iter()
        .map(|row| {
            (
                row["id"].as_i64().unwrap(),
                serde_json::from_str(row["payload"].as_str().unwrap()).unwrap(),
            )
        })
        .collect()
}

fn depends_on(project: &Project, ticket: i64) -> Vec<i64> {
    project
        .rows(&format!(
            "SELECT depends_on FROM dependency WHERE ticket = {ticket} ORDER BY depends_on"
        ))
        .into_iter()
        .map(|row| row["depends_on"].as_i64().unwrap())
        .collect()
}

/// A ticket on `plan`: its clone refuses writes, it proposes children and a
/// body edit, and the attempt ends with nothing raised; accepting them
/// blocks the parent, which the scheduler does not start once they are
/// done; `yard ticket done` then closes it and its dependents become ready.
/// The children are proposed parked and closed by hand. A later planning
/// ticket runs to its end after the children are done, so a restart of the
/// parent would have come first. The write probe first proves `/workspace`
/// is the clone. Control: a worker on a writable workflow in the same
/// project runs the same probe and its write succeeds.
///
/// Sabotage: make `admit::scheduled` start read-only tickets again; the
/// parent is admitted once its children are done. Or mount the clone
/// writable for `access = "read-only"`; the plan's write succeeds. Or
/// accept an edit proposal at the ticket's current revision; the second
/// edit applies and overwrites the plan.
#[test]
fn a_plan_proposes_children_that_block_it() {
    let answers = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen = answers.clone();
    let probe = "cd /workspace && test -d /workspace/.git && echo mounted; \
                 touch planted 2>/dev/null; echo \"write=$?\"";
    let machine = Machine::new("g15-plan", move |request| {
        if request.prompt().contains("Write here") {
            if request.opens() {
                return Reply::Tools(vec![bash(&format!(
                    "{probe}; git add -A && git commit -q -m 'Add planted' && echo committed"
                ))]);
            }
            seen.lock()
                .unwrap()
                .push(request.tool_results()[0].1.clone());
            return Reply::Text("done".into());
        }
        if !request.prompt().contains("Plan the release") {
            return Reply::Text("nothing to plan".into());
        }
        match request.tool_results().len() {
            0 => Reply::Tools(vec![
                bash(probe),
                propose(
                    json!({ "kind": "ticket", "key": "a", "title": "Child A", "parked": true }),
                ),
                propose(
                    json!({ "kind": "ticket", "key": "b", "title": "Child B", "parked": true }),
                ),
                propose(json!({ "kind": "edit", "body": "The plan: A, then B." })),
                propose(json!({ "kind": "edit", "body": "Stale: must not apply." })),
            ]),
            _ => {
                seen.lock()
                    .unwrap()
                    .push(request.tool_results()[0].1.clone());
                Reply::Text("planned".into())
            }
        }
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[workflows.write]\nreview = \"none\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Plan the release",
        "--workflow",
        "plan",
    ]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "After the release",
        "--workflow",
        "plan",
        "--depends-on",
        "Y-1",
    ]);
    let ended = watch.event("attempt.ended", &[("outcome", "planned")]);
    assert_eq!(ended["ticket"], "Y-1");
    let plan = answers.lock().unwrap()[0].clone();
    assert!(plan.contains("mounted"), "{plan}");
    assert!(
        plan.contains("write=") && !plan.contains("write=0"),
        "{plan}"
    );
    let raised: Vec<Value> = project
        .rows("SELECT kind FROM attention ORDER BY id")
        .into_iter()
        .map(|row| row["kind"].clone())
        .collect();
    assert_eq!(
        raised,
        [
            json!("proposal"),
            json!("proposal"),
            json!("proposal"),
            json!("proposal")
        ]
    );

    // Both edit proposals bind the revision the planning execution read.
    // The first accepted one applies; the second is stale and changes no row.
    let pending = proposals(&project);
    let stale = pending
        .iter()
        .find(|(_, payload)| payload["body"] == "Stale: must not apply.")
        .unwrap()
        .0;
    for (id, payload) in &pending {
        if payload["kind"] == "edit" {
            assert_eq!(payload["revision"], 1, "{payload}");
        }
        if *id != stale {
            project.json(&["proposal", "accept", &id.to_string()]);
        }
    }
    let seq = project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"].clone();
    let refused = project.refused(&["proposal", "accept", &stale.to_string()]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], 1);
    assert_eq!(refused["data"]["current"], 2);
    assert_eq!(
        project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"],
        seq
    );
    assert_eq!(
        project.rows(&format!("SELECT state FROM attention WHERE id = {stale}")),
        vec![json!({ "state": "open" })]
    );

    assert_eq!(depends_on(&project, 1), [3, 4]);
    let parent = project.json(&["ticket", "show", "Y-1"]);
    assert_eq!(parent["body"], "The plan: A, then B.");
    for child in ["Y-3", "Y-4"] {
        assert_eq!(project.json(&["ticket", "show", child])["parked"], true);
        project.json(&["ticket", "done", child, "--reason", "Done by hand"]);
    }
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Plan more",
        "--workflow",
        "plan",
    ]);
    watch.until("Y-5 planned", |event| {
        event["event"] == "attempt.ended" && event["ticket"] == "Y-5"
    });
    assert_eq!(
        project
            .rows("SELECT id FROM attempt WHERE ticket = 1")
            .len(),
        1,
        "the parent was started again"
    );
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE ticket = 2")
            .is_empty()
    );

    project.json(&["ticket", "done", "Y-1", "--reason", "Its children are done"]);
    let admitted = watch.event("attempt.admitted", &[]);
    assert_eq!(admitted["ticket"], "Y-2");

    project.json(&[
        "ticket",
        "new",
        "--title",
        "Write here",
        "--workflow",
        "write",
    ]);
    let approval = watch.until("Y-6 approval", |event| {
        event["event"] == "attention.raised" && event["ticket"] == "Y-6"
    });
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let control = answers.lock().unwrap()[1].clone();
    assert!(control.contains("mounted"), "{control}");
    assert!(control.contains("write=0"), "{control}");
}

/// Closing a ticket with a live attempt is refused, naming the attempt. The
/// attempt waits on its `approval` item with no execution running, so a
/// check keyed on running executions would let the close through. Control:
/// once the attempt is abandoned, closing succeeds.
///
/// Sabotage: key the live-attempt check in `admit::ticket_close` on running
/// executions; the ticket closes under its waiting attempt.
#[test]
fn closing_a_ticket_with_a_live_attempt_is_refused() {
    let machine = Machine::new("g15-close", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Busy"]);
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    assert!(
        project
            .rows("SELECT id FROM execution WHERE status = 'running'")
            .is_empty()
    );

    let refused = project.refused(&["ticket", "done", "Y-1", "--reason", "Not needed"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["attempt"], 1);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "open");

    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);
    project.json(&["ticket", "done", "Y-1", "--reason", "Not needed"]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
}

/// One execution proposing A and B, B depending on A: accepting both mints
/// A first and B's edge names it. B accepted first is refused and changes
/// nothing.
///
/// Sabotage: make `admit::resolve_reference` resolve a key from any
/// execution's proposals; B's edge names the other execution's A.
#[test]
fn proposals_of_one_execution_mint_in_order() {
    let machine = Machine::new("g15-siblings", |request| {
        let prefix = if request.prompt().contains("First plan") {
            "First"
        } else {
            "Second"
        };
        match request.tool_results().len() {
            0 => Reply::Tools(vec![
                propose(
                    json!({ "kind": "ticket", "key": "a", "title": format!("{prefix} A"), "parked": true }),
                ),
                propose(
                    json!({ "kind": "ticket", "key": "b", "title": format!("{prefix} B"),
                                "parked": true, "depends_on": ["a"] }),
                ),
            ]),
            _ => Reply::Text("planned".into()),
        }
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "First plan",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Second plan",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let raised = proposals(&project);
    let titled = |title: &str| {
        raised
            .iter()
            .find(|(_, payload)| payload["title"] == title)
            .unwrap()
            .0
    };
    // The first execution's A, accepted first, is the key another
    // execution's B must not reach.
    let (first_a, second_a, second_b) = (titled("First A"), titled("Second A"), titled("Second B"));
    project.json(&["proposal", "accept", &first_a.to_string()]);

    let seq = project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"].clone();
    let refused = project.refused(&["proposal", "accept", &second_b.to_string()]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(
        project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"],
        seq
    );

    let a = project.json(&["proposal", "accept", &second_a.to_string()]);
    let b = project.json(&["proposal", "accept", &second_b.to_string()]);
    let name = |value: &Value| {
        value["ticket"]
            .as_str()
            .unwrap()
            .trim_start_matches("Y-")
            .parse::<i64>()
            .unwrap()
    };
    assert_eq!(
        project.json(&["ticket", "show", a["ticket"].as_str().unwrap()])["title"],
        "Second A"
    );
    assert_eq!(depends_on(&project, name(&b)), [name(&a)]);
}
