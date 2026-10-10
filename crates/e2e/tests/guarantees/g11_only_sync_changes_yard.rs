//! G11: Only the operator's sync changes `.yard` and canonical from outside.

use e2e::*;
use serde_json::json;

fn unreviewed() -> String {
    config("").replace("review = [\"correctness\"]", "review = \"none\"")
}

const BASE_GATE: &str =
    "\n[gates.base]\ncommand = \"true\"\nstage = \"candidate\"\nruns_in = \"host\"\n";
const EXTRA_GATE: &str =
    "\n[gates.extra]\ncommand = \"true\"\nstage = \"candidate\"\nruns_in = \"host\"\n";

/// A worker commit under `.yard` comes back with the reason and no gate
/// runs on it: the project's candidate gate runs only on the repaired head.
/// The same change through `yard sync` is in force for the next execution.
/// The change adds a second candidate gate: refused from the worker, it runs
/// on the candidate once the operator syncs it.
///
/// Then a checkout and canonical that each hold a commit the other lacks:
/// both directions refuse naming both heads; a fast-forward passes.
/// Canonical moves by the candidate's landing, the checkout by an operator
/// commit.
///
/// Sabotage: make `supervise::implement` skip its `.yard` check; the
/// worker's commit becomes the candidate and the gate runs on it. Make
/// `admit::sync` import whenever the heads differ; the operator's commit
/// replaces the landing in canonical.
#[test]
fn only_the_operators_sync_changes_yard_and_canonical() {
    let gated = format!("{}{BASE_GATE}", unreviewed());
    let configured = format!("{gated}{EXTRA_GATE}");
    let change = configured.replace('\'', "'\\''");
    let machine = Machine::new("g11-yard", move |request| {
        if request.opens() && request.last_user().contains(".yard") && request.turn() > 0 {
            return Reply::Tools(vec![bash(
                "cd /workspace && git reset -q --hard HEAD~1 && printf 'feature\\n' > feature.txt \
                 && git add -A && git commit -q -m 'Add feature' && echo committed",
            )]);
        }
        act(
            request,
            vec![bash(&format!(
                "cd /workspace && printf '%s' '{change}' > .yard/config.toml && printf 'feature\\n' > feature.txt \
                 && git add -A && git commit -q -m 'Add feature and a gate' && git rev-parse HEAD"
            ))],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &gated);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let first = watch.attention();
    assert_eq!(first["data"]["kind"], "approval", "{first}");
    let head = first["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        project.rows(
            "SELECT reason, outcome FROM execution WHERE kind = 'implementation' ORDER BY id"
        ),
        vec![
            json!({ "reason": "first", "outcome": "refused" }),
            json!({ "reason": "repair", "outcome": "candidate" }),
        ]
    );
    // The refused head, as git in the box printed it.
    let refused = machine
        .model
        .requests()
        .iter()
        .find_map(|request| request.tool_results().first().cloned())
        .expect("the first execution's commit")
        .1
        .trim()
        .to_string();
    assert_ne!(refused, head);
    assert_eq!(
        project.rows("SELECT name, head, outcome FROM execution WHERE kind = 'gate'"),
        vec![json!({ "name": "base", "head": head, "outcome": "pass" })]
    );

    project.reconfigure(&configured);
    let second = watch.attention();
    assert_eq!(second["data"]["kind"], "approval", "{second}");
    assert_eq!(second["data"]["payload"]["head"], head.as_str());
    // The synced gate is in force for the candidate.
    assert_eq!(
        project.rows("SELECT head, outcome FROM execution WHERE kind = 'gate' AND name = 'extra'"),
        vec![json!({ "head": head, "outcome": "pass" })]
    );

    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    watch.event("landing.recorded", &[]);
    let landed = project.canonical_head();

    project.write("notes.md", "notes\n");
    project.git(&["add", "notes.md"]);
    project.git(&["commit", "--quiet", "-m", "Operator notes"]);
    let local = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let refused = project.refused(&["sync"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(
        refused["data"],
        json!({ "checkout": local, "canonical": landed })
    );
    assert_eq!(project.canonical_head(), landed);
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), local);

    project.git(&["reset", "--quiet", "--hard", "HEAD~1"]);
    assert_eq!(project.json(&["sync"])["sync"], "consumed");
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), landed);
    project.git(&["cherry-pick", &local]);
    assert_eq!(project.json(&["sync"])["sync"], "imported");
    assert_eq!(
        project.canonical_head(),
        project.git(&["rev-parse", "HEAD"]).trim()
    );
}

fn branch_head(project: &Project, branch: &str) -> String {
    git(
        &project.canonical(),
        &["rev-parse", &format!("refs/heads/{branch}")],
    )
    .trim()
    .to_string()
}

