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
/// template changes and a proof-only repair. Only candidate changes rejudge.
///
/// The new commit keeps the tree, so only the head tells the two candidates
/// apart. The edit's implementer run makes no commit: the same head is
/// judged afresh at the new revision, without `stopped:unchanged`, so the
/// revision binding shows in the revision each judging execution records.
/// Gate, workflow and seat sync keeps the admitted judgments. The proof-only repair
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
fn a_new_head_revision_or_proof_rejudges_but_template_sync_keeps_checks() {
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
    let admitted = config(GATE).replace(
        "[workflows.default]\n",
        "[workflows.default]\ninstructions = \"Admitted workflow instructions\"\nmax_session_executions = 1\n",
    );
    let project = Project::new(&machine, "p", &admitted);
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
        "--target",
        "main",
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

    // G2: late template lookup or digest invalidation would add judgments.
    // The subsequent proof repair and its attention form the observation barrier.
    project.reconfigure(&config(&GATE.replace("test -f", "test -s")));
    project.reconfigure(
        &config(&GATE.replace("test -f", "test -s"))
            .replace(
                "Review for correctness.",
                "Review for correctness and naming.",
            )
            .replace(
                "[workflows.default]\n",
                "[workflows.default]\ninstructions = \"New workflow instructions\"\nmax_session_executions = 1\n",
            ),
    );
    let fifth = third.clone();

    // A repair that changes only `/yard/proof` is a new candidate.
    // Sabotage: match only base and head in `checks::current`; the old gate
    // counts for the new proof and no gate reruns.
    let old_proof = proof(&fifth);
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
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
    let repair_prompt = machine
        .model
        .requests()
        .iter()
        .rev()
        .find(|request| request.opens() && !request.has_tool("yard_publish_review"))
        .unwrap()
        .prompt();
    assert!(
        repair_prompt.contains("Admitted workflow instructions"),
        "{repair_prompt}"
    );
    assert!(
        !repair_prompt.contains("New workflow instructions"),
        "{repair_prompt}"
    );
    assert_eq!(
        kinds(&project),
        [
            "gate", "review", "gate", "review", "gate", "review", "gate", "review"
        ]
    );
    assert_eq!(
        project
            .rows("SELECT DISTINCT plan_revision FROM execution WHERE kind IN ('gate', 'review')"),
        vec![json!({"plan_revision": 1})]
    );
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
        "attempt", "approve", "--target", "main", "Y-1", "--head", &candidate, "--proof",
        &old_proof,
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], old_proof.as_str());
    assert_eq!(refused["data"]["current"], sixth["payload"]["proof"]);
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    let refused = project.refused(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &first,
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], first.as_str());
    assert_eq!(refused["data"]["current"], candidate.as_str());
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    let new_proof = proof(&sixth);
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &candidate, "--proof",
        &new_proof,
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
    assert_eq!(raised["payload"]["plan_revision"], 1);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|request| request.opens())
            .count(),
        1
    );
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

    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
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

/// G2: manual policy synced while the auto-approved candidate's landing is
/// held withdraws approval on the same plan, with no new model turn or check.
/// Sabotage: read approve from the admitted plan; the held landing continues.
/// The protected-path scenario exercises the other policy input.
#[test]
fn manual_policy_withdraws_auto_approval_without_rejudgment() {
    let machine = notes_machine("g2-manual");
    let (project, mut watch, mut release, head) = held_after_policy_change(
        &machine,
        "[gates.check]\ncommand = \"test -f docs/notes.md\"\nstage = \"candidate\"\n\n",
        |config| config.replace("approve = \"auto\"", "approve = \"manual\""),
    );
    release.write_all(b"go\n").unwrap();
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn");
    let raised = approval(&mut watch);
    assert_eq!(raised["payload"]["head"], head);
    assert_eq!(raised["payload"]["plan_revision"], 1);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|request| request.opens())
            .count(),
        1
    );
    assert_eq!(
        project.rows("SELECT actor, state FROM approval"),
        vec![json!({"actor": "auto", "state": "withdrawn"})]
    );
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'gate' AND reason = 'candidate'")
            .len(),
        1
    );
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    release.write_all(b"go\n").unwrap();
    watch.event("landing.recorded", &[]);
}

