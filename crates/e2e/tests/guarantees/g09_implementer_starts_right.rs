//! G9: Each implementer execution starts from the right place.

use e2e::*;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn unreviewed(extra: &str) -> String {
    config(extra).replace("review = [\"correctness\"]", "review = \"none\"")
}

/// The opening request of each implementer execution, in order.
fn openings(machine: &Machine) -> Vec<ModelRequest> {
    machine
        .model
        .requests()
        .into_iter()
        .filter(|request| request.opens() && !request.has_tool("yard_publish_review"))
        .collect()
}

fn edit_body(project: &Project, body: &str) {
    let revision = project.ok("ticket.show", json!({"ticket": "Y-1"}))["revision"]
        .as_i64()
        .unwrap();
    project.ok(
        "ticket.edit",
        json!({"revision": revision, "body": body, "ticket": "Y-1"}),
    );
}

fn reasons(project: &Project) -> Vec<Value> {
    project
        .rows("SELECT reason FROM execution WHERE kind = 'implementation' AND attempt IN (SELECT id FROM attempt WHERE ticket = 1) ORDER BY id")
        .into_iter()
        .map(|row| row["reason"].clone())
        .collect()
}

fn assert_operator_diff(prompt: &str, removed: &[&str], added: &[&str]) {
    let mut lines = prompt.lines().peekable();
    let mut annotation = "";
    while let Some(line) = lines.next() {
        if !line.starts_with("diff --git ") {
            if !line.trim().is_empty() {
                annotation = line;
            }
            continue;
        }
        let diff: Vec<_> = std::iter::from_fn(|| {
            lines.next_if(|line| {
                line.starts_with([' ', '+', '-', '\\'])
                    || line.starts_with("index ")
                    || line.starts_with("@@ ")
            })
        })
        .collect();
        let contains = |prefix: char, identities: &[&str]| {
            identities.iter().all(|identity| {
                diff.iter().any(|line| {
                    line.starts_with(prefix)
                        && !line.starts_with("--- ")
                        && !line.starts_with("+++ ")
                        && line.contains(identity)
                })
            })
        };
        if contains('-', removed) && contains('+', added) {
            assert!(
                annotation
                    .split(|character: char| !character.is_alphanumeric() && character != '_')
                    .any(|word| word.eq_ignore_ascii_case("operator")),
                "diff must attribute the change to operator: {annotation:?}"
            );
            return;
        }
        annotation = "";
    }
    panic!("missing operator diff deleting {removed:?} and adding {added:?}: {prompt}");
}

