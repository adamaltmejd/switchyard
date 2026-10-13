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
