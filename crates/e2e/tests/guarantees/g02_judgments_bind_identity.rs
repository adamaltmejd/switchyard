//! G2: Judgments bind exact identity.

use e2e::*;
use serde_json::{Value, json};
use std::io::Write;

const GATE: &str = "[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n";

/// Seats publish no findings; the implementer commits feature.txt, or runs
/// `repair` when its execution's prompt carries `key`.
fn worker(key: &'static str, repair: ToolCall) -> impl Fn(&ModelRequest) -> Reply + Send + Sync {
    let repair = std::sync::Mutex::new(Some(repair));
    move |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains(key) {
            let call = repair.lock().unwrap().take().expect("one repair");
            return Reply::Tools(vec![call]);
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    }
}

fn unreviewed(extra: &str) -> String {
    config(extra).replace("review = [\"correctness\"]", "review = \"none\"")
}

fn auto(config: String) -> String {
    config.replace("approve = \"manual\"", "approve = \"auto\"")
}

/// The next attention item, which must be an approval: its data.
#[track_caller]
fn approval(watch: &mut Watch) -> Value {
    let raised = watch.attention();
    assert_eq!(raised["data"]["kind"], "approval", "{raised}");
    raised["data"].clone()
}

fn head(item: &Value) -> String {
    item["payload"]["head"].as_str().unwrap().to_string()
}

/// Gate and review executions on `head`, in order: (kind, ticket revision).
fn judged(project: &Project, head: &str) -> Vec<(String, i64)> {
    project
        .rows(&format!(
            "SELECT kind, ticket_revision FROM execution
             WHERE kind IN ('gate', 'review') AND head = '{head}' ORDER BY id"
        ))
        .iter()
        .map(|row| {
            (
                row["kind"].as_str().unwrap().to_string(),
                row["ticket_revision"].as_i64().unwrap(),
            )
        })
        .collect()
}

fn kinds(project: &Project) -> Vec<String> {
    project
        .rows("SELECT kind FROM execution WHERE kind IN ('gate', 'review') ORDER BY id")
        .iter()
        .map(|row| row["kind"].as_str().unwrap().to_string())
        .collect()
}

fn last_seq(project: &Project) -> i64 {
    project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap()
}

/// A passed gate and review, then a new commit: the candidate is
/// unverified. The new commit keeps the tree, so only the head tells the
/// two candidates apart.
///
/// Sabotage: drop `head` from `checks::current`'s match; the empty commit
/// reaches approval on the first commit's gate and review.
#[test]
fn a_new_commit_leaves_the_candidate_unverified() {
    let machine = Machine::new(
        "g2-commit",
        worker(
            "Once more",
            bash("cd /workspace && git commit -q --allow-empty -m Again && echo committed"),
        ),
    );
    machine.start();
    let project = Project::new(&machine, "p", &config(GATE));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = head(&approval(&mut watch));
    project.json(&[
        "attempt",
        "reject",
        "Y-1",
        "--head",
        &first,
        "--text",
        "Once more",
    ]);
    let second = approval(&mut watch);

    let canonical = project.canonical();
    let tree = |commit: &str| git(&canonical, &["rev-parse", &format!("{commit}^{{tree}}")]);
    assert_ne!(head(&second), first);
    assert_eq!(tree(&head(&second)), tree(&first));
    let once = vec![("gate".to_string(), 1), ("review".to_string(), 1)];
    assert_eq!(judged(&project, &first), once);
    assert_eq!(judged(&project, &head(&second)), once);
    let checks: Vec<i64> = project
        .rows(&format!(
            "SELECT id FROM \"check\" WHERE head = '{}' ORDER BY id",
            head(&second)
        ))
        .iter()
        .map(|row| row["id"].as_i64().unwrap())
        .collect();
    assert_eq!(second["payload"]["checks"], json!(checks));
}

