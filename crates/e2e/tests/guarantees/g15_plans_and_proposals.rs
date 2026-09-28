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
/// parent would have come first.
///
/// Sabotage: make `admit::scheduled` start read-only tickets again; the
/// parent is admitted once its children are done.
#[test]
fn a_plan_proposes_children_that_block_it() {
    let answer = Arc::new(Mutex::new(String::new()));
    let seen = answer.clone();
    let machine = Machine::new("g15-plan", move |request| {
        if !request.prompt().contains("Plan the release") {
            return Reply::Text("nothing to plan".into());
        }
        match request.tool_results().len() {
            0 => Reply::Tools(vec![
                bash("cd /workspace && touch planted 2>/dev/null; echo \"write=$?\""),
                propose(
                    json!({ "kind": "ticket", "key": "a", "title": "Child A", "parked": true }),
                ),
                propose(
                    json!({ "kind": "ticket", "key": "b", "title": "Child B", "parked": true }),
                ),
                propose(json!({ "kind": "edit", "body": "The plan: A, then B." })),
            ]),
            _ => {
                *seen.lock().unwrap() = request.tool_results()[0].1.clone();
                Reply::Text("planned".into())
            }
        }
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
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
    assert!(!answer.lock().unwrap().contains("write=0"));
    let raised: Vec<Value> = project
        .rows("SELECT kind FROM attention ORDER BY id")
        .into_iter()
        .map(|row| row["kind"].clone())
        .collect();
    assert_eq!(
        raised,
        [json!("proposal"), json!("proposal"), json!("proposal")]
    );

    for (id, _) in proposals(&project) {
        project.json(&["proposal", "accept", &id.to_string()]);
    }
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
}

/// Closing a ticket with a live attempt is refused, naming the attempt.
/// Control: once the attempt is abandoned, closing succeeds.
///
/// Sabotage: drop the live-attempt check in `admit::ticket_close`; the
/// ticket closes under its running worker.
#[test]
fn closing_a_ticket_with_a_live_attempt_is_refused() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g15-close", move |_| {
        Reply::Hold(held.clone(), Box::new(Reply::Text("done".into())))
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    project.json(&["ticket", "new", "--title", "Busy"]);
    hold.wait_held();

    let refused = project.refused(&["ticket", "done", "Y-1", "--reason", "Not needed"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["attempt"], 1);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "open");

    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);
    project.json(&["ticket", "done", "Y-1", "--reason", "Not needed"]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
    hold.release();
}

/// One execution proposing A and B, B depending on A: accepting both mints
/// A first and B's edge names it. B accepted first is refused, naming its
/// unresolved reference, and changes nothing.
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
    assert_eq!(raised.len(), 4);
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
    assert!(refused["message"].as_str().unwrap().contains("\"a\""));
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
