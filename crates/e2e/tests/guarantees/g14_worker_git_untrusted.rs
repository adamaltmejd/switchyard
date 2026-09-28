//! G14: Worker git is untrusted, and Yard stays local.

use e2e::*;
use serde_json::json;

/// A worker that plants `core.fsmonitor`, a hook, a clean filter and a
/// remote in its clone and corrupts an object: none of them runs on the
/// host, canonical's objects are intact, the candidate is refused by name.
/// It also links its harness state's `agent/models.json` to a host path:
/// the next execution's staging writes no file through it. Each plant
/// names a file under a host path that does not exist in the box, so a
/// marker proves it acted on the host. The corrupt object is a commit that
/// is well-formed on disk but fails fsck, on top of a good one. Control:
/// the next execution resets to the good commit, past the same plants, and
/// its candidate is taken.
///
/// Sabotage: drop `transfer.fsckObjects=true` from `git::Git`; the corrupt
/// commit becomes the candidate. Or run a host git command with its cwd in
/// the clone (e.g. `git status`); the fsmonitor marker appears. Or write
/// `models.json` with `std::fs::write` in `pi::stage_state`; the
/// `models.json` marker appears (and the next execution's Pi, reading the
/// link, misses its route).
#[test]
fn planted_worker_git_never_runs_on_the_host() {
    let markers = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let dir = markers.clone();
    let machine = Machine::new("g14-plants", move |request| {
        let dir = dir.lock().unwrap().clone();
        if request.opens() && request.last_user().contains("Continue") {
            return Reply::Tools(vec![bash(
                "cd /workspace && git -c core.hooksPath=/dev/null reset -q --hard HEAD~1 && echo reset",
            )]);
        }
        act(
            request,
            vec![bash(&format!(
                "cd /workspace && git config core.fsmonitor 'touch {dir}/fsmonitor; false' \
                 && git config filter.plant.clean 'touch {dir}/filter; cat' \
                 && printf '* filter=plant\\n' > .git/info/attributes \
                 && git remote add plant 'ext::sh -c touch% {dir}/remote' \
                 && for hook in pre-commit post-commit post-checkout reference-transaction pre-push; do \
                      printf '#!/bin/sh\\ntouch {dir}/hook\\n' > .git/hooks/$hook; chmod +x .git/hooks/$hook; done \
                 && printf 'feature\\n' > feature.txt && git add feature.txt \
                 && git -c core.hooksPath=/dev/null commit -q -m 'Add feature' \
                 && bad=$(printf 'tree %s\\nparent %s\\nauthor bad\\ncommitter bad\\n\\nbad\\n' \
                      $(git rev-parse 'HEAD^{{tree}}') $(git rev-parse HEAD) \
                      | git hash-object -t commit --literally -w --stdin) \
                 && git -c core.hooksPath=/dev/null update-ref HEAD $bad \
                 && ln -sf {dir}/models.json /yard/state/agent/models.json && echo planted"
            ))],
        )
    });
    let marker_dir = machine.root.join("markers");
    std::fs::create_dir_all(&marker_dir).unwrap();
    *markers.lock().unwrap() = marker_dir.display().to_string();
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let stopped = watch.attention();
    assert_eq!(
        (&stopped["data"]["kind"], &stopped["data"]["reason"]),
        (&json!("stopped"), &json!("failed")),
        "{stopped}"
    );
    assert_eq!(
        project.rows("SELECT outcome FROM execution"),
        vec![json!({ "outcome": "refused" })]
    );
    git(&project.canonical(), &["fsck", "--strict", "--no-dangling"]);

    project.json(&["attempt", "start", "Y-1"]);
    let approval = watch.attention();
    let planted: Vec<_> = std::fs::read_dir(&marker_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert!(planted.is_empty(), "ran on the host: {planted:?}");
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:feature.txt")]
        ),
        "feature\n"
    );
}

/// Across G7's path every request that reaches the fixture behind the
/// model's origin came through the model route, and the daemon listens on
/// its unix socket and the MCP listener only.
///
/// Sabotage: bind the MCP listener on `0.0.0.0` in `daemon::serve`; the
/// daemon listens beyond loopback.
#[test]
fn yard_reaches_only_its_routes_and_listens_locally() {
    let machine = Machine::new("g14-local", |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    project.json(&["attempt", "approve", "Y-1", "--head", head]);
    watch.event("landing.recorded", &[]);

    let requests = machine.model.requests();
    assert!(!requests.is_empty());
    for request in &requests {
        assert_eq!(request.path, "/api/v1/chat/completions");
    }

    let pid = format!("pid={},", machine.daemon_pid());
    let listening = |args: &[&str]| -> Vec<String> {
        let out = std::process::Command::new("ss")
            .args(args)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| line.contains(&pid))
            .map(str::to_string)
            .collect()
    };
    let inet = listening(&["-H", "-l", "-t", "-u", "-n", "-p"]);
    assert_eq!(inet.len(), 1, "{inet:?}");
    assert!(inet[0].contains("127.0.0.1:"), "{inet:?}");
    let unix = listening(&["-H", "-l", "-x", "-p"]);
    assert_eq!(unix.len(), 1, "{unix:?}");
    assert!(
        unix[0].contains(&machine.state.join("yard/yard.sock").display().to_string()),
        "{unix:?}"
    );
}
