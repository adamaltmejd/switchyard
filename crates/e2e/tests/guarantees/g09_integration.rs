//! G2/G9: integration consumes exact publications in its own retained writer.
use e2e::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};

fn configuration() -> String {
    let mut extra = include_str!("../../../../docs/examples/integration.toml").to_string();
    for (agent, model) in [
        ("a", "source-a"),
        ("b", "source-b"),
        ("integrator", "integration"),
    ] {
        extra.push_str(&format!(
            "\n[agents.{agent}]\nharness = 'pi'\nprovider = 'openrouter'\nmodel = '{model}'\n"
        ));
    }
    config(&extra).replace("review = [\"correctness\"]", "review = \"none\"")
        .replace("git diff --check \"$YARD_BASE\" HEAD", "if test -f conflict.txt; then test \"$(cat conflict.txt)\" = resolved; else test \"$(cat a.txt)\" = A && test \"$(cat b.txt)\" = B && test \"$(cat partial.txt)\" = partial; fi")
}

fn settle_planner(project: &Project<'_>, watch: &mut Watch) {
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Combine inputs",
        "--workflow",
        "plan",
    ]);
    let planner = watch.event("attempt.ended", &[("outcome", "planned")]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner["attempt"]
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
}

fn start(project: &Project<'_>) -> i64 {
    let mut operator = project.machine.operator();
    operator.initialize();
    let answer = operator.call(
        "yard_attempt_start",
        json!({"project": project.path,
        "ticket": "Y-1", "preset": "integration"}),
    );
    assert!(answer.get("error").is_none(), "{answer}");
    answer["result"]["attempt"].as_i64().unwrap()
}

fn continue_integration(project: &Project<'_>) {
    let mut operator = project.machine.operator();
    operator.initialize();
    let answer = operator.call(
        "yard_attempt_start",
        json!({"project": project.path,
        "ticket": "Y-1", "workspace": "integrate"}),
    );
    assert!(answer.get("error").is_none(), "{answer}");
}

fn approval(watch: &mut Watch) -> Value {
    watch.until("integrated approval", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    })["data"]["payload"]
        .clone()
}

// Fetch only the named immutable bundle ref. Git supplies the expected head.
fn fetch(input: &str) -> String {
    format!(
        "(bundle=/yard/candidates/{input}/candidate.bundle; git fetch \"$bundle\" \"$(git bundle list-heads \"$bundle\" | cut -d' ' -f2)\" && test \"$(git rev-parse FETCH_HEAD)\" = \"$(git -c safe.directory=/yard/candidates/{input}/source -C /yard/candidates/{input}/source rev-parse HEAD)\")"
    )
}