/// G2: a delayed acceptance cannot cross an edit, even before the held edit
/// worker changes the head. After release it cannot cross the new head either.
/// Refusals name the changed identity and write no rows; the current item's
/// repair exit and its completed approval supply the same-project control.
/// Expected revisions are the two CLI edits; heads come from host git.
/// Sabotage: omit ticket revision or head from candidate decisions, or let a
/// repaired decision carry to the next candidate's check.
#[test]
fn delayed_candidate_acceptance_names_the_changed_identity_and_writes_nothing() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-accept-stale", move |request| {
        if request
            .prompt()
            .contains("Your workspace is read-only. Plan the work")
        {
            return planned(request);
        }
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains("The operator edited") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![bash(
                    "cd /workspace && git commit --allow-empty -qm Edited",
                )])),
            );
        }
        let command = if request.last_user().contains("The gate check failed") {
            "cd /workspace && touch ready.txt && git add -A && git commit -qm Ready"
        } else {
            "cd /workspace && touch feature.txt && git add -A && git commit -qm Feature"
        };
        act(request, vec![bash(command)])
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f ready.txt\"\nstage = \"candidate\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Delayed acceptance",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == 1
    });
    watch.find("cleanup reservation released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let mut operator = machine.operator();
    operator.initialize();
    let admitted = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workflow": "default",
            "overrides": {"candidate-gate-check": {"operator_failure": true}}
        }),
    );
    assert!(admitted.get("error").is_none(), "{admitted}");
    let first = watch.attention();
    assert_eq!(first["data"]["kind"], "candidate");
    let first_item = project.json(&["attempt", "show", "Y-1"])["attention"][0].clone();
    let first_head = git(
        &project.path.join(".yard/local/attempts/2/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    let delayed = json!({
        "project": project.path, "ticket": "Y-1", "attention": first_item["attention"], "check": first_item["payload"]["evidence"]["check"]["check"], "plan_revision": 1,
        "target": "main", "head": first_head, "proof": first_item["payload"]["proof"], "text": "Accept the old failure"
    });
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        "1",
        "--body",
        "New brief",
    ]);
    hold.wait_held();
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_accept", delayed.clone());
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "revision", "expected": 1, "current": 2})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    hold.release();
    let second = watch.attention();
    assert_eq!(second["data"]["kind"], "candidate");
    let second_head = git(
        &project.path.join(".yard/local/attempts/2/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    assert_ne!(first_head, second_head);
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_accept", delayed);
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "head", "expected": first_head, "current": second_head})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    let item = project.json(&["attempt", "show", "Y-1"])["attention"][0].clone();
    let repaired = operator.call("yard_attempt_repair", json!({
        "project": project.path, "ticket": "Y-1", "attention": item["attention"], "check": item["payload"]["evidence"]["check"]["check"], "plan_revision": 1,
        "target": "main", "head": second_head, "proof": item["payload"]["proof"], "text": "Use the admitted repair route"
    }));
    assert_eq!(repaired["result"]["accepted"], false, "{repaired}");
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    let payload = &approval["data"]["payload"];
    let head = payload["head"].as_str().unwrap();
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:ready.txt")]
        ),
        ""
    );
    project.json(&[
        "attempt", "approve", "Y-1", "--target", "main", "--head", head,
    ]);
    watch.event("landing.recorded", &[]);
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", head, "refs/heads/main"],
    );
}

