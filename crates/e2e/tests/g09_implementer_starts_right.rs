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
        .filter(ModelRequest::opens)
        .collect()
}

fn reasons(project: &Project) -> Vec<Value> {
    project
        .rows("SELECT reason FROM execution WHERE kind = 'implementation' ORDER BY id")
        .into_iter()
        .map(|row| row["reason"].clone())
        .collect()
}

/// The first execution commits, then its next request is held; the second
/// execution, if its prompt carries `Rename`, commits again.
fn held_worker(hold: &Latch) -> impl Fn(&ModelRequest) -> Reply + Send + Sync + 'static {
    let hold = hold.clone();
    move |request| {
        if request.opens() && request.last_user().contains("Rename") {
            return Reply::Tools(vec![commit_file("renamed.txt", "renamed\n", "Rename")]);
        }
        if !request.opens() && request.last_user().contains("Rename") {
            return Reply::Text("done".into());
        }
        match request.tool_results().len() {
            0 => Reply::Tools(vec![commit_file("feature.txt", "feature\n", "Add feature")]),
            _ => Reply::Hold(hold.clone(), Box::new(Reply::Text("done".into()))),
        }
    }
}

/// A nudge mid-execution lets the execution end on its own and reaches the
/// next prompt: while the worker is held the execution keeps running, and
/// once released it ends as a candidate and the next execution opens with
/// the nudge.
///
/// Sabotage: make `supervise::implement` leave a nudge queued after a
/// candidate; review none approves the first head and the nudge is never
/// delivered.
#[test]
fn a_nudge_mid_execution_reaches_the_next_prompt() {
    let hold = Latch::new();
    let machine = Machine::new("g9-nudge", held_worker(&hold));
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed(""));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    hold.wait_held();

    project.json(&["attempt", "nudge", "Y-1", "--text", "Rename it too"]);
    assert!(hold.is_held(), "the nudge interrupted the worker");
    assert_eq!(
        project.rows("SELECT status FROM execution"),
        vec![json!({ "status": "running" })]
    );
    hold.release();

    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    assert_eq!(reasons(&project), [json!("first"), json!("nudge")]);
    assert_eq!(
        project.rows("SELECT outcome FROM execution WHERE id = 1"),
        vec![json!({ "outcome": "candidate" })]
    );
    let openings = openings(&machine);
    assert_eq!(openings.len(), 2);
    assert!(openings[1].last_user().contains("Rename it too"));
    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:renamed.txt")]
        ),
        "renamed\n"
    );
}

/// `stop` delivers a queued nudge sooner: the worker is still held when the
/// execution ends, and the next execution opens with the nudge.
///
/// Sabotage: make `admit::attempt_stop` skip notifying the execution; it
/// runs on and nothing reaches the next prompt.
#[test]
fn stop_delivers_a_nudge_sooner() {
    let hold = Latch::new();
    let machine = Machine::new("g9-stop", held_worker(&hold));
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed(""));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    hold.wait_held();

    project.json(&["attempt", "nudge", "Y-1", "--text", "Rename it now"]);
    project.json(&["attempt", "stop", "Y-1"]);
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    assert!(hold.is_held(), "the worker's request was answered");
    assert_eq!(reasons(&project), [json!("first"), json!("nudge")]);
    assert!(openings(&machine)[1].last_user().contains("Rename it now"));
    hold.release();
}

/// A worker that leaves an untracked file gets no review and the next
/// prompt lists the file. The file is nested in a new directory, so only a
/// listing of every untracked file names it.
///
/// Sabotage: drop `--untracked-files=all` from the clean check; the prompt
/// names only the directory.
#[test]
fn an_untracked_file_gets_no_review_and_is_named() {
    let machine = Machine::new("g9-dirty", |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains("not clean") {
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
}

/// With `max_session_executions = 2`: the second execution resumes the
/// first, the third resumes nothing and its prompt is the brief, the fourth
/// resumes the third, the fifth resumes nothing. Each execution stops
/// unchanged and `start` begins the next.
///
/// Sabotage: compare `count <= max_session_executions` in
/// `supervise::implement`; the third execution resumes the second.
#[test]
fn sessions_resume_up_to_max_session_executions() {
    let machine = Machine::new("g9-sessions", |_| Reply::Text("nothing to do".into()));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &unreviewed("").replace(
            "review = \"none\"\n\n[workflows.plan]",
            "review = \"none\"\nmax_session_executions = 2\n\n[workflows.plan]",
        ),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    for round in 0..5 {
        let stopped = watch.attention();
        assert_eq!(
            (&stopped["data"]["kind"], &stopped["data"]["reason"]),
            (&json!("stopped"), &json!("unchanged")),
            "{stopped}"
        );
        if round < 4 {
            project.json(&["attempt", "start", "Y-1"]);
        }
    }

    let resumed: Vec<Value> = project
        .rows("SELECT resumed FROM execution WHERE kind = 'implementation' ORDER BY id")
        .into_iter()
        .map(|row| row["resumed"].clone())
        .collect();
    assert_eq!(
        resumed,
        [Value::Null, json!(1), Value::Null, json!(3), Value::Null]
    );
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
    let third = &openings(&machine)[2];
    assert!(third.last_user().contains("You are working on ticket Y-1"));
}
