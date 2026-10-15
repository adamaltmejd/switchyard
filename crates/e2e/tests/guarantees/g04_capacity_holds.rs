//! G4: Capacity holds.

use e2e::*;
use serde_json::{Value, json};
use std::io::Write;

/// Run `yard attempt start` for each ticket at once; each one's JSON result
/// or error.
fn race(project: &Project, tickets: &[&str]) -> Vec<Result<Value, Value>> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = tickets
            .iter()
            .map(|ticket| {
                scope.spawn(move || {
                    let out = project.yard(&["attempt", "start", ticket, "--json"]);
                    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
                    if out.status.success() {
                        Ok(value)
                    } else {
                        Err(value["error"].clone())
                    }
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn live(project: &Project, ticket: i64) -> usize {
    project
        .rows(&format!(
            "SELECT id FROM attempt WHERE ticket = {ticket} AND state = 'live'"
        ))
        .len()
}

fn is_approval(event: &Value, ticket: &str) -> bool {
    event["event"] == "attention.raised"
        && event["ticket"] == ticket
        && event["data"]["kind"] == "approval"
}

/// The ticket's first approval item's head, whenever it was raised.
#[track_caller]
fn approval_of(watch: &mut Watch, ticket: &str) -> String {
    let event = watch.find(ticket, |event| is_approval(event, ticket));
    event["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Two `attempt start`s race for the last slot: one wins and a ticket never
/// has two live attempts. Planning tickets that ran once are started only
/// by hand, and each later run is held. First two starts of one ticket race
/// with both lanes free, so only the one-live-attempt rule stops the second;
/// then starts of two tickets race for the last lane.
///
/// Sabotage: drop the `attempt_one_live` unique index, the live-attempt
/// clause of `tickets::BLOCKER` and the live check in
/// `admit::attempt_start`; qualified together with the weak response
/// observer absent, these make the retained first live count fail at 2 vs 1.
/// Make `admit::free_lanes` ignore the lanes held in the project; both tickets
/// are admitted.
#[test]
fn racing_starts_admit_one_attempt() {
    let hold = Latch::new();
    let held = hold.clone();
    let seen = std::sync::Mutex::new(std::collections::HashSet::new());
    let machine = Machine::new("g4-race", move |request| {
        if !request.opens() {
            return Reply::Text("done".into());
        }
        if seen.lock().unwrap().insert(request.prompt()) {
            planned(request)
        } else {
            Reply::Hold(held.clone(), Box::new(Reply::Text("planned".into())))
        }
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    for title in ["Plan A", "Plan B", "Plan C"] {
        project.json(&["ticket", "new", "--title", title, "--workflow", "plan"]);
    }
    for ticket in ["Y-1", "Y-2", "Y-3"] {
        watch.find(ticket, |event| {
            event["event"] == "attempt.ended" && event["ticket"] == ticket
        });
    }

    let _ = race(&project, &["Y-1", "Y-1"]);
    assert_eq!(live(&project, 1), 1);

    let results = race(&project, &["Y-2", "Y-3"]);
    let refused: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();
    assert_eq!(refused.len(), 1, "{results:?}");
    assert_eq!(refused[0]["data"]["reason"], "capacity");
    assert_eq!(live(&project, 2) + live(&project, 3), 1);
    hold.release();
}

/// Two registered projects, each running its first execution, so both are
/// execution 1 in their own stores: the first ending leaves the second's
/// bearer working. The second's worker is held until the first has ended,
/// then records a progress note.
///
/// Sabotage: make `Grants::revoke` compare only the execution id; the
/// second's call is refused and no note is recorded.
#[test]
fn an_execution_ending_revokes_only_its_own_project_grant() {
    let first_hold = Latch::new();
    let second_hold = Latch::new();
    let (first_held, second_held) = (first_hold.clone(), second_hold.clone());
    let machine = Machine::new("g4-grants", move |request| {
        if request.opens() && request.prompt().contains("First") {
            return Reply::Hold(first_held.clone(), Box::new(Reply::Text("done".into())));
        }
        if request.opens() && request.prompt().contains("Second") {
            let note = tool("yard_progress", json!({ "note": "second's note" }));
            return Reply::Hold(second_held.clone(), Box::new(Reply::Tools(vec![note])));
        }
        Reply::Text("done".into())
    });
    machine.start();
    let first = Project::new(&machine, "a", &config(""));
    let second = Project::new(&machine, "b", &config(""));
    let mut first_watch = first.watch(0);
    let mut second_watch = second.watch(0);

    first.json(&["ticket", "new", "--title", "First"]);
    first_hold.wait_held();
    second.json(&["ticket", "new", "--title", "Second"]);
    second_hold.wait_held();
    let ids = |project: &Project| project.rows("SELECT id FROM execution");
    assert_eq!(ids(&first), ids(&second));

    first_hold.release();
    let stopped = first_watch.attention();
    assert_eq!(stopped["data"]["kind"], "stopped", "{stopped}");
    second_hold.release();
    let stopped = second_watch.attention();
    assert_eq!(stopped["data"]["kind"], "stopped", "{stopped}");
    assert_eq!(
        second.rows("SELECT progress FROM execution"),
        vec![json!({ "progress": "second's note" })]
    );
}

/// G4/G5/G13: two readers in one attempt race an independent worker for two
/// effect slots. A join requires both exact ended results. Crash with left
/// published and right held before publication; restart must retain left and
/// rerun only the incomplete right after checked ownership. A new revision
/// then requires fresh results instead of consuming the old join's inputs.
/// Expected names, counts, cap and routes come from the literal graph; Git,
/// real boxes/locks and store rows supply candidate and supplier identities.
/// Existing worker races have one execution per attempt and no exact join.
/// Sabotage: count attempts, reserve after launch, replay finished branches,
/// consume any publication or a pre-revision result, rewrite original admission,
/// or mount a reader writable or an ordinary command read-only.
/// G2/G8/G13: the post-join picker refuses foreign and historical sources
/// before publishing a subset and a duplicate with both exact contributors.
/// Sabotage: select by prose/index/latest result, drop a contributor, erase raw
/// findings, forward all findings to repair, or let picker exclusion pass a
/// required block, or bypass operator handling on an empty join carrying that
/// block. Fixture prose and
/// model profiles supply values; public store finding identities supply sources.
/// G5/G8: real successful picker teardown leaves an inherited host owner held.
/// Sabotage: clear ownership before lock freedom, or recognize only pass/fail
/// at checked release. The delegated runtime, PID, box list and same-inode lock
/// prove ownership; the original result and exact attention supply identities.
#[test]
fn concurrent_readers_share_capacity_and_join_exact_results_across_restart() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let implementation = Latch::new();
    let independent = Latch::new();
    let left = Latch::new();
    let right = Latch::new();
    let restarted = Latch::new();
    let revised = Latch::new();
    let held = [
        implementation.clone(),
        independent.clone(),
        left.clone(),
        right.clone(),
        restarted.clone(),
        revised.clone(),
    ];
    let right_runs = Arc::new(AtomicUsize::new(0));
    let runs = right_runs.clone();
    let picked = Latch::new();
    let picker_hold = picked.clone();
    let historical = Arc::new(std::sync::Mutex::new(None::<Value>));
    let picker_history = historical.clone();
    let foreign_source = Arc::new(std::sync::Mutex::new(None::<Value>));
    let picker_foreign = foreign_source.clone();
    let (reader_opened, reader_openings) = std::sync::mpsc::channel();
    let (repair_sent, repair_received) = std::sync::mpsc::channel();
    let (classified, classification_inputs) = std::sync::mpsc::channel();
    let machine = Machine::new("g4-graph-join", move |request| {
        if request
            .prompt()
            .contains("Classify these frozen failed-check inputs: ")
        {
            let prompt = request.prompt();
            let frozen = prompt
                .split("Classify these frozen failed-check inputs: ")
                .nth(1)
                .unwrap();
            let inputs = serde_json::Deserializer::from_str(frozen)
                .into_iter::<Value>()
                .next()
                .unwrap()
                .unwrap();
            if request.opens() {
                classified
                    .send(inputs["feedback"]["value"]["groups"].clone())
                    .unwrap();
            }
            return act(
                request,
                vec![tool(
                    "yard_publish_report",
                    json!({
                        "outcome": "operator", "identity": inputs["identity"],
                        "report": {"summary": "Required block remains; exact groups selected", "findings": []}
                    }),
                )],
            );
        }
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        let prompt = request.prompt();
        if request.has_tool("yard_publish_selection") {
            let frozen = prompt
                .split("Frozen inputs:\n")
                .nth(1)
                .unwrap()
                .split("\n\nCall yard_publish_selection")
                .next()
                .unwrap();
            let inputs: Value = serde_json::from_str(frozen).unwrap();
            let findings = |name: &str| {
                inputs[name]["value"]["review"]["findings"]
                    .as_array()
                    .unwrap()
            };
            let source = |name: &str, body: &str| {
                findings(name)
                    .iter()
                    .find(|finding| finding["body"] == body)
                    .unwrap()["source"]
                    .clone()
            };
            let useful = source("left", "useful");
            let left_shared = source("left", "shared");
            let right_shared = source("right", "shared");
            let foreign = picker_foreign.lock().unwrap().clone().unwrap();
            let old = picker_history.lock().unwrap().clone();
            let refusals = if old.is_some() { 4 } else { 3 };
            let count = request.tool_results().len();
            let response = || {
                let text = request.last_tool_result().unwrap().1;
                serde_json::from_str::<Value>(
                    &text[text.find('{').unwrap()..=text.rfind('}').unwrap()],
                )
                .unwrap()
            };
            let rejected = |step: usize| match step {
                0 => json!({"groups": [{"reason": "foreign", "sources": [foreign.clone()]}]}),
                1 => json!({"groups": [{"reason": "   ", "sources": [useful.clone()]}]}),
                2 => {
                    json!({"groups": [{"reason": "repeated", "sources": [useful.clone(), useful.clone()]}]})
                }
                _ => {
                    json!({"groups": [{"reason": "historical", "sources": [old.clone().unwrap()]}]})
                }
            };
            if count > 0 && count <= refusals {
                let response = response();
                assert_eq!(response["code"], "refused", "{response}");
                assert_eq!(response["data"]["reason"], "selection_input");
                assert_eq!(
                    response["data"]["identity"],
                    if count == 2 { "group" } else { "source" }
                );
                let expected = if count == 2 {
                    rejected(1)["groups"][0].clone()
                } else {
                    rejected(count - 1)["groups"][0]["sources"][0].clone()
                };
                assert_eq!(response["data"]["expected"], expected);
            }
            if count < refusals {
                return Reply::Tools(vec![tool("yard_publish_selection", rejected(count))]);
            }
            if count == refusals {
                return Reply::Tools(vec![tool(
                    "yard_publish_selection",
                    json!({"groups": [
                        {"reason": "useful subset", "sources": [useful.clone()]},
                        {"reason": "shared duplicate", "sources": [left_shared, right_shared]}
                    ]}),
                )]);
            }
            let response = response();
            assert!(response["published"].is_i64(), "{response}");
            if old.is_none() {
                *picker_history.lock().unwrap() = Some(useful);
                return Reply::Hold(picker_hold.clone(), Box::new(Reply::Text("done".into())));
            }
            return Reply::Text("done".into());
        }
        if request.has_tool("yard_publish_review") {
            if prompt.contains("Graph") && request.opens() {
                let latch = if prompt.contains("Left reader.") {
                    reader_opened.send("review-0").unwrap();
                    Some(&held[2])
                } else if prompt.contains("Revised right reader.") {
                    Some(&held[5])
                } else if prompt.contains("Right reader.") {
                    reader_opened.send("review-1").unwrap();
                    Some(if runs.fetch_add(1, Ordering::SeqCst) == 0 {
                        &held[3]
                    } else {
                        &held[4]
                    })
                } else {
                    None
                };
                if let Some(latch) = latch {
                    let model = if prompt.contains("Left reader.") {
                        "left-model"
                    } else {
                        "right-model"
                    };
                    assert_eq!(request.body["model"], model);
                    let findings = if model == "left-model" {
                        json!([{"priority": if prompt.contains("Changed candidate fixture.") { "P1" } else { "P3" }, "body": "unselected"},
                            {"priority": "P3", "body": "shared"}, {"priority": "P3", "body": "useful"}])
                    } else {
                        json!([{"priority": if prompt.contains("Changed candidate fixture.") { "P1" } else { "P3" }, "body": "shared"}])
                    };
                    return Reply::Hold(
                        latch.clone(),
                        Box::new(Reply::Tools(vec![publish(findings)])),
                    );
                }
            }
            return act(
                request,
                vec![publish(
                    if !prompt.contains("Graph") && prompt.contains("Left reader.") {
                        json!([{"priority": "P3", "body": "shared"}])
                    } else {
                        json!([])
                    },
                )],
            );
        }
        let graph = prompt.contains("Graph");
        let repairing = request
            .last_user()
            .contains("Repair these selected finding groups");
        if repairing && request.opens() {
            let brief = request.last_user();
            assert!(!brief.contains("unselected"));
            let groups = brief
                .split("Repair these selected finding groups with their original contributors:\n")
                .nth(1)
                .unwrap();
            let mut values = serde_json::Deserializer::from_str(groups).into_iter::<Value>();
            repair_sent.send(values.next().unwrap().unwrap()).unwrap();
        }
        let reply = act(
            request,
            vec![commit_file(
                if repairing {
                    "repair.txt"
                } else if graph {
                    "graph.txt"
                } else {
                    "independent.txt"
                },
                // A resumed session keeps the original prompt; edits arrive in the current brief.
                if request.last_user().contains("Changed candidate fixture.") {
                    "fixture changed\n"
                } else {
                    "fixture\n"
                },
                "Fixture",
            )],
        );
        if request.opens() {
            if !graph {
                reader_opened.send("implementer").unwrap();
            }
            Reply::Hold(held[if graph { 0 } else { 1 }].clone(), Box::new(reply))
        } else {
            reply
        }
    });
    let picker_owner = machine.root.join("picker-owner");
    let picker_owner_said = machine.root.join("picker-owner-said");
    let picker_owner_release = machine.root.join("picker-owner-release");
    let picker_owner_ready = machine.root.join("picker-owner-ready");
    fifo(&picker_owner_said);
    fifo(&picker_owner_release);
    fifo(&picker_owner_ready);
    let mut release_picker_owner = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&picker_owner_release)
        .unwrap();
    // Successful teardown closes live up's stdin, without a box down call.
    // Delegate real exec, then retain its inherited lock in a child. Its FIFO
    // acknowledgment precedes exec completion; only child stdio is redirected.
    machine.wrapper("pinfold", &format!(
        "if [ \"$1 $2\" = 'box exec' ]; then\n\
         \"$REAL\" \"$@\"; result=$?\n\
         if [ -f '{}' ]; then\n\
         {{ read -r picker_box; read -r picker_lock; }} < '{}'\n\
         if [ \"$3\" = \"$picker_box\" ]; then\n\
         sh -c 'holder=missing; for fd in /dev/fd/*; do if [ \"$fd\" -ef \"$5\" ]; then holder=${{fd##*/}}; fi; done; echo \"$$ $1 $2 $holder\" > \"$3\"; read line < \"$4\"' sh \"$3\" \"$result\" '{}' '{}' \"$picker_lock\" </dev/null >/dev/null 2>&1 &\n\
         read line < '{}'; fi; fi\n\
         exit \"$result\"; fi",
        picker_owner.display(), picker_owner.display(), picker_owner_said.display(), picker_owner_release.display(), picker_owner_ready.display()
    ));
    machine.start();
    let fixture = config(r#"
[review.seats.left]
agent = "left"
instructions = "Left reader."
[review.seats.right]
agent = "right"
instructions = "Right reader."
[review.seats.join]
agent = "worker"
instructions = "Exact join."
[profiles.revised]
runner = {kind = "agent", agent = "right", instructions = "Revised right reader."}
access = "read-only"
outputs = {result = "verdict"}
outcomes = ["complete", "blocked"]
[agents.left]
harness = "pi"
provider = "openrouter"
model = "left-model"
[agents.right]
harness = "pi"
provider = "openrouter"
model = "right-model"
[profiles.picker]
runner = {kind = "agent", agent = "worker", instructions = "Pick exact findings."}
access = "read-only"
outputs = {result = "finding-selection"}
outcomes = ["complete"]
[gates.read]
stage = "candidate"
command = "if test -f graph.txt; then if touch forbidden 2>/dev/null; then exit 97; fi; else test -f independent.txt && touch forbidden && test -f forbidden; fi"
"#).replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 2")
        .replace("review = [\"correctness\"]", "review = [\"left\", \"right\", \"join\"]")
        .replace("[workflows.default]", "[workflows.default]\ninstructions = \"Implement fixture.\"");
    let project = Project::new(&machine, "p", &fixture);
    project.git(&["checkout", "-qb", "feature/a"]);
    project.json(&["sync", "feature/a"]);
    project.git(&["checkout", "-q", "main"]);

    let candidate = json!({"node": "implementer", "output": "result", "kind": "candidate"});
    let graph = json!({"classifiers": {"review-2": {
        "runner": {"kind": "agent", "agent": "worker", "instructions": "Classify the selected groups."},
        "outcomes": {"operator": {"kind": "operator"}}, "repeat": 1}},
        "inputs": {"feedback": "verdict", "merged": "candidate"}, "nodes": {
        "implementer": {"runner": {"kind": "agent", "agent": "worker", "instructions": "Implement fixture."},
            "access": "write-allowed", "inputs": {"feedback": {"node": null, "output": "feedback", "kind": "verdict"}},
            "outputs": {"result": "candidate"}, "outcomes": ["complete"], "requires": [],
            "routes": {"complete": [{"to": "candidate-gate-read", "repeat": null},
                {"to": "review-0", "repeat": null}, {"to": "review-1", "repeat": null}]}},
        "candidate-gate-read": {"runner": {"kind": "command", "gate": "read"}, "access": "read-only",
            "inputs": {"candidate": candidate}, "outputs": {"result": "verdict"}, "outcomes": ["complete", "failed"],
            "requires": ["candidate"], "routes": {
                "complete": [{"to": "review-2", "repeat": null}],
                "failed": [{"to": "implementer", "repeat": 1}]}},
        "review-0": {"runner": {"kind": "agent", "agent": "left", "instructions": "Left reader."}, "access": "read-only",
            "inputs": {"candidate": candidate}, "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"],
            "requires": ["candidate"], "routes": {"complete": [{"to": "review-2", "repeat": null}],
                "blocked": [{"to": "implementer", "repeat": 3}]}},
        "review-1": {"runner": {"kind": "agent", "agent": "right", "instructions": "Right reader."}, "access": "read-only", "advisory": true,
            "inputs": {"candidate": candidate}, "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"],
            "requires": ["candidate"], "routes": {"complete": [{"to": "review-2", "repeat": null}],
                "blocked": [{"to": "implementer", "repeat": 3}]}},
        "review-2": {"runner": {"kind": "agent", "agent": "worker", "instructions": "Exact join."}, "access": "read-only", "operator_failure": true,
            "inputs": {"candidate": candidate,
                "command": {"node": "candidate-gate-read", "output": "result", "kind": "verdict"},
                "left": {"node": "review-0", "output": "result", "kind": "verdict"},
                "right": {"node": "review-1", "output": "result", "kind": "verdict"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"],
            "requires": ["candidate", "command", "left", "right"], "routes": {
                "complete": [{"to": "selector", "repeat": null}], "blocked": [{"to": "implementer", "repeat": 3}]}},
        "selector": {"runner": {"kind": "agent", "agent": "worker", "instructions": "Pick exact findings."},
            "access": "read-only", "outputs": {"result": "finding-selection"}, "outcomes": ["complete"], "routes": {},
            "inputs": {"candidate": candidate,
            "left": {"node": "review-0", "output": "result", "kind": "verdict"},
            "right": {"node": "review-1", "output": "result", "kind": "verdict"},
            "join": {"node": "review-2", "output": "result", "kind": "verdict"}},
            "requires": ["candidate", "left", "right", "join"]}
    }});
    let mut operator = machine.operator();
    operator.initialize();
    let resolved = operator.call(
        "yard_plan_resolve",
        json!({"project": project.path, "graph": graph}),
    );
    assert_eq!(resolved["result"]["plan"], graph);
    assert_eq!(resolved["result"]["shape"], "read-only-graph");
    assert_eq!(resolved["result"]["runnable"], true);
    let mut watch = project.watch(0);
    // Planning runs once automatically. Its ended ticket is a supported seam
    // for explicit selected admission without racing the ordinary scheduler.
    for (ticket, title, target) in [
        ("Y-1", "Graph", "main"),
        ("Y-2", "Independent", "feature/a"),
    ] {
        project.json(&[
            "ticket",
            "new",
            "--title",
            title,
            "--target",
            target,
            "--workflow",
            "plan",
        ]);
        let planner = watch.find("planner ended", |event| {
            event["event"] == "attempt.ended"
                && event["ticket"] == ticket
                && event["data"]["outcome"] == "planned"
        });
        let cleanup = watch.find("planner cleanup", |event| {
            event["event"] == "execution.ended"
                && event["data"]["kind"] == "cleanup"
                && event["attempt"] == planner["attempt"]
        });
        watch.find("planner cleanup release", |event| {
            event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
        });
    }
    // Sabotage: strip selector before validating advisory declarations. Moving
    // the declaration from a reviewer to the picker must create no work.
    let mut invalid = graph.clone();
    invalid["nodes"]["review-1"]
        .as_object_mut()
        .unwrap()
        .remove("advisory");
    invalid["nodes"]["selector"]["advisory"] = json!(true);
    let attempts = project.rows("SELECT id FROM attempt ORDER BY id");
    let executions = project.rows("SELECT id FROM execution ORDER BY id");
    let refused = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "graph": invalid}),
    );
    assert_eq!(refused["error"]["code"], "invalid", "{refused}");
    assert_eq!(
        refused["error"]["data"],
        json!({"input": "nodes.selector.advisory", "reason": "invalid_input"})
    );
    assert_eq!(project.rows("SELECT id FROM attempt ORDER BY id"), attempts);
    assert_eq!(
        project.rows("SELECT id FROM execution ORDER BY id"),
        executions
    );
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "graph": graph}),
    );
    assert!(started.get("error").is_none(), "{started}");
    implementation.wait_held();
    let original_admission =
        project.json(&["attempt", "show", "Y-1"])["attempt"]["admission"].clone();
    assert_eq!(original_admission["plan"], graph);
    let mut racing_operator = machine.operator();
    racing_operator.initialize();
    std::thread::scope(|scope| {
        let racer = scope.spawn(|| {
            racing_operator.call(
                "yard_attempt_start",
                json!({"project": project.path, "ticket": "Y-2", "workflow": "default"}),
            )
        });
        implementation.release();
        let response = racer.join().unwrap();
        assert!(response.get("error").is_none(), "{response}");
    });
    let waiting = watch.find("branch/independent capacity race", |event| {
        event["event"] == "execution.waiting"
    });
    assert_eq!(waiting["data"]["reason"], "capacity");
    assert!(waiting["attempt"] == 3 || waiting["attempt"] == 4);
    // Observe fixture-held actors; the short command can release after denial.
    let mut expected_owners = Vec::new();
    for _ in 0..2 {
        let node = reader_openings.recv_timeout(e2e::DEADLINE).unwrap();
        match node {
            "review-0" => left.wait_held(),
            "review-1" => right.wait_held(),
            "implementer" => independent.wait_held(),
            _ => unreachable!(),
        }
        expected_owners
            .push(json!({"attempt": if node == "implementer" {4} else {3}, "node": node}));
    }
    expected_owners.sort_by_key(|row| {
        (
            row["attempt"].as_i64(),
            row["node"].as_str().unwrap().to_owned(),
        )
    });
    assert_eq!(
        project
            .rows("SELECT attempt, node FROM execution WHERE reserved = 1 ORDER BY attempt, node"),
        expected_owners
    );
    // G4: all native boxes, not only registered rows, obey the shared cap.
    // Sabotage: launch a third actor before recording its reservation.
    let held = project.rows("SELECT handle FROM execution WHERE reserved = 1 ORDER BY handle");
    assert_eq!(held.len(), 2);
    let listed = machine.boxes("dev.yard.project");
    let native = listed
        .iter()
        .find(|row| row["name"] == held[0]["handle"])
        .unwrap();
    let label = format!(
        "dev.yard.project={}",
        native["labels"]["dev.yard.project"].as_str().unwrap()
    );
    let mut actual: Vec<_> = machine
        .boxes(&label)
        .into_iter()
        .map(|row| row["name"].as_str().unwrap().to_owned())
        .collect();
    actual.sort();
    let expected: Vec<_> = held
        .iter()
        .map(|row| row["handle"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(actual.len(), 2);
    assert_eq!(actual, expected);
    independent.release();
    left.wait_held();
    right.wait_held();
    let readers = project.rows("SELECT id, attempt, node, handle, reserved, teardown FROM execution WHERE reserved = 1 ORDER BY node");
    assert_eq!(readers.len(), 2, "{readers:?}");
    assert_eq!(
        readers
            .iter()
            .map(|row| row["node"].clone())
            .collect::<Vec<_>>(),
        [json!("review-0"), json!("review-1")]
    );
    let boxes = machine.boxes("dev.yard.project");
    for reader in &readers {
        assert!(boxes.iter().any(|row| row["name"] == reader["handle"]));
    }
    let reviewer_opens = |instructions: &str| {
        // Picker context includes source instructions; count actual reviewer
        // turns, not occurrences of those instructions in frozen evidence.
        machine
            .model
            .requests()
            .iter()
            .filter(|request| {
                request.opens()
                    && request.has_tool("yard_publish_review")
                    && request.prompt().contains("Graph")
                    && request.prompt().contains(instructions)
            })
            .count()
    };
    assert_eq!(reviewer_opens("Left reader."), 1);
    assert_eq!(reviewer_opens("Right reader."), 1);
    assert_eq!(reviewer_opens("Exact join."), 0);
    let head = git(
        &project.path.join(".yard/local/attempts/3/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    left.release();
    watch.find("left checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == readers[0]["id"]
    });
    let published = project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'review-0'");
    assert_eq!(published[0]["reserved"], 0);
    assert_eq!(published[0]["teardown"], 1);
    assert!(
        project
            .rows("SELECT id FROM execution WHERE attempt = 3 AND node = 'review-2'")
            .is_empty()
    );
    // Settle the independent allowed control before the crash so no unrelated
    // implementation is interrupted while Right holds its reader slot.
    let independent_head = approval_of(&mut watch, "Y-2");
    let sources = project.rows("SELECT f.id AS finding, f.\"check\" AS \"check\", e.id AS execution, e.attempt, 'Y-' || a.ticket AS ticket, e.base, e.head, e.proof, e.ticket_revision, e.plan_revision, e.digest FROM finding f JOIN \"check\" c ON c.id = f.\"check\" JOIN execution e ON e.id = c.execution JOIN attempt a ON a.id = e.attempt WHERE e.attempt = 4 ORDER BY f.id");
    assert_eq!(sources.len(), 1);
    *foreign_source.lock().unwrap() = Some(sources[0].clone());
    assert_eq!(project.rows("SELECT exit_code, outcome FROM execution WHERE attempt = 4 AND node = 'candidate-gate-read'"),
        vec![json!({"exit_code": 0, "outcome": "pass"})]);
    let command =
        project.rows("SELECT id FROM execution WHERE node = 'candidate-gate-read' AND attempt = 3");
    assert_eq!(command.len(), 1);
    watch.find("command checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == command[0]["id"]
    });
    assert_eq!(project.rows("SELECT exit_code, outcome FROM execution WHERE node = 'candidate-gate-read' AND attempt = 3"),
        vec![json!({"exit_code": 0, "outcome": "pass"})]);
    // Right's live box and inherited effect lock hold the crash point.
    let lock = std::fs::File::open(project.path.join(format!(
        ".yard/local/attempts/3/effects/{}/lock",
        readers[1]["id"]
    )))
    .unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert_eq!(
        project.rows(
            "SELECT status, reserved, result FROM execution WHERE attempt = 3 AND node = 'review-1'"
        ),
        vec![json!({"status": "running", "reserved": 1, "result": null})]
    );
    drop(watch);
    machine.kill();
    machine.start();
    restarted.wait_held();
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == readers[1]["handle"])
    );
    lock.try_lock().unwrap();
    lock.unlock().unwrap();
    assert_eq!(
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'review-0'"),
        published
    );
    assert_eq!(reviewer_opens("Left reader."), 1);
    assert_eq!(reviewer_opens("Right reader."), 2);
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE attempt = 3 AND node = 'candidate-gate-read'")
            .len(),
        1
    );
    assert!(
        project
            .rows("SELECT id FROM execution WHERE attempt = 3 AND node = 'review-2'")
            .is_empty()
    );
    right.release();
    restarted.release();
    picked.wait_held();
    let pending_picker = project.rows("SELECT id, handle, report, result, reserved, teardown, head, proof FROM execution WHERE attempt = 3 AND node = 'selector'");
    assert_eq!(pending_picker.len(), 1);
    assert!(pending_picker[0]["report"].is_string());
    assert_eq!(pending_picker[0]["result"], Value::Null);
    assert_eq!(pending_picker[0]["reserved"], 1);
    assert_eq!(pending_picker[0]["head"], head);
    assert!(
        project
            .rows("SELECT id FROM approval WHERE attempt = 3")
            .is_empty()
    );
    let source_rows = project.rows("SELECT * FROM execution WHERE attempt = 3 AND kind = 'review' AND node != 'selector' AND status = 'ended' AND plan_revision = 1 ORDER BY id");
    let picker_id = pending_picker[0]["id"].as_i64().unwrap();
    let picker_box = pending_picker[0]["handle"].as_str().unwrap();
    let picker_lock_path = project
        .path
        .join(format!(".yard/local/attempts/3/effects/{picker_id}/lock"));
    let picker_lock = std::fs::File::open(&picker_lock_path).unwrap();
    std::fs::write(
        &picker_owner,
        format!("{picker_box}\n{}\n", picker_lock_path.display()),
    )
    .unwrap();
    // Start at the currently held publication, excluding pre-restart history.
    let since = project.rows("SELECT COALESCE(MAX(seq), 0) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap();
    let mut watch = project.watch(since);
    picked.release();
    let owner = watch.said_before_end(&picker_owner_said, Some(picker_id));
    let owner_fields: Vec<_> = owner.split_whitespace().collect();
    assert_eq!(owner_fields.len(), 4);
    assert_eq!(&owner_fields[1..3], &[picker_box, "0"]);
    // The child compares its inherited descriptor to the actual effect inode.
    assert!(owner_fields[3].parse::<u32>().unwrap() > 2);
    assert!(
        std::process::Command::new("kill")
            .args(["-0", owner_fields[0]])
            .status()
            .unwrap()
            .success()
    );
    // Acknowledge only after the observer has received and checked the holder.
    std::fs::write(&picker_owner_ready, "held\n").unwrap();
    let ownership = watch.find("selected picker retains owner", |event| {
        event["event"] == "attention.raised"
            && event["execution"] == picker_id
            && event["data"]["kind"] == "red"
            && event["data"]["reason"] == "ownership"
    });
    assert_eq!(ownership["attempt"], 3);
    let held_selection = project.rows(&format!("SELECT report, result, outcome, reserved, teardown, lock_contract, mcp FROM execution WHERE id = {picker_id}"));
    assert_eq!(held_selection[0]["outcome"], "selected");
    assert_eq!(held_selection[0]["reserved"], 1);
    assert_eq!(held_selection[0]["teardown"], 0);
    assert_eq!(held_selection[0]["lock_contract"], 1);
    assert_eq!(held_selection[0]["mcp"], "registered");
    let committed: Value =
        serde_json::from_str(held_selection[0]["result"].as_str().unwrap()).unwrap();
    assert_eq!(committed["infrastructure"]["outcome"], "completed");
    assert_eq!(committed["outputs"]["result"]["kind"], "finding-selection");
    // G7: a publication whose inherited owner is still live is not applied.
    // Sabotage: count the published report before checked teardown.
    let pending = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert_eq!(
        pending["result"]["trials"]["choices"][0]["application"]["status"],
        "unapplied"
    );
    assert!(
        pending["result"]["trials"]["models"]
            .as_array()
            .unwrap()
            .iter()
            .all(|model| model["selected_count"] == 0)
    );
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == picker_box)
    );
    assert!(matches!(
        picker_lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(project.rows("SELECT id FROM attention WHERE attempt = 3 AND kind = 'approval' AND state = 'open'").is_empty());
    // Reconcile the ended publication while its real inherited owner is held.
    // The same result and exact ownership item must remain ineligible.
    drop(watch);
    machine.kill();
    machine.start();
    let mut watch = project.watch(0);
    assert_eq!(project.rows(&format!("SELECT report, result, outcome, reserved, teardown, lock_contract, mcp FROM execution WHERE id = {picker_id}")), held_selection);
    assert!(
        std::process::Command::new("kill")
            .args(["-0", owner_fields[0]])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT state FROM attention WHERE id = {}",
            ownership["attention"]
        )),
        vec![json!({"state": "open"})]
    );
    assert!(matches!(
        picker_lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(project.rows("SELECT id FROM attention WHERE attempt = 3 AND kind = 'approval' AND state = 'open'").is_empty());
    std::fs::remove_file(&picker_owner).unwrap();
    writeln!(release_picker_owner, "release").unwrap();
    watch.find("selected picker checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == picker_id
    });
    let resolved = watch.find("selected picker ownership reconciled", |event| {
        event["event"] == "attention.resolved" && event["attention"] == ownership["attention"]
    });
    assert_eq!(resolved["execution"], picker_id);
    assert_eq!(resolved["data"]["resolution"], "reconciled");
    assert_eq!(
        project.rows(&format!(
            "SELECT state FROM attention WHERE id = {}",
            ownership["attention"]
        )),
        vec![json!({"state": "resolved"})]
    );
    let released_selection = project.rows(&format!(
        "SELECT report, result, outcome, reserved, teardown FROM execution WHERE id = {picker_id}"
    ));
    assert_eq!(released_selection[0]["report"], held_selection[0]["report"]);
    assert_eq!(released_selection[0]["result"], held_selection[0]["result"]);
    assert_eq!(released_selection[0]["outcome"], "selected");
    assert_eq!(released_selection[0]["reserved"], 0);
    assert_eq!(released_selection[0]["teardown"], 1);
    use std::os::unix::fs::MetadataExt;
    let inherited = picker_lock.metadata().unwrap();
    let current = std::fs::metadata(
        project
            .path
            .join(format!(".yard/local/attempts/3/effects/{picker_id}/lock")),
    )
    .unwrap();
    assert_eq!(
        (current.dev(), current.ino()),
        (inherited.dev(), inherited.ino())
    );
    picker_lock.try_lock().unwrap();
    picker_lock.unlock().unwrap();
    drop(watch);
    let mut watch = project.watch(0);
    assert_eq!(approval_of(&mut watch, "Y-1"), head);
    let first_join =
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'review-2'");
    assert_eq!(first_join.len(), 1);
    let first_inputs: Value =
        serde_json::from_str(first_join[0]["inputs"].as_str().unwrap()).unwrap();
    assert_eq!(
        first_inputs["command"]["source"],
        json!({"execution": command[0]["id"],
        "node": "candidate-gate-read", "output": "result", "plan_revision": 1})
    );
    assert_eq!(
        first_inputs["left"]["source"]["execution"],
        readers[0]["id"]
    );
    let right_supplier = project.rows(
        "SELECT id FROM execution WHERE attempt = 3 AND node = 'review-1' AND outcome = 'pass'",
    );
    assert_eq!(right_supplier.len(), 1);
    assert_eq!(
        first_inputs["right"]["source"]["execution"],
        right_supplier[0]["id"]
    );
    assert_eq!(first_inputs["left"]["source"]["plan_revision"], 1);
    assert_eq!(first_inputs["right"]["source"]["plan_revision"], 1);
    assert_eq!(reviewer_opens("Exact join."), 1);
    let raw_findings = project.rows("SELECT f.*, e.id AS execution, e.model FROM finding f JOIN \"check\" c ON c.id = f.\"check\" JOIN execution e ON e.id = c.execution WHERE e.attempt = 3 AND e.plan_revision = 1 ORDER BY f.id");
    assert_eq!(raw_findings.len(), 4);
    let assert_selection = |row: &Value, revision: i64, expected_head: &str| {
        let result: Value = serde_json::from_str(row["result"].as_str().unwrap()).unwrap();
        assert_eq!(result["outputs"]["result"]["kind"], "finding-selection");
        assert_eq!(result["judgment"], Value::Null);
        let value = &result["outputs"]["result"]["value"];
        assert_eq!(value["picker"]["execution"], row["id"]);
        assert_eq!(value["inputs"]["candidate"]["value"]["head"], expected_head);
        let groups = value["groups"].as_array().unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0]["reason"], "useful subset");
        assert_eq!(groups[1]["reason"], "shared duplicate");
        let selected = project.rows(&format!("SELECT f.id, f.body, e.id AS execution, e.model FROM finding f JOIN \"check\" c ON c.id = f.\"check\" JOIN execution e ON e.id = c.execution WHERE e.attempt = 3 AND e.plan_revision = {revision} AND e.head = '{expected_head}' AND f.body != 'unselected' ORDER BY e.node, f.body"));
        let sources: Vec<_> = groups
            .iter()
            .flat_map(|group| group["sources"].as_array().unwrap())
            .collect();
        assert_eq!(sources.len(), 3);
        for finding in &selected {
            let group = if finding["body"] == "useful" { 0 } else { 1 };
            let source = groups[group]["sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|source| source["finding"] == finding["id"])
                .unwrap();
            assert_eq!(source["execution"], finding["execution"]);
            assert_eq!(source["plan_revision"], revision);
            assert_eq!(source["head"], expected_head);
            assert_eq!(source["ticket_revision"], row["ticket_revision"]);
            assert_eq!(source["proof"], row["proof"]);
            assert_eq!(source["ticket"], "Y-1");
        }
        assert_eq!(groups[0]["sources"].as_array().unwrap().len(), 1);
        assert_eq!(groups[1]["sources"].as_array().unwrap().len(), 2);
        assert_eq!(row["reserved"], 0);
        assert_eq!(row["teardown"], 1);
        assert_eq!(
            serde_json::from_str::<Value>(row["report"].as_str().unwrap()).unwrap(),
            *value
        );
    };
    let first_selection =
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'selector'");
    assert_eq!(first_selection.len(), 1);
    assert_selection(&first_selection[0], 1, &head);
    // G7: the fixture selected left useful/shared and right shared. The join
    // repeats the same review edges. Count each durable finding once and keep
    // both contributors to the combined group. Sabotages: attribute to picker,
    // sum duplicate input edges, or lose a combined finding contributor.
    let stats = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    for (name, published, selected, groups) in [("left-model", 3, 2, 2), ("right-model", 1, 1, 1)] {
        let model = stats["result"]["trials"]["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|model| model["identity"]["requested"]["model"] == name)
            .unwrap();
        assert_eq!(model["published_count"], published);
        assert_eq!(model["eligible_count"], published);
        assert_eq!(model["selected_count"], selected);
        assert_eq!(model["selected_group_count"], groups);
        assert_eq!(model["selected_rate"], selected as f64 / published as f64);
        let expected: std::collections::BTreeSet<_> = raw_findings
            .iter()
            .filter(|finding| finding["model"] == name && finding["body"] != "unselected")
            .map(|finding| format!("finding:{}", finding["id"]))
            .collect();
        assert_eq!(model["selected_sources"], json!(expected));
    }
    assert_eq!(
        stats["result"]["trials"]["choices"][0]["application"]["status"],
        "applied"
    );
    assert_eq!(
        project.rows("SELECT choice FROM attempt WHERE id = 3"),
        vec![json!({"choice": null})]
    );
    // Reopen an already committed selection. A supported revision below gives
    // a scheduler observation without polling or waiting out a clock.
    drop(watch);
    machine.kill();
    machine.start();
    let mut watch = project.watch(0);

    // An explicit revision invalidates branch consumers, never old evidence.
    let changed = operator.call(
        "yard_attempt_revise",
        json!({"project": project.path, "ticket": "Y-1", "plan_revision": 1,
        "overrides": {"review-1": {"profile": "revised"}}}),
    );
    assert_eq!(changed["result"]["plan_revision"], 2, "{changed}");
    revised.wait_held();
    // An applied historical selection is superseded by the admitted revision.
    let obsolete = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert_eq!(
        obsolete["result"]["trials"]["choices"][0]["application"]["status"],
        "superseded"
    );
    assert!(
        obsolete["result"]["trials"]["models"]
            .as_array()
            .unwrap()
            .iter()
            .all(|model| model["current_selected_count"] == 0)
    );
    assert_eq!(
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'selector'"),
        first_selection
    );
    // Sabotage: erase or rewrite old findings on revision. New reviewers may
    // already publish, so compare the complete original revision only.
    assert_eq!(project.rows("SELECT f.*, e.id AS execution, e.model FROM finding f JOIN \"check\" c ON c.id = f.\"check\" JOIN execution e ON e.id = c.execution WHERE e.attempt = 3 AND e.plan_revision = 1 ORDER BY f.id"), raw_findings);
    assert_eq!(
        project.rows(
            "SELECT * FROM execution WHERE attempt = 3 AND node = 'review-2' AND plan_revision = 1"
        ),
        first_join
    );
    assert!(project.rows("SELECT id FROM execution WHERE attempt = 3 AND node = 'review-2' AND plan_revision = 2").is_empty());
    revised.release();
    let approved = watch.until("revision two join approves", |event| {
        is_approval(event, "Y-1") && event["data"]["payload"]["plan_revision"] == 2
    });
    assert_eq!(approved["data"]["payload"]["head"], head);
    let suppliers = project.rows("SELECT id, node FROM execution WHERE attempt = 3 AND plan_revision = 2 AND kind = 'review' AND node != 'selector' ORDER BY node");
    assert_eq!(suppliers.len(), 3, "{suppliers:?}");
    let joined = project.rows("SELECT inputs FROM execution WHERE attempt = 3 AND node = 'review-2' AND plan_revision = 2");
    let inputs: Value = serde_json::from_str(joined[0]["inputs"].as_str().unwrap()).unwrap();
    let fresh_command = project.rows("SELECT id FROM execution WHERE attempt = 3 AND node = 'candidate-gate-read' AND plan_revision = 2");
    assert_eq!(fresh_command.len(), 1);
    assert_eq!(
        inputs["command"]["source"],
        json!({"execution": fresh_command[0]["id"],
        "node": "candidate-gate-read", "output": "result", "plan_revision": 2})
    );
    assert_ne!(fresh_command[0]["id"], command[0]["id"]);
    for (name, index) in [("left", 0), ("right", 1)] {
        assert_eq!(
            inputs[name]["source"],
            json!({"execution": suppliers[index]["id"], "node": format!("review-{index}"), "output": "result", "plan_revision": 2})
        );
        assert_ne!(
            inputs[name]["source"]["execution"],
            first_inputs[name]["source"]["execution"]
        );
    }
    assert_eq!(reviewer_opens("Left reader."), 2);
    assert_eq!(reviewer_opens("Revised right reader."), 1);
    assert_eq!(reviewer_opens("Exact join."), 2);
    let selections =
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'selector' ORDER BY id");
    assert_eq!(selections.len(), 2);
    assert_eq!(selections[0], first_selection[0]);
    assert_selection(&selections[1], 2, &head);
    let selected_inputs: Value =
        serde_json::from_str(selections[1]["inputs"].as_str().unwrap()).unwrap();
    for (name, index) in [("left", 0), ("right", 1), ("join", 2)] {
        assert_eq!(
            selected_inputs[name]["source"]["execution"],
            suppliers[index]["id"]
        );
    }
    let mut revised_graph = graph.clone();
    revised_graph["nodes"]["review-1"]["runner"]["instructions"] = json!("Revised right reader.");
    let shown = project.json(&["attempt", "show", "Y-1"]);
    assert_eq!(shown["attempt"]["admission"], original_admission);
    assert_eq!(shown["attempt"]["plan"]["plan"], revised_graph);
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"].to_string();
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        "Changed candidate fixture.",
    ]);
    let failure = watch.until("excluded required finding still blocks", |event| {
        event["event"] == "attention.raised"
            && event["ticket"] == "Y-1"
            && event["data"]["kind"] == "candidate"
    });
    let decision = |item: &Value| {
        json!({"project": project.path, "ticket": "Y-1",
        "attention": item["attention"], "check": item["data"]["payload"]["evidence"]["check"]["check"],
        "plan_revision": 2, "target": "main", "head": item["data"]["payload"]["head"],
        "proof": item["data"]["payload"]["proof"], "text": "Exact required failure decision"})
    };
    // Sabotage: judge the join only by its own passing check instead of its
    // unresolved carried blocks, or repair those blocks after exact acceptance.
    assert_eq!(
        failure["data"]["payload"]["evidence"]["check"]["verdict"],
        "pass"
    );
    assert_eq!(failure["data"]["payload"]["evidence"]["node"], "review-2");
    let required = project.rows("SELECT c.id, c.verdict FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE e.attempt = 3 AND e.node = 'review-0' ORDER BY e.id DESC LIMIT 1");
    assert_eq!(required[0]["verdict"], "fail");
    assert_eq!(
        failure["data"]["payload"]["evidence"]["result"]["outcome"],
        "blocked"
    );
    assert_eq!(
        failure["data"]["payload"]["evidence"]["result"]["outputs"]["result"]["value"]["blocked"],
        json!([required[0]["id"]])
    );
    let before_repair = project.rows("SELECT * FROM finding ORDER BY id");
    let reviews_before_repair = project.rows("SELECT e.*, c.id AS check_id, c.verdict AS check_verdict FROM execution e JOIN \"check\" c ON c.execution = e.id WHERE e.attempt = 3 ORDER BY e.id");
    let selected = project.rows("SELECT result FROM execution WHERE attempt = 3 AND node = 'selector' ORDER BY id DESC LIMIT 1");
    let selected: Value = serde_json::from_str(selected[0]["result"].as_str().unwrap()).unwrap();
    let repair = operator.call("yard_attempt_repair", decision(&failure));
    assert_eq!(repair["result"]["accepted"], false, "{repair}");
    // The host observes the real resumed Pi request. Forwarding all findings
    // or losing either shared contributor fails this exact group comparison.
    let received = repair_received.recv_timeout(e2e::DEADLINE).unwrap();
    // G2/G8: classification and remediation consume the identical Y132 packet.
    // Sabotage: rebuild groups from raw reviews, strip a contributor, or treat
    // an omitted finding as acceptance of the independent required block.
    assert_eq!(
        classification_inputs.recv_timeout(e2e::DEADLINE).unwrap(),
        received
    );
    let groups = &selected["outputs"]["result"]["value"]["groups"];
    let mut routed = received.clone();
    for group in routed.as_array_mut().unwrap() {
        group.as_object_mut().unwrap().remove("contributors");
    }
    assert_eq!(&routed, groups);
    for group in received.as_array().unwrap() {
        let contributors = group["contributors"].as_array().unwrap();
        assert_eq!(
            contributors.len(),
            group["sources"].as_array().unwrap().len()
        );
        for (contributor, source) in contributors
            .iter()
            .zip(group["sources"].as_array().unwrap())
        {
            assert_eq!(&contributor["finding"]["source"], source);
            let review = selected["outputs"]["result"]["value"]["inputs"]
                .as_object()
                .unwrap()
                .values()
                .find(|input| {
                    input["value"]["review"]["findings"]
                        .as_array()
                        .is_some_and(|findings| {
                            findings.iter().any(|finding| finding["source"] == *source)
                        })
                })
                .unwrap();
            for field in ["check", "node", "settings", "inputs"] {
                assert_eq!(contributor[field], review["value"]["review"][field]);
            }
            assert_ne!(contributor["finding"]["body"], "unselected");
            assert_eq!(
                contributor["finding"]["body"],
                if group["reason"] == "useful subset" {
                    "useful"
                } else {
                    "shared"
                }
            );
        }
    }
    let failure = watch.until("repair retains required verdict control", |event| {
        event["event"] == "attention.raised"
            && event["ticket"] == "Y-1"
            && event["data"]["kind"] == "candidate"
            && event["attention"] != failure["attention"]
    });
    let mut unauthorized = decision(&failure);
    unauthorized["head"] = json!(head);
    let refused = operator.call("yard_attempt_accept", unauthorized);
    assert_eq!(refused["error"]["code"], "stale", "{refused}");
    assert_eq!(refused["error"]["data"]["identity"], "head");
    assert_eq!(refused["error"]["data"]["expected"], head);
    assert_eq!(
        refused["error"]["data"]["current"],
        failure["data"]["payload"]["head"]
    );
    assert!(project.rows("SELECT id FROM attention WHERE attempt = 3 AND kind = 'approval' AND state = 'open'").is_empty());
    let accepted = operator.call("yard_attempt_accept", decision(&failure));
    assert_eq!(accepted["result"]["accepted"], true, "{accepted}");
    let fresh = watch.until("exact acceptance permits approval", |event| {
        is_approval(event, "Y-1")
            && event["data"]["payload"]["head"] == failure["data"]["payload"]["head"]
    });
    let new_head = git(
        &project.path.join(".yard/local/attempts/3/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    assert_ne!(new_head, head);
    assert_eq!(fresh["data"]["payload"]["head"], new_head);
    let fresh_selections =
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'selector' ORDER BY id");
    assert_eq!(fresh_selections.len(), 4);
    assert_eq!(&fresh_selections[..2], selections.as_slice());
    assert_selection(
        &fresh_selections[2],
        2,
        selected["outputs"]["result"]["value"]["inputs"]["candidate"]["value"]["head"]
            .as_str()
            .unwrap(),
    );
    assert_selection(&fresh_selections[3], 2, &new_head);
    let retained = project.rows("SELECT * FROM finding ORDER BY id");
    for finding in before_repair {
        assert_eq!(
            retained.iter().find(|row| row["id"] == finding["id"]),
            Some(&finding)
        );
    }
    let reviews_after_repair = project.rows("SELECT e.*, c.id AS check_id, c.verdict AS check_verdict FROM execution e JOIN \"check\" c ON c.execution = e.id WHERE e.attempt = 3 ORDER BY e.id");
    for review in reviews_before_repair {
        assert_eq!(
            reviews_after_repair
                .iter()
                .find(|row| row["id"] == review["id"]),
            Some(&review)
        );
    }
    let repairs = project.rows("SELECT inputs FROM execution WHERE attempt = 3 AND reason = 'repair' AND kind = 'implementation'");
    assert_eq!(repairs.len(), 1);
    let feedback: Value = serde_json::from_str(repairs[0]["inputs"].as_str().unwrap()).unwrap();
    assert_eq!(feedback["feedback"]["value"]["groups"], received);
    assert_eq!(
        feedback["feedback"]["source"]["execution"],
        selected["outputs"]["result"]["value"]["picker"]["execution"]
    );
    let fresh_inputs: Value =
        serde_json::from_str(fresh_selections[3]["inputs"].as_str().unwrap()).unwrap();
    for name in ["left", "right", "join"] {
        assert_ne!(
            fresh_inputs[name]["source"]["execution"],
            selected_inputs[name]["source"]["execution"]
        );
    }
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "main",
        "Y-1",
        "--head",
        &new_head,
        "--plan-revision",
        "2",
    ]);
    watch.find("graph lands", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "feature/a",
        "Y-2",
        "--head",
        &independent_head,
    ]);
    watch.find("independent lands", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
    for attempt in [3, 4] {
        let removed = watch.find("cleanup", |event| {
            event["event"] == "execution.ended"
                && event["data"]["kind"] == "cleanup"
                && event["attempt"] == attempt
        });
        watch.find("cleanup checked release", |event| {
            event["event"] == "execution.released" && event["execution"] == removed["execution"]
        });
    }
    assert_eq!(
        git(&project.canonical(), &["show", "main:graph.txt"]),
        "fixture changed\n"
    );
    assert_eq!(
        project.rows("SELECT * FROM execution WHERE attempt = 3 AND node = 'selector' ORDER BY id"),
        fresh_selections
    );
    let retained = project.rows("SELECT f.*, e.id AS execution, e.model FROM finding f JOIN \"check\" c ON c.id = f.\"check\" JOIN execution e ON e.id = c.execution WHERE e.attempt = 3 ORDER BY f.id");
    for finding in &raw_findings {
        assert_eq!(
            retained.iter().find(|row| row["id"] == finding["id"]),
            Some(finding)
        );
    }
    assert_eq!(
        git(&project.canonical(), &["show", "feature/a:independent.txt"]),
        "fixture\n"
    );
    assert_eq!(project.rows("SELECT * FROM execution WHERE attempt = 3 AND kind = 'review' AND node != 'selector' AND status = 'ended' AND plan_revision = 1 ORDER BY id"), source_rows);
    let retained_stats = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    )["result"]["trials"]
        .clone();
    assert!(
        retained_stats["choices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|choice| choice["repair_consumers"]
                .as_array()
                .is_some_and(|consumers| consumers.len() == 1))
    );
    // Cleanup and reopen preserve the same source samples and application facts.
    drop(watch);
    machine.kill();
    machine.start();
    let reopened = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert_eq!(reopened["result"]["trials"], retained_stats);
    five_writer_candidates_survive_continuation_and_restart();
}

