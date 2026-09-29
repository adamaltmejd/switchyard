//! G10: The queue lands one at a time and re-judges what does not merge.

use e2e::*;
use serde_json::{Value, json};

fn unreviewed(extra: &str) -> String {
    config(extra)
        .replace("review = [\"correctness\"]", "review = \"none\"")
        .replace("max_lanes = 2", "max_lanes = 3")
}

fn is_approval(event: &Value, ticket: &str) -> bool {
    event["event"] == "attention.raised"
        && event["ticket"] == ticket
        && event["data"]["kind"] == "approval"
}

fn head(event: Value) -> String {
    event["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string()
}

/// The ticket's first approval item's head, whenever it was raised.
#[track_caller]
fn approval_of(watch: &mut Watch, ticket: &str) -> String {
    head(watch.find(ticket, |event| is_approval(event, ticket)))
}

/// The ticket's next approval item's head, from events not yet read.
#[track_caller]
fn next_approval_of(watch: &mut Watch, ticket: &str) -> String {
    head(watch.until(ticket, |event| is_approval(event, ticket)))
}

/// Merge the target from the repair's bundle, keeping this side of any
/// conflict.
fn merge_target() -> ToolCall {
    bash(
        "cd /workspace && git fetch -q /yard/input/target.bundle refs/heads/main \
         && git merge -q -X ours --no-edit FETCH_HEAD && echo merged",
    )
}

/// Three approved candidates, the second red on its merged ref: the first
/// lands, the second gets one repair and a second red raises `red`, the
/// third lands on the moved target with its own gate run. No two landings
/// overlap.
///
/// The failing gate writes over 8000 bytes to stderr, then a marker to stdout;
/// the red's detail ends with the marker.
///
/// Sabotage: make `queue::returned` reset `landing_reds` with the repair;
/// the second red buys another repair instead of `red`. Or store the gate's
/// `stdout + stderr` again; the detail ends with stderr, not the marker.
#[test]
fn the_queue_lands_in_order_and_rejudges_a_red_merge() {
    const MARKER: &str = "the-gate-ends-here";
    let gate = format!(
        "[gates.clean]\ncommand = \"test ! -f bad.txt && exit 0; \\
         yes 'stderr noise noise noise' | head -n 1000 >&2; sleep 1; echo {MARKER}; exit 1\"\n"
    );
    let machine = Machine::new("g10-order", |request| {
        let prompt = request.prompt();
        let file = if prompt.contains("Second") {
            "bad.txt"
        } else if prompt.contains("Third") {
            "third.txt"
        } else {
            "first.txt"
        };
        if request.opens() && request.last_user().contains("failed to land") {
            return Reply::Tools(vec![commit_file("again.txt", "again\n", "Try again")]);
        }
        act(request, vec![commit_file(file, "text\n", "Add a file")])
    });
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed(&gate));
    let mut watch = project.watch(0);
    for title in ["First", "Second", "Third"] {
        project.json(&["ticket", "new", "--title", title]);
    }
    let heads: Vec<String> = ["Y-1", "Y-2", "Y-3"]
        .iter()
        .map(|ticket| approval_of(&mut watch, ticket))
        .collect();
    for (ticket, head) in ["Y-1", "Y-2", "Y-3"].iter().zip(&heads) {
        project.json(&["attempt", "approve", ticket, "--head", head]);
    }

    let again = next_approval_of(&mut watch, "Y-2");
    project.json(&["attempt", "approve", "Y-2", "--head", &again]);
    // The second red raises `red` before anything else starts on Y-2.
    let red = watch.until("Y-2 red or a repair", |event| {
        event["ticket"] == "Y-2"
            && (event["event"] == "attention.raised"
                || (event["event"] == "execution.started"
                    && event["data"]["kind"] == "implementation"))
    });
    assert_eq!(red["data"]["kind"], "red", "{red}");
    assert_eq!(red["data"]["reason"], "landing");
    let detail = red["data"]["payload"]["detail"].as_str().unwrap();
    assert!(detail.trim_end().ends_with(MARKER), "{detail}");
    watch.find("Y-3 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-3"
    });

    let landed: Vec<Value> = project
        .rows("SELECT ticket FROM audit WHERE event = 'landing.recorded' ORDER BY seq")
        .into_iter()
        .map(|row| row["ticket"].clone())
        .collect();
    assert_eq!(landed, [json!(1), json!(3)]);
    let landings = project.rows(
        "SELECT execution.id, attempt.ticket, execution.outcome, execution.head
         FROM execution JOIN attempt ON attempt.id = execution.attempt
         WHERE execution.kind = 'landing' ORDER BY execution.id",
    );
    let second: Vec<&Value> = landings.iter().filter(|row| row["ticket"] == 2).collect();
    assert_eq!(
        second
            .iter()
            .map(|row| row["outcome"].clone())
            .collect::<Vec<_>>(),
        [json!("red"), json!("red")]
    );
    assert_eq!(second[1]["head"], again.as_str());

    // The third's gate ran on a merge whose first parent is the target the
    // first landing left.
    let third = landings
        .iter()
        .find(|row| row["ticket"] == 3 && row["outcome"] == "landed")
        .unwrap();
    let gate = project.rows(&format!(
        "SELECT base, head, outcome FROM execution WHERE parent = {}",
        third["id"]
    ));
    assert_eq!(gate.len(), 1);
    assert_eq!(gate[0]["outcome"], "pass");
    let merged = gate[0]["head"].as_str().unwrap();
    let parents = git(
        &project.canonical(),
        &["rev-list", "--parents", "-n", "1", merged],
    );
    let parents: Vec<&str> = parents.split_whitespace().skip(1).collect();
    assert_eq!(parents, [heads[0].as_str(), heads[2].as_str()]);

    // One at a time: each landing ended before the next started. Sabotage:
    // give each project two landing slots; two landings overlap.
    let spans = project.rows(
        "SELECT execution, event FROM audit WHERE event IN ('execution.started', 'execution.ended')
         AND execution IN (SELECT id FROM execution WHERE kind = 'landing') ORDER BY seq",
    );
    let mut open = None;
    for span in &spans {
        if span["event"] == "execution.started" {
            assert!(open.is_none(), "two landings overlap: {spans:?}");
            open = Some(span["execution"].clone());
        } else {
            assert_eq!(open.take(), Some(span["execution"].clone()));
        }
    }
}

