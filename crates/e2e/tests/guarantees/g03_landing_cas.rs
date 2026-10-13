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
/// while ticket edits and a valid plan adjustment refuse naming the intent.
/// The same adjustment succeeds before the intent; another ticket edits
/// successfully. Then the operator moves canonical.
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
        &config(concat!(
            "[gates.check]\ncommand = \"test -f feature.txt\"\nruns_in = \"host\"\n",
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
    let adjustment = json!({"implementer": {"profile": "revised"}});
    // G3: future requirements are selected before intent and run on the
    // prepared merge, including the target-only file. Host Git is the oracle.
    // Sabotage: freeze landing nodes or read original landing definitions.
    let landing_head = machine.root.join("selected-landing-head");
    let configuration = project.git(&["show", "HEAD:.yard/config.toml"]);
    project.write("other.txt", "target-only\n");
    project.write(
        ".yard/config.toml",
        &format!("{configuration}\n[gates.future]\nstage = 'landing'\nruns_in = 'host'\ncommand = 'test -f feature.txt && test -f other.txt && git rev-parse HEAD >> {}'\n", landing_head.display()),
    );
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

    project.json(&["ticket", "new", "--title", "Other", "--parked"]);

    std::fs::write(&armed, "").unwrap();
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
    assert_eq!(watch.said(&said), "held\n");
    std::fs::remove_file(&armed).unwrap();
    let intent =
        project.rows("SELECT id FROM execution WHERE kind = 'landing' AND intent_state = 'open'");
    assert_eq!(intent.len(), 1);
    let merged = git(
        &project.canonical(),
        &[
            "rev-parse",
            &format!("refs/yard/landings/{}", intent[0]["id"]),
        ],
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
    let plan = project.json(&["attempt", "show", "Y-1"]);
    let revisions = project.rows("SELECT * FROM plan_revision ORDER BY attempt, revision");
    let approvals = project.rows("SELECT * FROM approval ORDER BY id");
    let attention = project.rows("SELECT * FROM attention ORDER BY id");
    let audit = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call(
        "yard_attempt_revise",
        json!({"project": project.path, "ticket": "Y-1",
            "plan_revision": plan["attempt"]["plan_revision"], "overrides": adjustment,
            "definition_commit": definition_commit, "image_commit": definition_commit}),
    );
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
        vec![json!({ "outcome": "pass" }), json!({ "outcome": "pass" })]
    );
    git(&canonical, &["merge-base", "--is-ancestor", &head, merged]);
    git(&canonical, &["merge-base", "--is-ancestor", moved, merged]);
}

/// G2/G3: a held host child proves the prepared input before restart.
/// Sabotage: regenerate the unchanged merge, replay its passing child, promote
/// the interrupted child, or reuse either child after the target advances.
/// Rewrite an accepted failure to pass, inherit acceptance on fresh preparation,
/// consume it before teardown, or bypass the final target CAS.
/// Retain the ended parent's reservation: its historical phase release cannot
/// permit acceptance. Require the final release after the failed end.
/// Delete admit's predecessor proof or accept while ownership is retained.
/// Retire an unrelated landing instead of the changed-input continuation.
fn prepared_host_child_restart(advance: bool) {
    let machine = Machine::new(
        if advance {
            "g2-prepared-moved"
        } else {
            "g3-prepared-child"
        },
        worker(),
    );
    machine.wrapper("git", &distinct_merge_dates(&machine));
    let armed = machine.root.join("armed");
    let said_path = machine.root.join("said");
    let release = machine.root.join("release");
    let observed = machine.root.join("observed-heads");
    let failed_marker = machine.root.join("failed-middle");
    let final_probe = machine.root.join("final-probe");
    let probe_said = machine.root.join("probe-said");
    let probe_release = machine.root.join("probe-release");
    for fifo in [&said_path, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    let _writer = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&release)
        .unwrap();
    if !advance {
        for fifo in [&probe_said, &probe_release] {
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(fifo)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        // The failed host child releases first. Hold the following real
        // inventory reply, which belongs to the failed parent's final release.
        machine.wrapper(
            "pinfold",
            &format!(
                "if [ \"$1 $2\" = 'box list' ]; then\n\
               if [ -f '{}' ]; then\n\
                 mv '{}' '{}';\n\
               elif [ -f '{}' ]; then\n\
                 rm '{}';\n\
                 \"$REAL\" \"$@\"; result=$?\n\
                 echo \"$$ $result\" > '{}';\n\
                 read released < '{}';\n\
                 exit \"$result\"\n\
               fi\n\
             fi",
                failed_marker.display(),
                failed_marker.display(),
                final_probe.display(),
                final_probe.display(),
                final_probe.display(),
                probe_said.display(),
                probe_release.display()
            ),
        );
    }
    let middle = if advance {
        "false".to_string()
    } else {
        format!("touch '{}'; false", failed_marker.display())
    };
    let gates = format!(
        "[gates.first]\ncommand = \"git rev-parse HEAD >> {}\"\nstage = \"landing\"\nruns_in = \"host\"\n\
         [gates.middle]\ncommand = \"{middle}\"\nstage = \"landing\"\nruns_in = \"host\"\n\
         [gates.second]\ncommand = \"if [ -e '{}' ]; then echo $$ > {}; read line < {}; fi\"\nstage = \"landing\"\nruns_in = \"host\"\n",
        observed.display(),
        armed.display(),
        said_path.display(),
        release.display()
    );
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(&gates).replace("review = [\"correctness\"]", "review = \"none\""),
    );
    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.json(&["sync", "feature/a"]);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Selected merged failure",
        "--target",
        "feature/a",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workflow": "default",
            "overrides": {"landing-gate-middle": {"operator_failure": true}}
        }),
    );
    assert!(started.get("error").is_none(), "{started}");
    let approval = watch.attention();
    let head = approval["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();
    project.write("other.txt", "other\n");
    project.git(&["add", "other.txt"]);
    project.git(&["commit", "-qm", "Operator change"]);
    project.json(&["sync", "feature/a"]);
    let target = git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"])
        .trim()
        .to_string();
    std::fs::write(&armed, "").unwrap();
    project.json(&[
        "attempt",
        "approve",
        "Y-1",
        "--target",
        "feature/a",
        "--head",
        &head,
    ]);
    let failure = watch.attention();
    assert_eq!(failure["data"]["reason"], "landing");
    let evidence = &failure["data"]["payload"]["evidence"];
    let identity = &evidence["landing"];
    let ended = watch.find("failed preparation ended", |event| {
        event["event"] == "execution.ended"
            && event["execution"] == identity["execution"]
            && event["data"]["outcome"] == "fail"
            && event["seq"].as_i64().unwrap() > failure["seq"].as_i64().unwrap()
    });
    let decision = json!({
        "project": project.path, "ticket": "Y-1", "attention": failure["attention"],
        "check": evidence["check"]["check"], "plan_revision": 1, "target": "feature/a", "head": head,
        "proof": failure["data"]["payload"]["proof"], "text": "Accept pinned merged failure",
        "landing": identity["execution"], "child": evidence["check"]["execution"], "node": evidence["node"],
        "base": identity["base"], "revision": identity["revision"],
        "old": identity["prepared"]["old"], "merged": identity["prepared"]["merged"]
    });
    let failure_check = evidence["check"]["check"].as_i64().unwrap();
    let failure_snapshot = project.rows(&format!("SELECT c.*, e.result, e.inputs FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.id = {failure_check}"));
    assert_eq!(evidence["check"]["verdict"], "fail");
    assert_eq!(evidence["result"]["outcome"], "failed");
    if !advance {
        let probe = watch.said(&probe_said);
        let fields = probe.split_whitespace().collect::<Vec<_>>();
        assert_eq!(fields.len(), 2, "{probe}");
        assert_eq!(
            fields[1], "0",
            "real inventory succeeded before its held reply"
        );
        assert!(
            std::process::Command::new("kill")
                .args(["-0", fields[0]])
                .status()
                .unwrap()
                .success()
        );
        let parent = identity["execution"].as_i64().unwrap();
        assert_eq!(
            project.rows(&format!(
                "SELECT id, status, outcome, reserved FROM execution WHERE id = {parent}"
            )),
            vec![json!({"id": parent, "status": "ended", "outcome": "fail", "reserved": 1})]
        );
        let parent_lock_path = project
            .path
            .join(format!(".yard/local/landings/{parent}/lock"));
        let parent_lock = std::fs::OpenOptions::new()
            .read(true)
            .open(&parent_lock_path)
            .unwrap();
        parent_lock.try_lock().unwrap();
        let competing = std::fs::OpenOptions::new()
            .read(true)
            .open(&parent_lock_path)
            .unwrap();
        assert!(matches!(
            competing.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        let before = project.rows("SELECT * FROM audit ORDER BY seq");
        let questions = project.rows("SELECT * FROM attention ORDER BY id");
        let refused = operator.call("yard_attempt_accept", decision.clone());
        assert_eq!(refused["error"]["code"], "stale", "{refused}");
        let data = &refused["error"]["data"];
        assert_eq!(data["identity"], "ownership");
        assert_eq!(
            data["expected"],
            json!({"execution": parent, "released": true})
        );
        assert_eq!(data["current"]["execution"], parent);
        assert_eq!(data["current"]["status"], "ended");
        assert_eq!(data["current"]["outcome"], "fail");
        assert_eq!(data["current"]["reserved"], true);
        assert_eq!(data["current"]["prepared"], identity["prepared"]);
        assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
        assert_eq!(
            project.rows("SELECT * FROM attention ORDER BY id"),
            questions
        );
        parent_lock.unlock().unwrap();
        std::fs::write(&probe_release, "go\n").unwrap();
    }
    watch.find("failed preparation released after end", |event| {
        event["event"] == "execution.released"
            && event["execution"] == identity["execution"]
            && event["seq"].as_i64().unwrap() > ended["seq"].as_i64().unwrap()
    });
    assert_eq!(
        project.rows(&format!(
            "SELECT status, outcome, reserved FROM execution WHERE id = {}",
            identity["execution"]
        )),
        vec![json!({"status": "ended", "outcome": "fail", "reserved": 0})]
    );
    let accepted = operator.call("yard_attempt_accept", decision.clone());
    assert_eq!(accepted["result"]["accepted"], true, "{accepted}");
    let group = watch.said(&said_path).trim().to_string();
    drop(watch);
    let parent = project
        .rows("SELECT id, prepared, intent_state FROM execution WHERE kind = 'landing' AND prepared IS NOT NULL")[0]
        .clone();
    let parent_id = parent["id"].as_i64().unwrap();
    let merged = git(
        &project.canonical(),
        &["rev-parse", &format!("refs/yard/landings/{parent_id}")],
    )
    .trim()
    .to_string();
    let prepared: serde_json::Value =
        serde_json::from_str(parent["prepared"].as_str().unwrap()).unwrap();
    assert_eq!(
        prepared,
        json!({"target_ref": "refs/heads/feature/a", "old": target, "merged": merged})
    );
    assert_eq!(parent["intent_state"], serde_json::Value::Null);
    let completed = project.rows(&format!(
        "SELECT * FROM execution WHERE parent = {parent_id} AND name = 'first'"
    ));
    assert_eq!(completed[0]["outcome"], "pass");
    assert_eq!(completed[0]["teardown"], 1);
    let held_child = project
        .rows("SELECT id, parent FROM execution WHERE name = 'second' AND status = 'running'")[0]
        .clone();
    let child = held_child["id"].as_i64().unwrap();
    let old_continuation = held_child["parent"].as_i64().unwrap();
    assert_eq!(
        project.rows(&format!(
            "SELECT resumed FROM execution WHERE id = {old_continuation}"
        )),
        vec![json!({"resumed": parent_id})]
    );
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .open(
            project
                .path
                .join(format!(".yard/local/attempts/2/effects/{child}/lock")),
        )
        .unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    machine.kill();
    assert_eq!(project.rows(&format!("SELECT c.*, e.result, e.inputs FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.id = {failure_check}")), failure_snapshot);
    // Held until reconciliation kills the recorded host group. Canonical has
    // not moved and there is no mutation intent before restarting.
    assert!(
        std::process::Command::new("kill")
            .args(["-0", "--", &format!("-{group}")])
            .status()
            .unwrap()
            .success()
    );
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    assert_eq!(
        std::fs::read_to_string(&observed).unwrap(),
        format!("{merged}\n")
    );
    std::fs::remove_file(&armed).unwrap();
    let expected_target = if advance {
        let tree = git(
            &project.canonical(),
            &["rev-parse", &format!("{target}^{{tree}}")],
        );
        let moved = git(
            &project.canonical(),
            &[
                "commit-tree",
                tree.trim(),
                "-p",
                &target,
                "-m",
                "Advance before continuation",
            ],
        );
        git(
            &project.canonical(),
            &["update-ref", "refs/heads/feature/a", moved.trim(), &target],
        );
        moved.trim().to_string()
    } else {
        target.clone()
    };
    machine.start();
    let mut watch = project.watch(0);
    if advance {
        let fresh = watch.find("fresh merged failure", |event| {
            event["event"] == "attention.raised"
                && event["data"]["reason"] == "landing"
                && event["attention"] != failure["attention"]
        });
        let evidence = &fresh["data"]["payload"]["evidence"];
        let identity = &evidence["landing"];
        let ended = watch.find("fresh failed preparation ended", |event| {
            event["event"] == "execution.ended"
                && event["execution"] == identity["execution"]
                && event["data"]["outcome"] == "fail"
                && event["seq"].as_i64().unwrap() > fresh["seq"].as_i64().unwrap()
        });
        watch.find("fresh preparation released after end", |event| {
            event["event"] == "execution.released"
                && event["execution"] == identity["execution"]
                && event["seq"].as_i64().unwrap() > ended["seq"].as_i64().unwrap()
        });
        assert_eq!(
            project.rows(&format!(
                "SELECT status, outcome, reserved FROM execution WHERE id = {}",
                identity["execution"]
            )),
            vec![json!({"status": "ended", "outcome": "fail", "reserved": 0})]
        );
        let mut operator = machine.operator();
        operator.initialize();
        let before = project.rows("SELECT * FROM audit ORDER BY seq");
        let decisions = project.rows("SELECT * FROM attention ORDER BY id");
        let refused = operator.call("yard_attempt_accept", decision);
        assert_eq!(refused["error"]["code"], "stale");
        assert_eq!(
            refused["error"]["data"],
            json!({"identity": "old", "expected": target, "current": expected_target})
        );
        assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
        assert_eq!(
            project.rows("SELECT * FROM attention ORDER BY id"),
            decisions
        );
        let accepted = operator.call("yard_attempt_accept", json!({
            "project": project.path, "ticket": "Y-1", "attention": fresh["attention"],
            "check": evidence["check"]["check"], "plan_revision": 1, "target": "feature/a", "head": head,
            "proof": fresh["data"]["payload"]["proof"], "text": "Accept fresh prepared failure",
            "landing": identity["execution"], "child": evidence["check"]["execution"], "node": evidence["node"],
            "base": identity["base"], "revision": identity["revision"],
            "old": identity["prepared"]["old"], "merged": identity["prepared"]["merged"]
        }));
        assert_eq!(accepted["result"]["accepted"], true, "{accepted}");
    }
    watch.event("landing.recorded", &[]);
    assert_eq!(project.rows(&format!("SELECT c.*, e.result, e.inputs FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.id = {failure_check}")), failure_snapshot);
    assert!(
        !std::process::Command::new("kill")
            .args(["-0", "--", &format!("-{group}")])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    assert!(lock.try_lock().is_ok());
    let landed = git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"])
        .trim()
        .to_string();
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id = {}",
            completed[0]["id"]
        )),
        completed
    );
    let interrupted = project.rows(&format!(
        "SELECT outcome, teardown, result FROM execution WHERE id = {child}"
    ));
    assert_eq!(interrupted[0]["outcome"], "interrupted");
    assert_eq!(interrupted[0]["teardown"], 1);
    let result: serde_json::Value =
        serde_json::from_str(interrupted[0]["result"].as_str().unwrap()).unwrap();
    assert_eq!(result["outputs"], json!({}));
    assert_eq!(result["judgment"], serde_json::Value::Null);
    let final_row = project.rows("SELECT id, resumed, inputs, intent_old FROM execution WHERE kind = 'landing' AND outcome = 'landed'")[0].clone();
    assert_eq!(final_row["intent_old"], expected_target);
    let inputs: serde_json::Value =
        serde_json::from_str(final_row["inputs"].as_str().unwrap()).unwrap();
    if advance {
        assert_ne!(landed, merged);
        let fresh_parent = project.rows("SELECT id FROM execution WHERE kind = 'landing' AND prepared IS NOT NULL ORDER BY id DESC LIMIT 1")[0]["id"].clone();
        assert_eq!(final_row["resumed"], fresh_parent);
        assert!(
            !inputs["children"]
                .as_array()
                .unwrap()
                .contains(&completed[0]["id"])
        );
        assert!(
            !inputs["children"]
                .as_array()
                .unwrap()
                .contains(&evidence["check"]["execution"])
        );
        assert_eq!(
            std::fs::read_to_string(&observed).unwrap(),
            format!("{merged}\n{landed}\n")
        );
        // The held parent is interrupted by restart. Its exact successor
        // must retire the old preparation, regardless of other retired rows.
        let continuation = project.rows(&format!(
            "SELECT id FROM execution WHERE kind = 'landing' AND resumed = {old_continuation}"
        ));
        assert_eq!(continuation.len(), 1, "{continuation:?}");
        assert_eq!(
            project.rows(&format!(
                "SELECT outcome FROM execution WHERE id = {}",
                continuation[0]["id"]
            )),
            vec![json!({"outcome": "retired"})]
        );
    } else {
        assert_eq!(landed, merged);
        let previous = project
            .rows("SELECT id FROM execution WHERE kind = 'landing' AND outcome = 'interrupted'")[0]
            ["id"]
            .clone();
        assert_eq!(final_row["resumed"], previous);
        assert!(
            inputs["children"]
                .as_array()
                .unwrap()
                .contains(&completed[0]["id"])
        );
        assert!(
            inputs["children"]
                .as_array()
                .unwrap()
                .contains(&evidence["check"]["execution"])
        );
        assert_eq!(
            std::fs::read_to_string(&observed).unwrap(),
            format!("{merged}\n")
        );
    }
    assert_eq!(
        project.rows(&format!(
            "SELECT outcome, head FROM execution WHERE parent = {} AND name = 'second'",
            final_row["id"]
        )),
        vec![json!({"outcome": "pass", "head": landed})]
    );
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", &head, &landed],
    );
}

