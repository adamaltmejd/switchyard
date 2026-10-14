//! G4: Capacity holds.

use e2e::*;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

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

/// The default workflow on one lane and no review.
fn single_lane(extra: &str) -> String {
    config(extra)
        .replace("max_lanes = 2", "max_lanes = 1")
        .replace("review = [\"correctness\"]", "review = \"none\"")
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

/// With `max_lanes` 1, an approved candidate whose landing fails returns for
/// repair and waits for a lane. When a second attempt holding the only lane
/// is abandoned while a new ticket is ready, the freed lane goes to the
/// returning repair, not to the new admission.
///
/// Sabotage: run `admit::scheduled` before the live attempts reacquire; the
/// ready ticket is admitted first and the repair waits.
/// G4's pending-adjustment clause also observes the Live adjustment contract:
/// the actual sole-lane holder excludes repair startup, so a valid revision
/// refuses without writes. The same adjustment succeeds during held repair.
/// Other revision tests hold executing work, not a pending returning repair.
/// Oracles: fixture latches, read-only store identities and unchanged rows.
/// Sabotage to qualify on the host: remove
/// `(current.next.is_some() && !implementing)` in `attempt_revise`.
/// The store proves a candidate is present, so the adjacent candidate/
/// implementing eligibility guard cannot mask that mutation.
#[test]
fn a_returning_repair_takes_a_freed_lane_before_a_new_ticket() {
    let repair = Latch::new();
    let holder = Latch::new();
    let (held, holder_held) = (repair.clone(), holder.clone());
    let machine = Machine::new("g4-repair-lane", move |request| {
        let prompt = request.prompt();
        if request.opens()
            && prompt.contains("Repair")
            && request.last_user().contains("failed to land")
        {
            // Hold the repair so the lane stays taken while the test reads.
            return Reply::Hold(held.clone(), Box::new(Reply::Text("done".into())));
        }
        if prompt.contains("Repair") {
            return act(request, vec![commit_file("bad.txt", "bad\n", "Repair")]);
        }
        if request.opens() && prompt.contains("Holder") {
            return Reply::Hold(holder_held.clone(), Box::new(Reply::Text("done".into())));
        }
        act(
            request,
            vec![commit_file("holder.txt", "holder\n", "Holder")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &single_lane(
            "[gates.land]\ncommand = \"test ! -f bad.txt\"\n\
             [profiles.revised]\nrunner = {kind = \"agent\", agent = \"worker\", instructions = \"Repair with revised instructions.\"}\n\
             access = \"write-allowed\"\noutputs = {result = \"candidate\"}\noutcomes = [\"complete\"]\n",
        ),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Repair"]);
    let head = approval_of(&mut watch, "Y-1");
    project.json(&["ticket", "new", "--title", "Holder"]);
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);

    // The holder takes the lane as the approved candidate starts landing.
    watch.until("Y-2 admitted", |event| {
        event["event"] == "attempt.admitted" && event["ticket"] == "Y-2"
    });
    // The landing gate fails; Y-1 is returned for repair and waits.
    let landing = watch.until("Y-1's landing red", |event| {
        event["event"] == "execution.ended"
            && event["ticket"] == "Y-1"
            && event["data"]["kind"] == "landing"
            && event["data"]["outcome"] == "red"
    });
    assert_eq!(live(&project, 1), 1);
    assert_eq!(
        project.rows("SELECT lane FROM attempt WHERE ticket = 1"),
        vec![json!({ "lane": 0 })]
    );

    holder.wait_held();
    // Landing releases its slot between child gates. Only the release after
    // its terminal event settles the repair's ownership before snapshots.
    watch.find("landing checked release", |event| {
        event["event"] == "execution.released"
            && event["execution"] == landing["execution"]
            && event["seq"].as_i64().unwrap() > landing["seq"].as_i64().unwrap()
    });
    let pending_sql = "SELECT * FROM attempt WHERE ticket = 1";
    let pending = project.rows(pending_sql);
    assert_eq!(pending.len(), 1);
    let attempt = pending[0]["id"].as_i64().unwrap();
    assert_eq!(pending[0]["state"], "live");
    assert_eq!(pending[0]["lane"], 0);
    let repair_next: Value = serde_json::from_str(pending[0]["next"].as_str().unwrap()).unwrap();
    assert_eq!(repair_next["reason"], "repair");
    assert_eq!(pending[0]["head"], head);
    let running_sql = format!(
        "SELECT id FROM execution WHERE attempt = {attempt} AND kind = 'implementation' AND status = 'running'"
    );
    assert!(project.rows(&running_sql).is_empty());
    let owners = project.rows("SELECT execution.id, handle FROM execution JOIN attempt ON attempt.id = execution.attempt WHERE attempt.ticket = 2 AND attempt.lane = 1 AND execution.kind = 'implementation' AND execution.status = 'running'");
    assert_eq!(owners.len(), 1);
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|listed| listed["name"] == owners[0]["handle"])
    );
    let revision_sql = format!(
        "SELECT COALESCE((SELECT max(revision) FROM plan_revision WHERE attempt = {attempt}), json_extract(admission, '$.revision')) AS revision FROM attempt WHERE id = {attempt}"
    );
    let revisions = project.rows(&revision_sql);
    let revision = revisions[0]["revision"].as_i64().unwrap();
    let plans_sql =
        format!("SELECT * FROM plan_revision WHERE attempt = {attempt} ORDER BY revision");
    let plans = project.rows(&plans_sql);
    let attention = project.rows("SELECT * FROM attention ORDER BY id");
    let approvals = project.rows("SELECT * FROM approval ORDER BY id");
    let mut operator = machine.operator();
    operator.initialize();
    let audit = project.rows("SELECT * FROM audit ORDER BY seq");
    let adjustment = json!({
        "project": project.path, "ticket": "Y-1", "plan_revision": revision,
        "overrides": {"implementer": {"profile": "revised"}}
    });
    let refused = operator.call("yard_attempt_revise", adjustment.clone());
    assert_eq!(refused["error"]["code"], "refused", "{refused}");
    assert_eq!(
        refused["error"]["data"],
        json!({"reason": "active_plan", "attempt": attempt})
    );
    assert_eq!(project.rows(&revision_sql), revisions);
    assert_eq!(project.rows(&plans_sql), plans);
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), audit);
    assert_eq!(project.rows(pending_sql), pending);
    assert_eq!(
        project.rows("SELECT * FROM attention ORDER BY id"),
        attention
    );
    assert_eq!(
        project.rows("SELECT * FROM approval ORDER BY id"),
        approvals
    );

    // A ready ticket now competes with the repairing attempt for the lane.
    project.json(&["ticket", "new", "--title", "New"]);
    project.json(&["attempt", "abandon", "Y-2"]);
    let next = watch.until("Y-1's repair or Y-3's admission", |event| {
        (event["event"] == "execution.started"
            && event["ticket"] == "Y-1"
            && event["data"]["kind"] == "implementation"
            && event["data"]["reason"] == "repair")
            || (event["event"] == "attempt.admitted" && event["ticket"] == "Y-3")
    });
    assert_eq!(next["ticket"], "Y-1", "{next}");
    repair.wait_held();
    assert_eq!(
        project.rows(&running_sql),
        vec![json!({"id": next["execution"]})]
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT status, reserved FROM execution WHERE id = {}",
            owners[0]["id"]
        )),
        vec![json!({"status": "ended", "reserved": 0})]
    );
    assert_eq!(project.rows(&revision_sql), revisions);
    let adjusted = operator.call("yard_attempt_revise", adjustment);
    assert_eq!(adjusted["result"]["attempt"], attempt, "{adjusted}");
    assert_eq!(
        adjusted["result"]["plan_revision"],
        revision + 1,
        "{adjusted}"
    );
    assert_eq!(
        project.rows(&revision_sql),
        vec![json!({"revision": revision + 1})]
    );
    repair.release();
    holder.release();
}

