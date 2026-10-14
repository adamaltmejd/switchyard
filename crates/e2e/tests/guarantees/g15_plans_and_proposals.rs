//! G15: Plans and proposals resolve.

use e2e::*;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

pub(super) fn pre_target_binary() -> PathBuf {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY
        .get_or_init(|| archived_binary("c85369dfaec8f8554da49ddf02a4dbec03bcad02", "pre-target"))
        .clone()
}

/// G5/G6: actual Y-67 and Y-68 rows survive execution migrations unchanged.
/// The historical daemon creates a fresh store, completes verification,
/// approval, landing and cleanup. Host Git and the fixture supply identities.
/// Sabotage: omit a column, remap IDs, lose a foreign key or invent scope,
/// workspace or settings for retained historical rows.
/// G2 sabotage: infer picker provenance from the old implementation head.
/// The pre-target upgrade lacks Y-67's bound results and owner records.
#[test]
fn nullable_execution_upgrade_preserves_real_results_approvals_and_owners() {
    for (revision, label) in [
        ("fb3f727f793524534965480f86e71af06b0ad646", "pre-standalone"),
        ("c2e7986", "pre-writers"),
    ] {
        historical_execution_upgrade(revision, label);
    }
}

fn historical_execution_upgrade(revision: &str, label: &str) {
    let machine = Machine::new("g68-migration", |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        act(
            request,
            vec![
                bash("printf proof > /yard/proof/evidence.txt"),
                commit_file("feature.txt", "feature\n", "Feature"),
            ],
        )
    });
    machine.use_binary(Some(archived_binary(revision, label)));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(
            r#"
[gates.checked]
command = "test -f feature.txt"
runs_in = "box"
"#,
        ),
    );
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Historical candidate"]);
    let candidate = history.attention();
    assert_eq!(candidate["data"]["kind"], "approval");
    let head = git(
        &project.path.join(".yard/local/attempts/1/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    let snapshots = project.path.join(".yard/local/attempts/1/proof-snapshots");
    let proof = std::fs::read_dir(&snapshots)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();
    assert_eq!(
        std::fs::read(snapshots.join(&proof).join("evidence.txt")).unwrap(),
        b"proof"
    );
    assert_eq!(candidate["data"]["payload"]["proof"], proof);
    project.json(&[
        "attempt", "approve", "Y-1", "--target", "main", "--head", &head, "--proof", &proof,
    ]);
    history.event("landing.recorded", &[]);
    let cleanup = history.until("historical cleanup ended", |e| {
        e["event"] == "execution.ended" && e["data"]["kind"] == "cleanup"
    });
    history.find("historical cleanup release", |e| {
        e["event"] == "execution.released" && e["execution"] == cleanup["execution"]
    });
    assert_eq!(project.canonical_head(), head);
    drop(history);
    machine.kill();
    let mut executions = project.rows("SELECT * FROM execution ORDER BY id");
    for row in &mut executions {
        row.as_object_mut()
            .unwrap()
            .entry("scope")
            .or_insert(Value::Null);
        row["workspace"] = Value::Null;
        row["settings"] = Value::Null;
    }
    let tables = [
        "attempt",
        "check",
        "approval",
        "attention",
        "audit",
        "plan_revision",
    ];
    let mut preserved =
        tables.map(|table| project.rows(&format!("SELECT * FROM \"{table}\" ORDER BY rowid")));
    assert!(!preserved[1].is_empty());
    assert!(!preserved[2].is_empty());
    // These actual source revisions precede pickers. Every old field remains exact.
    for index in [0, 2] {
        for row in &mut preserved[index] {
            row["choice"] = Value::Null;
        }
    }
    machine.use_binary(None);
    for _ in 0..2 {
        machine.start();
        assert_eq!(
            project.rows("SELECT * FROM execution ORDER BY id"),
            executions
        );
        for (table, before) in tables.iter().zip(&preserved) {
            assert_eq!(
                &project.rows(&format!("SELECT * FROM \"{table}\" ORDER BY rowid")),
                before,
                "{table}"
            );
        }
        assert!(project.rows("PRAGMA foreign_key_check").is_empty());
        assert_eq!(project.canonical_head(), head);
        machine.kill();
    }
}

fn archived_binary(revision: &str, name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = yard().parent().unwrap().join(name);
    let source = target.join("source");
    std::fs::create_dir_all(&source).unwrap();
    // Compile real archived source; each old daemon creates its own fresh store.
    // Pre-target source predates runners; later source embeds their sealed bytes.
    let runners = Command::new("git")
        .current_dir(&root)
        .args(["ls-tree", "--name-only", revision, "--", "runners"])
        .output()
        .unwrap();
    assert!(
        runners.status.success(),
        "{}",
        String::from_utf8_lossy(&runners.stderr)
    );
    let mut archive = Command::new("git");
    archive.current_dir(&root).args([
        "archive",
        revision,
        "Cargo.toml",
        "Cargo.lock",
        "crates",
        ".agents/skills",
    ]);
    if !runners.stdout.is_empty() {
        archive.arg("runners");
    }
    let archive = archive.output().unwrap();
    assert!(
        archive.status.success(),
        "{}",
        String::from_utf8_lossy(&archive.stderr)
    );
    let path = target.join("source.tar");
    std::fs::write(&path, archive.stdout).unwrap();
    assert!(
        Command::new("tar")
            .args(["-xf"])
            .arg(&path)
            .arg("-C")
            .arg(&source)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&source)
            .args(["build", "-p", "yard", "--locked", "--target-dir"])
            .arg(&target)
            .status()
            .unwrap()
            .success()
    );
    target.join("debug/yard")
}