/// G2/G9's integration clauses. Unlike independent writer/selection scenarios,
/// C imports two exact publications and has a partially merged tree at restart.
/// A host-model latch holds the real box after its tool wrote partial.txt;
/// the store and host file prove that state before reconciliation.
/// Expected contents and target come from the fixture and host Git.
/// Sabotage: mount canonical into C, make candidate inputs writable, reuse
/// A's candidate/approval for C, or recreate C's clone after interruption.
/// After planner release, admission refuses feedback without a new attempt
/// or execution, then supported named sources succeed on the same ticket;
/// sabotage: allow that source to collide with internal repair feedback.
#[test]
fn integration_retains_partial_work_and_approves_only_its_output() {
    let hold = Latch::new();
    let held = hold.clone();
    let turns = AtomicUsize::new(0);
    let machine = Machine::new("g9-integration", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        if request.has_tool("yard_publish_selection") {
            return act(
                request,
                vec![tool("yard_publish_selection", json!({"candidate": "c"}))],
            );
        }
        let model = request.body["model"].as_str().unwrap();
        if model == "integration" {
            if request.opens() {
                let turn = turns.fetch_add(1, Ordering::SeqCst);
                let command = if turn == 0 {
                    format!(
                        "{} && git merge --no-edit FETCH_HEAD && printf 'partial\\n' > partial.txt && echo integration-held",
                        fetch("61")
                    )
                } else {
                    format!(
                        "test \"$(cat partial.txt)\" = partial && {} && git merge --no-edit FETCH_HEAD && git add partial.txt && git commit -qm Integrated",
                        fetch("62")
                    )
                };
                return Reply::Tools(vec![bash(&command)]);
            }
            if request
                .last_tool_result()
                .is_some_and(|(_, text)| text.contains("integration-held"))
            {
                return Reply::Hold(held.clone(), Box::new(Reply::Text("done".into())));
            }
            return Reply::Text("done".into());
        }
        let (file, contents) = if model == "source-a" {
            ("a.txt", "A")
        } else {
            ("b.txt", "B")
        };
        act(
            request,
            vec![bash(&format!(
                "printf '{contents}\\n' > {file} && git add {file} && git commit -qm Source"
            ))],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &configuration());
    let target = project.canonical_head();
    let mut watch = project.watch(0);
    settle_planner(&project, &mut watch);
    let attempts = project.rows("SELECT id FROM attempt ORDER BY id");
    let executions = project.rows("SELECT id FROM execution ORDER BY id");
    let mut operator = machine.operator();
    operator.initialize();
    let refused = operator.call(
        "yard_attempt_start",
        json!({"project": project.path,
        "ticket": "Y-1", "preset": "integration", "overrides": {"integrate": {
            "inputs": {
                "feedback": {"node": "implementer", "output": "result", "kind": "candidate"},
                "b": {"node": "writer-b", "output": "result", "kind": "candidate"}
            }, "requires": ["feedback", "b"]
        }}}),
    );
    assert_eq!(refused["error"]["code"], "invalid", "{refused}");
    assert_eq!(
        refused["error"]["data"],
        json!({
            "input": "nodes.integrate.inputs.feedback", "reason": "invalid_input"
        })
    );
    assert_eq!(project.rows("SELECT id FROM attempt ORDER BY id"), attempts);
    assert_eq!(
        project.rows("SELECT id FROM execution ORDER BY id"),
        executions
    );
    drop(operator);
    let attempt = start(&project);
    hold.wait_held();
    let row = project
        .rows("SELECT id, status, handle FROM execution WHERE node = 'integrate'")
        .pop()
        .unwrap();
    let clone = project.path.join(format!(
        ".yard/local/attempts/{attempt}/writers/696e74656772617465/clone"
    ));
    // These writes exercise actual mounts, not their declared specifications.
    let out = machine.pinfold(&[
        "box",
        "exec",
        row["handle"].as_str().unwrap(),
        "--",
        "sh",
        "-c",
        "printf changed > /yard/candidates/61/source/a.txt || exit 37",
    ]);
    assert_eq!(out.status.code(), Some(37));
    assert_eq!(std::fs::read_to_string(clone.join("a.txt")).unwrap(), "A\n");
    assert_eq!(project.canonical_head(), target);
    machine.kill();
    assert_eq!(
        project.rows("SELECT status FROM execution WHERE node = 'integrate'")[0]["status"],
        "running"
    );
    assert_eq!(
        std::fs::read_to_string(clone.join("partial.txt")).unwrap(),
        "partial\n"
    );
    drop(watch);
    machine.start();
    let mut watch = project.watch(0);
    continue_integration(&project);
    let bound = approval(&mut watch);
    hold.release();
    let head = bound["head"].as_str().unwrap();
    for (file, expected) in [
        ("a.txt", "A\n"),
        ("b.txt", "B\n"),
        ("partial.txt", "partial\n"),
    ] {
        assert_eq!(
            git(&project.canonical(), &["show", &format!("{head}:{file}")]),
            expected
        );
    }
    let sources = project.rows(&format!(
        "SELECT id FROM execution WHERE attempt = {attempt} AND node = 'implementer' AND outcome = 'candidate'"
    ));
    let source_head = git(
        &project.canonical(),
        &[
            "rev-parse",
            &format!("refs/yard/candidates/{attempt}/{}", sources[0]["id"]),
        ],
    )
    .trim()
    .to_string();
    assert_ne!(source_head, head);
    let mut operator = machine.operator();
    operator.initialize();
    let rejected = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path,
        "ticket": "Y-1", "target": "main", "head": source_head, "proof": bound["proof"],
        "choice": bound["choice"], "plan_revision": bound["plan_revision"]}),
    );
    assert_eq!(rejected["error"]["code"], "stale", "{rejected}");
    assert_eq!(rejected["error"]["data"]["expected"], source_head);
    assert_eq!(rejected["error"]["data"]["current"], bound["head"]);
    assert!(project.rows("SELECT id FROM approval").is_empty());
    let accepted = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path,
        "ticket": "Y-1", "target": "main", "head": bound["head"], "proof": bound["proof"],
        "choice": bound["choice"], "plan_revision": bound["plan_revision"]}),
    );
    assert!(accepted.get("error").is_none(), "{accepted}");
    watch.event("attempt.landed", &[]);
    let approved = project.rows("SELECT head FROM approval");
    assert_eq!(approved, vec![json!({"head": bound["head"]})]);
}

