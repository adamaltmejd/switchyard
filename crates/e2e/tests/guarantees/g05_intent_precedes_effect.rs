//! G5: Intent precedes effect, and a restart loses only the turn.

use e2e::*;
use serde_json::json;
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

/// The daemon killed mid-execution: on restart the execution is
/// `interrupted`, no second box exists, the tree is kept, and `start`
/// continues.
/// The worker wrote a file and is held on its next request when
/// the daemon dies.
///
/// Sabotage: make `reconcile::project` leave running implementations
/// running; the execution is not `interrupted`, and `start` is refused.
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
    project.ok("ticket.new", json!({"title": "Add feature"}));
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
    assert_eq!(
        machine.boxes(&label),
        Vec::<serde_json::Value>::new(),
        "the project has a box after restart"
    );

    let mut watch = project.watch(0);
    project.ok("attempt.start", json!({"ticket": "Y-1"}));
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
/// a FIFO and waits for a line on another. The re-queued landing runs the
/// gate again.
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
    // Held open for writing, so a gate's read blocks rather than its open,
    // and ends when the test drops this.
    let _writer = std::fs::OpenOptions::new()
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
    project.ok("ticket.new", json!({"title": "Add feature"}));
    let group = watch.said(&said).trim().to_string();
    drop(watch);
    machine.kill();

    let alive = |group: &str| {
        std::process::Command::new("kill")
            .args(["-0", "--", &format!("-{group}")])
            .stderr(std::process::Stdio::null())
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
    let status = project.ok("status", json!({}));
    assert_eq!(status["attention"], json!([]), "{status}");

    let mut watch = project.watch(0);
    // The re-queued landing's gate runs again.
    watch.said(&said);
}
