//! G3: Landing is compare-and-swap and proved.

use e2e::*;
use serde_json::json;

fn worker() -> impl Fn(&ModelRequest) -> Reply + Send + Sync + 'static {
    |request| {
        if request
            .prompt()
            .contains("Your workspace is read-only. Plan the work")
        {
            planned(request)
        } else {
            match request.turn() {
                0 => Reply::Tools(vec![commit_file("feature.txt", "feature\n", "Add feature")]),
                _ => Reply::Text("done".into()),
            }
        }
    }
}

/// A candidate awaiting approval, with canonical moved past its base by the
/// operator, so its landing is a real merge. Returns the candidate head.
fn candidate_behind_target(project: &Project, watch: &mut Watch) -> String {
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--target",
        "feature/a",
    ]);
    let raised = watch.until("approval raised", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });
    project.write("other.txt", "other\n");
    project.git(&["add", "other.txt"]);
    project.git(&["commit", "--quiet", "-m", "Operator change"]);
    project.json(&["sync", "feature/a"]);
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
/// Sabotage: omit the landing Git child's inherited host lock, so
/// `capacity::release` sees a free lock and absent box despite the held child.
/// Restart reads canonical at the old head and retires the intent, so the
/// released `update-ref` lands a candidate the store never records.
/// Recover through the default ref instead of intent_ref: main contains
/// neither recorded head, so start cannot resolve the feature/a landing.
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
        "if [ -e '{}' ]; then case \" $* \" in *' update-ref --no-deref refs/heads/feature/a '*)\n\
         kill -9 $PPID; read line < '{}'; \"$REAL\" \"$@\"; status=$?; echo done > '{}'; exit $status;;\n esac; fi",
        armed.display(),
        release.display(),
        done.display()
    ));
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed());
    let main = project.canonical_head();
    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.json(&["sync", "feature/a"]);
    let mut watch = project.watch(0);
    let head = candidate_behind_target(&project, &mut watch);
    let target = git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"])
        .trim()
        .to_string();

    std::fs::write(&armed, "").unwrap();
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "feature/a",
        "Y-1",
        "--head",
        &head,
    ]);
    machine.wait_dead();
    std::fs::remove_file(&armed).unwrap();

    // Before restart: nothing moved, the intent is open.
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    assert_eq!(project.canonical_head(), main);
    let intent = &project.rows(
        "SELECT id, intent_state, intent_ref, intent_merged FROM execution WHERE kind = 'landing'",
    )[0];
    assert_eq!(intent["intent_state"], "open");
    assert_eq!(intent["intent_ref"], "refs/heads/feature/a");
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
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        merged
    );
    assert_eq!(project.canonical_head(), main);
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
/// line on another before running the real command. It can then hold the
/// requeue's merge-tree before successor gates and intent. The daemon lives.
fn hold_update_ref(
    machine: &Machine,
    requeue_armed: &std::path::Path,
    requeue_said: &std::path::Path,
    requeue_release: &std::path::Path,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let armed = machine.root.join("armed");
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    for fifo in [
        said.as_path(),
        release.as_path(),
        requeue_said,
        requeue_release,
    ] {
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
             echo held > '{}'; read line < '{}';; esac; fi\n\
             if [ -e '{}' ]; then case \" $* \" in *' merge-tree --write-tree '*)\n\
             rm '{}'; echo held > '{}'; read line < '{}';; esac; fi",
            armed.display(),
            said.display(),
            release.display(),
            requeue_armed.display(),
            requeue_armed.display(),
            requeue_said.display(),
            requeue_release.display()
        ),
    );
    (armed, said, release)
}

