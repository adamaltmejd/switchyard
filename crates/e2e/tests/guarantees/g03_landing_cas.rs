//! G3: Landing is compare-and-swap and proved.

use e2e::*;
use serde_json::json;

fn worker() -> impl Fn(&ModelRequest) -> Reply + Send + Sync + 'static {
    |request| match request.turn() {
        0 => Reply::Tools(vec![commit_file("feature.txt", "feature\n", "Add feature")]),
        _ => Reply::Text("done".into()),
    }
}

/// A candidate awaiting approval, with canonical moved past its base by the
/// operator, so its landing is a real merge. Returns the candidate head.
fn candidate_behind_target(project: &Project, watch: &mut Watch) -> String {
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let raised = watch.until("approval raised", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });
    project.write("other.txt", "other\n");
    project.git(&["add", "other.txt"]);
    project.git(&["commit", "--quiet", "-m", "Operator change"]);
    project.json(&["sync"]);
    raised["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string()
}

fn unreviewed() -> String {
    config("").replace("review = [\"correctness\"]", "review = \"none\"")
}

/// The daemon killed while its `update-ref` is held: restart keeps the
/// intent and raises `red`; once the command is released,
/// `start` records the landing once.
///
/// Sabotage: make `reconcile::decide_intent` skip its `lock_free` check; the
/// restart reads canonical at the old head and retires the intent, so the
/// released `update-ref` lands a candidate the store never records.
#[test]
fn a_landing_whose_update_ref_is_held_stays_undecided_until_released() {
    let machine = Machine::new("g3-held", worker());
    let armed = machine.root.join("armed");
    let release = machine.root.join("release");
    let done = machine.root.join("done");
    for fifo in [&release, &done] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    machine.wrapper("git", &format!(
        "if [ -e '{}' ]; then case \" $* \" in *' update-ref --no-deref refs/heads/main '*)\n\
         kill -9 $PPID; read line < '{}'; \"$REAL\" \"$@\"; status=$?; echo done > '{}'; exit $status;;\n esac; fi",
        armed.display(),
        release.display(),
        done.display()
    ));
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed());
    let mut watch = project.watch(0);
    let head = candidate_behind_target(&project, &mut watch);
    let target = project.canonical_head();

    std::fs::write(&armed, "").unwrap();
    project.json(&["attempt", "approve", "Y-1", "--head", &head]);
    machine.wait_dead();
    std::fs::remove_file(&armed).unwrap();

    // Before restart: nothing moved, the intent is open.
    assert_eq!(project.canonical_head(), target);
    let intent = &project
        .rows("SELECT id, intent_state, intent_merged FROM execution WHERE kind = 'landing'")[0];
    assert_eq!(intent["intent_state"], "open");
    let merged = intent["intent_merged"].as_str().unwrap().to_string();

    machine.start();
    let status = project.json(&["status"]);
    let red: Vec<_> = status["attention"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["kind"] == "red")
        .collect();
    assert_eq!(red.len(), 1, "{status}");
    assert_eq!(
        project.rows("SELECT intent_state FROM execution WHERE kind = 'landing'")[0]["intent_state"],
        "open"
    );
    // Release the held command: it moves canonical, and `start` records it.
    std::fs::write(&release, "go\n").unwrap();
    std::fs::read_to_string(&done).unwrap();
    assert_eq!(project.canonical_head(), merged);
    project.json(&["attempt", "start", "Y-1"]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
    assert_eq!(
        project
            .rows("SELECT seq FROM audit WHERE event = 'landing.recorded'")
            .len(),
        1
    );
}

/// A git wrapper that, while `armed` exists, holds the landing's
/// `update-ref` of the target: it says `held` on one FIFO and waits for a
/// line on another before running the real command. The daemon lives.
fn hold_update_ref(
    machine: &Machine,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let armed = machine.root.join("armed");
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    for fifo in [&said, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    machine.wrapper(
        "git",
        &format!(
            "if [ -e '{}' ]; then case \" $* \" in *' update-ref --no-deref refs/heads/main '*)\n\
             echo held > '{}'; read line < '{}';; esac; fi",
            armed.display(),
            said.display(),
            release.display()
        ),
    );
    (armed, said, release)
}

/// Canonical moved by hand between verify and land: the landing retires and
/// re-queues. On green, canonical is the verified ref and contains the head.
/// The landing is held at its `update-ref`, after its gates and intent,
/// while a ticket edit is refused naming the intent (another ticket edits
/// successfully), then the operator moves canonical.
///
/// Sabotage: make `queue::land` record the landing when `update_ref`
/// reports the old value did not match; the first landing is recorded
/// while canonical stays at the hand-made commit, so there is one landing
/// row and canonical is not its merged ref.
#[test]
fn a_landing_retires_when_canonical_moves_before_update_ref() {
    let machine = Machine::new("g3-moved", worker());
    let (armed, said, release) = hold_update_ref(&machine);
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f feature.txt\"\nruns_in = \"host\"\n")
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let raised = watch.attention();
    let head = raised["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();

    project.json(&["ticket", "new", "--title", "Other", "--parked"]);

    std::fs::write(&armed, "").unwrap();
    project.json(&["attempt", "approve", "Y-1", "--head", &head]);
    assert_eq!(watch.said(&said), "held\n");
    std::fs::remove_file(&armed).unwrap();
    // An edit on the intent is refused; the other ticket can be edited.
    // Sabotage: drop admit::edit's refuse_during_intent guard.
    let intent =
        project.rows("SELECT id FROM execution WHERE kind = 'landing' AND intent_state = 'open'");
    assert_eq!(intent.len(), 1);
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"].to_string();
    let refused = project.refused(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        "Changed",
    ]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["intent"], intent[0]["id"]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["body"], "");
    project.json(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "1",
        "--body",
        "Changed",
    ]);
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["body"], "Changed");

    // The operator moves canonical by hand while the landing is held.
    let canonical = project.canonical();
    let target = project.canonical_head();
    let tree = git(&canonical, &["rev-parse", &format!("{target}^{{tree}}")]);
    let moved = git(
        &canonical,
        &["commit-tree", tree.trim(), "-p", &target, "-m", "By hand"],
    );
    let moved = moved.trim();
    git(
        &canonical,
        &["update-ref", "refs/heads/main", moved, &target],
    );
    std::fs::write(&release, "go\n").unwrap();

    watch.event("landing.recorded", &[]);
    let landings = project.rows(
        "SELECT id, outcome, intent_old, intent_merged FROM execution WHERE kind = 'landing' ORDER BY id",
    );
    assert_eq!(landings.len(), 2, "{landings:?}");
    assert_eq!(landings[0]["outcome"], "retired");
    assert_eq!(landings[0]["intent_old"], target.as_str());
    assert_eq!(landings[1]["outcome"], "landed");
    assert_eq!(landings[1]["intent_old"], moved);
    // Canonical is the second landing's verified ref: its gate ran on it.
    let merged = landings[1]["intent_merged"].as_str().unwrap();
    assert_eq!(project.canonical_head(), merged);
    assert_eq!(
        project.rows(&format!(
            "SELECT outcome FROM execution WHERE parent = {} AND head = '{merged}'",
            landings[1]["id"]
        )),
        vec![json!({ "outcome": "pass" })]
    );
    git(&canonical, &["merge-base", "--is-ancestor", &head, merged]);
    git(&canonical, &["merge-base", "--is-ancestor", moved, merged]);
}