/// G15/G8: the old real daemon leaves a planned parent with open proposals
/// and a live limit candidate. Upgrade preserves both operator exits without
/// giving ended history a branch. The old binary, fixture proposal and git
/// supply the schema, expected edge and candidate identities.
/// Sabotage: require target_branch() for an ended legacy proposer, or omit
/// the limit attention target backfill; acceptance or override is refused.
/// G6 sabotage: fabricate historical admission evidence from today's config
/// or the recovered candidate base. Both real historical rows must stay unknown.
/// G8 sabotage: lose IDs, payloads, resolutions or audit history when rebuilding
/// the attention kind constraint. Open and resolved proposals must survive two
/// reopens; the only limit payload change is the existing target backfill.
#[test]
fn an_upgrade_preserves_ended_planner_proposals_and_limit_overrides() {
    let machine = Machine::new("g15-upgrade", |request| {
        if request.has_tool("yard_publish_review") {
            return act(
                request,
                vec![publish(json!([{ "priority": "P0", "body": "block" }]))],
            );
        }
        if request.prompt().contains("Legacy plan") {
            return act(
                request,
                vec![
                    propose(json!({
                        "kind": "ticket", "title": "Legacy child", "parked": true
                    })),
                    propose(json!({
                        "kind": "ticket", "title": "Rejected legacy child", "parked": true
                    })),
                ],
            );
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.use_binary(Some(pre_target_binary()));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("max_rounds = 3", "max_rounds = 1"),
    );
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Legacy plan",
        "--workflow",
        "plan",
    ]);
    watch.until("legacy planner cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });
    let proposal = proposals(&project)[0].0;
    let rejected = proposals(&project)[1].0;
    project.json(&["proposal", "reject", &rejected.to_string()]);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let limit = watch.attention();
    assert_eq!(
        (&limit["data"]["kind"], &limit["data"]["reason"]),
        (&json!("stopped"), &json!("limit"))
    );
    // Quiescent upgrade: both workers ended and the planner's cleanup finished.
    assert!(
        project
            .rows("SELECT id FROM execution WHERE status = 'running'")
            .is_empty()
    );
    let old = project.rows("SELECT id, outcome FROM attempt ORDER BY id");
    assert_eq!(old[0]["outcome"], "planned");
    let limit_id = limit["attention"].as_i64().unwrap();
    let payload = serde_json::from_str::<Value>(
        project.rows(&format!(
            "SELECT payload FROM attention WHERE id = {limit_id}"
        ))[0]["payload"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert!(payload.get("target").is_none());
    machine.kill();
    let proposals_before =
        project.rows("SELECT * FROM attention WHERE kind = 'proposal' ORDER BY id");
    let limit_before = project.rows(&format!(
        "SELECT id, kind, reason, ticket, attempt, execution, state, resolution, created_at, resolved_at FROM attention WHERE id = {limit_id}"
    ));
    let audit_before = project.rows("SELECT * FROM audit ORDER BY seq");
    // G5/G6: preserve every column actually recorded by the historical binary,
    // including old proofs and handles. New scope stays unknown, never inferred.
    // Sabotage: drop or relabel an old execution/check during nullable rebuild.
    let execution_columns = project
        .rows("PRAGMA table_info(execution)")
        .iter()
        .map(|column| format!("\"{}\"", column["name"].as_str().unwrap()))
        .collect::<Vec<_>>()
        .join(", ");
    let execution_query = format!("SELECT {execution_columns} FROM execution ORDER BY id");
    let executions_before = project.rows(&execution_query);
    let check_columns = project
        .rows("PRAGMA table_info(\"check\")")
        .iter()
        .map(|column| format!("\"{}\"", column["name"].as_str().unwrap()))
        .collect::<Vec<_>>()
        .join(", ");
    let check_query = format!("SELECT {check_columns} FROM \"check\" ORDER BY id");
    let checks_before = project.rows(&check_query);
    machine.use_binary(None);
    machine.start();
    machine.kill();
    machine.start();
    assert_eq!(
        project.rows("SELECT * FROM attention WHERE kind = 'proposal' ORDER BY id"),
        proposals_before
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT id, kind, reason, ticket, attempt, execution, state, resolution, created_at, resolved_at FROM attention WHERE id = {limit_id}"
        )),
        limit_before
    );
    let mut expected_payload = payload.clone();
    expected_payload["target"] = json!("main");
    assert_eq!(
        serde_json::from_str::<Value>(
            project.rows(&format!(
                "SELECT payload FROM attention WHERE id = {limit_id}"
            ))[0]["payload"]
                .as_str()
                .unwrap()
        )
        .unwrap(),
        expected_payload
    );
    let audit_after = project.rows("SELECT * FROM audit ORDER BY seq");
    assert_eq!(&audit_after[..audit_before.len()], audit_before.as_slice());
    assert!(project.rows("PRAGMA foreign_key_check").is_empty());
    assert_eq!(project.rows(&execution_query), executions_before);
    assert_eq!(project.rows(&check_query), checks_before);
    assert!(
        project
            .rows("SELECT id FROM execution WHERE scope IS NOT NULL")
            .is_empty()
    );
    for ticket in ["Y-1", "Y-2"] {
        assert_eq!(
            project.json(&["attempt", "show", ticket])["attempt"]["admission"],
            json!({"status": "unknown"})
        );
    }
    let child = project.json(&["proposal", "accept", &proposal.to_string()]);
    assert_eq!(depends_on(&project, 1), [3]);
    assert_eq!(
        project.rows("SELECT target_branch FROM attempt WHERE ticket = 1"),
        vec![json!({ "target_branch": null })]
    );
    assert!(
        project.json(&["ticket", "show", child["ticket"].as_str().unwrap()])["target"].is_null()
    );
    let head = payload["head"].as_str().unwrap();
    let mut args = vec![
        "attempt", "approve", "Y-2", "--target", "main", "--head", head,
    ];
    if !payload["proof"].as_str().unwrap().is_empty() {
        args.extend(["--proof", payload["proof"].as_str().unwrap()]);
    }
    project.json(&args);
    let mut upgraded = project.watch(0);
    upgraded.event("landing.recorded", &[]);
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", head, "refs/heads/main"],
    );
    assert_eq!(
        project.rows("SELECT overrode FROM approval WHERE attempt = 2"),
        vec![json!({ "overrode": 1 })]
    );
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["state"], "done");
}