#[test]
fn prepared_input_and_completed_child_survive_host_child_restart() {
    prepared_host_child_restart(false);
}

#[test]
fn target_advance_retires_prepared_input_and_requires_fresh_children() {
    prepared_host_child_restart(true);
}

/// G3: the final prepared-ref Git read is held with no mutation intent.
/// Sabotage: consume prepared evidence before checked parent teardown, or
/// regenerate its head when the surviving Git owner finally releases.
#[test]
fn prepared_input_waits_for_its_surviving_git_owner() {
    let machine = Machine::new("g3-prepared-owner", worker());
    let armed = machine.root.join("armed");
    let said_path = machine.root.join("said");
    let release = machine.root.join("release");
    let done = machine.root.join("done");
    for fifo in [&said_path, &release, &done] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    machine.wrapper("git", &format!(
        "{}if [ -e '{}' ]; then case \" $* \" in *' rev-parse --verify --quiet --end-of-options refs/yard/landings/'*) \"$REAL\" \"$@\"; status=$?; echo $$ > '{}'; read line < '{}'; echo done > '{}'; exit $status;; esac; fi",
        distinct_merge_dates(&machine), armed.display(), said_path.display(), release.display(), done.display()
    ));
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed());
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
        "Y-1",
        "--target",
        "feature/a",
        "--head",
        &head,
    ]);
    let pid = watch.said(&said_path).trim().to_string();
    drop(watch);
    machine.kill();
    let parent = project.rows(
        "SELECT id, prepared, status, reserved, intent_state FROM execution WHERE kind = 'landing'",
    )[0]
    .clone();
    let id = parent["id"].as_i64().unwrap();
    let merged = git(
        &project.canonical(),
        &["rev-parse", &format!("refs/yard/landings/{id}")],
    )
    .trim()
    .to_string();
    assert_eq!(parent["intent_state"], serde_json::Value::Null);
    assert_eq!(parent["status"], "running");
    assert_eq!(parent["reserved"], 1);
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid])
            .status()
            .unwrap()
            .success()
    );
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .open(project.path.join(format!(".yard/local/landings/{id}/lock")))
        .unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    std::fs::remove_file(&armed).unwrap();
    machine.start();
    let status = project.json(&["status"]);
    assert!(
        status["attention"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["reason"] == "ownership" && item["execution"] == id)
    );
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'landing'")
            .len(),
        1
    );
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    // The same project supplies the successful checked-teardown control.
    std::fs::write(&release, "go\n").unwrap();
    std::fs::read_to_string(&done).unwrap();
    let mut watch = project.watch(0);
    watch.until("old owner released", |event| {
        event["event"] == "execution.released" && event["execution"] == id
    });
    project.json(&["attempt", "start", "Y-1"]);
    watch.event("landing.recorded", &[]);
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        merged
    );
    assert_eq!(
        project.rows("SELECT resumed FROM execution WHERE kind = 'landing' AND outcome = 'landed'")
            [0]["resumed"],
        id
    );
}