/// G9/G13/G14: five graph-declared profiles act on one frozen ticket/target.
/// Sabotage: share a clone/index/state/proof path, pick the newest attempt head
/// for a reader, resume another workspace's session, bypass retained ownership,
/// overwrite an answered continuation, leave an edited stopped item blocking
/// another writer, or delete only the ordinary clone. Git, inode identities and fixture bytes
/// supply expectations. The host model holds each real harness after its edits;
/// the fifth writer's real box and inherited lock hold the external crash point.
/// Roots named selector and build-gate-model remain candidate producers through
/// publication, bound review, explicit continuation and recovery in this project.
/// Sabotage: skip roots by selector/gate spelling instead of admitted output.
fn five_writer_candidates_survive_continuation_and_restart() {
    use std::os::unix::fs::MetadataExt;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let held: Vec<_> = (0..5).map(|_| Latch::new()).collect();
    let readers: Vec<_> = (0..5).map(|_| Latch::new()).collect();
    let continued = Latch::new();
    let other_continued = Latch::new();
    let other_recovered = Latch::new();
    let writer_holds = held.clone();
    let reader_holds = readers.clone();
    let continuation_hold = continued.clone();
    let other_hold = other_continued.clone();
    let recovery_hold = other_recovered.clone();
    let phases: Vec<_> = (0..5).map(|_| AtomicUsize::new(0)).collect();
    let machine = Machine::new("g4-five-writers", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        if request.has_tool("yard_publish_review") {
            let prompt = request.prompt();
            let n = (0..5)
                .find(|n| prompt.contains(&format!("Frozen reader {n}.")))
                .unwrap();
            if request.opens() {
                return Reply::Hold(
                    reader_holds[n].clone(),
                    Box::new(Reply::Tools(vec![bash(&format!(
                        "test \"$(cat same.txt)\" = writer-{n} && test \"$(cat /yard/proof/evidence.txt)\" = proof-{n} && \
                     ! (echo bad > same.txt) 2>/dev/null && \
                     ! (echo bad > /yard/proof/evidence.txt) 2>/dev/null && echo frozen-{n}"
                    ))])),
                );
            }
            let results = request.tool_results();
            if results
                .last()
                .is_some_and(|(_, result)| result.contains(&format!("frozen-{n}")))
            {
                return Reply::Tools(vec![publish(json!([]))]);
            }
            return Reply::Text("done".into());
        }
        let model = request.body["model"].as_str().unwrap();
        let n = model
            .strip_prefix("trial-model-")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        if request.opens() {
            let phase = phases[n].fetch_add(1, Ordering::SeqCst);
            let mut calls = if phase == 0 {
                vec![
                    commit_file("same.txt", &format!("writer-{n}\n"), "Independent writer"),
                    bash(&format!("printf 'proof-{n}\\n' > /yard/proof/evidence.txt")),
                ]
            } else if n == 3 && phase == 1 {
                vec![bash("printf 'other-partial\\n' > other-partial.txt")]
            } else if n == 3 {
                vec![bash(
                    "test \"$(cat other-partial.txt)\" = other-partial && git add other-partial.txt && git commit -qm OtherRecovered",
                )]
            } else if phase == 1 {
                vec![bash(
                    "test \"$(cat partial.txt)\" = partial && git add partial.txt && git commit -qm Continued",
                )]
            } else {
                vec![bash(
                    "test \"$(cat interrupted.txt)\" = interrupted && git add interrupted.txt && git commit -qm Recovered",
                )]
            };
            if n == 4 && phase < 2 {
                calls.push(bash(if phase == 0 {
                    "printf 'partial\\n' > partial.txt"
                } else {
                    "printf 'interrupted\\n' > interrupted.txt"
                }));
            }
            calls.push(bash(&format!("echo writer-ready-{n}-{phase}")));
            let reply = Reply::Tools(calls);
            return if n == 3 && phase == 2 {
                Reply::Hold(recovery_hold.clone(), Box::new(reply))
            } else {
                reply
            };
        }
        let phase = phases[n].load(Ordering::SeqCst) - 1;
        assert!(
            request
                .tool_results()
                .last()
                .is_some_and(|(_, text)| text.contains(&format!("writer-ready-{n}-{phase}"))),
            "writer tools did not complete"
        );
        if phase == 0 {
            Reply::Hold(
                writer_holds[n].clone(),
                Box::new(Reply::Text("done".into())),
            )
        } else if phase == 1 {
            Reply::Hold(
                if n == 3 {
                    other_hold.clone()
                } else {
                    continuation_hold.clone()
                },
                Box::new(Reply::Text("done".into())),
            )
        } else {
            Reply::Text("done".into())
        }
    });
    let owner = machine.root.join("held-writer");
    let owner_said = machine.root.join("owner-said");
    let owner_release = machine.root.join("owner-release");
    fifo(&owner_said);
    fifo(&owner_release);
    let mut release_owner = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&owner_release)
        .unwrap();
    // Preserve stdin before asynchronous shell redirection, using a closed
    // descriptor so the real child's inherited execution lock is untouched.
    machine.wrapper(
        "pinfold",
        &format!(
            "if [ \"$1 $2\" = 'box exec' ]; then\n\
         input_fd=''; for fd in 3 4 5 6 7 8 9; do\n\
         if ! (eval \": <&$fd\") 2>/dev/null; then input_fd=$fd; break; fi; done\n\
         [ -n \"$input_fd\" ] || exit 1\n\
         eval \"exec $input_fd<&0\"\n\
         eval '\"$REAL\" \"$@\" <&'\"$input_fd $input_fd\"'<&- &'\n\
         child=$!; eval \"exec $input_fd<&-\"\n\
         wait \"$child\"; result=$?\n\
         if [ -f '{}' ] && [ \"$3\" = \"$(cat '{}')\" ]; then\n\
         echo \"$3 $result\" > '{}'; read line < '{}'; fi\n\
         exit \"$result\"; fi",
            owner.display(),
            owner.display(),
            owner_said.display(),
            owner_release.display()
        ),
    );
    machine.start();
    let mut extra = String::new();
    let mut nodes = serde_json::Map::new();
    for n in 0..5 {
        extra.push_str(&format!(
            "\n[agents.trial{n}]\nharness = \"pi\"\nprovider = \"openrouter\"\nmodel = \"trial-model-{n}\"\n\
             [profiles.trial{n}]\nrunner = {{kind = \"agent\", agent = \"trial{n}\", instructions = \"Same frozen instructions.\"}}\n\
             access = \"write-allowed\"\noutputs = {{result = \"candidate\"}}\noutcomes = [\"complete\"]\n\
             [review.seats.trial{n}]\nagent = \"worker\"\ninstructions = \"Frozen reader {n}.\"\n"
        ));
        let writer = match n {
            0 => "implementer".to_string(),
            2 => "selector".to_string(),
            3 => "build-gate-model".to_string(),
            _ => format!("writer-{n}"),
        };
        nodes.insert(
            writer.clone(),
            json!({"profile": format!("trial{n}"),
            "routes": {"complete": [{"to": format!("review-{n}")}]}}),
        );
        nodes.insert(format!("review-{n}"), json!({
            "runner": {"kind": "agent", "agent": "worker", "instructions": format!("Frozen reader {n}.")},
            "access": "read-only", "inputs": {"candidate": {"node": writer, "output": "result", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"], "requires": ["candidate"], "routes": {}
        }));
    }
    let fixture = config(&extra)
        .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 5")
        .replace(
            "review = [\"correctness\"]",
            "review = [\"trial0\", \"trial1\", \"trial2\", \"trial3\", \"trial4\"]",
        );
    let project = Project::new(&machine, "p", &fixture);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Five writer trial",
        "--workflow",
        "plan",
    ]);
    let planned = watch.find("trial planner ended", |e| e["event"] == "attempt.ended");
    let cleanup = watch.find("trial planner cleanup", |e| {
        e["event"] == "execution.ended"
            && e["data"]["kind"] == "cleanup"
            && e["attempt"] == planned["attempt"]
    });
    watch.find("trial planner released", |e| {
        e["event"] == "execution.released" && e["execution"] == cleanup["execution"]
    });
    let frozen_target = project.canonical_head();
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "graph": {"inputs": {}, "nodes": nodes.clone()}
        }),
    );
    assert!(started.get("error").is_none(), "{started}");
    for latch in &held {
        latch.wait_held();
    }
    let writers = writer_rows(
        &project,
        "SELECT * FROM execution WHERE attempt = 2 AND kind = 'implementation' ORDER BY node",
    );
    assert_eq!(writers.len(), 5);
    // The host runtime also holds unrelated projects. Pinfold supplies this
    // project's label on a held writer, without duplicating Yard's key logic.
    let worker = machine
        .boxes("dev.yard.project")
        .into_iter()
        .find(|listed| listed["name"] == writers[0]["handle"])
        .expect("the held writer has a box");
    let label = format!(
        "dev.yard.project={}",
        worker["labels"]["dev.yard.project"].as_str().unwrap()
    );
    let boxes = machine.boxes(&label);
    assert_eq!(boxes.len(), 5, "{boxes:?}");
    let root = project.path.join(".yard/local/attempts/2");
    let mut indexes = std::collections::BTreeSet::new();
    let mut states = std::collections::BTreeSet::new();
    let mut sessions = std::collections::BTreeSet::new();
    let mut expected_heads = Vec::new();
    for n in 0..5 {
        let name = match n {
            0 => "implementer".to_string(),
            2 => "selector".to_string(),
            3 => "build-gate-model".to_string(),
            _ => format!("writer-{n}"),
        };
        let row = writers.iter().find(|row| row["workspace"] == name).unwrap();
        assert!(boxes.iter().any(|listed| listed["name"] == row["handle"]));
        assert_eq!(row["base"], frozen_target);
        assert_eq!(row["settings"]["model"], format!("trial-model-{n}"));
        assert_eq!(row["ticket_revision"], writers[0]["ticket_revision"]);
        assert_eq!(row["plan_revision"], writers[0]["plan_revision"]);
        assert_eq!(row["status"], "running");
        assert_eq!(row["reserved"], 1);
        assert!(row["resumed"].is_null());
        let session = row["session_id"].as_str().unwrap();
        assert!(sessions.insert(session.to_string()));
        let workspace = writer_workspace(&root, &name);
        let clone = workspace.join("clone");
        assert_eq!(
            std::fs::read_to_string(clone.join("same.txt")).unwrap(),
            format!("writer-{n}\n")
        );
        assert_eq!(
            std::fs::read_to_string(workspace.join("proof/evidence.txt")).unwrap(),
            format!("proof-{n}\n")
        );
        assert!(indexes.insert(std::fs::metadata(clone.join(".git/index")).unwrap().ino()));
        assert!(states.insert(std::fs::metadata(workspace.join("state")).unwrap().ino()));
        assert!(!clone.join(".git/objects/info/alternates").exists());
        let head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
        let object = clone.join(".git/objects").join(&head[..2]).join(&head[2..]);
        assert_eq!(std::fs::metadata(object).unwrap().nlink(), 1);
        expected_heads.push(head);
    }
    let old = writers
        .iter()
        .find(|row| row["workspace"] == "writer-4")
        .unwrap();
    let refused = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "writer-4"}),
    );
    assert_eq!(
        refused["error"]["data"]["reason"], "workspace_owned",
        "{refused}"
    );
    assert_eq!(refused["error"]["data"]["execution"], old["id"]);
    for workspace in [None, Some("undeclared")] {
        let mut params = json!({"project": project.path, "ticket": "Y-1"});
        if let Some(workspace) = workspace {
            params["workspace"] = json!(workspace);
        }
        let refused = operator.call("yard_attempt_start", params);
        assert_eq!(
            refused["error"]["data"]["reason"], "undeclared_workspace",
            "{refused}"
        );
    }
    let refused = operator.call(
        "yard_attempt_stop",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert_eq!(
        refused["error"]["data"]["reason"], "ambiguous_execution",
        "{refused}"
    );
    let stopped = operator.call(
        "yard_attempt_stop",
        json!({"project": project.path, "ticket": "Y-1", "execution": old["id"]}),
    );
    assert!(stopped.get("error").is_none(), "{stopped}");
    watch.find("fifth checked stop", |e| {
        e["event"] == "execution.released" && e["execution"] == old["id"]
    });
    let fifth = writer_workspace(&root, "writer-4");
    let clone_inode = std::fs::metadata(fifth.join("clone")).unwrap().ino();
    assert_eq!(
        std::fs::read(fifth.join("clone/partial.txt")).unwrap(),
        b"partial\n"
    );
    held[4].release();
    let resumed = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "writer-4"}),
    );
    assert!(resumed.get("error").is_none(), "{resumed}");
    continued.wait_held();
    let successor = writer_rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'writer-4' ORDER BY id DESC LIMIT 1",
    )[0]
    .clone();
    std::fs::write(&owner, successor["handle"].as_str().unwrap()).unwrap();
    assert_eq!(successor["resumed"], old["id"]);
    assert_eq!(successor["session_id"], old["session_id"]);
    assert_eq!(
        std::fs::metadata(fifth.join("clone")).unwrap().ino(),
        clone_inode
    );
    for latch in &held[..4] {
        latch.release();
    }
    for latch in &readers[..4] {
        latch.wait_held();
    }
    let published = writer_rows(
        &project,
        "SELECT * FROM execution WHERE attempt = 2 AND kind = 'implementation' AND outcome = 'candidate' ORDER BY node",
    );
    assert_eq!(published.len(), 4);
    let mut proofs = std::collections::BTreeSet::new();
    for (n, expected_head) in expected_heads.iter().enumerate().take(4) {
        let name = match n {
            0 => "implementer".to_string(),
            2 => "selector".to_string(),
            3 => "build-gate-model".to_string(),
            _ => format!("writer-{n}"),
        };
        let producer = published
            .iter()
            .find(|row| row["workspace"] == name)
            .unwrap();
        let candidate = &producer["result"]["outputs"]["result"]["value"];
        assert_eq!(producer["exit_code"], 0);
        assert_eq!(candidate["candidate"], producer["id"]);
        assert_eq!(candidate["workspace"], name);
        assert_eq!(candidate["producer"]["execution"], producer["id"]);
        assert_eq!(candidate["producer"]["node"], name);
        assert_eq!(
            candidate["producer"]["settings"]["model"],
            format!("trial-model-{n}")
        );
        assert_eq!(
            candidate["producer"]["profile"]["name"],
            format!("trial{n}")
        );
        assert_eq!(candidate["head"], expected_head.as_str());
        let proof = candidate["proof"].as_str().unwrap();
        assert!(proofs.insert(proof.to_string()));
        assert_eq!(
            std::fs::read(
                root.join("proof-snapshots")
                    .join(proof)
                    .join("evidence.txt")
            )
            .unwrap(),
            format!("proof-{n}\n").as_bytes()
        );
        assert_eq!(
            git(
                &project.canonical(),
                &["show", &format!("{}:same.txt", expected_head)]
            ),
            format!("writer-{n}\n")
        );
        let reader = writer_rows(
            &project,
            &format!("SELECT * FROM execution WHERE attempt = 2 AND node = 'review-{n}'"),
        )[0]
        .clone();
        assert_eq!(
            reader["inputs"]["candidate"]["source"]["execution"],
            producer["id"]
        );
        assert_eq!(reader["inputs"]["candidate"]["value"], *candidate);
        assert_eq!(reader["head"], expected_head.as_str());
        assert_eq!(reader["proof"], proof);
    }
    // Mutate a live retained writer after publication while its reader is held.
    std::fs::write(
        writer_workspace(&root, "implementer").join("clone/same.txt"),
        "later-live\n",
    )
    .unwrap();
    std::fs::write(
        writer_workspace(&root, "implementer").join("proof/evidence.txt"),
        "later-live\n",
    )
    .unwrap();
    for (n, latch) in readers.iter().enumerate().take(4) {
        latch.release();
        let row = writer_rows(
            &project,
            &format!("SELECT id FROM execution WHERE attempt = 2 AND node = 'review-{n}'"),
        )[0]
        .clone();
        watch.find("frozen reader release", |e| {
            e["event"] == "execution.released" && e["execution"] == row["id"]
        });
        assert_eq!(
            writer_rows(
                &project,
                &format!(
                    "SELECT outcome, exit_code FROM execution WHERE id = {}",
                    row["id"]
                )
            ),
            vec![json!({"outcome": "pass", "exit_code": 0})]
        );
    }
    // A second writer is interrupted by the same crash. Both exact start
    // answers must survive while another stopped item or owner remains.
    let other = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workspace": "build-gate-model"
        }),
    );
    assert!(other.get("error").is_none(), "{other}");
    other_continued.wait_held();
    let other_writer = writer_rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'build-gate-model' ORDER BY id DESC LIMIT 1",
    )[0]
    .clone();
    let other_workspace = writer_workspace(&root, "build-gate-model");
    let other_inode = std::fs::metadata(other_workspace.join("clone"))
        .unwrap()
        .ino();
    let other_lock =
        std::fs::File::open(root.join(format!("effects/{}/lock", other_writer["id"]))).unwrap();
    assert!(matches!(
        other_lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == other_writer["handle"])
    );
    let lock = std::fs::File::open(root.join(format!("effects/{}/lock", successor["id"]))).unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(continued.is_held());
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == successor["handle"])
    );
    assert_eq!(
        std::fs::read(fifth.join("clone/interrupted.txt")).unwrap(),
        b"interrupted\n"
    );
    machine.kill();
    let said = watch.said(&owner_said);
    assert!(said.starts_with(successor["handle"].as_str().unwrap()));
    // Prove the held owner and partial tree externally before recovery.
    assert_eq!(
        writer_rows(
            &project,
            &format!(
                "SELECT status, reserved FROM execution WHERE id = {}",
                successor["id"]
            )
        ),
        vec![json!({"status": "running", "reserved": 1})]
    );
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert_eq!(
        std::fs::read(fifth.join("clone/interrupted.txt")).unwrap(),
        b"interrupted\n"
    );
    assert_eq!(
        std::fs::read(other_workspace.join("clone/other-partial.txt")).unwrap(),
        b"other-partial\n"
    );
    assert_eq!(
        writer_rows(
            &project,
            &format!(
                "SELECT status, reserved FROM execution WHERE id = {}",
                other_writer["id"]
            )
        ),
        vec![json!({"status": "running", "reserved": 1})]
    );
    machine.start();
    drop(watch);
    let mut watch = project.watch(0);
    let retained = watch.find("crashed writer retained owner", |e| {
        e["event"] == "attention.raised"
            && e["data"]["reason"] == "ownership"
            && e["execution"] == successor["id"]
    });
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == successor["handle"])
    );
    let refused = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "writer-4"}),
    );
    assert_eq!(
        refused["error"]["data"]["reason"], "workspace_owned",
        "{refused}"
    );
    assert_eq!(refused["error"]["data"]["execution"], successor["id"]);
    assert_eq!(
        writer_rows(
            &project,
            "SELECT id FROM execution WHERE workspace = 'writer-4'"
        )
        .len(),
        2
    );
    assert_eq!(
        writer_rows(
            &project,
            "SELECT * FROM execution WHERE attempt = 2 AND kind = 'implementation' AND outcome = 'candidate' ORDER BY node"
        ),
        published
    );
    std::fs::remove_file(&owner).unwrap();
    writeln!(release_owner, "release").unwrap();
    watch.find("crashed writer checked release", |e| {
        e["event"] == "execution.released" && e["execution"] == successor["id"]
    });
    let ended = watch.find("crashed writer interrupted", |e| {
        e["event"] == "execution.ended" && e["execution"] == successor["id"]
    });
    assert_eq!(ended["data"]["outcome"], "interrupted");
    let attention = project.json(&["attempt", "show", "Y-1"]);
    assert!(
        !attention["attention"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["attention"] == retained["attention"])
    );
    lock.try_lock().unwrap();
    lock.unlock().unwrap();
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == successor["handle"])
    );
    assert_eq!(
        writer_rows(
            &project,
            "SELECT * FROM execution WHERE attempt = 2 AND kind = 'implementation' AND outcome = 'candidate' ORDER BY node"
        ),
        published
    );
    assert_eq!(
        writer_rows(
            &project,
            "SELECT id FROM execution WHERE workspace = 'writer-4'"
        )
        .len(),
        2
    );
    assert_eq!(
        std::fs::read(fifth.join("clone/interrupted.txt")).unwrap(),
        b"interrupted\n"
    );
    continued.release();
    other_continued.release();
    let attention = project.json(&["attempt", "show", "Y-1"]);
    for predecessor in [&other_writer, &successor] {
        let item = attention["attention"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["execution"] == predecessor["id"] && item["reason"] == "interrupted")
            .unwrap();
        let resumed = operator.call(
            "yard_attempt_start",
            json!({
                "project": project.path, "ticket": "Y-1", "attention": item["attention"]
            }),
        );
        assert!(resumed.get("error").is_none(), "{resumed}");
        let current = writer_rows(
            &project,
            &format!(
                "SELECT * FROM execution WHERE workspace = '{}' ORDER BY id DESC LIMIT 1",
                predecessor["workspace"].as_str().unwrap()
            ),
        )[0]
        .clone();
        assert_ne!(current["id"], predecessor["id"]);
        assert_eq!(resumed["result"]["execution"], current["id"]);
    }
    other_recovered.wait_held();
    let other_successor = writer_rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'build-gate-model' ORDER BY id DESC LIMIT 1",
    )[0]
    .clone();
    assert_eq!(other_successor["resumed"], other_writer["id"]);
    assert_eq!(other_successor["session_id"], other_writer["session_id"]);
    assert_eq!(
        std::fs::metadata(other_workspace.join("clone"))
            .unwrap()
            .ino(),
        other_inode
    );
    other_recovered.release();
    readers[4].wait_held();
    let final_writer = writer_rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'writer-4' ORDER BY id DESC LIMIT 1",
    )[0]
    .clone();
    assert_eq!(final_writer["resumed"], successor["id"]);
    assert_eq!(final_writer["session_id"], old["session_id"]);
    assert_eq!(
        std::fs::metadata(fifth.join("clone")).unwrap().ino(),
        clone_inode
    );
    let head = git(&fifth.join("clone"), &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    let candidate = &final_writer["result"]["outputs"]["result"]["value"];
    assert_eq!(final_writer["exit_code"], 0);
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:partial.txt")]
        ),
        "partial\n"
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:interrupted.txt")]
        ),
        "interrupted\n"
    );
    let proof = candidate["proof"].as_str().unwrap();
    assert!(proofs.insert(proof.to_string()));
    assert_eq!(
        std::fs::read(
            root.join("proof-snapshots")
                .join(proof)
                .join("evidence.txt")
        )
        .unwrap(),
        b"proof-4\n"
    );
    assert_eq!(candidate["head"], head);
    assert_eq!(candidate["producer"]["execution"], final_writer["id"]);
    assert_eq!(candidate["producer"]["settings"]["model"], "trial-model-4");
    assert_eq!(candidate["producer"]["profile"]["name"], "trial4");
    let final_reader = writer_rows(
        &project,
        "SELECT * FROM execution WHERE attempt = 2 AND node = 'review-4'",
    )[0]
    .clone();
    assert_eq!(
        final_reader["inputs"]["candidate"]["source"]["execution"],
        final_writer["id"]
    );
    assert_eq!(final_reader["inputs"]["candidate"]["value"], *candidate);
    readers[4].release();
    watch.find("all candidates retained", |e| {
        e["event"] == "attention.raised" && e["data"]["reason"] == "candidates"
    });
    assert_eq!(
        writer_rows(
            &project,
            &format!(
                "SELECT outcome, exit_code FROM execution WHERE id = {}",
                final_reader["id"]
            )
        ),
        vec![json!({"outcome": "pass", "exit_code": 0})]
    );
    assert_eq!(
        project.json(&["attempt", "show", "Y-1"])["candidates"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(project.canonical_head(), frozen_target);
    let recovered = writer_rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'build-gate-model' ORDER BY id DESC LIMIT 1",
    )[0]
    .clone();
    assert_eq!(recovered["outcome"], "candidate");
    assert_eq!(recovered["exit_code"], 0);
    assert_eq!(
        git(
            &project.canonical(),
            &[
                "show",
                &format!(
                    "{}:other-partial.txt",
                    recovered["result"]["outputs"]["result"]["value"]["head"]
                        .as_str()
                        .unwrap()
                )
            ]
        ),
        "other-partial\n"
    );
    assert_eq!(
        writer_rows(
            &project,
            "SELECT * FROM execution WHERE workspace = 'build-gate-model' AND outcome = 'candidate' ORDER BY id"
        )[0],
        published
            .iter()
            .find(|row| row["workspace"] == "build-gate-model")
            .unwrap()
            .clone()
    );
    let untouched = project.path.join(".yard/local/attempts/987/sentinel");
    std::fs::create_dir_all(untouched.parent().unwrap()).unwrap();
    std::fs::write(&untouched, "other workspace\n").unwrap();
    project.json(&["attempt", "abandon", "Y-1"]);
    let cleanup = watch.find("all writer cleanup", |e| {
        e["event"] == "execution.ended" && e["data"]["kind"] == "cleanup" && e["attempt"] == 2
    });
    watch.find("all writer cleanup released", |e| {
        e["event"] == "execution.released" && e["execution"] == cleanup["execution"]
    });
    assert!(!root.exists());
    assert_eq!(std::fs::read(untouched).unwrap(), b"other workspace\n");
}

