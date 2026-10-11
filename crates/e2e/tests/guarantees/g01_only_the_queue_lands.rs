//! G1: Only the queue lands.

use e2e::*;
use serde_json::json;
use std::sync::{Arc, Mutex};

/// A worker with its MCP bearer and its clone: `git push` fails, no RPC
/// lands, canonical is not reachable from the box. In one model turn the
/// worker commits, pushes to canonical's host path, calls the daemon's
/// socket and asks the MCP route for a daemon method. Control: the queue
/// lands the same candidate.
///
/// Sabotage: mount the project's `.yard/local` into the worker box at its
/// host path; the push to canonical lands the commit. Mount the daemon's
/// state dir (the socket) into the worker box; the socket call connects.
/// Accept an MCP request that carries no `Mcp-Session-Id`: in `answer` in
/// crates/yard/src/mcp.rs change `else if !daemon.grants.in_session(..)` to
/// `else if false`; the probe gets a JSON-RPC answer,
/// not the 404. A daemon method arm in `mcp::answer` alone is not detected:
/// the sessionless probe is refused before the method is read.
#[test]
fn a_worker_cannot_land_by_push_rpc_or_canonical() {
    let probes = Arc::new(Mutex::new(String::new()));
    let seen = probes.clone();
    let script = Arc::new(Mutex::new(String::new()));
    let run = script.clone();
    let machine = Machine::new("g1", move |request| match request.tool_results().len() {
        0 => Reply::Tools(vec![
            commit_file("feature.txt", "feature\n", "Add feature"),
            bash(&run.lock().unwrap()),
        ]),
        _ => {
            *seen.lock().unwrap() = request.last_tool_result().unwrap().1;
            Reply::Text("done".into())
        }
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let canonical = project.canonical();
    let socket = machine.state.join("yard/yard.sock");
    // A call on the MCP route with the worker's bearer; `pattern` is what
    // its answer is searched for, printed after `name=`. The bearer alone has no
    // MCP session, so the route answers nothing but a 404 status.
    let call = |name: &str, body: serde_json::Value, pattern: &str| {
        format!(
            "curl -s -H \"Authorization: Bearer $YARD_MCP_BEARER\" -H 'content-type: application/json' \
             \"$YARD_MCP_ENDPOINT\" -d '{body}' -w '%{{http_code}}\n' | grep -o -e '{pattern}' | head -1 | sed 's/^/{name}=/'; "
        )
    };
    *script.lock().unwrap() = format!(
        "cd /workspace; \
         git push --quiet '{canonical}' HEAD:refs/heads/main 2>/dev/null; echo \"push-canonical=$?\"; \
         test -e '{canonical}'; echo \"canonical=$?\"; \
         curl -s --max-time 5 --unix-socket '{socket}' http://yard/rpc \
           -d '{{\"method\":\"attempt.approve\",\"params\":{{}}}}' >/dev/null 2>&1; echo \"socket=$?\"; \
         {method}",
        canonical = canonical.display(),
        socket = socket.display(),
        method = call(
            "method",
            json!({ "jsonrpc": "2.0", "id": 1, "method": "attempt.approve", "params": {} }),
            "404",
        ),
    );
    let target = project.canonical_head();
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");

    let probes = probes.lock().unwrap().clone();
    for line in ["push-canonical=", "canonical=1", "method=404"] {
        assert!(probes.contains(line), "{line} missing from:\n{probes}");
    }
    assert!(!probes.contains("push-canonical=0"), "{probes}");
    assert!(!probes.contains("socket=0"), "{probes}");
    assert_eq!(project.canonical_head(), target);

    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", head,
    ]);
    watch.event("landing.recorded", &[]);
    git(
        &canonical,
        &["merge-base", "--is-ancestor", head, "refs/heads/main"],
    );
}