/// A candidate that does not merge gets a repair naming the paths, and its
/// next head takes gates, review and approval again.
///
/// Sabotage: drop the conflicting paths from `queue::land`'s detail; the
/// repair prompt does not name `shared.txt`.
#[test]
fn a_candidate_that_does_not_merge_is_repaired_and_rejudged() {
    let machine = Machine::new("g10-conflict", |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains("failed to land") {
            return Reply::Tools(vec![merge_target()]);
        }
        let text = if request.prompt().contains("First") {
            "first\n"
        } else {
            "second\n"
        };
        act(
            request,
            vec![commit_file("shared.txt", text, "Write shared")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f shared.txt\"\nstage = \"candidate\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "First"]);
    project.json(&["ticket", "new", "--title", "Second"]);
    let first = approval_of(&mut watch, "Y-1");
    let second = approval_of(&mut watch, "Y-2");
    project.json(&["attempt", "approve", "Y-1", "--head", &first]);
    watch.find("Y-1 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    project.json(&["attempt", "approve", "Y-2", "--head", &second]);

    let repaired = next_approval_of(&mut watch, "Y-2");
    assert_ne!(repaired, second);
    let repair_prompt = machine
        .model
        .requests()
        .into_iter()
        .find(|request| request.opens() && request.last_user().contains("failed to land"))
        .unwrap()
        .last_user();
    assert!(repair_prompt.contains("shared.txt"), "{repair_prompt}");
    let judged = project.rows(&format!(
        "SELECT kind FROM execution WHERE head = '{repaired}' AND kind IN ('gate', 'review') ORDER BY id"
    ));
    assert_eq!(
        judged,
        vec![json!({ "kind": "gate" }), json!({ "kind": "review" })]
    );

    project.json(&["attempt", "approve", "Y-2", "--head", &repaired]);
    watch.find("Y-2 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
    assert_eq!(
        git(
            &project.canonical(),
            &["show", "refs/heads/main:shared.txt"]
        ),
        "second\n"
    );
}

/// A conflict against a target that changed `.yard/config.toml`, in a clone
/// made before it: the worker fetches the target from its bundle, merges,
/// and the new candidate's base is the target, so it passes the `.yard`
/// refusal and lands with the operator's configuration intact.
///
/// Sabotage: make `supervise::implement` keep the old base when the target
/// becomes an ancestor of the head; the merged candidate touches `.yard`
/// and is refused.
#[test]
fn a_conflict_with_a_config_change_merges_from_the_bundle() {
    let machine = Machine::new("g10-config", |request| {
        if request.opens() && request.last_user().contains("failed to land") {
            return Reply::Tools(vec![merge_target()]);
        }
        act(
            request,
            vec![commit_file("shared.txt", "worker\n", "Write shared")],
        )
    });
    machine.start();
    let base = config("").replace("review = [\"correctness\"]", "review = \"none\"");
    let project = Project::new(&machine, "p", &base);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Write shared"]);
    let head = approval_of(&mut watch, "Y-1");

    // The operator's conflicting change, with a configuration change beside it.
    let configured = base.replace("max_rounds = 3", "max_rounds = 4");
    project.write("shared.txt", "operator\n");
    project.write(".yard/config.toml", &configured);
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "Operator change"]);
    project.json(&["sync"]);
    let target = project.canonical_head();

    project.json(&["attempt", "approve", "Y-1", "--head", &head]);
    let repair = watch.until("Y-1's repair ended", |event| {
        event["event"] == "execution.ended"
            && event["ticket"] == "Y-1"
            && event["data"]["kind"] == "implementation"
    });
    assert_eq!(repair["data"]["outcome"], "candidate", "{repair}");
    let repaired = next_approval_of(&mut watch, "Y-1");
    let attempt = project.json(&["attempt", "show", "Y-1"]);
    assert_eq!(attempt["attempt"]["base"], target.as_str(), "{attempt}");

    project.json(&["attempt", "approve", "Y-1", "--head", &repaired]);
    watch.find("Y-1 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    assert_eq!(
        git(
            &project.canonical(),
            &["show", "refs/heads/main:.yard/config.toml"]
        ),
        configured
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["show", "refs/heads/main:shared.txt"]
        ),
        "worker\n"
    );
}
