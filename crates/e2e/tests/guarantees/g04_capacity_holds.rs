//! G4: Capacity holds.

use e2e::*;
use serde_json::Value;

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