/// G4: attempt lanes admit both racers, but the last shared effect slot
/// admits only one real worker. The other acts after checked teardown.
/// Existing lane races cannot catch an effect limit below max_lanes.
/// Sabotage: reserve after spawn, or count only attempt lanes.
#[test]
fn racing_workers_share_one_execution_slot() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g4-effects-race", move |request| {
        if request.opens() {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![commit_file(
                    "feature.txt",
                    "feature\n",
                    "Feature",
                )])),
            );
        }
        Reply::Text("done".into())
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("")
            .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1")
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    std::thread::scope(|scope| {
        let first = scope.spawn(|| project.json(&["ticket", "new", "--title", "First"]));
        let second = scope.spawn(|| project.json(&["ticket", "new", "--title", "Second"]));
        first.join().unwrap();
        second.join().unwrap();
    });
    hold.wait_held();
    let waiting = watch.event("execution.waiting", &[]);
    assert_eq!(waiting["data"]["reason"], "capacity");
    let owners = project.rows("SELECT id, handle FROM execution WHERE reserved = 1");
    assert_eq!(owners.len(), 1, "{owners:?}");
    // Pinfold supplies the project label on this fixture's held worker.
    // The presence-only label also includes unrelated host projects.
    let worker = machine
        .boxes("dev.yard.project")
        .into_iter()
        .find(|listed| listed["name"] == owners[0]["handle"])
        .expect("the held worker has a box");
    let label = format!(
        "dev.yard.project={}",
        worker["labels"]["dev.yard.project"].as_str().unwrap()
    );
    let boxes = machine.boxes(&label);
    assert_eq!(boxes.len(), 1, "{boxes:?}");
    assert_eq!(boxes[0]["name"], owners[0]["handle"]);
    assert_ne!(owners[0]["id"], waiting["execution"]);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|r| r.opens())
            .count(),
        1
    );
    hold.release();
    for ticket in ["Y-1", "Y-2"] {
        approval_of(&mut watch, ticket);
    }
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|r| r.opens())
            .count(),
        2
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
    let (reader_opened, reader_openings) = std::sync::mpsc::channel();
    let machine = Machine::new("g4-graph-join", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        let prompt = request.prompt();
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
                    return Reply::Hold(
                        latch.clone(),
                        Box::new(Reply::Tools(vec![publish(json!([]))])),
                    );
                }
            }
            return act(request, vec![publish(json!([]))]);
        }
        let graph = prompt.contains("Graph");
        let reply = act(
            request,
            vec![commit_file(
                if graph {
                    "graph.txt"
                } else {
                    "independent.txt"
                },
                "fixture\n",
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
    machine.start();
    let fixture = config(r#"
[review.seats.left]
agent = "worker"
instructions = "Left reader."
[review.seats.right]
agent = "worker"
instructions = "Right reader."
[review.seats.join]
agent = "worker"
instructions = "Exact join."
[profiles.revised]
runner = {kind = "agent", agent = "worker", instructions = "Revised right reader."}
access = "read-only"
outputs = {result = "verdict"}
outcomes = ["complete", "blocked"]
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
    let graph = json!({"inputs": {"feedback": "verdict", "merged": "candidate"}, "nodes": {
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
        "review-0": {"runner": {"kind": "agent", "agent": "worker", "instructions": "Left reader."}, "access": "read-only",
            "inputs": {"candidate": candidate}, "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"],
            "requires": ["candidate"], "routes": {"complete": [{"to": "review-2", "repeat": null}],
                "blocked": [{"to": "implementer", "repeat": 3}]}},
        "review-1": {"runner": {"kind": "agent", "agent": "worker", "instructions": "Right reader."}, "access": "read-only",
            "inputs": {"candidate": candidate}, "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"],
            "requires": ["candidate"], "routes": {"complete": [{"to": "review-2", "repeat": null}],
                "blocked": [{"to": "implementer", "repeat": 3}]}},
        "review-2": {"runner": {"kind": "agent", "agent": "worker", "instructions": "Exact join."}, "access": "read-only",
            "inputs": {"candidate": candidate,
                "command": {"node": "candidate-gate-read", "output": "result", "kind": "verdict"},
                "left": {"node": "review-0", "output": "result", "kind": "verdict"},
                "right": {"node": "review-1", "output": "result", "kind": "verdict"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"],
            "requires": ["candidate", "command", "left", "right"], "routes": {"blocked": [{"to": "implementer", "repeat": 3}]}}
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
    let opens = |instructions: &str| {
        machine
            .model
            .requests()
            .iter()
            .filter(|request| {
                request.opens()
                    && request.prompt().contains("Graph")
                    && request.prompt().contains(instructions)
            })
            .count()
    };
    assert_eq!(opens("Left reader."), 1);
    assert_eq!(opens("Right reader."), 1);
    assert_eq!(opens("Exact join."), 0);
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
    assert_eq!(opens("Left reader."), 1);
    assert_eq!(opens("Right reader."), 2);
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
    assert_eq!(opens("Exact join."), 1);

    // An explicit revision invalidates branch consumers, never old evidence.
    let changed = operator.call(
        "yard_attempt_revise",
        json!({"project": project.path, "ticket": "Y-1", "plan_revision": 1,
        "overrides": {"review-1": {"profile": "revised"}}}),
    );
    assert_eq!(changed["result"]["plan_revision"], 2, "{changed}");
    revised.wait_held();
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
    let suppliers = project.rows("SELECT id, node FROM execution WHERE attempt = 3 AND plan_revision = 2 AND kind = 'review' ORDER BY node");
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
    assert_eq!(opens("Left reader."), 2);
    assert_eq!(opens("Revised right reader."), 1);
    assert_eq!(opens("Exact join."), 2);
    let mut revised_graph = graph.clone();
    revised_graph["nodes"]["review-1"]["runner"]["instructions"] = json!("Revised right reader.");
    let shown = project.json(&["attempt", "show", "Y-1"]);
    assert_eq!(shown["attempt"]["admission"], original_admission);
    assert_eq!(shown["attempt"]["plan"]["plan"], revised_graph);
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "main",
        "Y-1",
        "--head",
        &head,
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
        "fixture\n"
    );
    assert_eq!(
        git(&project.canonical(), &["show", "feature/a:independent.txt"]),
        "fixture\n"
    );
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
    let edit_held = Latch::new();
    let edit_unchanged = Latch::new();
    let writer_holds = held.clone();
    let reader_holds = readers.clone();
    let continuation_hold = continued.clone();
    let other_hold = other_continued.clone();
    let recovery_hold = other_recovered.clone();
    let edit_hold = edit_held.clone();
    let unchanged_hold = edit_unchanged.clone();
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
        if request.prompt().contains("Edit handoff trial") {
            if request.opens() {
                let edited = request.context().contains("Edited handoff instructions.");
                if n == 0 && !edited {
                    return Reply::Hold(
                        unchanged_hold.clone(),
                        Box::new(Reply::Text("done".into())),
                    );
                }
                if n == 1 && !edited {
                    return Reply::Tools(vec![bash("printf 'kept\\n' > edit-partial.txt")]);
                }
                let mut calls = vec![bash(&format!(
                    "printf 'writer-{n}\\n' > same.txt; printf 'proof-{n}\\n' > /yard/proof/evidence.txt"
                ))];
                if edited {
                    calls.push(bash("printf 'edited\\n' > revision.txt"));
                    if n == 1 {
                        calls.push(bash("test \"$(cat edit-partial.txt)\" = kept"));
                    }
                }
                calls.push(bash(
                    "git add . && git commit -qm 'Read frozen instructions'",
                ));
                return Reply::Tools(calls);
            }
            if n == 1 && !request.context().contains("Edited handoff instructions.") {
                return Reply::Hold(edit_hold.clone(), Box::new(Reply::Text("done".into())));
            }
            return Reply::Text("done".into());
        }
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

    // G9: an edit answers the stopped writer after the other owner drains.
    // Sabotage: return on open stopped attention before processing the edit.
    // The unchanged writer and held partial-tree writer share the same five-root
    // graph. Fixture revision bytes and real exits prove every successor runs.
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Edit handoff trial",
        "--workflow",
        "plan",
    ]);
    let planned = watch.find("edit trial planned", |e| {
        e["event"] == "attempt.ended" && e["ticket"] == "Y-2"
    });
    let cleanup = watch.find("edit trial planner cleanup", |e| {
        e["event"] == "execution.ended"
            && e["data"]["kind"] == "cleanup"
            && e["attempt"] == planned["attempt"]
    });
    watch.find("edit trial planner released", |e| {
        e["event"] == "execution.released" && e["execution"] == cleanup["execution"]
    });
    let started = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-2", "graph": {"inputs": {}, "nodes": nodes}
        }),
    );
    assert!(started.get("error").is_none(), "{started}");
    let edit_attempt = started["result"]["attempt"].as_i64().unwrap();
    edit_held.wait_held();
    edit_unchanged.wait_held();
    edit_unchanged.release();
    let stopped = watch.find("unchanged writer before edit", |e| {
        e["event"] == "attention.raised"
            && e["attempt"] == edit_attempt
            && e["data"]["reason"] == "unchanged"
    });
    let before = writer_rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE attempt = {edit_attempt} AND kind = 'implementation' ORDER BY node"
        ),
    );
    assert_eq!(before.len(), 5);
    let held_writer = before
        .iter()
        .find(|row| row["workspace"] == "writer-1")
        .unwrap();
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == held_writer["handle"])
    );
    let edit_root = project
        .path
        .join(format!(".yard/local/attempts/{edit_attempt}"));
    let held_clone = writer_workspace(&edit_root, "writer-1").join("clone");
    let held_inode = std::fs::metadata(&held_clone).unwrap().ino();
    let edited = operator.call("yard_ticket_edit", json!({
        "project": project.path, "ticket": "Y-2", "revision": 1, "body": "Edited handoff instructions."
    }));
    assert!(edited.get("error").is_none(), "{edited}");
    let shown = project.json(&["attempt", "show", "Y-2"]);
    assert!(
        shown["attention"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["attention"] == stopped["attention"])
    );
    assert_eq!(writer_rows(&project, &format!(
        "SELECT id FROM execution WHERE attempt = {edit_attempt} AND kind = 'implementation'")).len(), 5);
    edit_held.release();
    watch.find("edited writers publish", |e| {
        e["event"] == "attention.raised"
            && e["attempt"] == edit_attempt
            && e["data"]["reason"] == "candidates"
    });
    let after = writer_rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE attempt = {edit_attempt} AND kind = 'implementation' AND ticket_revision = 2 ORDER BY node"
        ),
    );
    assert_eq!(after.len(), 5);
    for producer in &after {
        assert_eq!(producer["outcome"], "candidate");
        assert_eq!(producer["exit_code"], 0);
        let head = producer["result"]["outputs"]["result"]["value"]["head"]
            .as_str()
            .unwrap();
        assert_eq!(
            git(
                &project.canonical(),
                &["show", &format!("{head}:revision.txt")]
            ),
            "edited\n"
        );
        let reader = writer_rows(
            &project,
            &format!(
                "SELECT inputs, outcome, exit_code FROM execution WHERE attempt = {edit_attempt} AND kind = 'review' AND head = '{head}'"
            ),
        );
        assert_eq!(reader.len(), 1);
        assert_eq!(
            reader[0]["inputs"]["candidate"]["source"]["execution"],
            producer["id"]
        );
        assert_eq!(reader[0]["outcome"], "pass");
        assert_eq!(reader[0]["exit_code"], 0);
    }
    let continued = after
        .iter()
        .find(|row| row["workspace"] == "writer-1")
        .unwrap();
    assert_eq!(std::fs::metadata(&held_clone).unwrap().ino(), held_inode);
    assert_eq!(
        git(
            &project.canonical(),
            &[
                "show",
                &format!(
                    "{}:edit-partial.txt",
                    continued["result"]["outputs"]["result"]["value"]["head"]
                        .as_str()
                        .unwrap()
                )
            ]
        ),
        "kept\n"
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT state, resolution FROM attention WHERE id = {}",
            stopped["attention"]
        )),
        vec![json!({"state": "resolved", "resolution": "edit"})]
    );
    project.json(&["attempt", "abandon", "Y-2"]);
    let cleanup = watch.find("edit trial cleanup", |e| {
        e["event"] == "execution.ended"
            && e["data"]["kind"] == "cleanup"
            && e["attempt"] == edit_attempt
    });
    watch.find("edit trial cleanup released", |e| {
        e["event"] == "execution.released" && e["execution"] == cleanup["execution"]
    });
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

