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

/// A pinfold wrapper that, while `armed` exists, holds each `box <verb>`
/// call: it says `held` on one FIFO and waits for a line on another.
fn hold_pinfold(machine: &Machine, verb: &str) -> (std::path::PathBuf, Held) {
    let armed = machine.root.join("armed");
    let said = machine.root.join(format!("said-{verb}"));
    let release = machine.root.join(format!("release-{verb}"));
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

/// The daemon killed mid-execution: on restart the execution is
/// `interrupted`, no second box exists, the tree is kept, and `start`
/// continues. Its row and event exist while box up is held before invocation;
/// no command is answered while startup reconciliation is held.
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
    let (armed, held_up) = hold_pinfold(&machine, "up");
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut intent_watch = project.watch(0);
    std::fs::write(&armed, "").unwrap();
    project.json(&["ticket", "new", "--title", "Add feature"]);
    // Before invoking box up, the row and its event are already committed.
    // Sabotage: record the first execution only after box up returns.
    held_up.wait(Some(&mut intent_watch));
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

    std::fs::remove_file(&armed).unwrap();
    held_up.release();
    drop(intent_watch);
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

    // No command can observe the unreconciled running turn.
    // Sabotage: bind the daemon socket before reconciliation commits.
    let (armed, held_list) = hold_pinfold(&machine, "list");
    std::fs::write(&armed, "").unwrap();
    let lines = machine.spawn();
    held_list.wait(None);
    let refused = project.refused(&["status"]);
    assert_eq!(refused["code"], "daemon", "{refused}");
    std::fs::remove_file(&armed).unwrap();
    held_list.release();
    Machine::serving(&lines);
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
    project.json(&["ticket", "new", "--title", "Add feature"]);
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
    let status = project.json(&["status"]);
    assert_eq!(status["attention"], json!([]), "{status}");

    let mut watch = project.watch(0);
    // The re-queued landing's gate runs again.
    watch.said(&said);
}
