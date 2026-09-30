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
    project.json(&["attempt", "approve", "Y-1", "--head", head]);
    watch.event("landing.recorded", &[]);
    git(
        &canonical,
        &["merge-base", "--is-ancestor", head, "refs/heads/main"],
    );
}