/// Two projects on one machine. In `p` an edit mid-execution lets the
/// execution end on its own and reaches the next prompt as the full body
/// after changed definitions start a fresh session. The held worker commits
/// only once released, so its execution ends
/// as a candidate where a stop would end it unchanged. No gate runs on that
/// candidate before the next execution, which opens with the whole body
/// and a labelled operator plan diff. In `q`, `stop` delivers an edit
/// sooner: the worker is still held when the execution ends, `stop` still
/// raises `stopped` and starts nothing, and that item's `start` runs the
/// implementer with the edit's diff.
///
/// In `p`, a real worker holds the sole capacity slot before Y-1 starts.
/// Y-1's unreserved execution waits while consumed definitions, model/effort
/// and staged bytes are adopted without a plan or ticket change.
/// Releasing the owner lets that same execution open with its original
/// instructions. It commits; its source-only successor has the identical plan,
/// opens with adopted values and an operator diff, and is held before a real
/// second commit. Only then are its instruction profile and ticket revised.
/// A fresh third execution carrying the whole `Rename` body commits again
/// before any gate runs.
///
/// Sabotage: make `admit::steer` notify the stop signal of a running
/// implementer; the held execution ends unchanged. Load the execution from
/// the latest plan by replacing only the startup `attempts::for_execution`
/// in `supervise::implement` with `attempts::get`: the initial model prompt
/// after the capacity wait loses the original model, effort and staged bytes
/// in the source-only phase. Omit the
/// successor revision diff; the next model prompt fails. Make
/// `implementer_prompt` change only the plan heading to `Plan revision:`
/// while retaining the ticket actor heading and complete diff; the local
/// attribution assertion fails. Remove only the plan diff payload while
/// retaining its actor heading; the diff-change observer fails. Reword only
/// that heading to `Operator plan update:`; both observations still pass.
/// Or make
/// `supervise::implement` ignore the edit after a candidate; the first head
/// reaches approval and the edit is never delivered. Make
/// `admit::attempt_stop` skip notifying the execution; it runs on and no
/// `stopped` is raised. Let `supervise` schedule the edit instead of raising
/// `stopped` for a stopped run; no `stopped` item comes.
/// Launch every execution with the default worker's settings: q's host model
/// identity and recorded settings fail. Changed revision settings and
/// definition-only changes are adopted independently while p waits. Sabotages:
/// return the original attempt implementer from implementation_settings;
/// load the current attempt at held old startup; delete the consumed-definition
/// map from plan_handoff; or return no handoff when the actual predecessor and
/// successor plan objects are equal. Host model/system observations and Git
/// blobs catch each independently; instruction diffs alone cannot prove these bindings.
/// Omit the ticket body from a fresh session, or send only its resumed diff;
/// p's whole-body line assertion fails after the instruction definition changes.
#[test]
fn an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner() {
    let (edit_hold, stop_hold) = (Latch::new(), Latch::new());
    let owner_hold = Latch::new();
    let owner_held = owner_hold.clone();
    let holds = (edit_hold.clone(), stop_hold.clone());
    let successor_hold = Latch::new();
    let successor_held = successor_hold.clone();
    let source_successor = Arc::new(AtomicBool::new(false));
    let successor_phase = source_successor.clone();
    let machine = Machine::new("g9-edit", move |request| {
        if request.prompt().contains("Hold capacity") {
            return if request.opens() {
                Reply::Hold(owner_held.clone(), Box::new(Reply::Text("done".into())))
            } else {
                Reply::Text("done".into())
            };
        }
        if request.last_user().contains("Rename") {
            return act(
                request,
                vec![commit_file("renamed.txt", "renamed\n", "Rename")],
            );
        }
        if request.prompt().contains("Add feature") && successor_phase.load(Ordering::Acquire) {
            let reply = act(
                request,
                vec![commit_file("adopted.txt", "adopted\n", "Adopt source")],
            );
            return if request.opens() {
                Reply::Hold(successor_held.clone(), Box::new(reply))
            } else {
                reply
            };
        }
        let hold = if request.prompt().contains("Add widget") {
            &holds.1
        } else {
            &holds.0
        };
        if request.opens() {
            return Reply::Hold(
                hold.clone(),
                Box::new(Reply::Tools(vec![commit_file(
                    "feature.txt",
                    "feature\n",
                    "Add feature",
                )])),
            );
        }
        Reply::Text("done".into())
    });
    machine.start();
    let gate = "[gates.check]\ncommand = \"true\"\nstage = \"candidate\"\nruns_in = \"host\"\n";
    let profile = "[profiles.revised]\nrunner = {kind = \"agent\", agent = \"worker\", instructions = \"Revised implementation.\"}\naccess = \"write-allowed\"\noutputs = {result = \"candidate\"}\noutcomes = [\"complete\"]\n";
    let configuration = unreviewed(&format!("{gate}{profile}"))
        .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1")
        .replace(
            "implementer = \"worker\"",
            "implementer = \"worker\"\ninstructions = \"Original implementation.\"",
        )
        .replace(
            "model = \"fake-model\"",
            "model = \"fake-model\"\neffort = \"low\"",
        );
    let edited = Project::new(&machine, "p", &configuration);
    let staged_path = ".pi/extensions/startup/index.ts";
    let stage = |version: &str| {
        edited.write(staged_path, &format!(
            "export default function (pi) {{ pi.on('before_agent_start', async (event) => ({{systemPrompt: event.systemPrompt + '\\nstartup-bytes-{version} effort=' + pi.getThinkingLevel()}})); }}\n"
        ));
    };
    stage("v1");
    edited.git(&["add", "-A"]);
    edited.git(&["commit", "-qm", "Original startup input"]);
    edited.ok("sync", json!({}));
    let original_commit = edited.git(&["rev-parse", "HEAD"]).trim().to_string();
    let original_blob = edited
        .git(&["rev-parse", &format!("HEAD:{staged_path}")])
        .trim()
        .to_string();
    let stopped = Project::new(&machine, "q", &unreviewed(
        "[agents.stopper]\nharness = \"pi\"\nprovider = \"openrouter\"\nmodel = \"stopped-model\"\neffort = \"high\"\n",
    ).replace("implementer = \"worker\"", "implementer = \"stopper\""));
    let mut edit_watch = edited.watch(0);
    let mut stop_watch = stopped.watch(0);
    edited.ok(
        "ticket.new",
        json!({"title": "Add feature", "parked": true}),
    );
    edited.ok("ticket.new", json!({"title": "Hold capacity"}));
    owner_hold.wait_held();
    edited.ok("ticket.unpark", json!({"ticket": "Y-1"}));
    let waiting = edit_watch.until("Y-1 waits before implementation startup", |event| {
        event["event"] == "execution.waiting"
            && event["ticket"] == "Y-1"
            && event["data"]["reason"] == "capacity"
    });
    stopped.ok("ticket.new", json!({"title": "Add widget"}));
    stop_hold.wait_held();

    let execution = waiting["execution"].as_i64().unwrap();
    let frozen_query = format!(
        "SELECT id, attempt, plan_revision, inputs, agent, harness, provider, model, effort FROM execution WHERE id = {execution}"
    );
    let frozen = edited.rows(&frozen_query);
    assert_eq!(frozen[0]["plan_revision"], 1);
    assert_eq!(frozen[0]["agent"], "worker");
    assert_eq!(frozen[0]["harness"], "pi");
    assert_eq!(frozen[0]["provider"], "openrouter");
    assert_eq!(frozen[0]["model"], "fake-model");
    assert_eq!(frozen[0]["effort"], "low");
    let original_query = format!(
        "SELECT implementer, admission FROM attempt WHERE id = {}",
        frozen[0]["attempt"]
    );
    let original = edited.rows(&original_query);
    assert_eq!(
        edited.rows(&format!(
            "SELECT status, reserved FROM execution WHERE id = {execution}"
        )),
        vec![json!({"status": "running", "reserved": 0})]
    );
    let mut operator = machine.operator();
    operator.initialize();
    edited.write(
        ".yard/config.toml",
        &configuration
            .replace("model = \"fake-model\"", "model = \"adopted-model\"")
            .replace("effort = \"low\"", "effort = \"high\"")
            .replace("command = \"true\"", "command = \"test -f feature.txt\""),
    );
    stage("v2");
    edited.git(&["add", "-A"]);
    edited.git(&[
        "commit",
        "-qm",
        "Adopted settings definitions and startup input",
    ]);
    edited.ok("sync", json!({}));
    let adopted_commit = edited.git(&["rev-parse", "HEAD"]).trim().to_string();
    let adopted_blob = edited
        .git(&["rev-parse", &format!("HEAD:{staged_path}")])
        .trim()
        .to_string();
    let adopted = operator.call(
        "yard_attempt_revise",
        json!({
            "project": edited.path, "ticket": "Y-1", "plan_revision": 1,
            "definition_commit": adopted_commit, "staging_commit": adopted_commit
        }),
    );
    assert_eq!(adopted["result"]["plan_revision"], 2, "{adopted}");
    assert_eq!(edited.rows(&frozen_query), frozen);
    owner_hold.release();
    let owner_end = edit_watch.until("capacity owner stops", |event| {
        event["event"] == "attention.raised" && event["ticket"] == "Y-2"
    });
    assert_eq!(owner_end["data"]["kind"], "stopped", "{owner_end}");
    edit_hold.wait_held();
    let initial_request = openings(&machine)
        .into_iter()
        .find(|request| request.prompt().contains("Add feature"))
        .unwrap();
    assert_eq!(initial_request.body["model"], "fake-model");
    assert!(
        initial_request
            .system()
            .contains("startup-bytes-v1 effort=low")
    );
    assert!(!initial_request.system().contains("startup-bytes-v2"));
    let initial = initial_request.prompt();
    assert!(initial.contains("Original implementation."), "{initial}");
    assert!(!initial.contains("Revised implementation."), "{initial}");
    assert_eq!(edited.rows(&frozen_query), frozen);
    assert_eq!(edited.rows(&original_query), original);
    // Advance only after judging the actual initial opening. Routing must not
    // depend on the model, effort or staged bytes those assertions judge.
    source_successor.store(true, Ordering::Release);
    edit_hold.release();
    successor_hold.wait_held();
    // Host route holds the successor before its commit. Checked predecessor
    // release must already be complete; subsequent work cannot rewrite it.
    let ended_query = format!("SELECT * FROM execution WHERE id = {execution}");
    let ended = edited.rows(&ended_query);
    assert_eq!(ended[0]["teardown"], 1);
    assert_eq!(ended[0]["reserved"], 0);
    assert_eq!(ended[0]["outcome"], "candidate");
    let revisions_query = format!(
        "SELECT * FROM plan_revision WHERE attempt = {} AND revision = 2",
        frozen[0]["attempt"]
    );
    let revisions = edited.rows(&revisions_query);
    // Bind equality to the two executions that actually started, not to
    // unexecuted intermediate revisions. No instruction/profile edit yet.
    assert_eq!(edited.rows(&format!("SELECT plan_revision, ticket_revision FROM execution WHERE attempt = {} AND kind = 'implementation' ORDER BY id", frozen[0]["attempt"])),
        vec![json!({"plan_revision": 1, "ticket_revision": 1}), json!({"plan_revision": 2, "ticket_revision": 1})]);
    let admission: Value =
        serde_json::from_str(original[0]["admission"].as_str().unwrap()).unwrap();
    let source_revision: Value =
        serde_json::from_str(revisions[0]["evidence"].as_str().unwrap()).unwrap();
    assert_eq!(admission["plan"], source_revision["plan"]);
    let second_request = openings(&machine).last().unwrap().clone();
    assert_eq!(second_request.body["model"], "adopted-model");
    assert!(
        second_request
            .system()
            .contains("startup-bytes-v2 effort=high")
    );
    assert_operator_diff(
        &second_request.last_user(),
        &[&original_blob, "fake-model"],
        &[&adopted_blob, "adopted-model", "test -f feature.txt"],
    );
    let revised = operator.call(
        "yard_attempt_revise",
        json!({
            "project": edited.path, "ticket": "Y-1", "plan_revision": 2,
            "overrides": {"implementer": {"profile": "revised"}}
        }),
    );
    assert_eq!(revised["result"]["plan_revision"], 3, "{revised}");
    edit_body(&edited, "Rename it too");
    successor_hold.release();
    edit_body(&stopped, "Rename it now");
    stopped.ok("attempt.stop", json!({"ticket": "Y-1"}));

    let approval = edit_watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    assert_eq!(
        reasons(&edited),
        [json!("first"), json!("edit"), json!("edit")]
    );
    let first = edit_watch.find("the first execution ended", |event| {
        event["event"] == "execution.ended" && event["execution"] == waiting["execution"]
    });
    assert_eq!(first["data"]["outcome"], "candidate", "{first}");
    assert_eq!(edited.rows(&original_query), original);
    assert_eq!(edited.rows(&ended_query), ended);
    assert_eq!(edited.rows(&revisions_query), revisions);
    assert_eq!(
        edited.rows("SELECT agent, model, effort FROM execution WHERE kind = 'implementation' AND attempt IN (SELECT id FROM attempt WHERE ticket = 1) ORDER BY id"),
        vec![json!({"agent": "worker", "model": "fake-model", "effort": "low"}),
             json!({"agent": "worker", "model": "adopted-model", "effort": "high"}),
             json!({"agent": "worker", "model": "adopted-model", "effort": "high"})]
    );
    // The latest opening is p's third: q opens nothing more until its start.
    let third = openings(&machine).last().unwrap().last_user();
    assert!(third.lines().any(|line| line == "Rename it too"), "{third}");
    assert_operator_diff(
        &third,
        &["Original implementation."],
        &["Revised implementation."],
    );
    assert_eq!(
        edited
            .rows("SELECT plan_revision FROM execution WHERE kind = 'implementation' AND attempt IN (SELECT id FROM attempt WHERE ticket = 1) ORDER BY id"),
        vec![json!({"plan_revision": 1}), json!({"plan_revision": 2}), json!({"plan_revision": 3})]
    );
    assert_eq!(edited.rows(&frozen_query), frozen);
    let attempt = frozen[0]["attempt"].as_i64().unwrap();
    let current = &adopted["result"]["plan"]["inputs"];
    for (inputs, blob) in [
        (&admission["inputs"], &original_blob),
        (current, &adopted_blob),
    ] {
        let staged = inputs["trusted_staged_sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["path"] == staged_path)
            .unwrap();
        assert_eq!(staged["blob"], *blob);
    }
    assert_eq!(current["image_commit"], original_commit);
    for (key, revision) in [
        ("build_context_tree", format!("{original_commit}^{{tree}}")),
        (
            "dockerfile_blob",
            format!("{original_commit}:.yard/Dockerfile"),
        ),
    ] {
        assert_eq!(current[key], edited.git(&["rev-parse", &revision]).trim());
    }
    assert_eq!(current["definition_commit"], adopted_commit);
    assert_eq!(current["staging_commit"], adopted_commit);
    let clone = edited
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    assert_eq!(git(&clone, &["show", "HEAD:adopted.txt"]), "adopted\n");
    assert_eq!(git(&clone, &["show", "HEAD:renamed.txt"]), "renamed\n");
    // No gate ran on either earlier candidate: every gate ran on the third head.
    // Sabotage: retain the first candidate head after delivering the edit;
    // approval and gates agree with each other but differ from the worker's Git.
    assert_eq!(
        edited.rows("SELECT DISTINCT head FROM execution WHERE kind = 'gate'"),
        vec![json!({ "head": head })]
    );

    let item = stop_watch.attention();
    assert_eq!(item["data"]["kind"], "stopped", "{item}");
    assert!(stop_hold.is_held(), "the worker's request was answered");
    assert_eq!(reasons(&stopped), [json!("first")]);
    stopped.ok("attempt.start", json!({"ticket": "Y-1"}));
    let approval = stop_watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    assert_eq!(reasons(&stopped), [json!("first"), json!("restart")]);
    // The latest opening is q's second: p reached approval before it.
    let second = openings(&machine).last().unwrap().last_user();
    assert!(second.contains("+Rename it now"), "{second}");
    assert_eq!(
        openings(&machine).last().unwrap().body["model"],
        "stopped-model"
    );
    assert_eq!(
        stopped.rows(
            "SELECT agent, model, effort FROM execution WHERE kind = 'implementation' ORDER BY id"
        ),
        vec![json!({"agent": "stopper", "model": "stopped-model", "effort": "high"}); 2]
    );
    stop_hold.release();
}
