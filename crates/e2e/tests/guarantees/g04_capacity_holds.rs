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
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);

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
    repair.release();
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

/// G4/G5: hold the first target read inheriting the landing's host lock. Its
/// reservation precedes any prepared ref or child gate. Then, after an allowed
/// first host gate, hold the auto-approval protected-path
/// Git read before the second gate. The landing owns the only reservation;
/// release permits the second gate and final landing at limit one.
/// Expected gate names and landed bytes come from the fixture, and the parent
/// identity from the externally observed first child row.
/// Sabotage: omit the landing effect around gate-loop admission; the held
/// Git child has no reserved owner. Keep the parent slot during the child;
/// the second gate cannot complete at limit one.
/// Move the initial load outside preparation: the first owned target read
/// comes after the prepared ref, failing the initial-boundary observer.
#[test]
fn landing_gate_policy_reads_hold_the_shared_execution_slot() {
    let machine = Machine::new("g4-gate-policy-slot", |request| {
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
    let project = Project::new(
        &machine,
        "p",
        &config(&gates)
            .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1")
            .replace("approve = \"manual\"", "approve = \"auto\"")
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    std::fs::write(&initial_armed, "").unwrap();
    // Scheduler/config reads do not inherit the landing's ownership lock.
    // Compare actual open file identities, not a fixed descriptor number.
    machine.wrapper(
        "git",
        &format!(
            "case \" $* \" in *' check-ref-format refs/heads/main '*)\n\
         if [ -e '{}' ]; then\n\
           for owner in '{}'/*/lock; do for descriptor in /dev/fd/*; do\n\
             if [ \"$descriptor\" -ef \"$owner\" ] && rm '{}' 2>/dev/null; then\n\
               echo initial > '{}'; read line < '{}';\n\
             fi\n\
           done; done\n\
         fi;; esac",
            initial_armed.display(),
            project.path.join(".yard/local/landings").display(),
            initial_armed.display(),
            initial.display(),
            initial_release.display()
        ),
    );
    project.json(&["ticket", "new", "--title", "Feature"]);
    assert_eq!(watch.said(&initial), "initial\n");
    let landing = watch.find("landing started", |event| {
        event["event"] == "execution.started" && event["data"]["kind"] == "landing"
    });
    let initial_parent = landing["execution"].as_i64().unwrap();
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
        "case \" $* \" in *' diff --name-only '*) if rm '{}' 2>/dev/null; then echo held > '{}'; read line < '{}'; fi;; esac",
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
    let machine = Machine::new("g4-stop-preparation", |request| {
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
}
