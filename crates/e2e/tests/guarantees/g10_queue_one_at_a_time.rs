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
/// lands, the second gets one repair and a second red raises `red`. No two
/// landings overlap.
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

/// G2/G9/G10: main first warms the default inputs. A feature attempt then
/// starts at feature/a, despite its config declaring main. A missing target
/// creates no attempt; a revision-bound inactive edit supplies the control.
/// While the feature worker is held, target edits are refused by identity.
/// After approval, a conflicting operator commit supplies new feature gates
/// and image bytes. Repair must fetch feature/a's bundle, rebase its candidate
/// identity and take fresh branch judgments before it lands there.
///
/// Git commits, image marker files and host gate counters supply expected
/// values. Sabotage: use canonical HEAD for admission/landing, default
/// config/image inputs for the feature, main's repair bundle, or accept a
/// same-head approval naming main. Keep the old base, omit conflict paths,
/// or carry the retired approval into the repair. Existing G11 covers default switching
/// under a held default-bound execution and named sync's isolation.
#[test]
fn a_ticket_binds_feature_inputs_repairs_and_lands_on_its_target() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g10-target", move |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
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
            "[gates.counter]\ncommand = {}\nstage = \"candidate\"\nruns_in = \"host\"\n\n\
             [gates.image]\ncommand = \"test $(cat /target-input) = {marker}\"\nstage = \"candidate\"\n",
            serde_json::to_string(&command).unwrap(),
        )).replace("Review for correctness.", marker)
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
    project.write(".yard/config.toml", &configured("feature-v1"));
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}RUN echo feature-v1 > /target-input\n"),
    );
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "Feature inputs"]);
    let base = project.git(&["rev-parse", "HEAD"]).trim().to_string();
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

    // The synced inputs supersede judgments before the conflict is attempted.
    project.write("shared.txt", "operator\n");
    project.write(".yard/config.toml", &configured("feature-v2"));
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}RUN echo feature-v2 > /target-input\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Conflict and feature inputs"]);
    let target = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/a"]);
    assert_eq!(next_approval_of(&mut watch, "Y-2"), first);
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
        project.rows("SELECT state FROM approval WHERE attempt = 2"),
        vec![json!({ "state": "retired" })]
    );
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
    assert_eq!(
        std::fs::read_to_string(&counter).unwrap(),
        "main\nmain\nfeature-v1\nfeature-v2\nfeature-v2\nfeature-v2\n"
    );
    let images = project
        .rows("SELECT DISTINCT image_id FROM execution WHERE name = 'image' ORDER BY image_id");
    assert_eq!(images.len(), 3, "{images:?}");
    assert!(
        images
            .iter()
            .all(|row| row["image_id"].as_str().is_some_and(|id| !id.is_empty()))
    );
    let digests =
        project.rows("SELECT DISTINCT digest FROM execution WHERE name = 'image' ORDER BY digest");
    assert_eq!(digests.len(), 3, "{digests:?}");
    let reviews: Vec<_> = machine
        .model
        .requests()
        .into_iter()
        .filter(|request| request.opens() && request.has_tool("yard_publish_review"))
        .map(|request| request.prompt())
        .collect();
    for (prompt, marker) in reviews
        .iter()
        .zip(["main", "feature-v1", "feature-v2", "feature-v2"])
    {
        assert!(prompt.contains(marker), "{prompt}");
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

fn hold_feature_a(machine: &Machine) -> HeldTarget {
    let armed = machine.root.join("armed");
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    for path in [&said, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }
    machine.wrapper("git", &format!(
        "case \" $* \" in *' update-ref --no-deref refs/heads/feature/a '*) if [ -e '{}' ]; then echo held > '{}'; read line < '{}'; fi;; esac",
        armed.display(), said.display(), release.display()
    ));
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
    // Cancel the new worker so only the three approved targets compete next.
    project.json(&["attempt", "abandon", "Y-4"]);
    held.release();
    for id in 1..=3 {
        watch.find("waiting target lands", |event| {
            event["event"] == "landing.recorded" && event["ticket"] == format!("Y-{id}")
        });
        assert_eq!(target_head(&project, branches[id - 1]), heads[id - 1]);
    }
}
