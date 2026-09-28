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
