//! G10: The queue lands one at a time and re-judges what does not merge.

use e2e::*;
use serde_json::{Value, json};
use std::io::Write;

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
/// Omit committed scaffold skills from capture: complete fixture equality fails.
/// Substitute parser defaults for selected definitions: literal expectations
/// fail against the explicit non-default limits, timeouts and instructions
/// declared in configured(), independently of admission or inspection output.
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
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}RUN echo main > /target-input\n"),
    );
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
    project.write(
        ".agents/skills/admission/SKILL.md",
        "---\nname: admission\ndescription: Admission skill v1.\n---\nBody.\n",
    );
    project.write(".yard/config.toml", &configured("feature-v1"));
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}RUN echo feature-v1 > /target-input\n"),
    );
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "Feature inputs"]);
    let base = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let object = |revision: &str| project.git(&["rev-parse", revision]).trim().to_string();
    let expected_inputs = json!({
        "source_commit": base,
        "build_context_tree": object(&format!("{base}^{{tree}}")),
        "config_blob": object(&format!("{base}:.yard/config.toml")),
        "dockerfile_blob": object(&format!("{base}:.yard/Dockerfile")),
        "trusted_staged_sources": [
            {"path": ".agents/skills/admission/SKILL.md",
             "blob": object(&format!("{base}:.agents/skills/admission/SKILL.md")), "executable": false},
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
    project.write(
        ".agents/skills/admission/SKILL.md",
        "---\nname: admission\ndescription: Admission skill v2.\n---\nBody.\n",
    );
    project.write(".yard/config.toml", &configured("feature-v2"));
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}RUN echo feature-v2 > /target-input\n"),
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

fn target_head(project: &Project, branch: &str) -> String {
    git(
        &project.canonical(),
        &["rev-parse", &format!("refs/heads/{branch}")],
    )
    .trim()
    .to_string()
}

// Approval-needed candidates retain lanes; callers allow one per candidate.
fn target_candidates(project: &Project, watch: &mut Watch, branches: &[&str]) -> Vec<String> {
    for (index, branch) in branches.iter().enumerate() {
        if project.git(&["branch", "--list", branch]).trim().is_empty() {
            project.git(&["branch", branch]);
            project.json(&["sync", branch]);
        }
        project.json(&[
            "ticket",
            "new",
            "--title",
            &format!("Candidate {index}"),
            "--target",
            branch,
        ]);
    }
    (1..=branches.len())
        .map(|id| {
            approval_of(watch, &format!("Y-{id}"));
            let attempt =
                project.rows(&format!("SELECT id FROM attempt WHERE ticket = {id}"))[0]["id"]
                    .as_i64()
                    .unwrap();
            git(
                &project
                    .path
                    .join(format!(".yard/local/attempts/{attempt}/clone")),
                &["rev-parse", "HEAD"],
            )
            .trim()
            .to_string()
        })
        .collect()
}

fn target_worker(request: &ModelRequest) -> Reply {
    // Each ticket has independent bytes, including the second feature/a.
    let prompt = request.prompt();
    let file = if prompt.contains("Candidate 0") {
        "first.txt"
    } else if prompt.contains("Candidate 1") {
        "second.txt"
    } else {
        "third.txt"
    };
    act(request, vec![commit_file(file, "feature\n", "Feature")])
}

struct HeldTarget {
    armed: std::path::PathBuf,
    said: std::path::PathBuf,
    release: std::fs::File,
}

impl HeldTarget {
    fn release(&self) {
        let _ = std::fs::remove_file(&self.armed);
        let _ = (&self.release).write_all(b"go\n");
    }
}

impl Drop for HeldTarget {
    fn drop(&mut self) {
        self.release();
    }
}

struct HeldModel(Latch);

impl Drop for HeldModel {
    fn drop(&mut self) {
        self.0.release();
    }
}

fn held_effect(machine: &Machine, name: &str) -> HeldTarget {
    let armed = machine.root.join(format!("{name}-armed"));
    let said = machine.root.join(format!("{name}-said"));
    let release = machine.root.join(format!("{name}-release"));
    for path in [&said, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }
    let release = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(release)
        .unwrap();
    HeldTarget {
        armed,
        said,
        release,
    }
}

fn hold_feature_a(machine: &Machine) -> HeldTarget {
    let held = held_effect(machine, "target");
    machine.wrapper("git", &format!(
        "case \" $* \" in *' update-ref --no-deref refs/heads/feature/a '*) if [ -e '{}' ]; then echo held > '{}'; read line < '{}'; fi;; esac",
        held.armed.display(), held.said.display(), machine.root.join("target-release").display()
    ));
    held
}

/// G10/G3: hold feature/a after intent, while b lands and the second a waits.
/// Expected heads and ancestry come from git and the fixture's approved heads.
/// Sabotage: use a project-global busy predicate or canonical mutation lock;
/// b cannot land. Remove target serialization; the second a starts while held.
#[test]
fn unrelated_targets_land_while_the_same_target_waits() {
    let machine = Machine::new("g10-targets", target_worker);
    let held = hold_feature_a(&machine);
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &unreviewed("").replace("max_lanes = 2", "max_lanes = 3\nmax_executions = 2"),
    );
    let mut watch = project.watch(0);
    let branches = ["feature/a", "feature/b", "feature/a"];
    let heads = target_candidates(&project, &mut watch, &branches);
    let old = target_head(&project, "feature/a");
    let main = project.canonical_head();
    std::fs::write(&held.armed, "").unwrap();
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "feature/a",
        "Y-1",
        "--head",
        &heads[0],
    ]);
    assert_eq!(watch.said(&held.said), "held\n");
    for id in [3, 2] {
        project.json(&[
            "attempt",
            "approve",
            "--target",
            branches[id - 1],
            &format!("Y-{id}"),
            "--head",
            &heads[id - 1],
        ]);
    }
    watch.find("b lands while a is held", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
    assert_eq!(target_head(&project, "feature/b"), heads[1]);
    assert_eq!(target_head(&project, "feature/a"), old);
    assert!(project.rows("SELECT execution.id FROM execution JOIN attempt ON attempt.id = execution.attempt WHERE kind = 'landing' AND attempt.ticket = 3").is_empty());
    let intent = project.rows("SELECT id, intent_ref FROM execution WHERE intent_state = 'open'");
    assert_eq!(intent.len(), 1);
    assert_eq!(intent[0]["intent_ref"], "refs/heads/feature/a");
    // Named sync refuses the unresolved target by identity; b remains usable.
    let refused = project.refused(&["sync", "feature/a"]);
    assert_eq!(refused["code"], "refused");
    assert_eq!(refused["data"]["intent"], intent[0]["id"]);
    project.git(&["checkout", "--quiet", "feature/b"]);
    project.json(&["sync", "feature/b"]);
    held.release();
    for ticket in ["Y-1", "Y-3"] {
        watch.find(ticket, |event| {
            event["event"] == "landing.recorded" && event["ticket"] == ticket
        });
    }
    for head in [&heads[0], &heads[2]] {
        git(
            &project.canonical(),
            &["merge-base", "--is-ancestor", head, "refs/heads/feature/a"],
        );
    }
    assert_eq!(project.rows("SELECT ticket FROM audit WHERE event = 'landing.recorded' AND ticket IN (1, 3) ORDER BY seq"),
        vec![json!({"ticket": 1}), json!({"ticket": 3})]);
    assert_eq!(project.canonical_head(), main);
}