/// Canonical moved by hand between verify and land: the landing retires and
/// re-queues. On green, canonical is the verified ref and contains the head.
/// The landing is held at its `update-ref`, after its gates and intent,
/// while ticket edits and a valid plan adjustment refuse naming the intent.
/// The same adjustment succeeds before the intent; another ticket edits
/// successfully. After canonical moves, the resolved intent still refuses
/// a selected landing-definition change while the requeue's merge-tree holds.
/// Before the first intent, a selected gate fails once. Its candidate:landing
/// failure evidence binds the committed command before exact acceptance
/// continues preparation.
/// Sabotage original-landing-decision-evidence: use original_admission in
/// checks::evidence for parented checks instead of their recorded admission.
///
/// Sabotage: make `queue::land` record the landing when `update_ref`
/// reports the old value did not match; the first landing is recorded
/// while canonical stays at the hand-made commit, so there is one landing
/// row and canonical is not its merged ref.
#[test]
fn a_landing_retires_when_canonical_moves_before_update_ref() {
    let machine = Machine::new("g3-moved", worker());
    let requeue_armed = machine.root.join("requeue-armed");
    let requeue_said = machine.root.join("requeue-said");
    let requeue_release = machine.root.join("requeue-release");
    let (armed, said, release) =
        hold_update_ref(&machine, &requeue_armed, &requeue_said, &requeue_release);
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(concat!(
            "[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\nruns_in = \"host\"\n",
            "\n[profiles.revised]\n",
            "runner = {kind = \"agent\", agent = \"worker\", instructions = \"Keep the feature.\"}\n",
            "access = \"write-allowed\"\noutputs = {result = \"candidate\"}\noutcomes = [\"complete\"]\n",
        ))
            .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    watch.attention();
    // The admitted candidate accepts source and instructions selection
    // outside the intent boundary. Its agent and grants stay frozen.
    let adjustment = json!({"implementer": {"profile": "revised"},
        "landing-gate-future": {"operator_failure": true}});
    // G3: future requirements are selected before intent and run on the
    // prepared merge, including the target-only file. Host Git is the oracle.
    // Sabotage: freeze landing nodes or read original landing definitions.
    let landing_head = machine.root.join("selected-landing-head");
    let fail_once = machine.root.join("selected-landing-fail-once");
    let selected_command = format!(
        "test -f feature.txt && test -f other.txt && git rev-parse HEAD >> {} && if test -f {}; then rm {}; exit 1; fi",
        landing_head.display(),
        fail_once.display(),
        fail_once.display()
    );
    let configuration = project.git(&["show", "HEAD:.yard/config.toml"]);
    let selected_definition = format!("{configuration}\n[gates.future]\nstage = 'landing'\nruns_in = 'host'\ntimeout_minutes = 1\ncommand = '{selected_command}'\n")
        .replace("test -f feature.txt\"", "test -s feature.txt\"");
    project.write("other.txt", "target-only\n");
    project.write(".yard/config.toml", &selected_definition);
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "Future landing requirement"]);
    project.json(&["sync"]);
    let definition_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let mut operator = machine.operator();
    operator.initialize();
    let current = project.json(&["attempt", "show", "Y-1"]);
    let expected = current["attempt"]["plan_revision"].as_i64().unwrap();
    let adjusted = operator.call(
        "yard_attempt_revise",
        json!({"project": project.path, "ticket": "Y-1",
            "plan_revision": expected, "overrides": adjustment,
            "definition_commit": definition_commit, "image_commit": definition_commit}),
    );
    assert_eq!(
        adjusted["result"]["plan_revision"],
        expected + 1,
        "{adjusted}"
    );
    let raised = watch.until("revised approval", |event| {
        event["event"] == "attention.raised"
            && event["data"]["kind"] == "approval"
            && event["data"]["payload"]["plan_revision"] == expected + 1
    });
    let head = raised["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();

    // A supported candidate-stage definition is also a selected landing
    // requirement. Commit its next change without adopting it.
    project.write(
        ".yard/config.toml",
        &selected_definition.replace(
            "test -s feature.txt\"",
            "test -s feature.txt && test -f feature.txt\"",
        ),
    );
    project.git(&[
        "commit",
        "--quiet",
        "-am",
        "Later candidate and landing definition",
    ]);
    project.json(&["sync"]);
    let later_definition = project.git(&["rev-parse", "HEAD"]).trim().to_string();

    project.json(&["ticket", "new", "--title", "Other", "--parked"]);

    std::fs::write(&armed, "").unwrap();
    std::fs::write(&fail_once, "").unwrap();
    let plan_revision =
        project.json(&["attempt", "show", "Y-1"])["attempt"]["plan_revision"].to_string();
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "main",
        "Y-1",
        "--head",
        &head,
        "--plan-revision",
        &plan_revision,
    ]);
    let failure = watch.attention();
    assert_eq!(failure["data"]["kind"], "candidate");
    assert_eq!(failure["data"]["reason"], "landing");
    let evidence = &failure["data"]["payload"]["evidence"];
    let identity = &evidence["landing"];
    let failed_parent = identity["execution"].as_i64().unwrap();
    let failed_child = evidence["check"]["execution"].as_i64().unwrap();
    let failed_check = evidence["check"]["check"].as_i64().unwrap();
    assert_eq!(evidence["node"], "landing-gate-future");
    assert_eq!(evidence["check"]["name"], "future");
    assert_eq!(evidence["check"]["plan_revision"], expected + 1);
    assert_eq!(evidence["check"]["verdict"], "fail");
    assert_eq!(evidence["result"]["outcome"], "failed");
    assert_eq!(
        evidence["consumed_definition"]["gate"],
        json!({
            "name": "future", "command": selected_command, "stage": "landing",
            "runs_in": "host", "timeout_minutes": 1, "env": []
        })
    );
    let prepared = git(
        &project.canonical(),
        &["rev-parse", &format!("refs/yard/landings/{failed_parent}")],
    )
    .trim()
    .to_string();
    assert_eq!(identity["prepared"]["merged"], prepared);
    assert_eq!(evidence["check"]["head"], prepared);
    let ended = watch.find("selected failed preparation ended", |event| {
        event["event"] == "execution.ended"
            && event["execution"] == failed_parent
            && event["data"]["outcome"] == "fail"
            && event["seq"].as_i64().unwrap() > failure["seq"].as_i64().unwrap()
    });
    watch.find("selected failed preparation released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == failed_parent
            && event["seq"].as_i64().unwrap() > ended["seq"].as_i64().unwrap()
    });
    assert_eq!(project.rows(&format!(
        "SELECT id, parent, plan_revision, outcome, teardown, reserved FROM execution WHERE id = {failed_child}"
    )), vec![json!({"id": failed_child, "parent": failed_parent,
        "plan_revision": expected + 1, "outcome": "fail", "teardown": 1, "reserved": 0})]);
    assert_eq!(project.rows(&format!(
        "SELECT status, outcome, reserved, intent_state FROM execution WHERE id = {failed_parent}"
    )), vec![json!({"status": "ended", "outcome": "fail", "reserved": 0, "intent_state": null})]);
    let failed_snapshot = project.rows(&format!(
        "SELECT c.*, e.result, e.inputs FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.id = {failed_check}"
    ));
    let accepted = operator.call("yard_attempt_accept", json!({
        "project": project.path, "ticket": "Y-1", "attention": failure["attention"],
        "check": failed_check, "plan_revision": expected + 1, "target": "main", "head": head,
        "proof": failure["data"]["payload"]["proof"], "text": "Accept selected committed requirement",
        "landing": failed_parent, "child": failed_child, "node": evidence["node"],
        "base": identity["base"], "revision": identity["revision"],
        "old": identity["prepared"]["old"], "merged": prepared
    }));
    assert_eq!(accepted["result"]["accepted"], true, "{accepted}");
    assert_eq!(watch.said(&said), "held\n");
    std::fs::remove_file(&armed).unwrap();
    let intent =
        project.rows("SELECT id FROM execution WHERE kind = 'landing' AND intent_state = 'open'");
    assert_eq!(intent.len(), 1);
    // The accepted continuation reuses the failed parent's pinned input.
    let merged = git(
        &project.canonical(),
        &["rev-parse", &format!("refs/yard/landings/{failed_parent}")],
    )
    .trim()
    .to_string();
    assert_eq!(
        std::fs::read_to_string(&landing_head).unwrap(),
        format!("{merged}\n")
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{merged}:other.txt")]
        ),
        "target-only\n"
    );
    // G3 Shown-by: a valid plan adjustment racing the held intent refuses
    // naming it and changes no plan, approval, attention or audit; the same
    // adjustment succeeds before the intent. This maps Live adjustment.
    // attempt_revise guards this independently after ordinary validation.
    // Sabotage: remove only attempt_revise's refuse_during_intent call.
    // It must fail first at structured refusal, before identity and no-write
    // observations. The FIFO holds real Git after the mutation intent;
    // store snapshots supply the outside oracle.
    let refuse_adjustment = |operator: &mut Operator, mut request: serde_json::Value| {
        let plan = project.json(&["attempt", "show", "Y-1"]);
        let revisions = project.rows("SELECT * FROM plan_revision ORDER BY attempt, revision");
        let approvals = project.rows("SELECT * FROM approval ORDER BY id");
        let attention = project.rows("SELECT * FROM attention ORDER BY id");
        let audit = project.rows("SELECT * FROM audit ORDER BY seq");
        request["plan_revision"] = plan["attempt"]["plan_revision"].clone();
        let refused = operator.call("yard_attempt_revise", request);
        assert_eq!(refused["error"]["code"], "refused", "{refused}");
        assert_eq!(refused["error"]["data"]["intent"], intent[0]["id"]);
        let unchanged = project.json(&["attempt", "show", "Y-1"]);
        assert_eq!(
            unchanged["attempt"]["plan_revision"],
            plan["attempt"]["plan_revision"]
        );
        assert_eq!(unchanged["attempt"]["plan"], plan["attempt"]["plan"]);
        assert_eq!(unchanged["plan_revisions"], plan["plan_revisions"]);
        assert_eq!(
            project.rows("SELECT * FROM plan_revision ORDER BY attempt, revision"),
            revisions
        );
        assert_eq!(
            project.rows("SELECT * FROM approval ORDER BY id"),
            approvals
        );
        assert_eq!(
            project.rows("SELECT * FROM attention ORDER BY id"),
            attention
        );
        assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), audit);
    };
    refuse_adjustment(
        &mut operator,
        json!({"project": project.path, "ticket": "Y-1",
        "overrides": adjustment, "definition_commit": definition_commit,
        "image_commit": definition_commit}),
    );
    // An edit on the intent is refused; the other ticket can be edited.
    // Sabotage: drop admit::edit's refuse_during_intent guard.
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
    // G9: target edits also refuse the intent before waiting on canonical.
    // Sabotage: take canonical before the target guard; this call cannot
    // answer while the fixture holds update-ref. Y-2 supplies the control.
    let target_edit = project.refused(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--target",
        "main",
    ]);
    assert_eq!(target_edit["code"], "refused");
    assert_eq!(target_edit["data"]["intent"], intent[0]["id"]);
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
    project.json(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "2",
        "--target",
        "main",
    ]);
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["target"], "main");

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
    std::fs::write(&requeue_armed, "").unwrap();
    std::fs::write(&release, "go\n").unwrap();

    assert_eq!(watch.said(&requeue_said), "held\n");
    // G3: the predecessor's real intent is resolved and retired, but the
    // live attempt still cannot change its selected landing definitions.
    // The successor's real merge-tree holds before gates and intent.
    // Sabotage Y144-historical-landing-intent: delete only attempt_revise's historical
    // landing_requirements/intent_state guard, retaining refuse_during_intent.
    // The direct refused identity must fail, not a timeout or active_plan.
    // Store snapshots and host Git supply state and identity from outside.
    assert_eq!(project.rows("SELECT id, outcome, intent_state, intent_old, intent_merged FROM execution WHERE kind = 'landing' AND intent_state IS NOT NULL"),
        vec![json!({"id": intent[0]["id"], "outcome": "retired",
            "intent_state": "resolved", "intent_old": target, "intent_merged": merged})]);
    assert!(
        project
            .rows("SELECT id FROM execution WHERE intent_state = 'open'")
            .is_empty()
    );
    assert!(project.rows("SELECT id FROM execution WHERE attempt = 1 AND status = 'running' AND kind IN ('gate', 'review')").is_empty());
    assert_eq!(
        project.rows("SELECT state, next FROM attempt WHERE id = 1"),
        vec![json!({"state": "live", "next": null})]
    );
    assert_eq!(
        project.json(&["attempt", "show", "Y-1"])["attempt"]["head"],
        head
    );
    assert!(
        project
            .rows("SELECT id FROM attention WHERE attempt = 1 AND state = 'open'")
            .is_empty()
    );
    assert_eq!(
        project.rows("SELECT state FROM approval"),
        vec![json!({"state": "active"})]
    );
    assert_eq!(project.canonical_head(), moved);
    refuse_adjustment(
        &mut operator,
        json!({"project": project.path, "ticket": "Y-1",
        "overrides": adjustment, "definition_commit": later_definition}),
    );
    std::fs::write(&requeue_release, "go\n").unwrap();

    watch.event("landing.recorded", &[]);
    assert_eq!(project.rows(&format!(
        "SELECT c.*, e.result, e.inputs FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.id = {failed_check}"
    )), failed_snapshot);
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'landing'")
            .len(),
        3
    );
    let landings = project.rows(
        "SELECT id, outcome, intent_old, intent_merged FROM execution WHERE kind = 'landing' AND intent_state IS NOT NULL ORDER BY id",
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
        vec![json!({ "outcome": "pass" }), json!({ "outcome": "pass" })]
    );
    git(&canonical, &["merge-base", "--is-ancestor", &head, merged]);
    git(&canonical, &["merge-base", "--is-ancestor", moved, merged]);
}