// Locate the host seam; assertions above observe bytes and inode isolation.
fn writer_workspace(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    let key: String = name.bytes().map(|byte| format!("{byte:02x}")).collect();
    root.join("writers").join(key)
}

fn writer_rows(project: &Project<'_>, query: &str) -> Vec<Value> {
    project
        .rows(query)
        .into_iter()
        .map(|mut row| {
            for field in ["settings", "inputs", "result"] {
                if let Some(text) = row[field].as_str() {
                    row[field] = serde_json::from_str(text).unwrap();
                }
            }
            row
        })
        .collect()
}

fn fifo(path: &std::path::Path) {
    assert!(
        std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .unwrap()
            .success()
    );
}

/// G4: a full reviewed ticket at limit one runs sequential landing children
/// and the final Git effect. A held host gate excludes a new boxed worker,
/// even though approval released its attempt lane. The fixture's FIFO and
/// gate markers supply the expected acting effects; git supplies landed bytes.
/// Sabotage: keep a parent reservation while running children, count only
/// boxes/attempts, or release the host slot before its child ends.
#[test]
fn one_slot_lands_with_child_gates_and_excludes_a_worker() {
    let machine = Machine::new("g4-host-slot", |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        let file = if request.prompt().contains("Second") {
            "second.txt"
        } else {
            "first.txt"
        };
        act(request, vec![commit_file(file, "feature\n", "Feature")])
    });
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    let markers = machine.root.join("markers");
    fifo(&said);
    fifo(&release);
    let gates = format!(
        "[gates.candidate]\nstage = \"candidate\"\ncommand = \"test -f first.txt || test -f second.txt\"\n\
         [gates.first]\nruns_in = \"host\"\ncommand = \"echo held > {}; read line < {}; echo first >> {}\"\n\
         [gates.second]\nruns_in = \"host\"\ncommand = \"echo second >> {}\"\n",
        said.display(),
        release.display(),
        markers.display(),
        markers.display()
    );
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(&gates).replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1"),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "First"]);
    let head = approval_of(&mut watch, "Y-1");
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    assert_eq!(watch.said(&said), "held\n");
    project.json(&["ticket", "new", "--title", "Second"]);
    let waiting = watch.event("execution.waiting", &[]);
    assert_eq!(waiting["ticket"], "Y-2");
    let owner = project.rows("SELECT kind, parent, name FROM execution WHERE reserved = 1");
    assert_eq!(owner.len(), 1, "{owner:?}");
    assert_eq!(owner[0]["kind"], "gate");
    assert_eq!(owner[0]["name"], "first");
    assert!(owner[0]["parent"].is_i64());
    assert!(
        !machine
            .model
            .requests()
            .iter()
            .any(|r| r.prompt().contains("Second"))
    );
    std::fs::write(&release, "go\n").unwrap();
    watch.find("First landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    approval_of(&mut watch, "Y-2");
    assert_eq!(std::fs::read_to_string(markers).unwrap(), "first\nsecond\n");
    assert_eq!(
        git(&project.canonical(), &["show", "main:first.txt"]),
        "feature\n"
    );
}