/// G2/G9: real conflicting commits leave no candidate, choice or landing.
/// Ordinary dirty-tree tests have no integration inputs or selectable sources.
/// The same retained workspace resolves the conflict as the positive control.
/// Sabotage: select an input when integration fails, publish a dirty merge,
/// or discard the conflict tree before repair. Git supplies conflict state.
#[test]
fn integration_conflicts_require_repair_before_selection() {
    let turns = AtomicUsize::new(0);
    let machine = Machine::new("g9-integration-conflict", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        if request.has_tool("yard_publish_selection") {
            return act(
                request,
                vec![tool("yard_publish_selection", json!({"candidate": "c"}))],
            );
        }
        let model = request.body["model"].as_str().unwrap();
        if model == "integration" {
            if !request.opens() {
                return Reply::Text("done".into());
            }
            let turn = turns.fetch_add(1, Ordering::SeqCst);
            let command = match turn {
                0 => format!("{} && git merge --no-edit FETCH_HEAD && {} && git merge --no-edit FETCH_HEAD", fetch("61"), fetch("62")),
                1 => "git status --porcelain".into(),
                _ => "test -f .git/MERGE_HEAD && printf 'resolved\\n' > conflict.txt && git add conflict.txt && git commit -qm Resolved".into(),
            };
            return Reply::Tools(vec![bash(&command)]);
        }
        let content = if model == "source-a" { "A" } else { "B" };
        act(
            request,
            vec![bash(&format!(
                "printf '{content}\\n' > conflict.txt && git add conflict.txt && git commit -qm Source"
            ))],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &configuration());
    let target = project.canonical_head();
    let mut watch = project.watch(0);
    settle_planner(&project, &mut watch);
    let attempt = start(&project);
    watch.until("integration conflict attention", |event| {
        event["event"] == "attention.raised" && event["data"]["reason"] == "dirty"
    });
    let clone = project.path.join(format!(
        ".yard/local/attempts/{attempt}/writers/696e74656772617465/clone"
    ));
    assert!(!git(&clone, &["ls-files", "--unmerged"]).is_empty());
    assert_eq!(project.canonical_head(), target);
    assert!(project.rows("SELECT id FROM approval").is_empty());
    assert!(
        project
            .rows("SELECT id FROM execution WHERE node = 'selector'")
            .is_empty()
    );
    continue_integration(&project);
    let bound = approval(&mut watch);
    assert_eq!(
        git(
            &project.canonical(),
            &[
                "show",
                &format!("{}:conflict.txt", bound["head"].as_str().unwrap())
            ]
        ),
        "resolved\n"
    );
}
