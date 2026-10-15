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

/// The pre-target executable predates operator MCP. Its own CLI creates
/// genuine historical state; successor calls never enter this setup.
fn legacy_project<'a>(machine: &'a Machine, config: &str) -> Project<'a> {
    let project = Project::checkout(machine, "p");
    machine.legacy_cli(&project.path, &["init"]);
    project.write(".yard/config.toml", config);
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "yard"]);
    machine.legacy_cli(&project.path, &["sync"]);
    project
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
    let project = legacy_project(
        &machine,
        &config("").replace("max_rounds = 3", "max_rounds = 1"),
    );
    let mut watch = machine.legacy_watch(&project.path, 0);
    machine.legacy_cli(
        &project.path,
        &[
            "ticket",
            "new",
            "--title",
            "Legacy plan",
            "--workflow",
            "plan",
        ],
    );
    watch.until("legacy planner cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });
    let proposal = proposals(&project)[0].0;
    let rejected = proposals(&project)[1].0;
    machine.legacy_cli(
        &project.path,
        &["proposal", "reject", &rejected.to_string()],
    );
    machine.legacy_cli(&project.path, &["ticket", "new", "--title", "Add feature"]);
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
            project.ok("attempt.show", json!({"ticket": ticket}))["attempt"]["admission"],
            json!({"status": "unknown"})
        );
    }
    let child = project.ok("proposal.accept", json!({"attention": proposal}));
    assert_eq!(depends_on(&project, 1), [3]);
    assert_eq!(
        project.rows("SELECT target_branch FROM attempt WHERE ticket = 1"),
        vec![json!({ "target_branch": null })]
    );
    assert!(
        project.ok(
            "ticket.show",
            json!({"ticket": child["ticket"].as_str().unwrap()})
        )["target"]
            .is_null()
    );
    let head = payload["head"].as_str().unwrap();
    let mut args = json!({"ticket": "Y-2", "target": "main", "head": head});
    if !payload["proof"].as_str().unwrap().is_empty() {
        args["proof"] = payload["proof"].clone();
    }
    project.ok("attempt.approve", args);
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
    assert_eq!(
        project.ok("ticket.show", json!({"ticket": "Y-2"}))["state"],
        "done"
    );
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
/// done; `yard_ticket_done` then closes it and its dependents become ready.
/// G15: a read-only planner proposes children and a body edit. Same-run
/// sibling dependencies block the parent; stale edits write nothing. A writable
/// dependent keeps its own authority, and closing a live waiter refuses.
/// Fixture proposals and Git/file effects supply expectations.
/// Sabotage: allow planner writes, drop a declared dependency, accept a stale
/// revision or close a live attempt before its work ends.
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
    project.ok("sync", json!({"branch": "feature/a"}));
    let mut watch = project.watch(0);
    project.ok(
        "ticket.new",
        json!({"title": "Plan the release", "target": "feature/a", "workflow": "plan"}),
    );
    project.ok("ticket.new", json!({"title": "Write here", "target": "feature/a", "workflow": "write", "depends_on": ["Y-1"]}));
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
        project.ok("proposal.accept", json!({"attention": id}));
    }
    // The refusal changes no row. Cleanup executions for the ended plan
    // attempts land on their own schedule, so a `MAX(seq)` read would race
    // them; observe the body and revision the stale edit could change, and
    // the item that stays open.
    let refused = project.refused("proposal.accept", json!({"attention": stale}));
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], 1);
    assert_eq!(refused["data"]["current"], 2);
    let plan = project.ok("ticket.show", json!({"ticket": "Y-1"}));
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
    project.ok("proposal.reject", json!({"attention": stale}));

    assert_eq!(depends_on(&project, 1), [3, 4]);
    for child in ["Y-3", "Y-4"] {
        project.ok(
            "ticket.done",
            json!({"reason": "Done by hand", "ticket": child}),
        );
    }

    project.ok(
        "ticket.done",
        json!({"reason": "Its children are done", "ticket": "Y-1"}),
    );
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
    let child = project.ok("proposal.accept", json!({"attention": writable}));
    assert_eq!(depends_on(&project, 2), [1]);
    assert!(
        project.ok(
            "ticket.show",
            json!({"ticket": child["ticket"].as_str().unwrap()})
        )["target"]
            .is_null()
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
    let refused = project.refused(
        "ticket.done",
        json!({"reason": "Not needed", "ticket": "Y-2"}),
    );
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["attempt"], attempt);
    assert_eq!(
        project.ok("ticket.show", json!({"ticket": "Y-2"}))["state"],
        "open"
    );
    project.ok("ticket.park", json!({"ticket": "Y-2"}));
    project.ok("attempt.abandon", json!({"ticket": "Y-2"}));
    project.ok(
        "ticket.done",
        json!({"reason": "Not needed", "ticket": "Y-2"}),
    );
    assert_eq!(
        project.ok("ticket.show", json!({"ticket": "Y-2"}))["state"],
        "done"
    );
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
    let project = legacy_project(&machine, &configured);
    let mut watch = machine.legacy_watch(&project.path, 0);
    machine.legacy_cli(&project.path, &["ticket", "new", "--title", "Control"]);
    let control = watch.attention();
    assert_eq!(control["data"]["kind"], "approval");
    std::fs::write(&armed, "").unwrap();
    machine.legacy_cli(&project.path, &["ticket", "new", "--title", "Legacy"]);
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
    project.ok("sync", json!({}));
    project.ok("ticket.new", json!({"title": "Waiter"}));
    let waiting = watch.event("execution.waiting", &[]);
    assert_eq!(waiting["ticket"], "Y-3");
    assert_eq!(waiting["data"]["reason"], "capacity");
    std::fs::write(&release, "go\n").unwrap();
    assert_eq!(watch.said(&done), "done\n");
    drop(watch);
    machine.kill();
    machine.start();
    // Serving and this supported read occur after startup reconciliation.
    // Historical ownership attention alone cannot prove reconsideration.
    let status = project.ok("status", json!({}));
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
    let refused = project.refused("attempt.stop", json!({"ticket": "Y-2"}));
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