fn default_branch(project: &Project) -> String {
    git(&project.canonical(), &["symbolic-ref", "--short", "HEAD"])
        .trim()
        .to_string()
}

/// G6/G11: selected committed history alone moves. A second imported branch
/// and main stay fixed through import, divergence, and consume. Consume is
/// refused even when git could preserve unrelated staged/untracked changes,
/// and while another branch is checked out. Each refusal has a clean control.
/// Expected heads come from fixture git commits, never sync's output.
/// Sabotage: resolve named sync through HEAD, update main or feature/b,
/// accept divergence, or allow merge to preserve dirt or update a hidden ref.
/// Skip image validation when the canonical branch is absent: a commit with
/// no trusted Dockerfile creates a ref. The restored committed file controls it.
#[test]
fn named_sync_moves_only_committed_selected_history() {
    let machine = Machine::new("g11-named", |_| Reply::Text("unused".into()));
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let main = project.canonical_head();
    let missing = project.refused(&["sync", "feature/a"]);
    assert_eq!(missing["code"], "refused");
    assert_eq!(
        missing["data"],
        json!({ "reason": "missing_branch", "branch": "feature/a" })
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["for-each-ref", "--format=%(refname)", "refs/heads"]
        ),
        "refs/heads/main\n"
    );

    project.git(&["checkout", "--quiet", "-b", "feature/b"]);
    project.write("b.txt", "branch b\n");
    project.git(&["add", "b.txt"]);
    project.git(&["commit", "--quiet", "-m", "B"]);
    let b = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/b"]);
    project.git(&["checkout", "--quiet", "-b", "feature/a", "main"]);
    project.write("a.txt", "branch a\n");
    project.git(&["add", "a.txt"]);
    project.git(&["commit", "--quiet", "-m", "A"]);
    project.git(&["rm", "--quiet", ".yard/Dockerfile"]);
    project.git(&["commit", "--quiet", "-m", "Missing image input"]);
    let invalid_head = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let invalid = project.refused(&["sync", "feature/a"]);
    assert_eq!(invalid["code"], "invalid");
    assert!(invalid["message"].as_str().unwrap().contains(&invalid_head));
    assert!(
        invalid["message"]
            .as_str()
            .unwrap()
            .contains(".yard/Dockerfile")
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["for-each-ref", "--format=%(refname)", "refs/heads"]
        ),
        "refs/heads/feature/b\nrefs/heads/main\n"
    );
    project.git(&["checkout", "HEAD~1", "--", ".yard/Dockerfile"]);
    project.git(&["commit", "--quiet", "-m", "Restore image input"]);
    let a = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.git(&["checkout", "--quiet", "main"]);
    project.json(&["sync", "feature/a"]);
    assert_eq!(branch_head(&project, "feature/a"), a);
    assert_eq!(project.canonical_head(), main);
    assert_eq!(branch_head(&project, "feature/b"), b);
    assert_eq!(default_branch(&project), "main");
    assert_eq!(project.git(&["branch", "--show-current"]).trim(), "main");

    project.git(&["checkout", "--quiet", "feature/a"]);
    project.write("a.txt", "advanced a\n");
    project.git(&["commit", "--quiet", "-am", "Advance A"]);
    let advanced = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/a"]);
    project.git(&["reset", "--quiet", "--hard", &a]);
    project.git(&["checkout", "--quiet", "main"]);
    let wrong = project.refused(&["sync", "feature/a"]);
    assert_eq!(wrong["code"], "refused");
    assert_eq!(
        wrong["data"],
        json!({ "reason": "checkout_branch", "branch": "feature/a", "checked_out": "main", "checkout": a, "canonical": advanced })
    );
    assert_eq!(project.git(&["rev-parse", "feature/a"]).trim(), a);
    assert_eq!(project.git(&["branch", "--show-current"]).trim(), "main");

    project.git(&["checkout", "--quiet", "feature/a"]);
    project.write("README.md", "staged operator change\n");
    project.git(&["add", "README.md"]);
    project.write("untracked.txt", "operator change\n");
    let dirty = project.refused(&["sync", "feature/a"]);
    assert_eq!(dirty["code"], "refused");
    assert_eq!(
        dirty["data"],
        json!({ "reason": "dirty_checkout", "branch": "feature/a", "checkout": a, "canonical": advanced })
    );
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), a);
    assert_eq!(
        project.git(&["show", ":README.md"]),
        "staged operator change\n"
    );
    assert_eq!(
        std::fs::read_to_string(project.path.join("untracked.txt")).unwrap(),
        "operator change\n"
    );
    project.git(&["reset", "--quiet", "--hard", &a]);
    std::fs::remove_file(project.path.join("untracked.txt")).unwrap();
    project.json(&["sync", "feature/a"]);
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), advanced);
    assert_eq!(
        std::fs::read_to_string(project.path.join("a.txt")).unwrap(),
        "advanced a\n"
    );

    project.git(&["reset", "--quiet", "--hard", &a]);
    project.write("a.txt", "divergent a\n");
    project.git(&["commit", "--quiet", "-am", "Diverge A"]);
    let diverged = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let refused = project.refused(&["sync", "feature/a"]);
    assert_eq!(refused["code"], "refused");
    assert_eq!(
        refused["data"],
        json!({ "checkout": diverged, "canonical": advanced })
    );
    assert_eq!(branch_head(&project, "feature/a"), advanced);
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), diverged);
    project.git(&["reset", "--quiet", "--hard", &a]);
    project.json(&["sync", "feature/a"]);
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), advanced);
    assert_eq!(project.canonical_head(), main);
    assert_eq!(branch_head(&project, "feature/b"), b);
    assert_eq!(project.git(&["rev-parse", "main"]).trim(), main);
    assert_eq!(project.git(&["rev-parse", "feature/b"]).trim(), b);
    assert_eq!(default_branch(&project), "main");
}

