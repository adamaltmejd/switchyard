//! G2: Judgments bind exact identity.

use e2e::*;
use serde_json::{Value, json};
use std::io::Write;

const GATE: &str = "[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n";

/// Seats publish no findings; the implementer commits feature.txt and writes
/// its proof, or runs a repair once when its execution's prompt carries that
/// repair's key. Keys are tried in order, so the newest prompt's key comes
/// first: a fresh session's brief repeats the ticket body.
fn worker(repairs: Vec<(&'static str, ToolCall)>) -> impl Fn(&ModelRequest) -> Reply + Send + Sync {
    let repairs = std::sync::Mutex::new(
        repairs
            .into_iter()
            .map(|(key, call)| (key, Some(call)))
            .collect::<Vec<_>>(),
    );
    move |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() {
            let prompt = request.last_user();
            let mut repairs = repairs.lock().unwrap();
            if let Some((_, call)) = repairs.iter_mut().find(|(key, _)| prompt.contains(key)) {
                return Reply::Tools(vec![call.take().expect("one repair per key")]);
            }
        }
        act(
            request,
            vec![bash(
                "cd /workspace && printf 'feature\\n' > feature.txt && git add -A \
                 && git commit -q -m 'Add feature' && printf 'first' > /yard/proof/evidence.txt \
                 && echo committed",
            )],
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

fn proof(item: &Value) -> String {
    item["payload"]["proof"].as_str().unwrap().to_string()
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

/// A passed gate and review, then a new commit, a ticket edit, two synced
/// check changes and a proof-only repair in turn, each leaving the candidate
/// unverified.
///
/// The new commit keeps the tree, so only the head tells the two candidates
/// apart. The edit's implementer run makes no commit: the same head is
/// judged afresh at the new revision, without `stopped:unchanged`, so the
/// revision binding shows in the revision each judging execution records.
/// A synced gate change reruns the gate and keeps the review; then a seat
/// change reruns the review and keeps the new gate. The proof-only repair
/// reruns the gate on the same head under the new proof digest, and the old
/// check does not count. Delayed approves naming the old proof or the old
/// head write nothing; approving the current head and proof succeeds.
///
/// Sabotage: drop `head` from `checks::current`'s match; the empty commit
/// reaches approval on the first commit's gate and review. Make
/// `admit::steer` set nothing; no implementer runs for the edit and no
/// approval follows it. Keep `stop("unchanged")` for edit runs; the attempt
/// stops. Record the old revision on the gate and review executions; `judged`
/// no longer reads `revision + 1`. Make admit::approval_item accept an old
/// head; the stale approve writes.
#[test]
fn a_new_head_revision_check_config_or_proof_leaves_the_candidate_unverified() {
    let machine = Machine::new(
        "g2-identity",
        worker(vec![
            (
                "Change the proof",
                bash("printf 'second' > /yard/proof/evidence.txt"),
            ),
            ("nothing else", bash("cd /workspace && echo unchanged")),
            (
                "Once more",
                bash("cd /workspace && git commit -q --allow-empty -m Again && echo committed"),
            ),
        ]),
    );
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
    let first_item = approval(&mut watch);
    let first = head(&first_item);
    project.json(&[
        "attempt",
        "reject",
        "Y-1",
        "--head",
        &first,
        "--proof",
        &proof(&first_item),
        "--text",
        "Once more",
    ]);
    let second = approval(&mut watch);
    let candidate = head(&second);

    let canonical = project.canonical();
    let tree = |commit: &str| git(&canonical, &["rev-parse", &format!("{commit}^{{tree}}")]);
    assert_ne!(candidate, first);
    assert_eq!(tree(&candidate), tree(&first));
    let once = vec![("gate".to_string(), 1), ("review".to_string(), 1)];
    assert_eq!(judged(&project, &first), once);
    assert_eq!(judged(&project, &candidate), once);

    // A ticket edit supersedes the approval and the implementer runs.
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"]
        .as_i64()
        .unwrap();
    assert_eq!(second["payload"]["revision"], revision);
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision.to_string(),
        "--body",
        "Create feature.txt and nothing else",
    ]);
    let third = approval(&mut watch);

    assert_eq!(head(&third), candidate);
    assert_eq!(third["payload"]["revision"], revision + 1);
    let superseded =
        project.rows("SELECT resolution FROM attention WHERE kind = 'approval' ORDER BY id");
    assert_eq!(superseded[1]["resolution"], "superseded");
    let reasons =
        project.rows("SELECT reason FROM execution WHERE kind = 'implementation' ORDER BY id");
    assert_eq!(reasons.last().unwrap()["reason"], "edit");
    assert_eq!(
        judged(&project, &candidate),
        vec![
            ("gate".to_string(), revision),
            ("review".to_string(), revision),
            ("gate".to_string(), revision + 1),
            ("review".to_string(), revision + 1),
        ]
    );

    // A delayed edit on the old revision cannot overwrite the accepted edit.
    // Sabotage: drop admit::edit's revision comparison; this edit writes.
    let seq = last_seq(&project);
    let refused = project.refused(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision.to_string(),
        "--body",
        "Overwrite the accepted edit",
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], revision);
    assert_eq!(refused["data"]["current"], revision + 1);
    assert_eq!(last_seq(&project), seq);
    assert_eq!(
        project.json(&["ticket", "show", "Y-1"])["body"],
        "Create feature.txt and nothing else"
    );

    // A synced gate change reruns the gate and keeps the review.
    // Sabotage: hash the gates into `review_digest` too; the review reruns.
    let review = third["payload"]["checks"][1].clone();
    project.reconfigure(&config(&GATE.replace("test -f", "test -s")));
    let fourth = approval(&mut watch);
    assert_eq!(head(&fourth), candidate);
    assert_eq!(
        kinds(&project),
        ["gate", "review", "gate", "review", "gate", "review", "gate"]
    );
    let gate =
        project.rows("SELECT id FROM \"check\" WHERE kind = 'gate' ORDER BY id")[3]["id"].clone();
    assert_eq!(fourth["payload"]["checks"], json!([gate, review]));

    // The reverse change retains this new gate and replaces only the review.
    // Sabotage: hash the seats into gate_digest too; the gate reruns.
    project.reconfigure(&config(&GATE.replace("test -f", "test -s")).replace(
        "Review for correctness.",
        "Review for correctness and naming.",
    ));
    let fifth = approval(&mut watch);
    assert_eq!(head(&fifth), candidate);
    assert_eq!(
        kinds(&project),
        [
            "gate", "review", "gate", "review", "gate", "review", "gate", "review"
        ]
    );
    let review =
        project.rows("SELECT id FROM \"check\" WHERE kind = 'review' ORDER BY id")[3]["id"].clone();
    assert_eq!(fifth["payload"]["checks"], json!([gate, review]));

    // A repair that changes only `/yard/proof` is a new candidate.
    // Sabotage: match only base and head in `checks::current`; the old gate
    // counts for the new proof and no gate reruns.
    let old_proof = proof(&fifth);
    project.json(&[
        "attempt",
        "reject",
        "Y-1",
        "--head",
        &candidate,
        "--proof",
        &old_proof,
        "--text",
        "Change the proof",
    ]);
    let sixth = approval(&mut watch);
    assert_eq!(head(&sixth), candidate);
    let gates = project.rows(
        "SELECT c.id AS id, e.head AS head, e.proof AS proof FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.kind = 'gate' ORDER BY c.id DESC LIMIT 2",
    );
    assert_eq!(gates[0]["head"], candidate.as_str());
    assert_eq!(gates[1]["head"], candidate.as_str());
    assert_ne!(gates[0]["proof"], gates[1]["proof"]);
    assert_eq!(sixth["payload"]["checks"][0], gates[0]["id"]);
    assert_eq!(sixth["payload"]["proof"], gates[0]["proof"]);

    // A delayed approve naming the old proof writes nothing.
    // Sabotage: drop the proof comparison in `admit::approval_item`; the
    // delayed approve names the old proof and is given anyway.
    let seq = last_seq(&project);
    let refused = project.refused(&[
        "attempt", "approve", "Y-1", "--head", &candidate, "--proof", &old_proof,
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], old_proof.as_str());
    assert_eq!(refused["data"]["current"], sixth["payload"]["proof"]);
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    let refused = project.refused(&["attempt", "approve", "Y-1", "--head", &first]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], first.as_str());
    assert_eq!(refused["data"]["current"], candidate.as_str());
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    let new_proof = proof(&sixth);
    project.json(&[
        "attempt", "approve", "Y-1", "--head", &candidate, "--proof", &new_proof,
    ]);
    assert_eq!(
        project.rows("SELECT head, proof FROM approval"),
        vec![json!({ "head": candidate, "proof": new_proof })]
    );
}

