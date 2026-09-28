//! G5: Intent precedes effect, and a restart loses only the turn.

use e2e::*;
use serde_json::json;
use std::io::Write;
use std::path::Path;

fn mkfifo(path: &Path) {
    assert!(
        std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .unwrap()
            .success()
    );
}

/// A pinfold wrapper that, while `armed` exists, holds each `box <verb>`
/// call: it says `held` on one FIFO and waits for a line on another.
fn hold_pinfold(machine: &Machine, verb: &str) -> (std::path::PathBuf, Held) {
    let armed = machine.root.join("armed");
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    mkfifo(&said);
    mkfifo(&release);
    machine.wrapper(
        "pinfold",
        &format!(
            "if [ -e '{}' ] && [ \"$1 $2\" = 'box {verb}' ]; then echo held > '{}'; read line < '{}'; fi",
            armed.display(),
            said.display(),
            release.display()
        ),
    );
    (armed, Held { said, release })
}

struct Held {
    said: std::path::PathBuf,
    release: std::path::PathBuf,
}

impl Held {
    /// Block until the wrapper holds a call; `watch` fails it early.
    fn wait(&self, watch: Option<&mut Watch>) {
        let text = match watch {
            Some(watch) => watch.said(&self.said),
            None => said(&self.said),
        };
        assert_eq!(text, "held\n");
    }

    fn release(&self) {
        std::fs::write(&self.release, "go\n").unwrap();
    }
}

/// Intent is ordered before effect: an execution's audit event precedes
/// its box's creation. The worker's `box up` is held; while it is, the
/// execution's row and event exist.
///
/// Sabotage: make `admit::admit` record the first execution after its box
/// is up; the held `box up` finds no row.
#[test]
fn an_executions_event_precedes_its_box_and_request() {
    let machine = Machine::new("g5-intent", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    let (armed, held) = hold_pinfold(&machine, "up");
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    std::fs::write(&armed, "").unwrap();
    project.json(&["ticket", "new", "--title", "Add feature"]);
    held.wait(Some(&mut watch));

    let started =
        project.rows("SELECT execution, data FROM audit WHERE event = 'execution.started'");
    assert_eq!(started.len(), 1, "{started:?}");
    let execution = started[0]["execution"].as_i64().unwrap();
    assert_eq!(
        project.rows(&format!(
            "SELECT kind, status, handle FROM execution WHERE id = {execution}"
        )),
        vec![json!({ "kind": "implementation", "status": "running", "handle": null })]
    );

    // Let the held call go, so no wrapper outlives the test.
    std::fs::remove_file(&armed).unwrap();
    held.release();
}

/// The daemon killed mid-execution: on restart the execution is
/// `interrupted`, no second box exists, the tree is kept, and `start`
/// continues. The worker wrote a file and is held on its next request when
/// the daemon dies.
///
/// Sabotage: make `reconcile::project` leave running implementations
/// running; nothing raises `stopped`, and `start` is refused.
#[test]
fn a_daemon_killed_mid_execution_interrupts_it_and_start_continues() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g5-kill", move |request| {
        if request.last_user().contains("Continue") {
            return act(
                request,
                vec![bash(
                    "cd /workspace && git add -A && git commit -q -m 'Add feature' && echo committed",
                )],
            );
        }
        match request.tool_results().len() {
            0 => Reply::Tools(vec![bash(
                "cd /workspace && printf 'feature\\n' > feature.txt && echo written",
            )]),
            _ => Reply::Hold(held.clone(), Box::new(Reply::Text("never".into()))),
        }
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    project.json(&["ticket", "new", "--title", "Add feature"]);
    hold.wait_held();
    let clone = project.path.join(".yard/local/attempts/1/clone");
    // The project's label, as pinfold reports it on the held worker's box.
    let handle = project.rows("SELECT handle FROM execution WHERE id = 1")[0]["handle"].clone();
    let worker = machine
        .boxes("dev.yard.project")
        .into_iter()
        .find(|listed| listed["name"] == handle)
        .expect("the held worker has a box");
    let label = format!(
        "dev.yard.project={}",
        worker["labels"]["dev.yard.project"].as_str().unwrap()
    );
    assert_eq!(machine.boxes(&label).len(), 1);
    machine.kill();

    // Before restart: the execution is running and the tree holds the file.
    assert_eq!(
        project.rows("SELECT id, status FROM execution"),
        vec![json!({ "id": 1, "status": "running" })]
    );
    assert_eq!(
        std::fs::read_to_string(clone.join("feature.txt")).unwrap(),
        "feature\n"
    );

    machine.start();
    assert_eq!(
        project.rows("SELECT id, status, outcome FROM execution"),
        vec![json!({ "id": 1, "status": "ended", "outcome": "interrupted" })]
    );
    let status = project.json(&["status"]);
    let items: Vec<_> = status["attention"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| (item["kind"].clone(), item["reason"].clone()))
        .collect();
    assert_eq!(items, vec![(json!("stopped"), json!("interrupted"))]);
    assert_eq!(
        machine.boxes(&label),
        Vec::<serde_json::Value>::new(),
        "the project has a box after restart"
    );

    let mut watch = project.watch(0);
    project.json(&["attempt", "start", "Y-1"]);
    let candidate = watch.event("attempt.candidate", &[]);
    let head = candidate["data"]["head"].as_str().unwrap();
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:feature.txt")]
        ),
        "feature\n"
    );
    let executions =
        project.rows("SELECT reason FROM execution WHERE kind = 'implementation' ORDER BY id");
    assert_eq!(
        executions,
        vec![json!({ "reason": "first" }), json!({ "reason": "retry" })]
    );
    hold.release();
}

