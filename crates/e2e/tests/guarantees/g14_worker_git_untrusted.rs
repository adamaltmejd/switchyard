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
/// link, misses its route). Dropping git's error detail from the refusal
/// loses the corrupt commit identity reported by the fixture.
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
                 && ln -sf {dir}/models.json /yard/state/agent/models.json && echo planted $bad"
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
    let corrupt = machine
        .model
        .requests()
        .iter()
        .flat_map(ModelRequest::tool_results)
        .find_map(|(_, text)| {
            text.lines()
                .find_map(|line| line.strip_prefix("planted ").map(str::to_string))
        })
        .expect("the fixture reported its corrupt commit");
    let detail = stopped["data"]["payload"]["detail"].as_str().unwrap();
    assert!(detail.contains(&corrupt), "{stopped}");
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

/// The worker leaves a symlink in `/yard/proof`: its candidate is refused by
/// name and no gate runs. Control: a regular file is accepted and its
/// candidate gate runs.
///
/// Sabotage: follow links in `proof::snapshot`; the symlink is copied, the
/// candidate is accepted and a gate runs.
#[test]
fn a_proof_symlink_is_refused_by_name() {
    let machine = Machine::new("g14-proof", |request| {
        let command = if request.prompt().contains("Y-1") {
            "cd /workspace && printf 'feature' > feature.txt && git add -A && git commit -q -m 'Add feature' && ln -s /workspace/feature.txt /yard/proof/link && echo done"
        } else {
            "cd /workspace && printf 'feature' > feature.txt && git add -A && git commit -q -m 'Add feature' && printf 'evidence' > /yard/proof/evidence.txt && echo done"
        };
        act(request, vec![bash(command)])
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(
            "[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\nruns_in = \"host\"\n",
        )
        .replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Symlink proof"]);
    project.json(&["ticket", "new", "--title", "Good proof"]);

    let mut stopped = false;
    let mut approved = false;
    while !stopped || !approved {
        let event = watch.until("decision", |event| {
            event["event"] == "attention.raised"
                && (event["data"]["kind"] == "stopped" || event["data"]["kind"] == "approval")
        });
        if event["data"]["kind"] == "stopped" {
            assert_eq!(event["data"]["reason"], "failed", "{event}");
            stopped = true;
        } else {
            assert_eq!(event["ticket"], "Y-2", "{event}");
            approved = true;
        }
    }

    let refusals = project.rows(
        "SELECT attempt.ticket AS ticket, execution.detail AS detail
         FROM execution JOIN attempt ON attempt.id = execution.attempt
         WHERE execution.kind = 'implementation' AND execution.outcome = 'refused'",
    );
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0]["ticket"], 1, "{refusals:?}");
    let detail = refusals[0]["detail"].as_str().unwrap();
    assert!(detail.contains("link"), "{detail}");

    // No gate ran for the refused candidate; the accepted one ran its gate.
    let gates = project.rows(
        "SELECT attempt.ticket AS ticket FROM execution
         JOIN attempt ON attempt.id = execution.attempt
         WHERE execution.kind = 'gate'",
    );
    assert_eq!(gates, vec![json!({ "ticket": 2 })], "{gates:?}");
}
