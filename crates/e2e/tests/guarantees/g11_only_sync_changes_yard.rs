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
/// The same change through `yard sync` is in force for the next admission.
/// The change adds a second candidate gate: refused from the worker, it runs
/// on a future candidate once the operator syncs it.
///
/// Sabotage: make `supervise::implement` skip its `.yard` check; the
/// worker's commit becomes the candidate and the gate runs on it.
/// Omit `.yard` from supervise's returned refusal detail while retaining
/// the guard and repair route: the stored reason loses the resource identity.
#[test]
fn only_the_operators_sync_changes_yard_and_canonical() {
    let gated = format!("{}{BASE_GATE}", unreviewed());
    let configured = format!("{gated}{EXTRA_GATE}");
    let change = configured.replace('\'', "'\\''");
    let machine = Machine::new("g11-yard", move |request| {
        if request.prompt().contains("Future feature") {
            return act(
                request,
                vec![commit_file("future.txt", "future\\n", "Future feature")],
            );
        }
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
    // G11's reason names the reserved resource changed by the fixture's
    // `.yard/config.toml` commit, independently of refusal and gate routing.
    let refusal = project.rows(
        "SELECT detail FROM execution WHERE kind = 'implementation' AND reason = 'first' AND outcome = 'refused'",
    );
    assert!(
        refusal[0]["detail"].as_str().unwrap().contains(".yard"),
        "{refusal:?}"
    );

    project.reconfigure(&configured);
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    watch.event("landing.recorded", &[]);
    // G11/G2: the allowed operator sync supplies a future admitted gate.
    // Sabotage: keep using the previous admission for future attempts.
    project.json(&["ticket", "new", "--title", "Future feature"]);
    let second = watch.attention();
    assert_eq!(second["data"]["kind"], "approval");
    let future = second["data"]["payload"]["head"].as_str().unwrap();
    assert_eq!(
        project.rows("SELECT head, outcome FROM execution WHERE kind = 'gate' AND name = 'extra'"),
        vec![json!({"head": future, "outcome": "pass"})]
    );
    project.json(&["ticket", "park", "Y-2"]);
    project.json(&["attempt", "abandon", "Y-2"]);
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