/// A host observer responds to every protected-path Git call, including calls
/// without ownership. The external approval row distinguishes queue policy
/// from the scheduler's earlier decision to give automatic approval.
struct LandingPolicyProbe {
    armed: std::path::PathBuf,
    request: std::fs::File,
    reply: std::fs::File,
    observations: std::sync::Arc<std::sync::Mutex<Vec<Value>>>,
    reader: Option<std::thread::JoinHandle<()>>,
    releases: Vec<std::fs::File>,
    models: Vec<Latch>,
}

impl Drop for LandingPolicyProbe {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.armed);
        for model in &self.models {
            model.release();
        }
        for release in &mut self.releases {
            let _ = release.write_all(b"go\n");
        }
        let _ = self.reply.write_all(b"go\n");
        let _ = self.request.write_all(b"end\n");
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

/// G4/G5: a held worker excludes the first queue policy read after a default
/// switch and lowering sync. Then hold the first target read inheriting the
/// landing's host lock. Its reservation precedes any prepared ref or child
/// gate. Then, after an allowed first host gate, hold the protected-path
/// Git read before the second gate. The landing owns the only reservation;
/// release permits the second gate and final landing at limit one.
/// Expected gate names and landed bytes come from the fixture, and the parent
/// identity from the externally observed first child row.
/// Sabotage: omit the landing effect around gate-loop admission; the held
/// Git child has no reserved owner. Keep the parent slot during the child;
/// the second gate cannot complete at limit one.
/// Move the initial admission/load outside preparation: the first owned target read
/// comes after the prepared ref, failing the initial-boundary observer.
/// Put policy Git back in queue::next: the host observer records an unowned
/// read while the holder owns the slot. Allocate from main's target limit
/// rather than the synced current default: the landing does not wait behind
/// the holder at the lowered limit.
#[test]
fn landing_gate_policy_reads_hold_the_shared_execution_slot() {
    let feature_hold = Latch::new();
    let holder_hold = Latch::new();
    let (feature_held, holder_held) = (feature_hold.clone(), holder_hold.clone());
    let machine = Machine::new("g4-gate-policy-slot", move |request| {
        if request.opens() {
            if request.prompt().contains("Holder") {
                return Reply::Hold(holder_held.clone(), Box::new(Reply::Text("done".into())));
            }
            return Reply::Hold(
                feature_held.clone(),
                Box::new(Reply::Tools(vec![commit_file(
                    "feature.txt",
                    "feature\n",
                    "Feature",
                )])),
            );
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Feature")],
        )
    });
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    let armed = machine.root.join("armed");
    let markers = machine.root.join("markers");
    let initial = machine.root.join("initial");
    let initial_release = machine.root.join("initial-release");
    let initial_armed = machine.root.join("initial-armed");
    fifo(&said);
    fifo(&release);
    fifo(&initial);
    fifo(&initial_release);
    let gates = format!(
        "[gates.first]\nruns_in = \"host\"\ncommand = \"echo first > {}; read line < {}\"\n\
         [gates.second]\nruns_in = \"host\"\ncommand = \"echo second > {}\"\n",
        said.display(),
        release.display(),
        markers.display()
    );
    machine.start();
    let configured = config(&gates)
        .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 2")
        .replace("approve = \"manual\"", "approve = \"auto\"")
        .replace("review = [\"correctness\"]", "review = \"none\"");
    let project = Project::new(&machine, "p", &configured);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Feature"]);
    feature_hold.wait_held();
    project.json(&["ticket", "new", "--title", "Holder"]);
    holder_hold.wait_held();
    project.git(&["checkout", "-qb", "capacity"]);
    project.write(
        ".yard/config.toml",
        &configured
            .replace("max_executions = 2", "max_executions = 3")
            .replace("ref = \"main\"", "ref = \"capacity\""),
    );
    project.git(&["add", ".yard/config.toml"]);
    project.git(&["commit", "-qm", "Select capacity default"]);
    project.json(&["sync", "capacity", "--default"]);
    project.reconfigure(
        &configured
            .replace("max_executions = 2", "max_executions = 1")
            .replace("ref = \"main\"", "ref = \"capacity\""),
    );
    let policy_request = machine.root.join("policy-request");
    let policy_reply = machine.root.join("policy-reply");
    let policy_armed = machine.root.join("policy-armed");
    fifo(&policy_request);
    fifo(&policy_reply);
    let open_pipe = |path: &std::path::Path| {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap()
    };
    let request = open_pipe(&policy_request);
    let reply = open_pipe(&policy_reply);
    let input = request.try_clone().unwrap();
    let mut output = reply.try_clone().unwrap();
    let observations = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorded = observations.clone();
    let store = project.path.join(".yard/local/store.sqlite");
    let reader = std::thread::spawn(move || {
        let conn = rusqlite::Connection::open_with_flags(
            store,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        for line in std::io::BufReader::new(input).lines() {
            let line = line.unwrap();
            if line == "end" {
                break;
            }
            let approved: i64 = conn
                .query_row(
                    "SELECT count(*) FROM approval WHERE actor = 'auto' AND state = 'active'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            if approved != 0 {
                let mut query = conn
                    .prepare("SELECT id, kind FROM execution WHERE reserved = 1 ORDER BY id")
                    .unwrap();
                let owners = query
                    .query_map([], |row| {
                        Ok(json!({
                            "id": row.get::<_, i64>(0)?, "kind": row.get::<_, String>(1)?,
                        }))
                    })
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                recorded
                    .lock()
                    .unwrap()
                    .push(json!({ "owned": line == "yes", "owners": owners }));
            }
            output.write_all(b"go\n").unwrap();
        }
    });
    let probe = LandingPolicyProbe {
        armed: policy_armed.clone(),
        request,
        reply,
        observations,
        reader: Some(reader),
        releases: vec![open_pipe(&initial_release), open_pipe(&release)],
        models: vec![feature_hold.clone(), holder_hold.clone()],
    };
    std::fs::write(&policy_armed, "").unwrap();
    let policy_script = format!(
        "owned=no\nfor owner in '{}'/*/lock; do for descriptor in /dev/fd/*; do\n\
         if [ \"$descriptor\" -ef \"$owner\" ]; then owned=yes; fi\n\
         done; done\n\
         case \" $* \" in *' diff --name-only '*) if [ -e '{}' ]; then echo \"$owned\" > '{}'; read line < '{}'; fi;; esac\n",
        project.path.join(".yard/local/landings").display(),
        policy_armed.display(),
        policy_request.display(),
        policy_reply.display()
    );
    std::fs::write(&initial_armed, "").unwrap();
    // Scheduler/config reads do not inherit the landing's ownership lock.
    // Compare actual open file identities, not a fixed descriptor number.
    machine.wrapper(
        "git",
        &format!(
            "{policy_script}case \" $* \" in *' check-ref-format refs/heads/main '*)\n\
         if [ \"$owned\" = yes ] && rm '{}' 2>/dev/null; then\n\
           echo initial > '{}'; read line < '{}';\n\
         fi;; esac",
            initial_armed.display(),
            initial.display(),
            initial_release.display()
        ),
    );
    feature_hold.release();
    let waiting = watch.find("landing waits behind holder", |event| {
        event["event"] == "execution.waiting" && event["ticket"] == "Y-1"
    });
    assert_eq!(waiting["data"]["reason"], "capacity");
    let waiting_id = waiting["execution"].as_i64().unwrap();
    assert_eq!(
        project.rows(&format!(
            "SELECT kind, reserved FROM execution WHERE id = {waiting_id}"
        )),
        vec![json!({"kind": "landing", "reserved": 0})]
    );
    let holders = project.rows("SELECT execution.id FROM execution JOIN attempt ON attempt.id = execution.attempt WHERE attempt.ticket = 2 AND execution.reserved = 1");
    assert_eq!(holders.len(), 1);
    assert_eq!(
        project.rows("SELECT id FROM execution WHERE reserved = 1"),
        holders
    );
    assert!(
        probe.observations.lock().unwrap().is_empty(),
        "queue policy acted before capacity"
    );
    holder_hold.release();
    let stopped_holder = watch.find("holder ended", |event| {
        event["event"] == "attention.raised" && event["ticket"] == "Y-2"
    });
    assert_eq!(stopped_holder["data"]["kind"], "stopped");
    assert_eq!(stopped_holder["data"]["reason"], "unchanged");
    assert_eq!(watch.said(&initial), "initial\n");
    let landing = watch.find("landing started", |event| {
        event["event"] == "execution.started" && event["data"]["kind"] == "landing"
    });
    let initial_parent = landing["execution"].as_i64().unwrap();
    assert_eq!(initial_parent, waiting_id);
    assert_eq!(
        project.rows("SELECT id, kind, status FROM execution WHERE reserved = 1"),
        vec![json!({"id": initial_parent, "kind": "landing", "status": "running"})]
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["for-each-ref", "refs/yard/landings/"]
        ),
        ""
    );
    assert!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'gate' AND reason = 'landing'")
            .is_empty()
    );
    std::fs::write(&initial_release, "go\n").unwrap();
    assert_eq!(watch.said(&said), "first\n");
    let first = project.rows("SELECT parent FROM execution WHERE kind = 'gate' AND name = 'first'");
    assert_eq!(first.len(), 1);
    let parent = first[0]["parent"].as_i64().unwrap();
    assert_eq!(parent, initial_parent);
    // Arm after the first gate acts: the next diff is the second gate's
    // live approval read, rather than initial approval or final admission.
    std::fs::write(&armed, "").unwrap();
    machine.wrapper("git", &format!(
        "{policy_script}case \" $* \" in *' diff --name-only '*) if rm '{}' 2>/dev/null; then echo held > '{}'; read line < '{}'; fi;; esac",
        armed.display(), said.display(), release.display()));
    std::fs::write(&release, "go\n").unwrap();
    assert_eq!(watch.said(&said), "held\n");
    assert_eq!(
        project.rows("SELECT id, kind, status FROM execution WHERE reserved = 1"),
        vec![json!({"id": parent, "kind": "landing", "status": "running"})]
    );
    assert!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'gate' AND name = 'second'")
            .is_empty()
    );
    std::fs::write(&release, "go\n").unwrap();
    watch.event("landing.recorded", &[]);
    let reads = probe.observations.lock().unwrap();
    assert!(!reads.is_empty());
    for read in reads.iter() {
        assert_eq!(
            read,
            &json!({ "owned": true, "owners": [{ "id": parent, "kind": "landing" }] })
        );
    }
    drop(reads);
    assert_eq!(std::fs::read_to_string(markers).unwrap(), "second\n");
    assert_eq!(
        git(&project.canonical(), &["show", "main:feature.txt"]),
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

/// G4/G9: stop a capacity waiter while a real worker holds the sole slot.
/// Its row ends before release; the host model never sees that turn. A
/// deliberate start of the same ticket then reaches approval. Existing
/// G9 stops an acting worker and cannot expose the absent waiting handle.
/// Sabotage: stop only the in-memory handle, or reserve from a stale row
/// after stop ended it. Expected unchanged/stopped comes from the stop
/// contract; requests and the held turn come from the host fixture.
#[test]
fn stopping_a_capacity_waiter_prevents_its_turn_until_start() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g4-stop-waiter", move |request| {
        if request.opens() && request.prompt().contains("Holder") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![commit_file(
                    "holder.txt",
                    "holder\n",
                    "Holder",
                )])),
            );
        }
        act(
            request,
            vec![commit_file("waiter.txt", "waiter\n", "Waiter")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("")
            .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1")
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Holder"]);
    hold.wait_held();
    project.json(&["ticket", "new", "--title", "Waiter"]);
    let waiting = watch.event("execution.waiting", &[]);
    assert_eq!(waiting["ticket"], "Y-2");
    let execution = waiting["execution"].as_i64().unwrap();
    assert_eq!(
        project.json(&["attempt", "stop", "Y-2"])["stopping"],
        execution
    );
    let stopped = watch.until("waiting execution stopped", |event| {
        event["event"] == "attention.raised" && event["execution"] == execution
    });
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    assert_eq!(
        project.rows(&format!(
            "SELECT status, outcome, reserved FROM execution WHERE id = {execution}"
        )),
        vec![json!({"status": "ended", "outcome": "unchanged", "reserved": 0})]
    );
    hold.release();
    approval_of(&mut watch, "Y-1");
    assert!(
        !machine
            .model
            .requests()
            .iter()
            .any(|r| r.prompt().contains("Waiter"))
    );
    project.json(&["attempt", "start", "Y-2"]);
    approval_of(&mut watch, "Y-2");
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|r| r.opens() && r.prompt().contains("Waiter"))
            .count(),
        1
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT reserved FROM execution WHERE id = {execution}"
        )),
        vec![json!({"reserved": 0})]
    );
}

