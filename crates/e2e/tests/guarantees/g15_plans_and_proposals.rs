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
/// is the clone. Control: the dependent, a worker on a writable workflow in
/// the same project, runs the same probe and its write succeeds.
///
/// The later plan also proposes A and B, B depending on A, as the first
/// plan did: accepting both mints A first and B's edge names it. B accepted
/// first is refused and changes nothing, though the first plan's A is
/// already minted.
///
/// Sabotage: make `admit::scheduled` start read-only tickets again; the
/// parent is admitted once its children are done. Or mount the clone
/// writable for `access = "read-only"`; the plan's write succeeds. Or
/// accept an edit proposal at the ticket's current revision; the second
/// edit applies and overwrites the plan. Or make `admit::resolve_reference`
/// resolve a key from any execution's proposals; the later B's edge names
/// the first plan's A.
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
        if request.prompt().contains("Plan more") {
            return match request.tool_results().len() {
                0 => Reply::Tools(vec![
                    propose(
                        json!({ "kind": "ticket", "key": "a", "title": "More A", "parked": true }),
                    ),
                    propose(json!({ "kind": "ticket", "key": "b", "title": "More B",
                                    "parked": true, "depends_on": ["a"] })),
                ]),
                _ => Reply::Text("planned".into()),
            };
        }
        match request.tool_results().len() {
            0 => Reply::Tools(vec![
                bash(probe),
                propose(
                    json!({ "kind": "ticket", "key": "a", "title": "Child A", "parked": true }),
                ),
                propose(json!({ "kind": "ticket", "key": "b", "title": "Child B",
                                "parked": true, "depends_on": ["a"] })),
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
        "Write here",
        "--workflow",
        "write",
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
    for (_, payload) in &pending {
        if payload["kind"] == "edit" {
            assert_eq!(payload["revision"], 1, "{payload}");
        }
    }
    // The proposals are raised in any order; accept Child A before Child B,
    // whose edge names it.
    let mut accept: Vec<&(i64, Value)> = pending.iter().filter(|(id, _)| *id != stale).collect();
    accept.sort_by_key(|(_, payload)| payload["title"].as_str().unwrap_or_default().to_owned());
    for (id, _) in accept {
        project.json(&["proposal", "accept", &id.to_string()]);
    }
    // The refusal changes no row. Cleanup executions for the ended plan
    // attempts land on their own schedule, so a `MAX(seq)` read would race
    // them; observe the body and revision the stale edit could change, and
    // the item that stays open.
    let refused = project.refused(&["proposal", "accept", &stale.to_string()]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], 1);
    assert_eq!(refused["data"]["current"], 2);
    let plan = project.json(&["ticket", "show", "Y-1"]);
    assert_eq!(
        (plan["body"].as_str(), plan["revision"].as_i64()),
        (Some("The plan: A, then B."), Some(2))
    );
    assert_eq!(
        project.rows(&format!("SELECT state FROM attention WHERE id = {stale}")),
        vec![json!({ "state": "open" })]
    );
    // Reject it so no open edit proposal blocks the parent's readiness; the
    // `admit::scheduled` sabotage below must be free to start it again.
    project.json(&["proposal", "reject", &stale.to_string()]);

    assert_eq!(depends_on(&project, 1), [3, 4]);
    let parent = project.json(&["ticket", "show", "Y-1"]);
    assert_eq!(parent["body"], "The plan: A, then B.");
    for child in ["Y-3", "Y-4"] {
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

    // The later plan's B, accepted before its own A, must not reach the
    // first plan's A, already minted as Y-3. The refusal changes no row.
    // Cleanup executions for the ended plan attempts land on their own
    // schedule, so a `MAX(seq)` read would race them; observe the mint and
    // the item the refused accept could touch.
    let raised = proposals(&project);
    let titled = |title: &str| {
        raised
            .iter()
            .find(|(_, payload)| payload["title"] == title)
            .unwrap()
            .0
    };
    let (more_a, more_b) = (titled("More A"), titled("More B"));
    let refused = project.refused(&["proposal", "accept", &more_b.to_string()]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert!(
        project
            .rows("SELECT id FROM ticket WHERE title = 'More B'")
            .is_empty(),
        "the refused accept minted More B"
    );
    assert_eq!(
        project.rows(&format!("SELECT state FROM attention WHERE id = {more_b}")),
        vec![json!({ "state": "open" })]
    );

    let a = project.json(&["proposal", "accept", &more_a.to_string()]);
    let b = project.json(&["proposal", "accept", &more_b.to_string()]);
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
        "More A"
    );
    assert_eq!(depends_on(&project, name(&b)), [name(&a)]);

    project.json(&["ticket", "done", "Y-1", "--reason", "Its children are done"]);
    let admitted = watch.event("attempt.admitted", &[]);
    assert_eq!(admitted["ticket"], "Y-2");

    let approval = watch.until("Y-2 approval", |event| {
        event["event"] == "attention.raised" && event["ticket"] == "Y-2"
    });
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let control = answers.lock().unwrap()[1].clone();
    assert!(control.contains("mounted"), "{control}");
    assert!(control.contains("write=0"), "{control}");

    // A waiting attempt is live even with no execution running. Sabotage:
    // key admit::ticket_close on running executions; this close succeeds.
    // Control: abandon that attempt and the same close succeeds.
    let attempt = project.rows("SELECT id FROM attempt WHERE ticket = 2")[0]["id"].clone();
    assert!(
        project
            .rows(&format!(
                "SELECT id FROM execution WHERE attempt = {attempt} AND status = 'running'"
            ))
            .is_empty()
    );
    let refused = project.refused(&["ticket", "done", "Y-2", "--reason", "Not needed"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["attempt"], attempt);
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["state"], "open");
    project.json(&["ticket", "park", "Y-2"]);
    project.json(&["attempt", "abandon", "Y-2"]);
    project.json(&["ticket", "done", "Y-2", "--reason", "Not needed"]);
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["state"], "done");
}