/// A passed gate and review, then a ticket edit: the candidate is
/// unverified, though its head is the same.
///
/// Sabotage: drop `ticket_revision` from `checks::current`'s match; the
/// approval comes back on revision 1's gate and review.
#[test]
fn a_ticket_edit_leaves_the_candidate_unverified() {
    let machine = Machine::new("g2-edit", worker("never", bash("true")));
    machine.start();
    let project = Project::new(&machine, "p", &config(GATE));
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create feature.txt",
    ]);
    let first = approval(&mut watch);
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"]
        .as_i64()
        .unwrap();
    assert_eq!(first["payload"]["revision"], revision);
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision.to_string(),
        "--body",
        "Create feature.txt and nothing else",
    ]);
    let second = approval(&mut watch);

    assert_eq!(head(&second), head(&first));
    assert_eq!(second["payload"]["revision"], revision + 1);
    assert_eq!(
        judged(&project, &head(&first)),
        vec![
            ("gate".to_string(), revision),
            ("review".to_string(), revision),
            ("gate".to_string(), revision + 1),
            ("review".to_string(), revision + 1),
        ]
    );
}

/// A synced gate change reruns the gate and keeps the review.
///
/// Sabotage: hash the gates into `review_digest` too; the review reruns.
#[test]
fn a_synced_gate_change_reruns_the_gate_and_keeps_the_review() {
    let machine = Machine::new("g2-gate", worker("never", bash("true")));
    machine.start();
    let project = Project::new(&machine, "p", &config(GATE));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = approval(&mut watch);
    assert_eq!(kinds(&project), ["gate", "review"]);
    let review = first["payload"]["checks"][1].clone();

    project.reconfigure(&config(&GATE.replace("test -f", "test -s")));
    let second = approval(&mut watch);
    assert_eq!(head(&second), head(&first));
    assert_eq!(kinds(&project), ["gate", "review", "gate"]);
    let gate =
        project.rows("SELECT id FROM \"check\" WHERE kind = 'gate' ORDER BY id")[1]["id"].clone();
    assert_eq!(second["payload"]["checks"], json!([gate, review]));
}

/// A synced seat change reruns the review and keeps the gate.
///
/// Sabotage: hash the seats into `gate_digest` too; the gate reruns.
#[test]
fn a_synced_seat_change_reruns_the_review_and_keeps_the_gate() {
    let machine = Machine::new("g2-seat", worker("never", bash("true")));
    machine.start();
    let project = Project::new(&machine, "p", &config(GATE));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = approval(&mut watch);
    assert_eq!(kinds(&project), ["gate", "review"]);
    let gate = first["payload"]["checks"][0].clone();

    project.reconfigure(&config(GATE).replace(
        "Review for correctness.",
        "Review for correctness and naming.",
    ));
    let second = approval(&mut watch);
    assert_eq!(head(&second), head(&first));
    assert_eq!(kinds(&project), ["gate", "review", "review"]);
    let review =
        project.rows("SELECT id FROM \"check\" WHERE kind = 'review' ORDER BY id")[1]["id"].clone();
    assert_eq!(second["payload"]["checks"], json!([gate, review]));
}

/// An approval given with `--head`, then a red landing gate and a repair
/// commit: the approval retires with the red landing and does not carry to
/// the repair head, which raises `approval` again. Control: the repair head
/// is approved and lands.
///
/// Sabotage: make `queue::returned` leave the approval active; the queue
/// lands the first head again and its second red raises `red`.
#[test]
fn an_approval_does_not_carry_to_a_repair_commit() {
    let machine = Machine::new(
        "g2-repair",
        worker(
            "failed on the merged ref",
            commit_file("fixed.txt", "fixed\n", "Fix"),
        ),
    );
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &unreviewed("[gates.merged]\ncommand = \"test -f fixed.txt\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = head(&approval(&mut watch));
    project.json(&["attempt", "approve", "Y-1", "--head", &first]);
    let second = head(&approval(&mut watch));

    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{second}^")]).trim(),
        first
    );
    let landings = project.rows("SELECT head, outcome FROM execution WHERE kind = 'landing'");
    assert_eq!(landings, vec![json!({ "head": first, "outcome": "red" })]);

    project.json(&["attempt", "approve", "Y-1", "--head", &second]);
    watch.event("landing.recorded", &[]);
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", &second, "refs/heads/main"],
    );
}