fn propose(arguments: Value) -> ToolCall {
    tool("yard_propose", arguments)
}

/// The proposals raised so far, in order: (attention id, payload).
fn proposals(project: &Project) -> Vec<(i64, Value)> {
    project
        .rows("SELECT id, payload FROM attention WHERE kind = 'proposal' ORDER BY id")
        .into_iter()
        .map(|row| {
            (
                row["id"].as_i64().unwrap(),
                serde_json::from_str(row["payload"].as_str().unwrap()).unwrap(),
            )
        })
        .collect()
}

fn depends_on(project: &Project, ticket: i64) -> Vec<i64> {
    project
        .rows(&format!(
            "SELECT depends_on FROM dependency WHERE ticket = {ticket} ORDER BY depends_on"
        ))
        .into_iter()
        .map(|row| row["depends_on"].as_i64().unwrap())
        .collect()
}

/// A ticket on `plan`: its clone refuses writes, it proposes children and a
/// body edit, and the attempt ends with nothing raised; accepting them
/// blocks the parent, which the scheduler does not start once they are
/// done; `yard ticket done` then closes it and its dependents become ready.
/// The children are proposed parked and closed by hand. A later planning
/// ticket runs to its end after the children are done, so a restart of the
/// parent would have come first. The write probe first proves `/workspace`
/// is the clone. Control: the dependent, a worker on a writable workflow in
/// the same project, runs the same probe and its write succeeds.
///
/// The later plan also proposes A and B, B depending on A, as the first
/// plan did: accepting both mints A first and B's edge names it, though
/// the first plan's A is already minted.
///
/// Sabotage: make `admit::scheduled` start read-only tickets again; the
/// parent is admitted once its children are done. Or mount the clone
/// writable for `access = "read-only"`; the plan's write succeeds. Or
/// accept an edit proposal at the ticket's current revision; the second
/// edit applies and overwrites the plan. Or make `admit::resolve_reference`
/// resolve a key from any execution's proposals; the later B's edge names
/// the first plan's A.
/// Or classify proposals with the default config: main's plan is writable
/// and write is read-only, the reverse of feature/a. The planner loses its
/// child edges or the writable worker gains one.
/// Or copy the parent target to a proposed child; its override is not absent.
#[test]
fn a_plan_proposes_children_that_block_it() {
    let answers = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen = answers.clone();
    let probe = "cd /workspace && test -d /workspace/.git && echo mounted; \
                 touch planted 2>/dev/null; echo \"write=$?\"";
    let machine = Machine::new("g15-plan", move |request| {
        if request.prompt().contains("Write here") {
            if request.opens() {
                return Reply::Tools(vec![
                    bash(&format!(
                        "{probe}; git add -A && git commit -q -m 'Add planted' && echo committed"
                    )),
                    propose(json!({ "kind": "ticket", "title": "Writable child", "parked": true })),
                ]);
            }
            seen.lock()
                .unwrap()
                .push(request.tool_results()[0].1.clone());
            return Reply::Text("done".into());
        }
        if request.prompt().contains("Plan more") {
            return match request.tool_results().len() {
                0 => Reply::Tools(vec![
                    propose(
                        json!({ "kind": "ticket", "key": "a", "title": "More A", "parked": true }),
                    ),
                    propose(json!({ "kind": "ticket", "key": "b", "title": "More B",
                                    "parked": true, "depends_on": ["a"] })),
                    publish_report(),
                ]),
                _ => Reply::Text("planned".into()),
            };
        }
        match request.tool_results().len() {
            0 => Reply::Tools(vec![
                bash(probe),
                propose(
                    json!({ "kind": "ticket", "key": "a", "title": "Child A", "parked": true }),
                ),
                propose(json!({ "kind": "ticket", "key": "b", "title": "Child B",
                                "parked": true, "depends_on": ["a"] })),
                propose(json!({ "kind": "edit", "body": "The plan: A, then B." })),
                propose(json!({ "kind": "edit", "body": "Stale: must not apply." })),
                publish_report(),
            ]),
            _ => {
                seen.lock()
                    .unwrap()
                    .push(request.tool_results()[0].1.clone());
                Reply::Text("planned".into())
            }
        }
    });
    machine.start();
    let feature = config("[workflows.write]\nreview = \"none\"\n");
    let main = feature
        .replace("access = \"read-only\"", "access = \"write\"")
        .replace(
            "[workflows.write]",
            "[workflows.write]\naccess = \"read-only\"",
        );
    let project = Project::new(&machine, "p", &main);
    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.write(".yard/config.toml", &feature);
    project.git(&["commit", "--quiet", "-am", "Feature workflows"]);
    project.json(&["sync", "feature/a"]);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Plan the release",
        "--target",
        "feature/a",
        "--workflow",
        "plan",
    ]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Write here",
        "--target",
        "feature/a",
        "--workflow",
        "write",
        "--depends-on",
        "Y-1",
    ]);
    let ended = watch.event("attempt.ended", &[("outcome", "planned")]);
    assert_eq!(ended["ticket"], "Y-1");
    let plan = answers.lock().unwrap()[0].clone();
    assert!(plan.contains("mounted"), "{plan}");
    assert!(
        plan.contains("write=") && !plan.contains("write=0"),
        "{plan}"
    );
    let raised: Vec<Value> = project
        .rows("SELECT kind FROM attention ORDER BY id")
        .into_iter()
        .map(|row| row["kind"].clone())
        .collect();
    assert_eq!(
        raised,
        [
            json!("proposal"),
            json!("proposal"),
            json!("proposal"),
            json!("proposal")
        ]
    );

    // Both edit proposals bind the revision the planning execution read.
    // The first accepted one applies; the second is stale and changes no row.
    let pending = proposals(&project);
    let stale = pending
        .iter()
        .find(|(_, payload)| payload["body"] == "Stale: must not apply.")
        .unwrap()
        .0;
    for (_, payload) in &pending {
        if payload["kind"] == "edit" {
            assert_eq!(payload["revision"], 1, "{payload}");
        }
    }
    // The proposals are raised in any order; accept Child A before Child B,
    // whose edge names it.
    let mut accept: Vec<&(i64, Value)> = pending.iter().filter(|(id, _)| *id != stale).collect();
    accept.sort_by_key(|(_, payload)| payload["title"].as_str().unwrap_or_default().to_owned());
    for (id, _) in accept {
        project.json(&["proposal", "accept", &id.to_string()]);
    }
    // The refusal changes no row. Cleanup executions for the ended plan
    // attempts land on their own schedule, so a `MAX(seq)` read would race
    // them; observe the body and revision the stale edit could change, and
    // the item that stays open.
    let refused = project.refused(&["proposal", "accept", &stale.to_string()]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], 1);
    assert_eq!(refused["data"]["current"], 2);
    let plan = project.json(&["ticket", "show", "Y-1"]);
    assert_eq!(
        (plan["body"].as_str(), plan["revision"].as_i64()),
        (Some("The plan: A, then B."), Some(2))
    );
    assert_eq!(
        project.rows(&format!("SELECT state FROM attention WHERE id = {stale}")),
        vec![json!({ "state": "open" })]
    );
    // Reject it so no open edit proposal blocks the parent's readiness; the
    // `admit::scheduled` sabotage below must be free to start it again.
    project.json(&["proposal", "reject", &stale.to_string()]);

    assert_eq!(depends_on(&project, 1), [3, 4]);
    for child in ["Y-3", "Y-4"] {
        project.json(&["ticket", "done", child, "--reason", "Done by hand"]);
    }
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Plan more",
        "--target",
        "feature/a",
        "--workflow",
        "plan",
    ]);
    watch.until("Y-5 planned", |event| {
        event["event"] == "attempt.ended" && event["ticket"] == "Y-5"
    });
    assert_eq!(
        project
            .rows("SELECT id FROM attempt WHERE ticket = 1")
            .len(),
        1,
        "the parent was started again"
    );
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE ticket = 2")
            .is_empty()
    );

    // The later plan's B must name its own accepted A, not the first plan's
    // A, already minted as Y-3.
    let raised = proposals(&project);
    let titled = |title: &str| {
        raised
            .iter()
            .find(|(_, payload)| payload["title"] == title)
            .unwrap()
            .0
    };
    let (more_a, more_b) = (titled("More A"), titled("More B"));
    let a = project.json(&["proposal", "accept", &more_a.to_string()]);
    let b = project.json(&["proposal", "accept", &more_b.to_string()]);
    let name = |value: &Value| {
        value["ticket"]
            .as_str()
            .unwrap()
            .trim_start_matches("Y-")
            .parse::<i64>()
            .unwrap()
    };
    assert_eq!(
        project.json(&["ticket", "show", a["ticket"].as_str().unwrap()])["title"],
        "More A"
    );
    assert_eq!(depends_on(&project, name(&b)), [name(&a)]);

    project.json(&["ticket", "done", "Y-1", "--reason", "Its children are done"]);
    let admitted = watch.event("attempt.admitted", &[]);
    assert_eq!(admitted["ticket"], "Y-2");

    let approval = watch.until("Y-2 approval", |event| {
        event["event"] == "attention.raised"
            && event["ticket"] == "Y-2"
            && event["data"]["kind"] == "approval"
    });
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let control = answers.lock().unwrap()[1].clone();
    assert!(control.contains("mounted"), "{control}");
    assert!(control.contains("write=0"), "{control}");

    let writable = proposals(&project)
        .into_iter()
        .find(|(_, payload)| payload["title"] == "Writable child")
        .unwrap()
        .0;
    let child = project.json(&["proposal", "accept", &writable.to_string()]);
    assert_eq!(depends_on(&project, 2), [1]);
    assert!(
        project.json(&["ticket", "show", child["ticket"].as_str().unwrap()])["target"].is_null()
    );

    // A waiting attempt is live even with no execution running. Sabotage:
    // key admit::ticket_close on running executions; this close succeeds.
    // Control: abandon that attempt and the same close succeeds.
    let attempt = project.rows("SELECT id FROM attempt WHERE ticket = 2")[0]["id"].clone();
    assert!(
        project
            .rows(&format!(
                "SELECT id FROM execution WHERE attempt = {attempt} AND status = 'running'"
            ))
            .is_empty()
    );
    let refused = project.refused(&["ticket", "done", "Y-2", "--reason", "Not needed"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["attempt"], attempt);
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["state"], "open");
    project.json(&["ticket", "park", "Y-2"]);
    project.json(&["attempt", "abandon", "Y-2"]);
    project.json(&["ticket", "done", "Y-2", "--reason", "Not needed"]);
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["state"], "done");
}

