//! G10: The queue lands one at a time and re-judges what does not merge.

use e2e::*;
use serde_json::{Value, json};

fn unreviewed(extra: &str) -> String {
    config(extra).replace("review = [\"correctness\"]", "review = \"none\"")
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

/// Two approved candidates, the second red on its merged ref: the first
/// lands, the second gets one repair and a second red raises `red`.
///
/// The failing gate writes over 8000 bytes to stderr, then a marker to stdout;
/// the red's detail ends with the marker.
///
/// Sabotage: make `queue::returned` reset `landing_reds` with the repair;
/// the second red buys another repair instead of `red`. Or drop the `2>&1`
/// from `supervise::box_gate`'s wrapper; the detail ends with stderr, not the
/// marker.
#[test]
fn the_queue_lands_in_order_and_rejudges_a_red_merge() {
    const MARKER: &str = "the-gate-ends-here";
    let gate = format!(
        "[gates.clean]\ncommand = \"test ! -f bad.txt && exit 0; yes 'stderr noise noise noise' | head -n 1000 >&2; echo {MARKER}; exit 1\"\n"
    );
    let machine = Machine::new("g10-order", |request| {
        let prompt = request.prompt();
        let file = if prompt.contains("Second") {
            "bad.txt"
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
    for title in ["First", "Second"] {
        project.json(&["ticket", "new", "--title", title]);
    }
    let heads: Vec<String> = ["Y-1", "Y-2"]
        .iter()
        .map(|ticket| approval_of(&mut watch, ticket))
        .collect();
    for (ticket, head) in ["Y-1", "Y-2"].iter().zip(&heads) {
        project.json(&[
            "attempt", "approve", "--target", "main", ticket, "--head", head,
        ]);
    }

    let again = next_approval_of(&mut watch, "Y-2");
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-2", "--head", &again,
    ]);
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
    watch.find("Y-1 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });

    let landed: Vec<Value> = project
        .rows("SELECT ticket FROM audit WHERE event = 'landing.recorded' ORDER BY seq")
        .into_iter()
        .map(|row| row["ticket"].clone())
        .collect();
    assert_eq!(landed, [json!(1)]);
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
}

/// G2/G9/G10: main first warms the default inputs. A feature attempt then
/// starts at feature/a, despite its config declaring main. A missing target
/// creates no attempt; a revision-bound inactive edit supplies the control.
/// While the feature worker is held, target edits are refused by identity.
/// After approval, a conflicting operator commit supplies new feature gates
/// and image bytes. Repair must fetch feature/a's bundle, rebase its candidate
/// identity and take judgments with admitted inputs before it lands there.
///
/// Git commits, image marker files and host gate counters supply expected
/// values. Sabotage: use canonical HEAD for admission/landing, default
/// config/image inputs for the feature, main's repair bundle, or accept a
/// same-head approval naming main. Keep the old base, omit conflict paths,
/// or carry the retired approval into the repair. Existing G11 covers default switching
/// under a held default-bound execution and named sync's isolation.
/// G6 admission evidence uses literal selected definitions and host Git
/// objects. Sabotage: overwrite it from the changed repair base or regenerate
/// it from current templates on restart. No existing block observes immutable
/// admission evidence while the candidate base and current templates change.
/// Omit committed scaffold skills or their references from capture, including
/// test-audit's skill and sweep reference: complete fixture equality fails.
/// Substitute parser defaults for selected definitions: literal expectations
/// fail against the explicit non-default limits, timeouts and instructions
/// declared in configured(), independently of admission or inspection output.
/// G6: omit separate committed provenance, lose staging executable bits, or
/// build from a different whole context: host Git, staged bytes/mode and the
/// boxed COPY gate fail. Repair, sync and reopen exercise the changed-base case.
#[test]
fn a_ticket_binds_feature_inputs_repairs_and_lands_on_its_target() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g10-target", move |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.prompt().contains("Future target") {
            return act(
                request,
                vec![commit_file("future.txt", "future\\n", "Future work")],
            );
        }
        if request.opens() && request.last_user().contains("failed to land") {
            return Reply::Tools(vec![bash(
                "cd /workspace && git fetch -q /yard/input/target.bundle refs/heads/feature/a \
                 && git merge -q -X ours --no-edit FETCH_HEAD && echo merged",
            )]);
        }
        if request.opens() && request.prompt().contains("Feature target") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![commit_file(
                    "shared.txt",
                    "worker\n",
                    "Feature work",
                )])),
            );
        }
        act(
            request,
            vec![commit_file("main.txt", "main work\n", "Main work")],
        )
    });
    machine.start();
    let counter = machine.root.join("gate-inputs");
    let configured = |marker: &str| {
        let command = format!("printf '%s\\n' {marker} >> '{}'", counter.display());
        config(&format!(
            "[gates.counter]\ncommand = {}\ntimeout_minutes = 40\nstage = \"candidate\"\nruns_in = \"host\"\nenv = []\n\n\
             [gates.image]\ncommand = \"test $(cat /target-input) = {marker}\"\ntimeout_minutes = 41\nstage = \"candidate\"\nruns_in = \"box\"\nenv = []\n",
            serde_json::to_string(&command).unwrap(),
        ))
        .replace("[workflows.default]\n", "[workflows.default]\naccess = \"write\"\ninstructions = \"Implement the ticket.\"\nmax_session_executions = 7\ninactivity_timeout_minutes = 45\ntotal_work_timeout_minutes = 300\n")
        .replace("[review]\n", "[review]\ntimeout_minutes = 35\n")
        .replace("Review for correctness.", marker)
    };
    let project = Project::new(&machine, "p", &configured("main"));
    let dockerfile = project.git(&["show", "HEAD:.yard/Dockerfile"]);
    project.write("image-input", "main\n");
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}COPY image-input /target-input\nLABEL dev.yard.input=main\n"),
    );
    project.git(&["add", "image-input"]);
    project.git(&["commit", "--quiet", "-am", "Main image"]);
    project.json(&["sync"]);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Main target"]);
    let main_head = approval_of(&mut watch, "Y-1");
    project.json(&[
        "attempt", "approve", "Y-1", "--target", "main", "--head", &main_head,
    ]);
    watch.event("landing.recorded", &[]);
    let main = project.canonical_head();
    project.json(&["sync"]);

    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.write("shared.txt", "base\n");
    project.write("image-input", "feature-v1\n");
    project.write(
        ".agents/skills/admission/SKILL.md",
        "---\nname: admission\ndescription: Admission skill v1.\n---\nBody.\n",
    );
    project.write(".yard/config.toml", &configured("feature-v1"));
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
        project.path.join(".agents/skills/admission/SKILL.md"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}COPY image-input /target-input\nLABEL dev.yard.input=feature-v1\n"),
    );
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "Feature inputs"]);
    let base = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let object = |revision: &str| project.git(&["rev-parse", revision]).trim().to_string();
    let expected_inputs = json!({
        "source_commit": base,
        "definition_commit": base,
        "staging_commit": base,
        "image_commit": base,
        "build_context_tree": object(&format!("{base}^{{tree}}")),
        "config_blob": object(&format!("{base}:.yard/config.toml")),
        "dockerfile_blob": object(&format!("{base}:.yard/Dockerfile")),
        "trusted_staged_sources": [
            {"path": ".agents/skills/admission/SKILL.md",
             "blob": object(&format!("{base}:.agents/skills/admission/SKILL.md")), "executable": true},
            {"path": ".agents/skills/test-audit/SKILL.md",
             "blob": object(&format!("{base}:.agents/skills/test-audit/SKILL.md")), "executable": false},
            {"path": ".agents/skills/test-audit/sweep.md",
             "blob": object(&format!("{base}:.agents/skills/test-audit/sweep.md")), "executable": false},
            {"path": ".agents/skills/yard-drive/SKILL.md",
             "blob": object(&format!("{base}:.agents/skills/yard-drive/SKILL.md")), "executable": false},
            {"path": ".agents/skills/yard-file/SKILL.md",
             "blob": object(&format!("{base}:.agents/skills/yard-file/SKILL.md")), "executable": false},
            {"path": ".agents/skills/yard-operator/SKILL.md",
             "blob": object(&format!("{base}:.agents/skills/yard-operator/SKILL.md")), "executable": false},
            {"path": "AGENTS.md", "blob": object(&format!("{base}:AGENTS.md")), "executable": false}
        ]
    });
    project.json(&["sync", "feature/a"]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Feature target",
        "--target",
        "missing",
    ]);
    let missing = project.refused(&["attempt", "start", "Y-2"]);
    assert_eq!(missing["code"], "refused");
    assert_eq!(
        missing["data"],
        json!({ "reason": "missing_target", "target": "missing" })
    );
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE ticket = 2")
            .is_empty()
    );
    let stale = project.refused(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "0",
        "--target",
        "feature/a",
    ]);
    assert_eq!(stale["code"], "stale");
    assert_eq!(stale["data"], json!({ "expected": 0, "current": 1 }));
    project.json(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "1",
        "--target",
        "feature/a",
    ]);
    hold.wait_held();
    let admission = project.json(&["attempt", "show", "Y-2"])["attempt"]["admission"].clone();
    assert_eq!(admission["status"], "recorded");
    assert_eq!(admission["target"], "feature/a");
    assert_eq!(admission["config_head"], base);
    assert_eq!(admission["inputs"], expected_inputs);
    let staged = project
        .path
        .join(".yard/local/attempts/2/state/agent/skills/agents/admission/SKILL.md");
    assert_eq!(
        std::fs::read_to_string(&staged).unwrap(),
        project.git(&["show", &format!("{base}:.agents/skills/admission/SKILL.md")])
    );
    assert_ne!(
        std::fs::metadata(&staged).unwrap().permissions().mode() & 0o111,
        0
    );
    assert_eq!(
        admission["workflow"],
        json!({
            "implementer": "worker", "review": ["correctness"], "instructions": "Implement the ticket.",
            "read_only": false, "max_session_executions": 7,
            "inactivity_timeout_minutes": 45, "total_work_timeout_minutes": 300
        })
    );
    assert_eq!(
        admission["seats"],
        json!({
            "correctness": {"agent": "worker", "instructions": "feature-v1"}
        })
    );
    assert_eq!(
        admission["plan"]["nodes"]["review-0"]["runner"],
        json!({
            "kind": "agent", "agent": "worker", "instructions": "feature-v1"
        })
    );
    assert_eq!(
        admission["review"],
        json!({"max_rounds": 3, "timeout_minutes": 35, "blocking": 1})
    );
    assert_eq!(
        admission["gates"],
        json!([
            {"name": "counter", "command": format!("printf '%s\\n' feature-v1 >> '{}'", counter.display()),
             "timeout_minutes": 40, "stage": "candidate", "runs_in": "host", "env": []},
            {"name": "image", "command": "test $(cat /target-input) = feature-v1",
             "timeout_minutes": 41, "stage": "candidate", "runs_in": "box", "env": []}
        ])
    );
    let artifacts = machine.pinfold(&["artifacts"]);
    assert!(artifacts.status.success());
    let artifacts: Value = serde_json::from_slice(&artifacts.stdout).unwrap();
    let pi = artifacts
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "pi")
        .unwrap();
    assert_eq!(
        admission["agents"],
        json!({"worker": {
            "settings": {"harness": "pi", "provider": "openrouter", "model": "fake-model"},
            "runner": {"name": "pi", "version": pi["version"]}, "login_name": null
        }})
    );
    let binary_hash = if cfg!(target_os = "macos") {
        std::process::Command::new("shasum")
            .args(["-a", "256"])
            .arg(yard())
            .output()
            .unwrap()
    } else {
        std::process::Command::new("sha256sum")
            .arg(yard())
            .output()
            .unwrap()
    };
    assert!(binary_hash.status.success());
    let hash = String::from_utf8(binary_hash.stdout).unwrap();
    assert_eq!(
        admission["yard_runner"]["sha256"],
        hash.split_whitespace().next().unwrap()
    );
    assert_eq!(
        project.rows("SELECT target_branch, base FROM attempt WHERE ticket = 2"),
        vec![json!({ "target_branch": "feature/a", "base": base })]
    );
    let live = project.refused(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "2",
        "--target",
        "main",
    ]);
    assert_eq!(live["code"], "refused");
    let attempt_id = project.rows("SELECT id FROM attempt WHERE ticket = 2")[0]["id"].clone();
    assert_eq!(
        live["data"],
        json!({ "reason": "live_target", "attempt": attempt_id, "target": "feature/a" })
    );
    assert_eq!(
        project.rows("SELECT revision, target_branch FROM ticket WHERE id = 2"),
        vec![json!({ "revision": 2, "target_branch": "feature/a" })]
    );
    hold.release();
    let first = approval_of(&mut watch, "Y-2");
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{first}^")]).trim(),
        base
    );
    let approvals = project.rows("SELECT * FROM approval");
    let stale = project.refused(&[
        "attempt", "approve", "Y-2", "--target", "main", "--head", &first,
    ]);
    assert_eq!(stale["code"], "stale");
    assert_eq!(
        stale["data"],
        json!({ "expected": "main", "current": "feature/a" })
    );
    assert_eq!(project.rows("SELECT * FROM approval"), approvals);

    // G2: template sync keeps judgments; conflict repair keeps admitted inputs.
    // Sabotage: late gate/reviewer/image lookup or staging from repaired base.
    project.write("shared.txt", "operator\n");
    project.write("image-input", "feature-v2\n");
    project.write(
        ".agents/skills/admission/SKILL.md",
        "---\nname: admission\ndescription: Admission skill v2.\n---\nBody.\n",
    );
    project.write(".yard/config.toml", &configured("feature-v2"));
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}COPY image-input /target-input\nLABEL dev.yard.input=feature\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Conflict and feature inputs"]);
    let target = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/a"]);
    project.json(&[
        "attempt",
        "approve",
        "Y-2",
        "--target",
        "feature/a",
        "--head",
        &first,
    ]);
    let repaired = next_approval_of(&mut watch, "Y-2");
    assert_ne!(repaired, first);
    let repair_prompt = machine
        .model
        .requests()
        .into_iter()
        .find(|request| request.opens() && request.last_user().contains("failed to land"))
        .unwrap()
        .last_user();
    assert!(repair_prompt.contains("shared.txt"), "{repair_prompt}");
    assert_eq!(
        project.rows("SELECT target_branch, base, head FROM attempt WHERE ticket = 2"),
        vec![json!({ "target_branch": "feature/a", "base": target, "head": repaired })]
    );
    assert_eq!(
        project.json(&["attempt", "show", "Y-2"])["attempt"]["admission"],
        admission
    );
    assert_eq!(project.rows("SELECT DISTINCT plan_revision FROM execution WHERE attempt = 2 AND kind IN ('gate', 'review')"), vec![json!({"plan_revision": 1})]);
    // Restart at rest, after the repair's boxes have ended. Today's templates
    // and the mutable candidate base both differ from recorded admission inputs.
    assert!(
        project
            .rows("SELECT id FROM execution WHERE status = 'running'")
            .is_empty()
    );
    let results = project.rows("SELECT id, image_id, digest, plan_revision FROM execution WHERE attempt = 2 AND kind IN ('gate', 'review') ORDER BY id");
    let seq = project.json(&["status"])["seq"].as_i64().unwrap();
    drop(watch);
    machine.kill();
    machine.start();
    let mut watch = project.watch(seq);
    assert_eq!(
        project.json(&["attempt", "show", "Y-2"])["attempt"]["admission"],
        admission
    );
    assert_eq!(
        project.rows("SELECT state FROM approval WHERE attempt = 2"),
        vec![json!({ "state": "retired" })]
    );
    assert_eq!(project.rows("SELECT id, image_id, digest, plan_revision FROM execution WHERE attempt = 2 AND kind IN ('gate', 'review') ORDER BY id"), results);
    // The repair alone cannot land; its new exact candidate still awaits approval.
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    project.json(&[
        "attempt",
        "approve",
        "Y-2",
        "--target",
        "feature/a",
        "--head",
        &repaired,
    ]);
    let landed = watch.until("feature landing", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
    assert_eq!(landed["data"]["target"], "feature/a");
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        repaired
    );
    assert_eq!(project.canonical_head(), main);
    assert_eq!(
        git(
            &project.canonical(),
            &["show", "refs/heads/feature/a:shared.txt"]
        ),
        "worker\n"
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["show", "refs/heads/feature/a:.yard/config.toml"]
        ),
        configured("feature-v2")
    );
    assert_eq!(project.json(&["ticket", "show", "Y-2"])["state"], "done");
    // A future admission resolves the fixture's v2 definitions and Dockerfile.
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Future target",
        "--target",
        "feature/a",
    ]);
    approval_of(&mut watch, "Y-3");
    let future = project.json(&["attempt", "show", "Y-3"])["attempt"]["admission"].clone();
    assert_eq!(future["config_head"], repaired);
    assert_eq!(future["revision"], 1);
    assert_eq!(
        future["gates"][1]["command"],
        "test $(cat /target-input) = feature-v2"
    );
    assert_eq!(future["seats"]["correctness"]["instructions"], "feature-v2");
    assert_eq!(
        future["inputs"]["dockerfile_blob"],
        object(&format!("{target}:.yard/Dockerfile"))
    );
    project.json(&["ticket", "park", "Y-3"]);
    project.json(&["attempt", "abandon", "Y-3"]);
    assert_eq!(
        std::fs::read_to_string(&counter).unwrap(),
        "main\nmain\nfeature-v1\nfeature-v1\nfeature-v1\nfeature-v2\n"
    );
    let images = project
        .rows("SELECT DISTINCT image_id FROM execution WHERE name = 'image' ORDER BY image_id");
    assert!(images.len() >= 3, "{images:?}");
    assert!(
        images
            .iter()
            .all(|row| row["image_id"].as_str().is_some_and(|id| !id.is_empty()))
    );
    let digests =
        project.rows("SELECT DISTINCT digest FROM execution WHERE name = 'image' ORDER BY digest");
    assert_eq!(digests.len(), 3, "{digests:?}");
    // Both implementer executions use trusted v1 staging even after base repair.
    for request in machine.model.requests().iter().filter(|request| {
        request.opens()
            && (request.last_user().contains("Feature target")
                || request.last_user().contains("failed to land"))
    }) {
        assert!(
            request.system().contains("Admission skill v1."),
            "{}",
            request.prompt()
        );
        assert!(
            !request.system().contains("Admission skill v2."),
            "{}",
            request.prompt()
        );
    }
    let reviews: Vec<_> = machine
        .model
        .requests()
        .into_iter()
        .filter(|request| request.opens() && request.has_tool("yard_publish_review"))
        .map(|request| request.context())
        .collect();
    for (prompt, marker) in reviews
        .iter()
        .zip(["main", "feature-v1", "feature-v1", "feature-v2"])
    {
        assert!(prompt.contains(marker), "{prompt}");
        if marker.starts_with("feature-") {
            let skill = if marker == "feature-v1" {
                "Admission skill v1."
            } else {
                "Admission skill v2."
            };
            assert!(prompt.contains(skill), "{prompt}");
        }
    }
    assert_eq!(reviews.len(), 4);
}