/// G6/G11: a feature config may declare main, but selecting a new default
/// requires its committed config to declare that branch. A held real worker
/// keeps its admitted main target while selection succeeds.
/// Omitted sync and the next admission use the new symbolic HEAD.
/// Git and the held fixture supply identities.
/// Sabotage: change HEAD on named import, skip default/config validation,
/// retarget the held attempt, or keep resolving omission to main.
/// Revalidate an old imported config during consume, or require a matching
/// clean local branch for default selection: the subsequent operations fail.
#[test]
fn default_selection_is_explicit_validated_and_preserves_admitted_work() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g11-default", move |_| {
        Reply::Hold(held.clone(), Box::new(Reply::Text("planned".into())))
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let main = project.canonical_head();
    project.git(&["checkout", "--quiet", "-b", "feature/b"]);
    project.write("b.txt", "old b\n");
    project.git(&["add", "b.txt"]);
    project.git(&["commit", "--quiet", "-m", "B"]);
    let old_b = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/b"]);
    project.write("b.txt", "new b\n");
    project.git(&["commit", "--quiet", "-am", "Advance B"]);
    let new_b = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/b"]);
    project.git(&["reset", "--quiet", "--hard", &old_b]);
    project.git(&["checkout", "--quiet", "main"]);
    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.write("a.txt", "feature\n");
    project.git(&["add", "a.txt"]);
    project.git(&["commit", "--quiet", "-m", "Feature"]);
    project.json(&["sync", "feature/a"]);
    assert_eq!(default_branch(&project), "main");
    let mismatch = project.refused(&["sync", "feature/a", "--default"]);
    assert_eq!(mismatch["code"], "invalid");
    assert_eq!(
        mismatch["data"],
        json!({ "reason": "default_mismatch", "target": "main", "default": "feature/a" })
    );
    assert_eq!(default_branch(&project), "main");

    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Held plan",
        "--workflow",
        "plan",
    ]);
    hold.wait_held();

    project.write(
        ".yard/config.toml",
        &config("").replace("ref = \"main\"", "ref = \"feature/a\""),
    );
    project.git(&["commit", "--quiet", "-am", "Select default"]);
    let selected = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/a", "--default"]);
    assert_eq!(default_branch(&project), "feature/a");
    assert_eq!(branch_head(&project, "feature/a"), selected);
    assert_eq!(
        project.rows("SELECT target_branch, base FROM attempt WHERE state = 'live'"),
        vec![json!({ "target_branch": "main", "base": main })]
    );

    project.json(&["ticket", "park", "Y-1"]);
    hold.release();
    watch.until("the old attempt was removed", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE state = 'live'")
            .is_empty()
    );
    assert_eq!(
        project.rows("SELECT target_branch, outcome FROM attempt WHERE ticket = 1"),
        vec![json!({ "target_branch": "main", "outcome": "planned" })]
    );
    project.json(&["sync", "feature/a", "--default"]);
    assert_eq!(default_branch(&project), "feature/a");
    assert_eq!(branch_head(&project, "feature/a"), selected);
    assert_eq!(project.canonical_head(), main);
    project.write("a.txt", "new default\n");
    project.git(&["commit", "--quiet", "-am", "Advance default"]);
    let next = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync"]);
    assert_eq!(branch_head(&project, "feature/a"), next);
    assert_eq!(project.canonical_head(), main);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "New default plan",
        "--workflow",
        "plan",
    ]);
    watch.until("new default plan ended", |event| {
        event["event"] == "attempt.ended" && event["ticket"] == "Y-2"
    });
    assert_eq!(
        project.rows("SELECT target_branch, base FROM attempt WHERE ticket = 2"),
        vec![json!({ "target_branch": "feature/a", "base": next })]
    );

    // Already imported history can be consumed even though its config
    // declares the former default. It is not an incoming configuration.
    project.git(&["checkout", "--quiet", "feature/b"]);
    project.json(&["sync", "feature/b"]);
    assert_eq!(project.git(&["rev-parse", "HEAD"]).trim(), new_b);
    assert_eq!(
        std::fs::read_to_string(project.path.join("b.txt")).unwrap(),
        "new b\n"
    );

    // Default selection uses canonical history without requiring a local
    // checkout branch or reconciling any dirty operator files.
    project.json(&["sync", "main", "--default"]);
    project.git(&["branch", "--force", "feature/a", &selected]);
    project.json(&["sync", "feature/a", "--default"]);
    assert_eq!(default_branch(&project), "feature/a");
    assert_eq!(project.git(&["rev-parse", "feature/a"]).trim(), selected);
    assert_eq!(
        project.git(&["branch", "--show-current"]).trim(),
        "feature/b"
    );
    project.json(&["sync", "main", "--default"]);
    project.git(&["branch", "-D", "feature/a"]);
    project.write("operator.txt", "uncommitted\n");
    project.json(&["sync", "feature/a", "--default"]);
    assert_eq!(default_branch(&project), "feature/a");
    assert_eq!(branch_head(&project, "feature/a"), next);
    assert_eq!(
        project.git(&["branch", "--show-current"]).trim(),
        "feature/b"
    );
    assert_eq!(
        project.git(&[
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads/feature/a"
        ]),
        ""
    );
    assert_eq!(
        std::fs::read_to_string(project.path.join("operator.txt")).unwrap(),
        "uncommitted\n"
    );
}