/// G5: a real pre-capacity binary dies with its clone Git child held before
/// any box/handle exists. The fixture proves that PID survives SIGKILL.
/// Upgrade retains the old row and capacity rather than guessing from an
/// absent effects lock. A prior candidate reaching approval is the allowed
/// control. Three attempt lanes admit a waiter beside both old attempts.
/// Releasing the known fixture child does not invent a proof on the old
/// row; a second startup reconsiders ownership after completion and keeps
/// the ambiguous upgrade stopped for the Y80 rehearsal.
/// Stop refuses the retained owner without claiming an undeliverable action.
/// Stopping an admitted waiter succeeds; deliberately starting it again must
/// wait on capacity before the old child is released.
/// Sabotage: backfill only reservations without their legacy lock contract,
/// or accept absence of the new lock as proof for a legacy execution,
/// including when reconciliation probes it after the old child finishes.
/// Acknowledge stop without a live handle; the refusal and missing successful
/// stop audit fail. End or release the retained owner on refusal; its rows change.
#[test]
fn an_upgrade_retains_a_legacy_pre_box_child_without_an_ownership_proof() {
    let machine = Machine::new("g5-legacy-child", |request| {
        act(
            request,
            vec![commit_file("control.txt", "control\n", "Control")],
        )
    });
    let armed = machine.root.join("armed");
    let said_path = machine.root.join("said");
    let release = machine.root.join("release");
    let done = machine.root.join("done");
    for path in [&said_path, &release, &done] {
        assert!(Command::new("mkfifo").arg(path).status().unwrap().success());
    }
    machine.wrapper("git", &format!(
        "if [ -e '{}' ]; then case \" $* \" in *' clone '*) echo $$ > '{}'; read line < '{}'; \"$REAL\" \"$@\"; result=$?; echo done > '{}'; exit \"$result\";; esac; fi",
        armed.display(), said_path.display(), release.display(), done.display()));
    machine.use_binary(Some(pre_target_binary()));
    machine.start();
    let configured = config("").replace("review = [\"correctness\"]", "review = \"none\"");
    let project = Project::new(&machine, "p", &configured);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Control"]);
    let control = watch.attention();
    assert_eq!(control["data"]["kind"], "approval");
    std::fs::write(&armed, "").unwrap();
    project.json(&["ticket", "new", "--title", "Legacy"]);
    let pid = watch.said(&said_path).trim().to_string();
    drop(watch);
    machine.kill();
    assert!(
        Command::new("kill")
            .args(["-0", &pid])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        project.rows("SELECT id, status, handle FROM execution WHERE id = 2"),
        vec![json!({"id": 2, "status": "running", "handle": null})]
    );
    std::fs::remove_file(&armed).unwrap();
    machine.use_binary(None);
    machine.start();
    let mut watch = project.watch(0);
    let unknown = watch.find("legacy ownership attention", |event| {
        event["event"] == "attention.raised" && event["execution"] == 2
    });
    assert_eq!(unknown["data"]["kind"], "red");
    assert_eq!(unknown["data"]["reason"], "ownership");
    assert_eq!(unknown["data"]["payload"]["lock_contract"], false);
    project.write(
        ".yard/config.toml",
        &configured.replace("max_lanes = 2", "max_lanes = 3\nmax_executions = 1"),
    );
    git(&project.path, &["add", ".yard/config.toml"]);
    git(&project.path, &["commit", "-qm", "Set shared capacity"]);
    project.json(&["sync"]);
    project.json(&["ticket", "new", "--title", "Waiter"]);
    let waiting = watch.event("execution.waiting", &[]);
    assert_eq!(waiting["ticket"], "Y-3");
    assert_eq!(waiting["data"]["reason"], "capacity");
    let waiter = waiting["execution"].as_i64().unwrap();
    assert_eq!(
        project.json(&["attempt", "stop", "Y-3"])["stopping"],
        waiter
    );
    let stopped = watch.find("waiter stopped", |event| {
        event["event"] == "attention.raised" && event["execution"] == waiter
    });
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    assert_eq!(
        project.rows(&format!(
            "SELECT status, reserved, outcome FROM execution WHERE id = {waiter}"
        )),
        vec![json!({"status": "ended", "reserved": 0, "outcome": "unchanged"})]
    );
    project.json(&["attempt", "start", "Y-3"]);
    let renewed = watch.find("renewed capacity wait", |event| {
        event["event"] == "execution.waiting" && event["execution"] != waiter
    });
    assert_eq!(renewed["ticket"], "Y-3");
    assert_eq!(renewed["data"]["reason"], "capacity");
    std::fs::write(&release, "go\n").unwrap();
    assert_eq!(watch.said(&done), "done\n");
    drop(watch);
    machine.kill();
    machine.start();
    // Serving and this supported read occur after startup reconciliation.
    // Historical ownership attention alone cannot prove reconsideration.
    let status = project.json(&["status"]);
    let retained = status["attention"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["execution"] == 2 && item["reason"] == "ownership")
        .expect("the legacy owner still needs attention after reconciliation");
    assert_eq!(retained["kind"], "red");
    assert_eq!(retained["execution"], 2);
    assert_eq!(retained["reason"], "ownership");
    assert_eq!(
        project.rows("SELECT status, reserved, lock_contract FROM execution WHERE id = 2"),
        vec![json!({"status": "running", "reserved": 1, "lock_contract": 0})]
    );
    let before = project.rows("SELECT * FROM execution WHERE id = 2");
    let attempt_before = project.rows("SELECT * FROM attempt WHERE ticket = 2");
    let refused = project.refused(&["attempt", "stop", "Y-2"]);
    assert_eq!(refused["code"], "refused");
    assert_eq!(refused["data"]["reason"], "ownership");
    assert_eq!(refused["data"]["ticket"], "Y-2");
    assert_eq!(refused["data"]["execution"], 2);
    assert_eq!(refused["data"]["attempt"], attempt_before[0]["id"]);
    assert_eq!(refused["data"]["attention"], retained["attention"]);
    assert_eq!(project.rows("SELECT * FROM execution WHERE id = 2"), before);
    assert_eq!(
        project.rows("SELECT * FROM attempt WHERE ticket = 2"),
        attempt_before
    );
    assert!(
        project
            .rows("SELECT seq FROM audit WHERE event = 'attempt.stopped' AND execution = 2")
            .is_empty()
    );
    assert!(
        !machine
            .model
            .requests()
            .iter()
            .any(|r| r.prompt().contains("Waiter"))
    );
}