/// G4: at cap one, a held target Git effect excludes two other approved
/// targets and a new real worker. Release lets all waiting targets succeed.
/// The FIFO holds the actual child; pinfold supplies box absence, and the
/// model fixture supplies absence of worker actions. No elapsed-time assertion.
/// Sabotage: bypass capacity::effect for independent-target landing, or count
/// boxes alone; the waiting targets act instead of reporting capacity waits.
/// Launch a box without recording its handle: the scoped pinfold absence
/// observation still sees it. The label comes from the fixture's held worker.
#[test]
fn independent_target_landings_share_the_project_execution_cap() {
    let model_hold = Latch::new();
    let worker_hold = model_hold.clone();
    let machine = Machine::new("g4-targets", move |request| {
        let reply = target_worker(request);
        if request.opens() && request.prompt().contains("Candidate 0") {
            Reply::Hold(worker_hold.clone(), Box::new(reply))
        } else {
            reply
        }
    });
    let held = hold_feature_a(&machine);
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &unreviewed("").replace("max_lanes = 2", "max_lanes = 3\nmax_executions = 1"),
    );
    let mut watch = project.watch(0);
    let branches = ["feature/a", "feature/b", "feature/c"];
    let (heads, label) = std::thread::scope(|scope| {
        let _release = HeldModel(model_hold.clone());
        let candidates = scope.spawn(|| target_candidates(&project, &mut watch, &branches));
        model_hold.wait_held();
        let owners = project.rows("SELECT handle FROM execution WHERE reserved = 1");
        let worker = machine
            .boxes("dev.yard.project")
            .into_iter()
            .find(|listed| listed["name"] == owners[0]["handle"])
            .expect("the fixture's held worker has a pinfold box");
        let label = format!(
            "dev.yard.project={}",
            worker["labels"]["dev.yard.project"].as_str().unwrap()
        );
        assert_eq!(machine.boxes(&label).len(), 1);
        model_hold.release();
        (candidates.join().unwrap(), label)
    });
    std::fs::write(&held.armed, "").unwrap();
    project.json(&[
        "attempt",
        "approve",
        "--target",
        branches[0],
        "Y-1",
        "--head",
        &heads[0],
    ]);
    assert_eq!(watch.said(&held.said), "held\n");
    for id in [2, 3] {
        project.json(&[
            "attempt",
            "approve",
            "--target",
            branches[id - 1],
            &format!("Y-{id}"),
            "--head",
            &heads[id - 1],
        ]);
        let waiting = watch.find("target waits for capacity", |event| {
            event["event"] == "execution.waiting" && event["ticket"] == format!("Y-{id}")
        });
        assert_eq!(waiting["data"]["reason"], "capacity");
        assert_eq!(
            target_head(&project, branches[id - 1]),
            project.canonical_head()
        );
    }
    let turns = machine.model.requests().len();
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Candidate 4",
        "--target",
        "feature/c",
    ]);
    let waiting = watch.find("worker waits for the host slot", |event| {
        event["event"] == "execution.waiting" && event["ticket"] == "Y-4"
    });
    assert_eq!(waiting["data"]["reason"], "capacity");
    assert_eq!(machine.model.requests().len(), turns);
    let owners = project.rows("SELECT kind, intent_ref FROM execution WHERE reserved = 1");
    assert_eq!(
        owners,
        vec![json!({"kind": "landing", "intent_ref": "refs/heads/feature/a"})]
    );
    assert!(machine.boxes(&label).is_empty());
    // Park before abandonment so this ticket cannot be readmitted. The
    // same-target worker/landing overlap is held explicitly in the next case.
    project.json(&["ticket", "park", "Y-4"]);
    project.json(&["attempt", "abandon", "Y-4"]);
    held.release();
    for id in 1..=3 {
        watch.find("waiting target lands", |event| {
            event["event"] == "landing.recorded" && event["ticket"] == format!("Y-{id}")
        });
        assert_eq!(target_head(&project, branches[id - 1]), heads[id - 1]);
    }
}