/// Under `auto` a protected path still raises `approval`. The candidate
/// renames `AGENTS.md` away, so only its old path is protected. Control: an
/// unprotected candidate in the same project is approved automatically and
/// lands.
///
/// Sabotage: drop `--no-renames` from `git::changed_paths`; the rename shows
/// only its new path and the candidate is approved automatically.
#[test]
fn auto_approval_raises_approval_on_a_protected_path() {
    let machine = Machine::new("g2-protected", |request| {
        if request.prompt().contains("Move the guidance") {
            return act(
                request,
                vec![bash(
                    "cd /workspace && mkdir -p docs && git mv AGENTS.md docs/AGENTS.md \
                     && git commit -q -m Move && echo committed",
                )],
            );
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &auto(unreviewed("")));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Move the guidance"]);
    project.json(&["ticket", "new", "--title", "Add feature"]);

    let decided = watch.find("Y-1 decided", |event| {
        event["ticket"] == "Y-1"
            && (event["event"] == "attention.raised" || event["event"] == "approval.given")
    });
    assert_eq!(decided["event"], "attention.raised", "{decided}");
    assert_eq!(decided["data"]["kind"], "approval");
    assert_eq!(
        decided["data"]["payload"]["protected"],
        json!(["AGENTS.md"])
    );

    watch.find("Y-2 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
    assert!(
        project
            .rows("SELECT approval.id FROM approval JOIN attempt ON attempt.id = approval.attempt WHERE attempt.ticket = 1")
            .is_empty()
    );
}

/// An automatic approval, a landing held in its host gate, and a sync that
/// changes policy: the landing withdraws the approval when it records its
/// intent, raises `approval`, and reruns no check. Control: the operator's
/// approval of the same head lands.
///
/// Sabotage for "reruns no check": hash `approve` or `protected_paths` into
/// the gate digest; the sync reruns the candidate gate.
fn policy_change_withdraws(test: &str, change: impl Fn(String) -> String) {
    let machine = notes_machine(test);
    let (project, mut watch, mut release, head) = held_after_policy_change(&machine, change);
    release.write_all(b"go\n").unwrap();
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn", "{ended}");
    let raised = approval(&mut watch);
    assert_eq!(raised["payload"]["head"], head.as_str());
    assert_eq!(
        project.rows("SELECT actor, state FROM approval"),
        vec![json!({ "actor": "auto", "state": "withdrawn" })]
    );
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'gate' AND reason = 'candidate'")
            .len(),
        1,
        "a check reran"
    );

    project.json(&["attempt", "approve", "Y-1", "--head", &head]);
    release.write_all(b"go\n").unwrap();
    watch.event("landing.recorded", &[]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
}

fn notes_machine(test: &str) -> Machine {
    Machine::new(test, |request| {
        act(
            request,
            vec![commit_file("docs/notes.md", "notes\n", "Add notes")],
        )
    })
}

/// An automatic approval whose landing is held in its host gate, after a
/// sync that changes policy: the project, its watch, the gate's release and
/// the approved head.
fn held_after_policy_change(
    machine: &Machine,
    change: impl Fn(String) -> String,
) -> (Project<'_>, Watch, std::fs::File, String) {
    let hold = machine.root.join("hold");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&hold)
            .status()
            .unwrap()
            .success()
    );
    // Held open for writing, so each line released stays until a gate reads it.
    let release = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&hold)
        .unwrap();
    let gates = format!(
        "[gates.check]\ncommand = \"test -f docs/notes.md\"\nstage = \"candidate\"\n\n\
         [gates.held]\ncommand = \"read line < {}\"\nruns_in = \"host\"\n",
        hold.display()
    );
    machine.start();
    let base = auto(unreviewed(&gates));
    let project = Project::new(machine, "p", &base);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add notes"]);
    let approved = watch.event("approval.given", &[("actor", "auto")]);
    let head = approved["data"]["head"].as_str().unwrap().to_string();
    watch.event("execution.started", &[("name", "held")]);

    project.reconfigure(&change(base));
    (project, watch, release, head)
}

/// An automatic approval, then a sync that protects its path: the landing
/// withdraws it, raises `approval` and reruns no check.
///
/// Sabotage: make `queue::land` re-read the approval at its intent against
/// the configuration it started with; the landing records an intent and
/// retires on the moved target instead of withdrawing.
#[test]
fn a_sync_that_protects_an_auto_approved_path_withdraws_it() {
    policy_change_withdraws("g2-protect", |config| {
        config.replace("\".pi/\"]", "\".pi/\", \"docs/\"]")
    });
}