/// G11: default selection refuses an open landing intent by its execution
/// identity while the real git child holds update-ref. It must answer before
/// the fixture releases the child. The completed landing controls selection
/// in the same project. Expected ids come from the store, heads from host git.
/// Sabotage: omit the intent identity from refusal, or wait for canonical
/// before checking live work; the refusal cannot answer while the child holds.
#[test]
fn default_selection_names_a_held_landing_intent_then_succeeds() {
    let machine = Machine::new("g11-intent", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Feature")],
        )
    });
    let armed = machine.root.join("armed");
    let said = machine.root.join("said");
    let release = machine.root.join("release");
    for fifo in [&said, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    machine.wrapper(
        "git",
        &format!(
            "if [ -e '{}' ]; then case \" $* \" in *' update-ref --no-deref refs/heads/main '*)\n\
         echo held > '{}'; read line < '{}';; esac; fi",
            armed.display(),
            said.display(),
            release.display()
        ),
    );
    machine.start();
    let project = Project::new(&machine, "p", &unreviewed());
    let main = project.canonical_head();
    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.write("a.txt", "feature a\n");
    project.git(&["add", "a.txt"]);
    project.git(&["commit", "--quiet", "-m", "A"]);
    let imported = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "feature/a"]);
    project.write(
        ".yard/config.toml",
        &unreviewed().replace("ref = \"main\"", "ref = \"feature/a\""),
    );
    project.git(&["commit", "--quiet", "-am", "New default"]);
    let selected = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.git(&["checkout", "--quiet", "main"]);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let raised = watch.attention();
    let head = raised["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();
    std::fs::write(&armed, "").unwrap();
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    assert_eq!(watch.said(&said), "held\n");
    std::fs::remove_file(&armed).unwrap();
    let intent = project
        .rows("SELECT id, attempt FROM execution WHERE kind = 'landing' AND intent_state = 'open'");
    assert_eq!(intent.len(), 1);
    let refused = project.refused(&["sync", "feature/a", "--default"]);
    assert_eq!(refused["code"], "refused");
    assert_eq!(
        refused["data"],
        json!({ "reason": "live_target", "default": "main", "branch": "feature/a", "attempts": [intent[0]["attempt"]], "intents": [intent[0]["id"]] })
    );
    assert_eq!(default_branch(&project), "main");
    assert_eq!(project.canonical_head(), main);
    assert_eq!(branch_head(&project, "feature/a"), imported);
    std::fs::write(&release, "go\n").unwrap();
    watch.event("landing.recorded", &[]);
    assert!(
        project
            .rows("SELECT id FROM execution WHERE intent_state = 'open'")
            .is_empty()
    );
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE state = 'live'")
            .is_empty()
    );
    let landed = project.canonical_head();
    assert_ne!(landed, main);
    project.json(&["sync", "feature/a", "--default"]);
    assert_eq!(default_branch(&project), "feature/a");
    assert_eq!(branch_head(&project, "feature/a"), selected);
    assert_eq!(project.canonical_head(), landed);
}
