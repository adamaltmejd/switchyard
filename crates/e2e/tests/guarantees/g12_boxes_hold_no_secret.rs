//! G12: Boxes hold nothing secret.

use e2e::*;
use std::sync::{Arc, Mutex};

/// The key is absent from the box's environment and clone; the fixture
/// behind the injecting route receives it. The worker searches its own
/// environment, its harness's, the box's init, the clone and every
/// directory Yard mounts.
///
/// Sabotage: make `supervise::worker_spec` pass the key as a literal
/// variable; the worker's environment holds it. Or stage the placeholder
/// with no injecting route; the fixture never receives the key.
#[test]
fn the_key_is_absent_from_the_box_and_reaches_the_route() {
    let found = Arc::new(Mutex::new(String::new()));
    let seen = found.clone();
    // The key split in two, so the command itself, and the transcript and
    // session that record it, never hold it whole.
    let (head, tail) = SECRET.split_at(8);
    let key = format!("'{head}''{tail}'");
    let search = format!(
        "for f in /proc/self/environ /proc/$PPID/environ /proc/1/environ; do \
           tr '\\0' '\\n' < $f | grep -c {key}; done; \
         grep -rsl {key} /workspace /yard /tmp | wc -l"
    );
    let machine = Machine::new("g12", move |request| match request.tool_results().len() {
        0 => Reply::Tools(vec![bash(&search)]),
        1 => {
            *seen.lock().unwrap() = request.last_tool_result().unwrap().1;
            Reply::Tools(vec![commit_file("feature.txt", "feature\n", "Add feature")])
        }
        _ => Reply::Text("done".into()),
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    watch.until("approval", |event| event["event"] == "attention.raised");

    let counts: Vec<String> = found
        .lock()
        .unwrap()
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    assert_eq!(counts, ["0", "0", "0", "0"], "the key was found in the box");
    let requests = machine.model.requests();
    assert!(!requests.is_empty());
    for request in &requests {
        assert_eq!(
            request.header("authorization"),
            Some(format!("Bearer {SECRET}").as_str())
        );
    }
}

/// The claude login token is absent from the box's environment, files and
/// clone; the fixture behind pinfold's login route receives it as a Bearer
/// header. The same search runs through Claude's `Bash` tool.
///
/// Sabotage: set `CLAUDE_CODE_OAUTH_TOKEN` in the box env in `claude::env`;
/// the search finds it. Or drop the login route's `from`; the fixture gets
/// the placeholder and no Bearer.
#[test]
fn the_claude_token_is_absent_from_the_box_and_reaches_the_route() {
    let found = Arc::new(Mutex::new(String::new()));
    let seen = found.clone();
    let (head, tail) = SECRET.split_at(8);
    let key = format!("'{head}''{tail}'");
    let search = format!(
        "echo PROBE-BEGIN; \
         for f in /proc/self/environ /proc/$PPID/environ /proc/1/environ; do \
           tr '\\0' '\\n' < $f | grep -c {key}; done; \
         grep -rsl {key} /workspace /yard /tmp | wc -l; \
         echo PROBE-END"
    );
    let machine = Machine::new("g12-claude", move |request| {
        match request.tool_results().len() {
            0 => Reply::Tools(vec![claude_bash(&search)]),
            1 => {
                *seen.lock().unwrap() = request.last_tool_result().unwrap().1;
                Reply::Tools(vec![claude_commit_file(
                    "feature.txt",
                    "feature\n",
                    "Add feature",
                )])
            }
            _ => Reply::Text("done".into()),
        }
    });
    machine.write_claude_env();
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &claude_config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    watch.until("approval", |event| event["event"] == "attention.raised");

    let counts: Vec<String> = found
        .lock()
        .unwrap()
        .lines()
        .skip_while(|line| !line.contains("PROBE-BEGIN"))
        .skip(1)
        .take_while(|line| !line.contains("PROBE-END"))
        .map(|line| line.trim().to_string())
        .collect();
    assert_eq!(
        counts,
        ["0", "0", "0", "0"],
        "the token was found in the box"
    );
    let requests = machine.model.requests();
    let anthropic: Vec<_> = requests
        .iter()
        .filter(|request| request.path.starts_with("/v1/messages"))
        .collect();
    assert!(!anthropic.is_empty(), "the login route was never called");
    for request in anthropic {
        assert_eq!(
            request.header("authorization"),
            Some(format!("Bearer {SECRET}").as_str())
        );
    }
}