/// G4/G9: the allowed first ticket reaches approval, then the fixture holds
/// a real clone child after reservation but before any box handle exists.
/// Stop must retain that child's slot and prevent the second model turn.
/// A deliberate later start reaches approval in the same project.
/// Unlike the waiter and acting-worker scenarios, this stops preparation.
/// Sabotage: register the stop handle only in run_harness, ignore preparation
/// stop checks, or release the reservation at stop acceptance.
/// Expected identities come from fixture ticket order; the PID, reservation
/// and model requests are external observations of the held preparation.
#[test]
fn stopping_reserved_preparation_keeps_ownership_and_prevents_the_turn() {
    stopped_preparation(false);
}

/// G4/G5/G9: the same held child completes real Git work, then reports 73.
/// Sabotage: propagate preparation Err before honoring the acknowledged stop.
/// Success alone cannot expose that error handoff. The fixture supplies the
/// exits, the host observes the PID and the store retains ownership and detail.
#[test]
fn stopping_failed_preparation_keeps_ownership_and_prevents_the_turn() {
    stopped_preparation(true);
}

fn stopped_preparation(fail_after_git: bool) {
    let name = if fail_after_git {
        "g4-stop-failed-preparation"
    } else {
        "g4-stop-preparation"
    };
    let machine = Machine::new(name, |request| {
        let file = if request.prompt().contains("Control") {
            "control.txt"
        } else {
            "prepared.txt"
        };
        act(request, vec![commit_file(file, "feature\n", "Feature")])
    });
    let armed = machine.root.join("armed");
    let said_path = machine.root.join("said");
    let release = machine.root.join("release");
    fifo(&said_path);
    fifo(&release);
    let git_exit = machine.root.join("native-git-exit");
    let after_git = if fail_after_git {
        // The wrapper removes only the clone it just created before refusing.
        // Later operator starts need no private-state repair by the test.
        format!(
            "\"$REAL\" \"$@\"; code=$?; echo \"$code\" > '{}'; [ \"$code\" -eq 0 ] || exit \"$code\"; for clone_destination in \"$@\"; do :; done; rm -rf -- \"$clone_destination\" || exit $?; exit 73",
            git_exit.display()
        )
    } else {
        String::new()
    };
    machine.wrapper("git", &format!(
        "if [ -e '{}' ]; then case \" $* \" in *' clone '*) echo $$ > '{}'; read line < '{}'; {};; esac; fi",
        armed.display(), said_path.display(), release.display(), after_git));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("")
            .replace(
                "max_lanes = 2",
                if fail_after_git {
                    "max_lanes = 4\nmax_executions = 1"
                } else {
                    "max_lanes = 2\nmax_executions = 1"
                },
            )
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Control"]);
    approval_of(&mut watch, "Y-1");
    std::fs::write(&armed, "").unwrap();
    project.json(&["ticket", "new", "--title", "Prepared"]);
    let pid = watch.said(&said_path).trim().to_string();
    let held = vec![json!({"id": 2, "status": "running", "reserved": 1, "handle": null})];
    assert_eq!(
        project.rows("SELECT id, status, reserved, handle FROM execution WHERE id = 2"),
        held
    );
    assert_eq!(project.json(&["attempt", "stop", "Y-2"])["stopping"], 2);
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        project.rows("SELECT id, status, reserved, handle FROM execution WHERE id = 2"),
        held
    );
    std::fs::remove_file(&armed).unwrap();
    std::fs::write(&release, "go\n").unwrap();
    let stopped = watch.attention();
    assert_eq!(stopped["execution"], 2);
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    assert_eq!(
        project.rows("SELECT status, outcome, handle FROM execution WHERE id = 2"),
        vec![json!({"status": "ended", "outcome": "unchanged", "handle": null})]
    );
    assert!(
        !machine
            .model
            .requests()
            .iter()
            .any(|r| r.prompt().contains("Prepared"))
    );
    if fail_after_git {
        assert_eq!(std::fs::read_to_string(&git_exit).unwrap().trim(), "0");
        assert_eq!(
            project.rows(
                "SELECT instr(detail, '73') > 0 AS reported_exit FROM execution WHERE id = 2"
            ),
            vec![json!({"reported_exit": 1})]
        );
    }
    project.json(&["attempt", "start", "Y-2"]);
    approval_of(&mut watch, "Y-2");
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|r| r.opens() && r.prompt().contains("Prepared"))
            .count(),
        1
    );
    assert_eq!(
        project.rows("SELECT reserved FROM execution WHERE id = 2"),
        vec![json!({"reserved": 0})]
    );
    if fail_after_git {
        // The same child failure without an acknowledged stop remains error.
        std::fs::write(&armed, "").unwrap();
        project.json(&["ticket", "new", "--title", "Failure"]);
        watch.said(&said_path);
        std::fs::remove_file(&armed).unwrap();
        std::fs::write(&release, "go\n").unwrap();
        let failed = watch.attention();
        assert_eq!(failed["execution"], 4);
        assert_eq!(failed["data"]["kind"], "red");
        assert_eq!(failed["data"]["reason"], "error");
        assert_eq!(std::fs::read_to_string(&git_exit).unwrap().trim(), "0");
        assert_eq!(
            project.rows(
                "SELECT status, outcome, reserved, instr(detail, '73') > 0 AS reported_exit
                 FROM execution WHERE id = 4"
            ),
            vec![json!({"status": "ended", "outcome": "error", "reserved": 0, "reported_exit": 1})]
        );
        assert!(
            !machine
                .model
                .requests()
                .iter()
                .any(|r| r.prompt().contains("Failure"))
        );
        project.json(&["attempt", "start", "Y-3"]);
        approval_of(&mut watch, "Y-3");
        stopped_box_preparation(&machine, &project, &mut watch);
    }
}

