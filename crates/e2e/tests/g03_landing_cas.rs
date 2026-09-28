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

/// The daemon killed after `update-ref`, before the landing is recorded:
/// before restart canonical is the merged head, the intent is unresolved and
/// no landing is recorded; on restart it is recorded once, no merge runs,
/// the ticket closes.
///
/// Sabotage: make `reconcile::decide_intent` retire an intent whose target
/// equals the merged head; the ticket stays open after restart.
#[test]
fn a_landing_killed_after_update_ref_is_recorded_once_on_restart() {
    let machine = Machine::new("g3-after", worker());
    let armed = machine.root.join("armed");
    machine.git_wrapper(&format!(
        "if [ -e '{}' ]; then case \" $* \" in *' update-ref --no-deref refs/heads/main '*)\n\
         \"$REAL\" \"$@\"; status=$?; kill -9 $PPID; exit $status;; esac; fi",
        armed.display()
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

    // Before restart: canonical is the merged head, and only that.
    let merged = project.canonical_head();
    let parents = git(
        &project.canonical(),
        &["rev-list", "--parents", "-n", "1", &merged],
    );
    assert_eq!(
        parents.split_whitespace().skip(1).collect::<Vec<_>>(),
        vec![target.as_str(), head.as_str()],
        "canonical is not the merge of the target and the candidate"
    );
    let intents = project.rows(
        "SELECT id, intent_state, intent_old, intent_merged FROM execution WHERE kind = 'landing'",
    );
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0]["intent_state"], "open");
    assert_eq!(intents[0]["intent_old"], target.as_str());
    assert_eq!(intents[0]["intent_merged"], merged.as_str());
    assert!(
        project
            .rows("SELECT seq FROM audit WHERE event = 'landing.recorded'")
            .is_empty()
    );

    machine.start();
    // Reconciliation committed before the daemon served.
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
    assert_eq!(
        project
            .rows("SELECT seq FROM audit WHERE event = 'landing.recorded'")
            .len(),
        1
    );
    assert_eq!(project.canonical_head(), merged, "a merge ran again");
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'landing'")
            .len(),
        1,
        "a second landing ran"
    );
}

/// The daemon killed while its `update-ref` is held: restart keeps the
/// intent, refuses the queue and raises `red`; once the command is released,
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
    machine.git_wrapper(&format!(
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
        .filter(|item| item["kind"] == "red" && item["reason"] == "intent")
        .collect();
    assert_eq!(red.len(), 1, "{status}");
    assert_eq!(
        project.rows("SELECT intent_state FROM execution WHERE kind = 'landing'")[0]["intent_state"],
        "open"
    );
    // The queue is refused while the intent is open: `start` decides nothing.
    let refused = project.refused(&["attempt", "start", "Y-1"]);
    assert_eq!(refused["code"], "refused");

    // Release the held command: it moves canonical, and `start` records it.
    std::fs::write(&release, "go\n").unwrap();
    std::fs::read_to_string(&done).unwrap();
    assert_eq!(project.canonical_head(), merged);
    let started = project.json(&["attempt", "start", "Y-1"]);
    assert_eq!(started["intent"], json!("decided"));
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
    assert_eq!(
        project
            .rows("SELECT seq FROM audit WHERE event = 'landing.recorded'")
            .len(),
        1
    );
    assert_eq!(project.canonical_head(), merged);
}
