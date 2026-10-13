//! G9: Each implementer execution starts from the right place.

use e2e::*;
use serde_json::{Value, json};

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
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"].to_string();
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        body,
    ]);
}

fn reasons(project: &Project) -> Vec<Value> {
    project
        .rows("SELECT reason FROM execution WHERE kind = 'implementation' AND attempt IN (SELECT id FROM attempt WHERE ticket = 1) ORDER BY id")
        .into_iter()
        .map(|row| row["reason"].clone())
        .collect()
}

/// Two projects on one machine. In `p` an edit mid-execution lets the
/// execution end on its own and reaches the next prompt as a diff of the
/// body: the held worker commits only once released, so its execution ends
/// as a candidate where a stop would end it unchanged. No gate runs on that
/// candidate before the next execution, which opens with the edit's diff
/// and is labelled as the operator's. In `q`, `stop` delivers an edit
/// sooner: the worker is still held when the execution ends, `stop` still
/// raises `stopped` and starts nothing, and that item's `start` runs the
/// implementer with the edit's diff.
///
/// In `p`, a real worker holds the sole capacity slot before Y-1 starts.
/// Y-1's unreserved execution waits while its instruction profile is revised.
/// Releasing the owner lets that same execution open with its original
/// instructions. Its model request is then held for the ticket edit and,
/// once released, commits; a later prompt carrying `Rename` commits again.
///
/// Sabotage: make `admit::steer` notify the stop signal of a running
/// implementer; the held execution ends unchanged. Load the execution from
/// the latest plan by replacing only the startup `attempts::for_execution`
/// in `supervise::implement` with `attempts::get`: the initial model prompt
/// after the capacity wait loses the original instructions. Omit the
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
/// definition-only handoff sabotages need the dependent source adoption API;
/// this scenario uses only supported instruction revisions.
#[test]
fn an_edit_reaches_the_next_prompt_and_stop_delivers_it_sooner() {
    let (edit_hold, stop_hold) = (Latch::new(), Latch::new());
    let owner_hold = Latch::new();
    let owner_held = owner_hold.clone();
    let holds = (edit_hold.clone(), stop_hold.clone());
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
    let edited = Project::new(
        &machine,
        "p",
        &unreviewed(&format!("{gate}{profile}"))
            .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1")
            .replace(
                "implementer = \"worker\"",
                "implementer = \"worker\"\ninstructions = \"Original implementation.\"",
            )
            .replace(
                "model = \"fake-model\"",
                "model = \"fake-model\"\neffort = \"low\"",
            ),
    );
    let stopped = Project::new(&machine, "q", &unreviewed(
        "[agents.stopper]\nharness = \"pi\"\nprovider = \"openrouter\"\nmodel = \"stopped-model\"\neffort = \"high\"\n",
    ).replace("implementer = \"worker\"", "implementer = \"stopper\""));
    let mut edit_watch = edited.watch(0);
    let mut stop_watch = stopped.watch(0);
    edited.json(&["ticket", "new", "--title", "Add feature", "--parked"]);
    edited.json(&["ticket", "new", "--title", "Hold capacity"]);
    owner_hold.wait_held();
    edited.json(&["ticket", "unpark", "Y-1"]);
    let waiting = edit_watch.until("Y-1 waits before implementation startup", |event| {
        event["event"] == "execution.waiting"
            && event["ticket"] == "Y-1"
            && event["data"]["reason"] == "capacity"
    });
    stopped.json(&["ticket", "new", "--title", "Add widget"]);
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
    let revised = operator.call(
        "yard_attempt_revise",
        json!({
        "project": edited.path, "ticket": "Y-1", "plan_revision": 1,
        "overrides": {"implementer": {"profile": "revised"}}}),
    );
    assert_eq!(revised["result"]["plan_revision"], 2, "{revised}");
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
    let initial = initial_request.prompt();
    assert!(initial.contains("Original implementation."), "{initial}");
    assert!(!initial.contains("Revised implementation."), "{initial}");
    assert_eq!(edited.rows(&frozen_query), frozen);
    assert_eq!(edited.rows(&original_query), original);
    edit_body(&edited, "Rename it too");
    edit_hold.release();
    edit_body(&stopped, "Rename it now");
    stopped.json(&["attempt", "stop", "Y-1"]);

    let approval = edit_watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    assert_eq!(reasons(&edited), [json!("first"), json!("edit")]);
    let first = edit_watch.find("the first execution ended", |event| {
        event["event"] == "execution.ended" && event["execution"] == waiting["execution"]
    });
    assert_eq!(first["data"]["outcome"], "candidate", "{first}");
    assert_eq!(edited.rows(&original_query), original);
    assert_eq!(
        edited.rows("SELECT agent, model, effort FROM execution WHERE kind = 'implementation' AND attempt IN (SELECT id FROM attempt WHERE ticket = 1) ORDER BY id"),
        vec![json!({"agent": "worker", "model": "fake-model", "effort": "low"}); 2]
    );
    assert_eq!(
        openings(&machine).last().unwrap().body["model"],
        "fake-model"
    );
    // The latest opening is p's second: q opens nothing more until its start.
    let second = openings(&machine).last().unwrap().last_user();
    assert!(second.contains("+Rename it too"), "{second}");
    let mut lines = second.lines().peekable();
    let mut annotation = "";
    let mut plan_annotation = None;
    while let Some(line) = lines.next() {
        if !line.starts_with("diff --git ") {
            if !line.trim().is_empty() {
                annotation = line;
            }
            continue;
        }
        // Consume native Git lines so no previous diff supplies the actor.
        let diff: Vec<_> = std::iter::from_fn(|| {
            lines.next_if(|line| {
                line.starts_with([' ', '+', '-', '\\'])
                    || line.starts_with("index ")
                    || line.starts_with("@@ ")
            })
        })
        .collect();
        if diff.iter().any(|line| {
            line.starts_with('-')
                && !line.starts_with("--- ")
                && line.contains("Original implementation.")
        }) && diff.iter().any(|line| {
            line.starts_with('+')
                && !line.starts_with("+++ ")
                && line.contains("Revised implementation.")
        }) {
            plan_annotation = Some(annotation);
            break;
        }
        annotation = "";
    }
    let annotation =
        plan_annotation.expect("plan Git diff must delete original and add revised instructions");
    assert!(
        annotation
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .any(|word| word.eq_ignore_ascii_case("operator")),
        "plan diff annotation must attribute the change to operator: {annotation:?}"
    );
    assert_eq!(
        edited
            .rows("SELECT plan_revision FROM execution WHERE kind = 'implementation' AND attempt IN (SELECT id FROM attempt WHERE ticket = 1) ORDER BY id"),
        vec![json!({"plan_revision": 1}), json!({"plan_revision": 2})]
    );
    assert_eq!(edited.rows(&frozen_query), frozen);
    let attempt = frozen[0]["attempt"].as_i64().unwrap();
    let clone = edited
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    assert_eq!(git(&clone, &["show", "HEAD:renamed.txt"]), "renamed\n");
    // No gate ran on the first candidate: every gate ran on the second head.
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
    stopped.json(&["attempt", "start", "Y-1"]);
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

/// A worker that leaves an untracked file gets no review and the next
/// prompt lists the file. The file is nested in a new directory, so only a
/// listing of every untracked file names it. A second worker that leaves
/// the file after being told is `stopped:dirty`, not sent back again.
///
/// Sabotage: drop `--untracked-files=all` from the clean check; the prompt
/// names only the directory. Or drop the `dirty` reason check in
/// `supervise`; the second worker is sent back a third time.
#[test]
fn an_untracked_file_gets_no_review_and_is_named() {
    let machine = Machine::new("g9-dirty", |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains("not clean") {
            if request.prompt().contains("Keep notes") {
                return Reply::Text("the notes stay".into());
            }
            return Reply::Tools(vec![bash("cd /workspace && rm -r notes && echo removed")]);
        }
        act(
            request,
            vec![bash(
                "cd /workspace && printf 'feature\\n' > feature.txt && git add feature.txt \
                 && git commit -q -m 'Add feature' && mkdir -p notes && echo scratch > notes/scratch.log",
            )],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");

    let executions = project.rows("SELECT kind, reason, outcome FROM execution ORDER BY id");
    assert_eq!(
        executions,
        vec![
            json!({ "kind": "implementation", "reason": "first", "outcome": "dirty" }),
            json!({ "kind": "implementation", "reason": "dirty", "outcome": "candidate" }),
            json!({ "kind": "review", "reason": "first", "outcome": "pass" }),
        ]
    );
    assert!(
        openings(&machine)[1]
            .last_user()
            .contains("notes/scratch.log")
    );

    project.json(&["ticket", "new", "--title", "Keep notes"]);
    // Stopped, or sent back a third time: fail on either rather than wait.
    let starts = std::cell::Cell::new(0);
    let stopped = watch.until("Y-2 stopped or a third start", |event| {
        if event["ticket"] == "Y-2"
            && event["event"] == "execution.started"
            && event["data"]["kind"] == "implementation"
        {
            starts.set(starts.get() + 1);
        }
        event["event"] == "attention.raised" || starts.get() == 3
    });
    assert_eq!(stopped["data"]["kind"], "stopped", "{stopped}");
    assert_eq!(stopped["data"]["reason"], "dirty", "{stopped}");
    assert_eq!(
        project.rows("SELECT reason, outcome FROM execution WHERE attempt = 2 ORDER BY id"),
        vec![
            json!({ "reason": "first", "outcome": "dirty" }),
            json!({ "reason": "dirty", "outcome": "dirty" }),
        ]
    );
}

/// With `max_session_executions = 2`: the second execution resumes the
/// first, the third resumes nothing and its prompt is the brief, the fourth
/// resumes the third, the fifth resumes nothing. The first execution
/// commits and the operator rejects it; each later one stops unchanged and
/// `start` begins the next. The third's brief carries the ticket's title
/// and the attempt's diff, which names the committed file.
///
/// Sabotage: compare `count <= max_session_executions` in
/// `supervise::implement`; the third execution resumes the second.
#[test]
fn sessions_resume_up_to_max_session_executions() {
    let machine = Machine::new("g9-sessions", |request| {
        act(
            request,
            vec![bash(
                "cd /workspace && if [ ! -f knob.txt ]; then printf 'knob\\n' > knob.txt \
                 && git add knob.txt && git commit -q -m 'Polish'; fi; echo ok",
            )],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &unreviewed("").replace(
            "review = \"none\"\n",
            "review = \"none\"\nmax_session_executions = 2\n",
        ),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Polish the widget"]);
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
        "Y-1",
        "--head",
        head,
        "--text",
        "Polish more",
    ]);
    for round in 0..4 {
        let stopped = watch.attention();
        assert_eq!(
            (&stopped["data"]["kind"], &stopped["data"]["reason"]),
            (&json!("stopped"), &json!("unchanged")),
            "{stopped}"
        );
        if round < 3 {
            project.json(&["attempt", "start", "Y-1"]);
        }
    }

    // What each execution sent the model: a fresh session has one user
    // message, the brief.
    let users: Vec<usize> = openings(&machine)
        .iter()
        .map(|request| {
            request.body["messages"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|message| message["role"] == "user")
                .count()
        })
        .collect();
    assert_eq!(users, [1, 2, 1, 2, 1]);
    let third = openings(&machine)[2].last_user();
    assert!(third.contains("Polish the widget"), "{third}");
    assert!(third.contains("knob.txt"), "{third}");
}