/// G4/G5/G9: native mount refusal and a lost ready reply are distinct startup
/// failures. The latter leaves a real box and inherited host owner alive.
/// Sabotage: propagate up_worker Err before honoring stop, discard a successful
/// LiveBox at a stop check, or release a failed startup on box absence alone.
/// Native Pinfold supplies diagnostics and inventory; the fixture holds stdin,
/// observes the inherited lock, and supplies ticket/execution order.
fn stopped_box_preparation(machine: &Machine, project: &Project, watch: &mut Watch) {
    let armed = machine.root.join("startup-armed");
    let said = machine.root.join("startup-said");
    let release = machine.root.join("startup-release");
    let native_failure = machine.root.join("startup-native-failure");
    fifo(&said);
    fifo(&release);
    // Preserve the native rejection as stderr, then exit with its real code.
    // Yard observes EOF only after this native validation child has finished.
    machine.wrapper("pinfold", &format!(
        "if [ -e '{}' ] && [ \"$1 $2\" = 'box up' ]; then echo $$ > '{}'; read line < '{}'; \"$REAL\" \"$@\" > '{}'; code=$?; cat '{}' >&2; exit \"$code\"; fi",
        armed.display(), said.display(), release.display(), native_failure.display(), native_failure.display()));
    let clone = project.path.join(".yard/local/attempts/4/clone");
    let parked = project.path.join(".yard/local/attempts/4/parked-clone");
    for (execution, stop) in [(6, true), (7, false)] {
        std::fs::write(&armed, "").unwrap();
        if execution == 6 {
            project.json(&["ticket", "new", "--title", "Startup"]);
        } else {
            project.json(&["attempt", "start", "Y-4"]);
        }
        let pid = watch.said(&said).trim().to_string();
        let held = vec![json!({"status": "running", "reserved": 1, "handle": null})];
        assert_eq!(
            project.rows(&format!(
                "SELECT status, reserved, handle FROM execution WHERE id = {execution}"
            )),
            held
        );
        if stop {
            assert_eq!(
                project.json(&["attempt", "stop", "Y-4"])["stopping"],
                execution
            );
        }
        assert!(
            std::process::Command::new("kill")
                .args(["-0", &pid])
                .status()
                .unwrap()
                .success()
        );
        assert_eq!(
            project.rows(&format!(
                "SELECT status, reserved, handle FROM execution WHERE id = {execution}"
            )),
            held
        );
        // Refuse only this fixture's mount, after the real child is held.
        std::fs::rename(&clone, &parked).unwrap();
        std::fs::remove_file(&armed).unwrap();
        std::fs::write(&release, "go\n").unwrap();
        let attention = watch.attention();
        std::fs::rename(&parked, &clone).unwrap();
        assert_eq!(attention["execution"], execution);
        assert_eq!(
            attention["data"]["kind"],
            if stop { "stopped" } else { "red" }
        );
        assert_eq!(
            attention["data"]["reason"],
            if stop { "unchanged" } else { "error" }
        );
        let native_reply = std::fs::read_to_string(&native_failure).unwrap();
        let native: Value = serde_json::from_str(native_reply.lines().next().unwrap()).unwrap();
        assert!(matches!(
            native["event"].as_str(),
            Some("refused" | "failed")
        ));
        let rows = project.rows(&format!(
            "SELECT status, outcome, handle, detail FROM execution WHERE id = {execution}"
        ));
        assert_eq!(rows[0]["status"], "ended");
        assert_eq!(rows[0]["outcome"], if stop { "unchanged" } else { "error" });
        assert_eq!(rows[0]["handle"], Value::Null);
        // Native Pinfold's diagnostic is the external expected value.
        assert!(
            rows[0]["detail"]
                .as_str()
                .unwrap()
                .contains(native_reply.trim())
        );
        if stop {
            watch.until("native startup refusal checked release", |event| {
                event["event"] == "execution.released" && event["execution"] == execution
            });
        }
        assert!(
            !machine
                .model
                .requests()
                .iter()
                .any(|r| r.prompt().contains("Startup"))
        );
    }

    // Lose the actual ready reply after native startup. Keep native stdin
    // open with a FIFO, so the box and its controller survive the failed
    // caller. No protocol frame is invented.
    let native_output = machine.root.join("startup-ready-ack");
    let ready = machine.root.join("startup-ready");
    let owner_release = machine.root.join("startup-owner-release");
    let down = machine.root.join("startup-down");
    let host_release = machine.root.join("startup-host-release");
    let listing_armed = machine.root.join("startup-listing-armed");
    let listing_said = machine.root.join("startup-listing-said");
    let listing_release = machine.root.join("startup-listing-release");
    let spec_path = machine.root.join("startup-spec");
    for path in [
        &native_output,
        &ready,
        &owner_release,
        &down,
        &host_release,
        &listing_said,
        &listing_release,
    ] {
        fifo(path);
    }
    // Scoped shell redirections save Yard's pipes on extra descriptors.
    // Surviving pipeline shells must redirect permanently before they hold.
    machine.wrapper(
        "pinfold",
        &format!(
            r#"
if [ "$1 $2" = 'box list' ] && [ -e '{listing_armed}' ]; then
    "$REAL" "$@"; code=$?
    echo "$code" > '{listing_said}'; read line < '{listing_release}'
    exit "$code"
fi
if [ "$1 $2" = 'box up' ] && [ -e '{armed}' ]; then
    read -r spec
    printf '%s\n' "$spec" > '{spec_path}'
    echo $$ > '{said}'; read line < '{release}'
    ( exec 2> '{feed_stderr}'
      printf '%s\n' "$spec"; read line < '{owner_release}' ) |
        "$REAL" "$@" 2> '{stderr}' |
        ( exec > '{remaining}' 2> '{drain_stderr}'
          read -r line; printf '%s\n' "$line" > '{ready}'; echo captured > '{native_output}'
          cat; echo down > '{down}'; read line < '{host_release}' ) &
    read line < '{native_output}'
    exit 73
fi
"#,
            listing_armed = listing_armed.display(),
            listing_said = listing_said.display(),
            listing_release = listing_release.display(),
            armed = armed.display(),
            spec_path = spec_path.display(),
            said = said.display(),
            release = release.display(),
            owner_release = owner_release.display(),
            native_output = native_output.display(),
            stderr = machine.root.join("startup-native-stderr").display(),
            feed_stderr = machine.root.join("startup-feed-stderr").display(),
            ready = ready.display(),
            remaining = machine.root.join("startup-remaining").display(),
            down = down.display(),
            host_release = host_release.display(),
            drain_stderr = machine.root.join("startup-drain-stderr").display()
        ),
    );
    std::fs::write(&armed, "").unwrap();
    project.json(&["attempt", "start", "Y-4"]);
    let pid = watch.said(&said).trim().to_string();
    assert_eq!(project.json(&["attempt", "stop", "Y-4"])["stopping"], 8);
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        project.rows("SELECT status, reserved, handle FROM execution WHERE id = 8"),
        vec![json!({"status": "running", "reserved": 1, "handle": null})]
    );
    std::fs::remove_file(&armed).unwrap();
    std::fs::write(&release, "go\n").unwrap();
    let native: Value = serde_json::from_str(&watch.said(&ready)).unwrap();
    assert_eq!(native["event"], "ready");
    let spec: Value = serde_json::from_slice(&std::fs::read(&spec_path).unwrap()).unwrap();
    let label = format!(
        "dev.yard.project={}",
        spec["labels"]["dev.yard.project"].as_str().unwrap()
    );
    let box_name = spec["name"].as_str().unwrap();
    let boxes = machine.boxes(&label);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0]["name"], box_name);
    let lock =
        std::fs::File::open(project.path.join(".yard/local/attempts/4/effects/8/lock")).unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    let stopped = watch.attention();
    assert_eq!(stopped["execution"], 8);
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    let ownership = watch.attention();
    assert_eq!(ownership["execution"], 8);
    assert_eq!(ownership["data"]["kind"], "red");
    assert_eq!(ownership["data"]["reason"], "ownership");
    assert_eq!(
        project.rows("SELECT status, outcome, reserved, handle, length(detail) > 0 AS diagnostic FROM execution WHERE id = 8"),
        vec![json!({"status": "ended", "outcome": "unchanged", "reserved": 1, "handle": null, "diagnostic": 1})]
    );
    assert!(
        !machine
            .model
            .requests()
            .iter()
            .any(|r| r.prompt().contains("Startup"))
    );

    // Box absence alone is insufficient: the drain child retains the same
    // inherited lock after native down. Two real listing replies prove that
    // a checked release has actually run with box absence and that host owner.
    std::fs::write(&owner_release, "go\n").unwrap();
    assert_eq!(watch.said(&down).trim(), "down");
    assert!(machine.boxes(&label).is_empty());
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    std::fs::write(&listing_armed, "").unwrap();
    assert_eq!(watch.said(&listing_said).trim(), "0");
    std::fs::write(&listing_release, "go\n").unwrap();
    assert_eq!(watch.said(&listing_said).trim(), "0");
    assert_eq!(
        project.rows("SELECT reserved FROM execution WHERE id = 8"),
        vec![json!({"reserved": 1})]
    );
    assert_eq!(
        project.rows("SELECT id FROM execution WHERE attempt = 4 ORDER BY id"),
        vec![json!({"id": 6}), json!({"id": 7}), json!({"id": 8})]
    );
    std::fs::write(&host_release, "go\n").unwrap();
    std::fs::remove_file(&listing_armed).unwrap();
    std::fs::write(&listing_release, "go\n").unwrap();
    watch.until("failed startup checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == 8
    });
    assert!(machine.boxes(&label).is_empty());
    lock.try_lock().unwrap();
    lock.unlock().unwrap();
    assert_eq!(
        project.rows("SELECT reserved FROM execution WHERE id = 8"),
        vec![json!({"reserved": 0})]
    );
    // Unchanged executions retain both operator exits. Answer the exact
    // ownership item after proof; the stopped item still prevents startup.
    project.json(&[
        "attempt",
        "start",
        "Y-4",
        "--attention",
        &ownership["attention"].as_i64().unwrap().to_string(),
    ]);
    // A successful reply must keep its returned owner on the existing
    // teardown path. Hold real stdin EOF after the pending stop is consumed.
    machine.wrapper(
        "pinfold",
        &format!(
            r#"
if [ "$1 $2" = 'box up' ] && [ -e '{armed}' ]; then
    read -r spec
    echo $$ > '{said}'; read line < '{release}'
    {{ printf '%s\n' "$spec"; cat; echo down > '{down}'; read line < '{host_release}'; }} |
        "$REAL" "$@"
    exit $?
fi
"#,
            armed = armed.display(),
            said = said.display(),
            release = release.display(),
            down = down.display(),
            host_release = host_release.display()
        ),
    );
    std::fs::write(&armed, "").unwrap();
    project.json(&["attempt", "start", "Y-4"]);
    watch.said(&said);
    assert_eq!(project.json(&["attempt", "stop", "Y-4"])["stopping"], 9);
    std::fs::remove_file(&armed).unwrap();
    std::fs::write(&release, "go\n").unwrap();
    assert_eq!(watch.said(&down).trim(), "down");
    let boxes = machine.boxes(&label);
    assert_eq!(boxes.len(), 1);
    assert_eq!(
        project.rows("SELECT status, reserved, handle FROM execution WHERE id = 9"),
        vec![json!({"status": "running", "reserved": 1, "handle": boxes[0]["name"]})]
    );
    let lock =
        std::fs::File::open(project.path.join(".yard/local/attempts/4/effects/9/lock")).unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(
        !machine
            .model
            .requests()
            .iter()
            .any(|r| r.prompt().contains("Startup"))
    );
    std::fs::write(&host_release, "go\n").unwrap();
    let stopped = watch.attention();
    assert_eq!(stopped["execution"], 9);
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    watch.until("successful startup checked teardown", |event| {
        event["event"] == "execution.released" && event["execution"] == 9
    });
    assert!(machine.boxes(&label).is_empty());
    lock.try_lock().unwrap();
    lock.unlock().unwrap();
    project.json(&["attempt", "start", "Y-4"]);
    approval_of(watch, "Y-4");
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|r| r.opens() && r.prompt().contains("Startup"))
            .count(),
        1
    );
}