/// G2/G8: explicit skipped acceptance follows a full default landing in the
/// same project. A reduced plan retains one candidate gate and no seats.
/// An edit holds the new worker before changing head: delayed decisions name
/// the old revision, then old head, and write nothing. Wrong plan and retained
/// node identities also refuse; current identities record the operator's text.
/// Expected nodes and revisions are fixture literals; base/head come from git.
/// The pre-decision execution/check snapshot must survive acceptance, landing
/// and a quiescent reopen. Existing omission coverage has no explicit decision.
/// Sabotage: manufacture a passing check/result for an accepted omission, drop
/// skipped-node or plan binding, drop revision/head binding, or mark the empty
/// panel reviewed on acceptance. Remove approval evidence persistence to lose
/// the decision at landing or reopen. Accept blank text to lose the required
/// operator explanation.
#[test]
fn selected_skipped_decisions_bind_identity_without_manufacturing_results() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-skipped", move |request| {
        if request
            .prompt()
            .contains("Your workspace is read-only. Plan the work")
        {
            return planned(request);
        }
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains("The operator edited") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![bash(
                    "cd /workspace && git commit --allow-empty -qm Edited",
                )])),
            );
        }
        let path = if request.context().contains("Reduced") {
            "reduced.txt"
        } else {
            "full.txt"
        };
        act(request, vec![commit_file(path, "feature\n", "Feature")])
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(
            "[gates.check]\ncommand = \"git rev-parse HEAD\"\nstage = \"candidate\"\n\
         [gates.keep]\ncommand = \"git rev-parse HEAD\"\nstage = \"candidate\"\n\
         [gates.other]\ncommand = \"git rev-parse HEAD\"\nstage = \"candidate\"\n",
        ),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Full"]);
    let full = approval(&mut watch);
    project.json(&[
        "attempt",
        "approve",
        "Y-1",
        "--target",
        "main",
        "--head",
        full["payload"]["head"].as_str().unwrap(),
    ]);
    watch.event("landing.recorded", &[]);
    // Ordinary approval must not silently become explicit skipped acceptance.
    assert_eq!(
        project.rows("SELECT accepted_skipped FROM approval"),
        vec![json!({"accepted_skipped": null})]
    );
    let base = project.canonical_head();
    project.json(&["ticket", "new", "--title", "Reduced", "--workflow", "plan"]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let planner = project.json(&["attempt", "show", "Y-2"])["attempt"]["attempt"].clone();
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let graph = json!({"inputs": {"feedback": "verdict"}, "nodes": {
        "implementer": {
            "runner": {"kind": "agent", "agent": "worker", "instructions": ""},
            "access": "write-allowed", "inputs": {"feedback": {"output": "feedback", "kind": "verdict"}},
            "outputs": {"result": "candidate"}, "outcomes": ["complete"], "requires": [],
            "routes": {"complete": [{"to": "candidate-gate-keep"}]}
        },
        "candidate-gate-keep": {
            "runner": {"kind": "command", "gate": "keep"}, "access": "write-allowed",
            "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"], "requires": ["candidate"], "routes": {}
        }
    }});
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-2", "graph": graph}),
    );
    assert!(started.get("error").is_none(), "{started}");
    let first = approval(&mut watch);
    let shown = project.json(&["attempt", "show", "Y-2"]);
    let first_attention = shown["attention"][0]["attention"].clone();
    let attempt = shown["attempt"]["attempt"].as_i64().unwrap();
    let clone = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let first_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    let delayed = json!({"project": project.path, "ticket": "Y-2", "target": "main", "base": base,
        "head": first_head, "proof": first["payload"]["proof"], "revision": 1, "plan_revision": 1,
        "attention": first_attention,
        "accept_skipped": ["candidate-gate-check", "review-0"], "text": "Accept the selected candidate omissions"});
    project.json(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "1",
        "--body",
        "Reduced, edited",
    ]);
    hold.wait_held();
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_approve", delayed.clone());
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "revision", "expected": 1, "current": 2})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    hold.release();
    let current = approval(&mut watch);
    let current_attention =
        project.json(&["attempt", "show", "Y-2"])["attention"][0]["attention"].clone();
    let head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    assert_ne!(head, first_head);
    let mut decision = delayed;
    decision["head"] = json!(head);
    decision["revision"] = json!(2);
    decision["attention"] = current_attention.clone();
    for (key, old, identity, expected) in [
        ("head", json!(first_head), "head", json!(head)),
        (
            "attention",
            first_attention.clone(),
            "attention",
            current_attention.clone(),
        ),
        ("plan_revision", json!(0), "plan_revision", json!(1)),
        (
            "accept_skipped",
            json!(["candidate-gate-keep"]),
            "skipped",
            json!(["candidate-gate-check", "candidate-gate-other", "review-0"]),
        ),
    ] {
        let mut stale = decision.clone();
        stale[key] = old.clone();
        let before = project.rows("SELECT * FROM audit ORDER BY seq");
        let approvals = project.rows("SELECT * FROM approval ORDER BY id");
        let refused = operator.call("yard_attempt_approve", stale);
        assert_eq!(refused["error"]["code"], "stale");
        assert_eq!(
            refused["error"]["data"],
            json!({"identity": identity, "expected": old, "current": expected})
        );
        assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
        assert_eq!(
            project.rows("SELECT * FROM approval ORDER BY id"),
            approvals
        );
    }
    // This snapshot proves the acceptance adds no execution, result or check.
    // The retained gate already finished; omissions have no rows to accept.
    let judgments = format!(
        "SELECT e.node, e.head, e.result, c.verdict FROM execution e LEFT JOIN \"check\" c ON c.execution = e.id WHERE e.attempt = {attempt} AND e.kind IN ('gate', 'review') ORDER BY e.id"
    );
    let original = project.rows(&judgments);
    assert_eq!(original.len(), 2); // first candidate, then the edited candidate
    assert!(
        original
            .iter()
            .all(|row| row["node"] == "candidate-gate-keep" && row["verdict"] == "pass")
    );
    assert_eq!(current["payload"]["unreviewed"], true);
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let mut blank = decision.clone();
    blank["text"] = json!("   ");
    let refused = operator.call("yard_attempt_approve", blank);
    assert_eq!(refused["error"]["code"], "invalid");
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    let accepted = operator.call("yard_attempt_approve", decision.clone());
    assert!(accepted.get("error").is_none(), "{accepted}");
    let expected = json!({"identity": {"attempt": attempt, "target": "main", "base": base, "head": head, "proof": current["payload"]["proof"], "revision": 2, "plan_revision": 1,
        "skipped": ["candidate-gate-check", "candidate-gate-other", "review-0"]}, "attention": current_attention, "nodes": ["candidate-gate-check", "review-0"], "actor": "operator", "text": decision["text"]});
    let evidence =
        format!("SELECT accepted_skipped, actor, overrode FROM approval WHERE attempt = {attempt}");
    let rows =
        vec![json!({"accepted_skipped": expected.to_string(), "actor": "operator", "overrode": 1})];
    assert_eq!(project.rows(&evidence), rows);
    watch.event("landing.recorded", &[]);
    let cleanup = watch.find("accepted candidate cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == attempt
    });
    watch.find("cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", &head, "refs/heads/main"],
    );
    assert_eq!(project.rows(&evidence), rows);
    assert_eq!(project.rows(&judgments), original);
    let admission = format!(
        "SELECT json_extract(admission, '$.unreviewed') AS unreviewed FROM attempt WHERE id = {attempt}"
    );
    assert_eq!(project.rows(&admission), vec![json!({"unreviewed": 1})]);
    machine.kill();
    machine.start();
    project.json(&["status"]);
    assert_eq!(project.rows(&evidence), rows);
    assert_eq!(project.rows(&judgments), original);
    assert_eq!(project.rows(&admission), vec![json!({"unreviewed": 1})]);
}