/// G4/G5: an old Git child is held in clone before any worker starts.
/// Killing the daemon leaves the inherited lock and reservation. A second
/// ticket is admitted after restart but cannot act until that exact child
/// exits. Unlike the existing restart test, the old owner survives startup.
/// Sabotage: release capacity when the daemon dies, omit the inherited
/// lock on a pre-box Git effect, or change ownership attention from red.
#[test]
fn an_old_host_child_retains_capacity_across_restart() {
    let machine = Machine::new("g4-old-child", |request| {
        act(
            request,
            vec![commit_file("second.txt", "second\n", "Second")],
        )
    });
    let armed = machine.root.join("armed");
    let said_path = machine.root.join("said");
    let release = machine.root.join("release");
    fifo(&said_path);
    fifo(&release);
    machine.wrapper("git", &format!(
        "if [ -e '{}' ]; then case \" $* \" in *' clone '*) echo $$ > '{}'; read line < '{}';; esac; fi",
        armed.display(), said_path.display(), release.display()));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("")
            .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1")
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    std::fs::write(&armed, "").unwrap();
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "First"]);
    let pid = watch.said(&said_path).trim().to_string();
    drop(watch);
    machine.kill();
    let old = project.rows("SELECT id, status, reserved, handle FROM execution");
    assert_eq!(
        old,
        vec![json!({"id": 1, "status": "running", "reserved": 1, "handle": null})]
    );
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid])
            .status()
            .unwrap()
            .success()
    );
    std::fs::remove_file(&armed).unwrap();
    machine.start();
    let mut watch = project.watch(0);
    let attention = watch.attention();
    assert_eq!(attention["execution"], 1);
    assert_eq!(attention["data"]["kind"], "red");
    assert_eq!(attention["data"]["reason"], "ownership");
    project.json(&["ticket", "new", "--title", "Second"]);
    let waiting = watch.event("execution.waiting", &[]);
    assert_eq!(waiting["ticket"], "Y-2");
    assert!(machine.model.requests().is_empty());
    assert_eq!(
        project.rows("SELECT id FROM execution WHERE reserved = 1"),
        vec![json!({"id": 1})]
    );
    std::fs::write(&release, "go\n").unwrap();
    approval_of(&mut watch, "Y-2");
    assert_eq!(
        project.rows("SELECT reserved, outcome FROM execution WHERE id = 1"),
        vec![json!({"reserved": 0, "outcome": "interrupted"})]
    );
}
