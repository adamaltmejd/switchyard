//! G4: Capacity holds.

use e2e::*;
use serde_json::{Value, json};

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
/// `admit::attempt_start`; both starts of one ticket are admitted. Make
/// `admit::free_lanes` ignore the lanes held in the project; both tickets
/// are admitted.
#[test]
fn racing_starts_admit_one_attempt() {
    let hold = Latch::new();
    let held = hold.clone();
    let seen = std::sync::Mutex::new(std::collections::HashSet::new());
    let machine = Machine::new("g4-race", move |request| {
        if seen.lock().unwrap().insert(request.prompt()) {
            Reply::Text("planned".into())
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

    let results = race(&project, &["Y-1", "Y-1"]);
    let refused: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();
    assert_eq!(refused.len(), 1, "{results:?}");
    assert_eq!(refused[0]["code"], "refused");
    assert_eq!(live(&project, 1), 1);

    let results = race(&project, &["Y-2", "Y-3"]);
    let refused: Vec<_> = results.iter().filter_map(|r| r.as_ref().err()).collect();
    assert_eq!(refused.len(), 1, "{results:?}");
    assert_eq!(refused[0]["data"]["reason"], "capacity");
    assert_eq!(live(&project, 2) + live(&project, 3), 1);
    assert_eq!(live(&project, 1), 1);
    hold.release();
}

/// Two registered projects: `YARD_MAX_LANES` bounds attempts across both,
/// and each keeps its own store. Control: abandoning the first project's
/// attempt frees the lane and the second's ticket is admitted.
///
/// Sabotage: make `admit::free_lanes` count only the project's own lanes;
/// the scheduler admits the second project's ticket, and its start is
/// refused as live rather than `capacity`.
#[test]
fn machine_lanes_bound_attempts_across_projects() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g4-machine", move |request| {
        if request.prompt().contains("Hold") {
            return Reply::Hold(held.clone(), Box::new(Reply::Text("done".into())));
        }
        Reply::Text("done".into())
    });
    let env = std::fs::read_to_string(machine.operator_env()).unwrap();
    machine.write_operator_env(&format!("{env}YARD_MAX_LANES=1\n"));
    machine.start();
    let first = Project::new(&machine, "a", &config(""));
    let second = Project::new(&machine, "b", &config(""));

    first.json(&["ticket", "new", "--title", "Hold"]);
    hold.wait_held();
    second.json(&["ticket", "new", "--title", "Waits"]);
    let refused = second.refused(&["attempt", "start", "Y-1"]);
    assert_eq!(refused["data"]["reason"], "capacity", "{refused}");
    assert_eq!(live(&second, 1), 0);

    let mut watch = second.watch(0);
    // Parked first, so the freed lane cannot go back to it.
    first.json(&["ticket", "park", "Y-1"]);
    first.json(&["attempt", "abandon", "Y-1"]);
    watch.event("attempt.admitted", &[]);
    hold.release();

    let titles = |project: &Project| -> Vec<Value> {
        project
            .rows("SELECT title FROM ticket")
            .into_iter()
            .map(|row| row["title"].clone())
            .collect()
    };
    assert_eq!(titles(&first), ["Hold"]);
    assert_eq!(titles(&second), ["Waits"]);
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
#[test]
fn a_returning_repair_takes_a_freed_lane_before_a_new_ticket() {
    let repair = Latch::new();
    let held = repair.clone();
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
        act(
            request,
            vec![commit_file("holder.txt", "holder\n", "Holder")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &single_lane("[gates.land]\ncommand = \"test ! -f bad.txt\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Repair"]);
    let head = approval_of(&mut watch, "Y-1");
    project.json(&["ticket", "new", "--title", "Holder"]);
    project.json(&["attempt", "approve", "Y-1", "--head", &head]);

    // The holder takes the lane as the approved candidate starts landing.
    watch.until("Y-2 admitted", |event| {
        event["event"] == "attempt.admitted" && event["ticket"] == "Y-2"
    });
    // The landing gate fails; Y-1 is returned for repair and waits.
    watch.until("Y-1's landing red", |event| {
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
    assert_eq!(live(&project, 3), 0);
    repair.release();
}