/// The daemon killed while a host landing gate runs: the gate's group is
/// gone after restart and the landing re-queues. The gate says its group on
/// a FIFO and waits for a line on another. Control: the re-queued landing
/// runs the gate again and lands once it is released.
///
/// Sabotage: make `reconcile::kill_group` skip the kill; the gate's group
/// outlives the restart and its lock raises `red`.
#[test]
fn a_daemon_killed_during_a_host_landing_gate_leaves_no_group() {
    let machine = Machine::new("g5-host-gate", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    let said = machine.root.join("said");
    let hold = machine.root.join("hold");
    mkfifo(&said);
    mkfifo(&hold);
    // Held open for writing, so a line released stays until a gate reads it.
    let mut release = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&hold)
        .unwrap();
    let gate = format!(
        "[gates.held]\ncommand = \"echo $$ > {}; read line < {}\"\nruns_in = \"host\"\n",
        said.display(),
        hold.display()
    );
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(&gate)
            .replace("review = [\"correctness\"]", "review = \"none\"")
            .replace("approve = \"manual\"", "approve = \"auto\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let group = watch.said(&said).trim().to_string();
    drop(watch);
    machine.kill();

    let alive = |group: &str| {
        std::process::Command::new("kill")
            .args(["-0", "--", &format!("-{group}")])
            .status()
            .unwrap()
            .success()
    };
    assert!(alive(&group), "the gate's group died with the daemon");
    assert_eq!(
        project.rows("SELECT status FROM execution WHERE kind = 'landing'"),
        vec![json!({ "status": "running" })]
    );

    machine.start();
    assert!(!alive(&group), "the gate's group survived the restart");
    assert_eq!(
        project.rows("SELECT outcome FROM execution WHERE kind = 'landing' ORDER BY id")[0],
        json!({ "outcome": "interrupted" })
    );
    let status = project.json(&["status"]);
    assert_eq!(status["attention"], json!([]), "{status}");

    let mut watch = project.watch(0);
    // The re-queued landing's gate runs again.
    watch.said(&said);
    release.write_all(b"go\n").unwrap();
    watch.event("landing.recorded", &[]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
}

/// No command is answered before reconciliation has committed. The
/// restart's reconciliation is held at its first pinfold call, with an
/// interrupted execution still to reconcile: the CLI finds no daemon.
/// Released, the first answer already shows the execution interrupted.
///
/// Sabotage: make `daemon::serve` bind the socket before reconciling;
/// `status` answers while the execution still reads `running`.
#[test]
fn no_command_is_answered_before_reconciliation() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g5-serve", move |_| {
        Reply::Hold(held.clone(), Box::new(Reply::Text("never".into())))
    });
    let (armed, pinfold) = hold_pinfold(&machine, "list");
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    project.json(&["ticket", "new", "--title", "Hold"]);
    hold.wait_held();
    machine.kill();

    std::fs::write(&armed, "").unwrap();
    let lines = machine.spawn();
    pinfold.wait(None);
    let refused = project.refused(&["status"]);
    assert_eq!(refused["code"], "daemon", "{refused}");
    assert_eq!(
        project.rows("SELECT status FROM execution"),
        vec![json!({ "status": "running" })]
    );

    std::fs::remove_file(&armed).unwrap();
    pinfold.release();
    Machine::serving(&lines);
    let status = project.json(&["status"]);
    assert_eq!(status["attention"][0]["reason"], "interrupted", "{status}");
    hold.release();
}