/// An automatic approval, then a sync that sets `approve = "manual"`: the
/// landing withdraws it, raises `approval` and reruns no check.
///
/// Sabotage: make `queue::holds` skip the `approve` check for an automatic
/// approval; the retired landing re-queues and lands.
#[test]
fn a_sync_that_sets_manual_withdraws_an_auto_approval() {
    policy_change_withdraws("g2-manual", |config| {
        config.replace("approve = \"auto\"", "approve = \"manual\"")
    });
}

/// An automatic approval, a sync that sets `approve = "manual"`, and the
/// attempt abandoned while the landing reads the candidate's protected paths
/// at its intent: the landing withdraws, and no `approval` is raised for the
/// ended attempt. Control: `a_sync_that_sets_manual_withdraws_an_auto_approval`.
///
/// Sabotage: make `queue::lapsed` raise `approval` without re-reading the
/// attempt; an approval item stays open for an attempt that ended.
#[test]
fn an_attempt_abandoned_while_its_landing_rereads_policy_raises_nothing() {
    let machine = notes_machine("g2-abandon");
    let (project, mut watch, mut release, _) = held_after_policy_change(&machine, |config| {
        config.replace("approve = \"auto\"", "approve = \"manual\"")
    });
    let fifo = |name: &str| {
        let path = machine.root.join(name);
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        path
    };
    let (said, resume) = (fifo("said"), fifo("resume"));
    let armed = machine.root.join("armed");
    std::fs::write(&armed, "").unwrap();
    // Armed after the sync, so the next protected-path read is the intent's.
    machine.wrapper(
        "git",
        &format!(
            "case \" $* \" in *' diff --name-only '*) if rm '{}' 2>/dev/null; then echo held > '{}'; read line < '{}'; fi;; esac",
            armed.display(),
            said.display(),
            resume.display()
        ),
    );
    release.write_all(b"go\n").unwrap();
    assert_eq!(watch.said(&said), "held\n");
    project.json(&["attempt", "abandon", "Y-1"]);
    std::fs::write(&resume, "go\n").unwrap();
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn", "{ended}");
    assert_eq!(
        project.rows("SELECT kind FROM attention WHERE state = 'open'"),
        Vec::<Value>::new()
    );
}

/// An approve naming an old candidate gets a stale result and changes no
/// row, though an approval item is open on the newer candidate. Control:
/// the approve naming the newer candidate is given.
///
/// Sabotage: make `admit::approval_item` accept any head while an item is
/// open; the approve of the old head is given.
#[test]
fn an_approve_naming_an_old_candidate_is_stale() {
    let machine = Machine::new(
        "g2-old-head",
        worker("Once more", commit_file("more.txt", "more\n", "More")),
    );
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed(""));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = head(&approval(&mut watch));
    project.json(&[
        "attempt",
        "reject",
        "Y-1",
        "--head",
        &first,
        "--text",
        "Once more",
    ]);
    let second = head(&approval(&mut watch));

    let seq = last_seq(&project);
    let refused = project.refused(&["attempt", "approve", "Y-1", "--head", &first]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], first.as_str());
    assert_eq!(refused["data"]["current"], second.as_str());
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    project.json(&["attempt", "approve", "Y-1", "--head", &second]);
    assert_eq!(
        project.rows("SELECT head FROM approval"),
        vec![json!({ "head": second })]
    );
}

/// An edit naming an old revision gets a stale result and changes no row.
/// Control: the edit naming the current revision is applied.
///
/// Sabotage: drop the revision comparison in `admit::edit`; the second edit
/// overwrites the first.
#[test]
fn an_edit_naming_an_old_revision_is_stale() {
    let machine = Machine::new("g2-old-revision", |_| Reply::Text("unused".into()));
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    project.json(&["ticket", "new", "--title", "Plan", "--parked"]);
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"]
        .as_i64()
        .unwrap()
        .to_string();
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        "First",
    ]);

    let seq = last_seq(&project);
    let refused = project.refused(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        "Second",
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"].to_string(), revision);
    assert_eq!(
        refused["data"]["current"],
        revision.parse::<i64>().unwrap() + 1
    );
    assert_eq!(last_seq(&project), seq);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["body"], "First");
}