/// A seat held mid-review, then a ticket edit: the seat stops and its box
/// is gone before the implementer runs for the edit, and the round count
/// is unchanged. Control: the seat's box is listed while the fixture holds
/// it.
///
/// Sabotage: make `admit::steer` return no executions to stop; the seat
/// stays held, its box stays listed and no implementer starts.
#[test]
fn an_edit_stops_a_held_seat_and_costs_no_round() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-held", move |request| {
        if request.has_tool("yard_publish_review") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![publish(json!([]))])),
            );
        }
        if request.opens() && request.last_user().contains("The operator edited") {
            return Reply::Tools(vec![bash(
                "cd /workspace && git commit -q --allow-empty -m Again && echo committed",
            )]);
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create it",
    ]);
    hold.wait_held();
    let handle =
        project.rows("SELECT handle FROM execution WHERE kind = 'review'")[0]["handle"].clone();
    let listed = |machine: &Machine| {
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|listed| listed["name"] == handle)
    };
    assert!(
        listed(&machine),
        "the held seat's box {handle} is not listed"
    );

    let revision = project.json(&["ticket", "show", "Y-1"])["revision"].to_string();
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        "Create it, twice",
    ]);
    watch.until("the edit's implementer", |event| {
        event["event"] == "execution.started"
            && event["ticket"] == "Y-1"
            && event["data"]["reason"] == "edit"
    });
    assert!(!listed(&machine), "the seat's box {handle} is still up");
    assert_eq!(
        project.rows("SELECT outcome FROM execution WHERE kind = 'review' ORDER BY id")[0]["outcome"],
        "stopped"
    );
    assert_eq!(
        project.rows("SELECT rounds FROM attempt")[0]["rounds"],
        json!(0)
    );
    hold.release();
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
}