/// G4/G10: two real held effects drain after the live cap is lowered to one.
/// A landing gate holds one slot; a same-target worker's clone holds the other.
/// Releasing the gate makes final landing wait for capacity. Releasing the
/// clone must let the worker launch, finish and free the slot for landing.
/// Sabotage: take the final landing target lock before capacity::effect;
/// the reserved worker cannot launch and neither positive control completes.
/// Git supplies the target heads; the fixture supplies held-child identities
/// and model actions. This forces the lock cycle instead of racing admission.
#[test]
fn same_target_worker_and_landing_drain_at_one_slot() {
    let model_hold = Latch::new();
    let worker_hold = model_hold.clone();
    let machine = Machine::new("g4-target-drain", move |request| {
        let reply = target_worker(request);
        if request.opens() && request.prompt().contains("Candidate 1") {
            Reply::Hold(worker_hold.clone(), Box::new(reply))
        } else {
            reply
        }
    });
    let gate = held_effect(&machine, "gate");
    let clone = held_effect(&machine, "clone");
    machine.wrapper("git", &format!(
        "case \" $* \" in *' clone '*) if [ -e '{}' ]; then echo held > '{}'; read line < '{}'; fi;; esac",
        clone.armed.display(), clone.said.display(), machine.root.join("clone-release").display()
    ));
    let gates = format!(
        "[gates.held]\nstage = \"landing\"\nruns_in = \"host\"\ncommand = \"if [ -e '{}' ]; then echo held > '{}'; read line < '{}'; fi\"\n",
        gate.armed.display(),
        gate.said.display(),
        machine.root.join("gate-release").display()
    );
    let configured =
        unreviewed(&gates).replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 2");
    machine.start();
    let project = Project::new(&machine, "p", &configured);
    let mut watch = project.watch(0);
    let head = target_candidates(&project, &mut watch, &["feature/a"]).remove(0);
    let old = target_head(&project, "feature/a");
    std::fs::write(&gate.armed, "").unwrap();
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "feature/a",
        "Y-1",
        "--head",
        &head,
    ]);
    assert_eq!(watch.said(&gate.said), "held\n");
    std::fs::write(&clone.armed, "").unwrap();
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Candidate 1",
        "--target",
        "feature/a",
    ]);
    assert_eq!(watch.said(&clone.said), "held\n");
    let owners = project.rows(
        "SELECT execution.kind, attempt.target_branch, execution.handle FROM execution JOIN attempt ON attempt.id = execution.attempt WHERE reserved = 1 ORDER BY execution.kind",
    );
    assert_eq!(owners.len(), 2, "{owners:?}");
    assert_eq!(owners[0]["kind"], "gate");
    assert_eq!(
        owners[1],
        json!({"kind": "implementation", "target_branch": "feature/a", "handle": null})
    );
    assert_eq!(owners[0]["target_branch"], "feature/a");
    project.reconfigure(&configured.replace("max_executions = 2", "max_executions = 1"));
    gate.release();
    let waiting = watch.find("final landing waits for the worker slot", |event| {
        event["event"] == "execution.waiting" && event["ticket"] == "Y-1"
    });
    assert_eq!(waiting["data"]["reason"], "capacity");
    assert_eq!(target_head(&project, "feature/a"), old);
    assert_eq!(
        project.rows("SELECT intent_ref FROM execution WHERE kind = 'landing'"),
        vec![json!({"intent_ref": null})]
    );
    let _release = HeldModel(model_hold.clone());
    clone.release();
    model_hold.wait_held();
    let owners = project.rows(
        "SELECT execution.kind, attempt.ticket, execution.handle FROM execution JOIN attempt ON attempt.id = execution.attempt WHERE reserved = 1",
    );
    assert_eq!(owners.len(), 1, "{owners:?}");
    assert_eq!(owners[0]["kind"], "implementation");
    assert_eq!(owners[0]["ticket"], 2);
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|listed| listed["name"] == owners[0]["handle"])
    );
    assert_eq!(target_head(&project, "feature/a"), old);
    model_hold.release();
    approval_of(&mut watch, "Y-2");
    let attempt = project.rows("SELECT id FROM attempt WHERE ticket = 2")[0]["id"]
        .as_i64()
        .unwrap();
    let second = git(
        &project
            .path
            .join(format!(".yard/local/attempts/{attempt}/clone")),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    watch.find("first target lands after worker drainage", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    assert_eq!(target_head(&project, "feature/a"), head);
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "feature/a",
        "Y-2",
        "--head",
        &second,
    ]);
    watch.find("same-target worker candidate lands", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
    for candidate in [&head, &second] {
        git(
            &project.canonical(),
            &[
                "merge-base",
                "--is-ancestor",
                candidate,
                "refs/heads/feature/a",
            ],
        );
    }
}