/// G1/G3/G8: one project lands its default chain, a reduced panel with a
/// missing middle seat, then an empty verification selection under auto
/// policy with a protected change. Expectations are fixture node identities,
/// scripted findings and host git trees, not resolver output. Existing full
/// chains cannot expose hidden omitted gates or a stale panel predecessor.
/// Sabotage: run configured landing gates despite selection; their rows appear.
/// Fabricate passes for skipped nodes; the checked node/verdict sequence differs. Derive
/// unreviewed from workflow.review; the empty selection is marked reviewed.
/// Keep review-2's old predecessor or discard its incoming block; the reduced
/// panel cannot repair once before approval. Auto-approve protected paths;
/// canonical moves before the operator answers the empty selection.
#[test]
fn operator_reduced_verification_stays_skipped_and_queue_authorized() {
    use serde_json::Value;
    use std::sync::atomic::{AtomicBool, Ordering};

    let blocked = AtomicBool::new(false);
    let machine = Machine::new("g1-reduced", move |request| {
        if request
            .prompt()
            .contains("Your workspace is read-only. Plan the work")
        {
            return planned(request);
        }
        if request.has_tool("yard_publish_review") {
            let findings = if request.context().contains("Reduced")
                && request.prompt().contains("First seat.")
                && request.opens()
                && !blocked.swap(true, Ordering::SeqCst)
            {
                json!([{ "priority": "P1", "body": "Retained first seat blocks" }])
            } else {
                json!([])
            };
            return act(request, vec![publish(findings)]);
        }
        let (path, contents) = if request.context().contains("Empty") {
            (
                "AGENTS.md",
                "Rule: every file ends with a newline.\nOperator-selected change.\n",
            )
        } else if request.context().contains("Review blocked") {
            ("repair.txt", "repaired\n")
        } else if request.context().contains("Reduced") {
            ("reduced.txt", "reduced\n")
        } else {
            ("full.txt", "full\n")
        };
        act(request, vec![commit_file(path, contents, "Selected work")])
    });
    machine.start();
    let fixture = config(
        r#"
[review.seats.middle]
agent = "worker"
instructions = "Middle seat."
[review.seats.last]
agent = "worker"
instructions = "Last seat."
[gates.check]
command = "git rev-parse HEAD"
stage = "candidate"
[gates.final]
command = "git rev-parse HEAD"
"#,
    )
    .replace(
        "review = [\"correctness\"]",
        "review = [\"correctness\", \"middle\", \"last\"]",
    )
    .replace("Review for correctness.", "First seat.");
    let project = Project::new(&machine, "p", &fixture);
    let mut watch = project.watch(0);
    let mut operator = machine.operator();
    operator.initialize();
    let implementer = json!({
        "runner": {"kind": "agent", "agent": "worker", "instructions": ""},
        "access": "write-allowed", "inputs": {"feedback": {"output": "feedback", "kind": "verdict"}},
        "outputs": {"result": "candidate"}, "outcomes": ["complete"], "requires": []
    });
    let mut reduced = json!({"inputs": {"feedback": "verdict", "merged": "candidate"}, "nodes": {
        "implementer": implementer,
        "review-0": {
            "runner": {"kind": "agent", "agent": "worker", "instructions": "First seat."},
            "access": "read-only", "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"], "requires": ["candidate"],
            "routes": {"complete": [{"to": "review-2"}]}
        },
        "review-2": {
            "runner": {"kind": "agent", "agent": "worker", "instructions": "Last seat."},
            "access": "read-only", "inputs": {
                "candidate": {"node": "implementer", "output": "result", "kind": "candidate"},
                "panel": {"node": "review-1", "output": "result", "kind": "verdict"}
            },
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "blocked"], "requires": ["candidate", "panel"],
            "routes": {"complete": [{"to": "landing-gate-final"}], "blocked": [{"to": "implementer", "repeat": 3}]}
        },
        "landing-gate-final": {
            "runner": {"kind": "command", "gate": "final"}, "access": "write-allowed",
            "inputs": {"merged": {"output": "merged", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"], "requires": ["merged"], "routes": {}
        }
    }});
    reduced["nodes"]["implementer"]["routes"] = json!({"complete": [{"to": "review-0"}]});
    let empty = json!({"inputs": {"feedback": "verdict"}, "nodes": {"implementer": implementer}});
    let omitted = [
        json!([]),
        json!(["candidate-gate-check", "landing-gate-check", "review-1"]),
        json!([
            "candidate-gate-check",
            "landing-gate-check",
            "landing-gate-final",
            "review-0",
            "review-1",
            "review-2"
        ]),
    ];
    for (index, title) in ["Full", "Reduced", "Empty"].iter().enumerate() {
        let ticket = format!("Y-{}", index + 1);
        if index == 2 {
            project.reconfigure(&fixture.replace("approve = \"manual\"", "approve = \"auto\""));
        }
        if index == 0 {
            project.json(&["ticket", "new", "--title", title]);
        } else {
            project.json(&["ticket", "new", "--title", title, "--workflow", "plan"]);
            watch.event("attempt.ended", &[("outcome", "planned")]);
            let started = operator.call(
                "yard_attempt_start",
                json!({
                    "project": project.path, "ticket": ticket,
                    "graph": if index == 1 { &reduced } else { &empty }
                }),
            );
            assert!(started.get("error").is_none(), "{started}");
        }
        let approval = watch.attention();
        assert_eq!(approval["data"]["kind"], "approval", "{approval}");
        assert_eq!(approval["data"]["payload"]["unreviewed"], index == 2);
        assert_eq!(approval["data"]["payload"]["skipped"], omitted[index]);
        let show = project.json(&["attempt", "show", &ticket]);
        let attempt = show["attempt"]["attempt"].as_i64().unwrap();
        assert_eq!(show["attempt"]["admission"]["skipped"], omitted[index]);
        assert_eq!(show["attempt"]["admission"]["unreviewed"], index == 2);
        if index == 1 {
            assert_eq!(
                show["attempt"]["admission"]["plan"]["nodes"]["review-2"]["inputs"]["panel"]["node"],
                "review-0"
            );
            assert_eq!(project.rows(&format!("SELECT node FROM execution WHERE attempt = {attempt} AND kind = 'review' ORDER BY id")), vec![
                json!({"node": "review-0"}), json!({"node": "review-2"}),
                json!({"node": "review-0"}), json!({"node": "review-2"})
            ]);
            assert_eq!(project.rows(&format!("SELECT reason FROM execution WHERE attempt = {attempt} AND kind = 'implementation' ORDER BY id")), vec![json!({"reason": "first"}), json!({"reason": "repair"})]);
        }
        // Force a real merge after approval; even the empty selection must
        // record this exact tree, not fabricate a verification pass.
        let before = project.canonical_head();
        if index == 2 {
            assert_eq!(
                approval["data"]["payload"]["protected"],
                json!(["AGENTS.md"])
            );
            assert_eq!(
                project.rows(&format!(
                    "SELECT id FROM execution WHERE attempt = {attempt} AND kind = 'landing'"
                )),
                Vec::<Value>::new()
            );
        }
        project.json(&["sync"]);
        project.write("target.txt", &format!("target {index}\n"));
        project.git(&["add", "target.txt"]);
        project.git(&["commit", "-qm", "Advance target"]);
        project.json(&["sync"]);
        let old = project.canonical_head();
        assert_ne!(old, before);
        let head = approval["data"]["payload"]["head"].as_str().unwrap();
        let expected_tree = git(
            &project.canonical(),
            &["merge-tree", "--write-tree", &old, head],
        );
        project.json(&[
            "attempt", "approve", "--target", "main", &ticket, "--head", head,
        ]);
        watch.event("landing.recorded", &[]);
        let merged = project.canonical_head();
        let parents = git(
            &project.canonical(),
            &["show", "-s", "--format=%P", &merged],
        );
        assert_eq!(parents.trim(), format!("{old} {head}"));
        let tree = git(
            &project.canonical(),
            &["rev-parse", &format!("{merged}^{{tree}}")],
        );
        assert_eq!(tree.trim(), expected_tree.lines().next().unwrap());
        let landing = project.rows(&format!("SELECT intent_old, intent_merged, intent_state FROM execution WHERE attempt = {attempt} AND kind = 'landing'"));
        assert_eq!(
            landing,
            vec![json!({"intent_old": old, "intent_merged": merged, "intent_state": "resolved"})]
        );
        let gates = project.rows(&format!("SELECT node, head FROM execution WHERE attempt = {attempt} AND kind = 'gate' ORDER BY id"));
        let expected = match index {
            0 => vec![
                json!({"node": "candidate-gate-check", "head": head}),
                json!({"node": "landing-gate-check", "head": merged}),
                json!({"node": "landing-gate-final", "head": merged}),
            ],
            1 => vec![json!({"node": "landing-gate-final", "head": merged})],
            _ => vec![],
        };
        assert_eq!(gates, expected);
        // Checks are candidate/review publications. Landing children carry
        // their judgments on execution; their exact nodes and heads are above.
        let checks = project.rows(&format!(
            "SELECT e.node, c.verdict FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE e.attempt = {attempt} ORDER BY c.id"
        ));
        let expected_checks = match index {
            0 => vec![
                json!({"node": "candidate-gate-check", "verdict": "pass"}),
                json!({"node": "review-0", "verdict": "pass"}),
                json!({"node": "review-1", "verdict": "pass"}),
                json!({"node": "review-2", "verdict": "pass"}),
            ],
            1 => vec![
                json!({"node": "review-0", "verdict": "fail"}),
                json!({"node": "review-2", "verdict": "pass"}),
                json!({"node": "review-0", "verdict": "pass"}),
                json!({"node": "review-2", "verdict": "pass"}),
            ],
            _ => vec![],
        };
        assert_eq!(checks, expected_checks);
    }
}