/// An automatic approval, a landing held in its host gate, and a sync that
/// protects its path: the landing withdraws the approval when it records its
/// intent, raises `approval`, and reruns no check. Control: the operator's
/// approval of the same head lands.
///
/// Sabotage: make `queue::land` re-read the approval at its intent against
/// the configuration it started with; the landing records an intent and
/// retires on the moved target instead of withdrawing. Hash
/// `protected_paths` into the gate digest; the sync reruns the candidate
/// gate.
#[test]
fn a_sync_that_protects_an_auto_approved_path_withdraws_it() {
    let machine = notes_machine("g2-protect");
    let (project, mut watch, mut release, head) = held_after_policy_change(
        &machine,
        "[gates.check]\ncommand = \"test -f docs/notes.md\"\nstage = \"candidate\"\n\n",
        |config| config.replace("\".pi/\"]", "\".pi/\", \"docs/\"]"),
    );
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
/// the approved head. `gates` goes before the held landing gate.
fn held_after_policy_change<'m>(
    machine: &'m Machine,
    gates: &str,
    change: impl Fn(String) -> String,
) -> (Project<'m>, Watch, std::fs::File, String) {
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
        "{gates}[gates.held]\ncommand = \"read line < {}\"\nruns_in = \"host\"\n",
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

/// An automatic approval, a sync that sets `approve = "manual"`, and the
/// attempt abandoned while the landing reads the candidate's protected paths
/// at its intent: the landing withdraws, and no `approval` is raised for the
/// ended attempt. Control: `a_sync_that_protects_an_auto_approved_path_withdraws_it`,
/// whose attempt lives, raises `approval`.
///
/// Sabotage: make `queue::lapsed` raise `approval` without re-reading the
/// attempt; an approval item stays open for an attempt that ended.
#[test]
fn an_attempt_abandoned_while_its_landing_rereads_policy_raises_nothing() {
    let machine = notes_machine("g2-abandon");
    let (project, mut watch, mut release, _) = held_after_policy_change(&machine, "", |config| {
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
