//! G7: A ticket lands end to end and leaves only rows.

use e2e::*;
use serde_json::json;

/// G7 ordinary routes cross a failed gate, then a blocking first seat and
/// passing final seat, before a fresh candidate reaches approval. The final
/// seat must carry the earlier block; checking only its own pass loses repair.
/// Expected order and files come from this fixture, heads from host git.
/// G6: a graph missing the ordinary implementer refuses as custom-graph;
/// the selected ordinary control runs in the same project. Sabotage: admit
/// that shape as ordinary; its named refusal or no-write assertion fails.
/// G7: candidate and landing command order differs from configuration, and
/// a target advance separates candidate and merged inputs before landing.
/// Sabotage: keep the configured scheduler alongside graph scheduling;
/// executions duplicate. Route a final seat's own verdict instead of its
/// panel result; approval arrives before reviewed.txt exists. Ignore the
/// failed gate back edge or its feedback; ready.txt never reaches approval.
#[test]
fn ordinary_candidate_routes_repair_then_reach_approval() {
    let machine = Machine::new("g7-routes", |request| {
        if request
            .prompt()
            .contains("Your workspace is read-only. Plan the work")
        {
            return planned(request);
        }
        if request.has_tool("yard_publish_review") {
            let blocks = request.prompt().contains("correctness reviewer")
                && !request.prompt().contains("reviewed.txt");
            return act(
                request,
                vec![publish(if blocks {
                    json!([{ "priority": "P1", "body": "add reviewed.txt" }])
                } else {
                    json!([])
                })],
            );
        }
        let prompt = request.last_user();
        let file = if prompt.contains("The gate pre failed") {
            "ready.txt"
        } else if prompt.contains("Review blocked") {
            "reviewed.txt"
        } else {
            "feature.txt"
        };
        act(
            request,
            vec![commit_file(file, "feature\n", "Advance candidate")],
        )
    });
    machine.start();
    let configuration = config(
        "[review.seats.final]\nagent = \"worker\"\ninstructions = \"Final seat.\"\n\
        [gates.pre]\ncommand = \"test -f ready.txt\"\nstage = \"candidate\"\n\
        [gates.post]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n",
    )
    .replace(
        "review = [\"correctness\"]",
        "review = [\"correctness\", \"final\"]",
    );
    let project = Project::new(&machine, "p", &configuration);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Ordinary routes",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let mut operator = machine.operator();
    operator.initialize();
    // The missing ordinary implementer is reachable through trusted input,
    // but is refused before admission, not rewritten by ordinary translation.
    let refused = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "graph": {"nodes": {"reporter": {
                "runner": {"kind": "agent", "agent": "worker", "instructions": "Report"},
                "access": "read-only", "outputs": {"result": "report"}, "outcomes": ["complete"]
            }}}
        }),
    );
    assert_eq!(refused["error"]["code"], "refused");
    assert_eq!(refused["error"]["data"]["shape"], "custom-graph");
    assert_eq!(project.rows("SELECT id FROM attempt").len(), 1);
    let admitted = operator.call("yard_attempt_start", json!({
        "project": project.path, "ticket": "Y-1", "workflow": "default", "overrides": {
            "implementer": {"routes": {"complete": [{"to": "candidate-gate-post"}]}},
            "candidate-gate-post": {"routes": {
                "complete": [{"to": "candidate-gate-pre"}], "failed": [{"to": "implementer", "repeat": 1}]}},
            "candidate-gate-pre": {"routes": {
                "complete": [{"to": "review-0"}], "failed": [{"to": "implementer", "repeat": 1}]}},
            "review-1": {"routes": {
                "complete": [{"to": "landing-gate-post"}], "blocked": [{"to": "implementer", "repeat": 3}]}},
            "landing-gate-post": {"routes": {"complete": [{"to": "landing-gate-pre"}]}},
            "landing-gate-pre": {"routes": {}}
        }
    }));
    assert!(admitted.get("error").is_none(), "{admitted}");
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let shown = project.json(&["attempt", "show", "Y-1"]);
    let rows = shown["executions"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["binding"]["node"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "implementer",
            "candidate-gate-post",
            "candidate-gate-pre",
            "implementer",
            "candidate-gate-post",
            "candidate-gate-pre",
            "review-0",
            "review-1",
            "implementer",
            "candidate-gate-post",
            "candidate-gate-pre",
            "review-0",
            "review-1"
        ]
    );
    let head = git(
        &project.path.join(".yard/local/attempts/2/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    assert_eq!(approval["data"]["payload"]["head"], head);
    for file in ["feature.txt", "ready.txt", "reviewed.txt"] {
        assert_eq!(
            git(&project.canonical(), &["show", &format!("{head}:{file}")]),
            "feature\n"
        );
    }
    assert_eq!(rows[2]["result"]["outcome"], "failed");
    assert_eq!(rows[7]["result"]["judgment"]["verdict"], "pass");
    assert_eq!(rows[7]["result"]["outcome"], "blocked");
    assert_eq!(rows[12]["result"]["outcome"], "complete");
    // G7 promises selected input/result agreement. The predecessor results
    // are immutable, including the block that a passing final seat carries.
    for (seat, previous) in [(7, 6), (12, 11)] {
        assert_eq!(
            rows[seat]["binding"]["inputs"]["panel"]["source"]["execution"],
            rows[previous]["execution"]
        );
        assert_eq!(
            rows[seat]["binding"]["inputs"]["panel"]["value"],
            rows[previous]["result"]["outputs"]["result"]["value"]
        );
    }
    for (repair, trigger) in [(3, 2), (8, 7)] {
        assert_eq!(rows[repair]["reason"], "repair");
        assert_eq!(
            rows[repair]["binding"]["inputs"]["feedback"]["source"]["execution"],
            rows[trigger]["execution"]
        );
        assert_eq!(rows[trigger]["result_eligible"], true);
    }
    assert_eq!(
        project.rows("SELECT rounds, next FROM attempt WHERE id = 2"),
        vec![json!({"rounds": 1, "next": null})]
    );

    project.write("target.txt", "advanced\n");
    project.git(&["add", "target.txt"]);
    project.git(&["commit", "-qm", "Advance target"]);
    let target = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "main"]);
    let proof = approval["data"]["payload"]["proof"].as_str().unwrap();
    project.json(&[
        "attempt", "approve", "Y-1", "--target", "main", "--head", &head, "--proof", proof,
    ]);
    watch.event("landing.recorded", &[]);
    watch.until("cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });
    let merged = project.canonical_head();
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{merged}^1")]).trim(),
        target
    );
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{merged}^2")]).trim(),
        head
    );
    let shown = project.json(&["attempt", "show", "Y-1"]);
    let gates: Vec<_> = shown["executions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["kind"] == "gate")
        .collect();
    assert_eq!(
        gates
            .iter()
            .map(|row| row["binding"]["node"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "candidate-gate-post",
            "candidate-gate-pre",
            "candidate-gate-post",
            "candidate-gate-pre",
            "candidate-gate-post",
            "candidate-gate-pre",
            "landing-gate-post",
            "landing-gate-pre"
        ]
    );
    for gate in &gates[4..6] {
        assert_eq!(
            gate["binding"]["inputs"]["candidate"]["value"]["head"],
            head
        );
        assert_eq!(gate["result_eligible"], true);
    }
    for gate in &gates[6..] {
        assert_eq!(gate["binding"]["inputs"]["merged"]["value"]["base"], target);
        assert_eq!(gate["binding"]["inputs"]["merged"]["value"]["head"], merged);
        assert_eq!(gate["result_eligible"], true);
    }
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
}

/// New ticket, worker commit, candidate gate, review pass, approval, green
/// landing: canonical moves and the ticket is done. Afterwards every
/// decision has one audit event naming its target and text, the execution
/// rows carry tokens, cost, model and start reason, every audit event is one
/// the spec names, so no handle event exists, the attempt directory and its
/// boxes are gone, and another live attempt's directory is untouched.
///
/// Sabotage: make cleanup skip `remove(&project.attempt_dir(..))`; the
/// landed attempt's directory survives and the test fails. Record
/// `set_handle` as an audit event; its name is not in the spec's list.
#[test]
fn a_ticket_lands_end_to_end_and_leaves_only_rows() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g7", move |request| {
        if request.has_tool("yard_publish_review") {
            return match request.turn() {
                0 => Reply::Tools(vec![publish(json!([
                    { "priority": "P3", "file": "feature.txt", "line": 1, "body": "a nit" }
                ]))]),
                _ => Reply::Text("published".into()),
            };
        }
        if request.prompt().contains("Y-2") {
            return Reply::Hold(held.clone(), Box::new(Reply::Text("never mind".into())));
        }
        match request.turn() {
            0 => Reply::Tools(vec![bash(
                "cd /workspace && printf 'feature\\n' > feature.txt && git add -A && git commit -q -m 'Add feature' && printf 'evidence' > /yard/proof/evidence.txt && echo committed",
            )]),
            _ => Reply::Text("done".into()),
        }
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n"),
    );
    let start = project.canonical_head();
    let mut watch = project.watch(0);

    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create feature.txt",
    ]);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Hold",
        "--body",
        "The fixture holds this one",
    ]);
    hold.wait_held();

    let raised = watch.until("approval raised", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });
    assert_eq!(raised["ticket"], "Y-1");
    let head = git(
        &project.path.join(".yard/local/attempts/1/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    assert_eq!(raised["data"]["payload"]["head"], head);
    // The candidate is the worker's commit on top of the target.
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:feature.txt")]
        ),
        "feature\n"
    );
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{head}^")]).trim(),
        start
    );

    // The snapshot is on the host before landing, holding what the worker
    // wrote into `/yard/proof`.
    let snapshots = project.path.join(".yard/local/attempts/1/proof-snapshots");
    let proof = std::fs::read_dir(&snapshots)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();
    assert_eq!(raised["data"]["payload"]["proof"], proof);
    let snapshot = project.path.join(format!(
        ".yard/local/attempts/1/proof-snapshots/{proof}/evidence.txt"
    ));
    assert_eq!(std::fs::read(&snapshot).unwrap(), b"evidence");

    // Separate candidate and merged inputs with a real target advance.
    project.write("target.txt", "target advance\n");
    project.git(&["add", "target.txt"]);
    project.git(&["commit", "-qm", "Advance target before landing"]);
    let target = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync", "main"]);

    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head, "--proof", &proof,
        "--text", "ship it",
    ]);
    watch.event("landing.recorded", &[]);
    watch.until("cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });

    // Canonical merges the candidate and the advanced target.
    let merged = project.canonical_head();
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{merged}^1")]).trim(),
        target
    );
    assert_eq!(
        git(&project.canonical(), &["rev-parse", &format!("{merged}^2")]).trim(),
        head
    );
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");

    // G7 node/input/result evidence survives cleanup. Sabotage: attribute a
    // result to another node/input, or feed landing gates the candidate.
    // Names and revision are literal fixture admission; commits come from git,
    // proof from the host snapshot. Execution IDs only establish reference
    // agreement, including CLI/store agreement after cleanup.
    let shown = project.json(&["attempt", "show", "Y-1"]);
    let rows = shown["executions"].as_array().unwrap();
    let nodes: Vec<_> = rows
        .iter()
        .filter(|row| row["binding"]["status"] == "recorded")
        .collect();
    assert_eq!(
        nodes
            .iter()
            .map(|row| row["binding"]["node"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "implementer",
            "candidate-gate-check",
            "review-0",
            "landing-gate-check"
        ]
    );
    let candidate = json!({"base": start, "head": head, "proof": proof});
    assert_eq!(nodes[0]["binding"]["inputs"], json!({}));
    assert_eq!(
        nodes[0]["result"]["outputs"]["result"],
        json!({"kind": "candidate", "value": candidate})
    );
    for row in &nodes {
        assert_eq!(row["plan_revision"], 1);
        assert_eq!(row["result_eligible"], true);
        assert_eq!(row["teardown"], true);
        assert_eq!(row["result"]["outcome"], "complete");
        assert_eq!(row["result"]["infrastructure"]["outcome"], "completed");
    }
    for row in &nodes[1..3] {
        assert_eq!(
            row["binding"]["inputs"],
            json!({"candidate": {
                "kind": "candidate", "source": {"execution": nodes[0]["execution"], "node": "implementer", "output": "result"},
                "value": candidate,
            }})
        );
        assert_eq!(row["result"]["outputs"]["result"]["kind"], "verdict");
        assert_eq!(row["result"]["judgment"]["verdict"], "pass");
    }
    assert_eq!(
        nodes[3]["binding"]["inputs"],
        json!({"merged": {
            "kind": "candidate", "source": {"external": "merged", "execution": nodes[3]["parent"]},
            "value": {"base": target, "head": merged, "proof": proof},
        }})
    );
    assert_eq!(nodes[3]["result"]["judgment"]["verdict"], "pass");
    let stored = project.rows("SELECT node, inputs, result, plan_revision, teardown FROM execution WHERE attempt = 1 AND node IS NOT NULL ORDER BY id");
    assert_eq!(stored.len(), nodes.len());
    for (row, saved) in nodes.iter().zip(&stored) {
        assert_eq!(saved["node"], row["binding"]["node"]);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(saved["inputs"].as_str().unwrap()).unwrap(),
            row["binding"]["inputs"]
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(saved["result"].as_str().unwrap()).unwrap(),
            row["result"]
        );
        assert_eq!(saved["plan_revision"], row["plan_revision"]);
        assert_eq!(saved["teardown"], 1);
    }

    // Every decision has one audit event naming its target, with its text.
    let events = project.rows("SELECT event, ticket, text FROM audit ORDER BY seq");
    let count = |name: &str, ticket: i64| {
        events
            .iter()
            .filter(|event| event["event"] == name && event["ticket"] == ticket)
            .count()
    };
    for name in [
        "ticket.new",
        "attempt.admitted",
        "approval.given",
        "landing.intent",
        "landing.recorded",
        "ticket.done",
    ] {
        assert_eq!(count(name, 1), 1, "{name} for Y-1 in {events:?}");
    }
    let approval = events
        .iter()
        .find(|event| event["event"] == "approval.given")
        .unwrap();
    assert_eq!(approval["text"], "ship it");
    // G7 requires the rows and their decision events to agree, including
    // after cleanup while another attempt is still live. Sabotage: omit or
    // duplicate the audit call in executions::start/end or checks::record,
    // or give it another execution, attempt or check identity.
    let executions = project
        .rows("SELECT attempt, id AS execution FROM execution WHERE attempt = 1 ORDER BY id");
    for event in ["execution.started", "execution.ended"] {
        assert_eq!(
            project.rows(&format!(
                "SELECT attempt, execution FROM audit WHERE event = '{event}'
                 AND (attempt = 1 OR execution IN (SELECT id FROM execution WHERE attempt = 1))
                 ORDER BY execution"
            )),
            executions,
            "{event} must name each execution exactly once"
        );
    }
    assert_eq!(
        project.rows(
            "SELECT json_extract(data, '$.check') AS check_id, attempt, execution FROM audit
             WHERE event = 'check.recorded'
             AND (attempt = 1 OR execution IN (SELECT id FROM execution WHERE attempt = 1))
             ORDER BY check_id"
        ),
        project.rows(
            "SELECT c.id AS check_id, e.attempt, c.execution FROM \"check\" c
             JOIN execution e ON e.id = c.execution WHERE e.attempt = 1 ORDER BY c.id"
        ),
        "each check must have exactly one event naming its check, attempt and execution"
    );
    // Handles and usage are column writes, never events: every event is
    // one of the spec's closed list (ARCHITECTURE.md, Store).
    // Sabotage: emit an unlisted handle or usage audit event on completion.
    let spec = [
        "sync.imported",
        "sync.consumed",
        "sync.default",
        "ticket.new",
        "ticket.edited",
        "ticket.parked",
        "ticket.unparked",
        "ticket.linked",
        "ticket.done",
        "ticket.abandoned",
        "attempt.admitted",
        "attempt.candidate",
        "attempt.stopped",
        "attempt.abandoned",
        "attempt.ended",
        "execution.started",
        "execution.ended",
        "execution.waiting",
        "execution.reserved",
        "execution.released",
        "check.recorded",
        "approval.given",
        "approval.ended",
        "landing.prepared",
        "landing.continued",
        "landing.intent",
        "landing.recorded",
        "attention.raised",
        "attention.resolved",
        "report.published",
        "tool.refused",
    ];
    for event in &events {
        let name = event["event"].as_str().unwrap();
        assert!(
            spec.contains(&name),
            "{name} is not a spec event: {events:?}"
        );
    }

    // The worker rows carry the statistics.
    let workers = project.rows(
        "SELECT kind, reason, model, provider, tokens_in, tokens_out, cost FROM execution
         WHERE attempt = 1 AND kind IN ('implementation', 'review') ORDER BY id",
    );
    assert_eq!(workers.len(), 2, "{workers:?}");
    let implementation = &workers[0];
    assert_eq!(implementation["kind"], "implementation");
    assert_eq!(implementation["reason"], "first");
    assert_eq!(implementation["model"], "fake-model");
    assert_eq!(implementation["provider"], "openrouter");
    // Two model turns at the fixture's fixed usage.
    assert_eq!(implementation["tokens_in"], 2 * PROMPT_TOKENS as i64);
    assert_eq!(implementation["tokens_out"], 2 * COMPLETION_TOKENS as i64);
    assert!(implementation["cost"].as_f64().unwrap() > 0.0);

    // The landed attempt left rows only: no directory, no box, and no proof
    // snapshot; the check and approval rows carry the proof digest.
    assert!(!project.path.join(".yard/local/attempts/1").exists());
    let attempt_proof = project.rows("SELECT proof FROM attempt WHERE id = 1")[0]["proof"].clone();
    assert!(
        attempt_proof
            .as_str()
            .is_some_and(|proof| !proof.is_empty()),
        "{attempt_proof:?}"
    );
    let check_proofs = project
        .rows("SELECT e.proof AS proof FROM \"check\" c JOIN execution e ON e.id = c.execution");
    assert!(!check_proofs.is_empty(), "no checks");
    assert!(
        check_proofs.iter().all(|row| row["proof"] == attempt_proof),
        "{check_proofs:?}"
    );
    let approval_proofs = project.rows("SELECT proof FROM approval");
    assert_eq!(approval_proofs, vec![json!({ "proof": attempt_proof })]);
    let handles = project.rows(
        "SELECT handle FROM execution WHERE attempt = 1 AND kind IN ('implementation', 'gate', 'review')",
    );
    assert!(!handles.is_empty());
    let boxes = machine.boxes("dev.yard.project");
    for handle in &handles {
        assert!(
            boxes
                .iter()
                .all(|listed| listed["name"] != handle["handle"]),
            "{handle} still has a box"
        );
    }
    // The other live attempt kept its clone.
    assert!(
        project
            .path
            .join(".yard/local/attempts/2/clone/.git")
            .is_dir()
    );
    // G14: all requests across this lifecycle reached the model route, and
    // the daemon has only its local MCP listener and operator socket. Only
    // the route puts the operator's key in place of Pi's placeholder.
    // Sabotage: stage the upstream origin as Pi's provider URL; no request
    // carries the key. Bind daemon::serve's listener on 0.0.0.0.
    let requests = machine.model.requests();
    assert!(!requests.is_empty());
    let authorization = format!("Bearer {SECRET}");
    for request in &requests {
        assert_eq!(request.path, "/api/v1/chat/completions");
        assert_eq!(
            request.header("authorization"),
            Some(authorization.as_str())
        );
    }

    // lsof, which both platforms have: the daemon's own sockets.
    let pid = machine.daemon_pid().to_string();
    let sockets = |args: &[&str]| -> Vec<String> {
        let out = std::process::Command::new("lsof")
            .args(["-nP", "-a", "-p", &pid])
            .args(args)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .skip(1)
            .map(str::to_string)
            .collect()
    };
    let tcp = sockets(&["-iTCP", "-sTCP:LISTEN"]);
    assert_eq!(tcp.len(), 1, "{tcp:?}");
    assert!(tcp[0].contains("127.0.0.1:"), "{tcp:?}");
    let udp = sockets(&["-iUDP"]);
    assert!(udp.is_empty(), "{udp:?}");
    // Every unix socket bound to a path is the daemon's own; a connection
    // accepted on it names the same path.
    let socket = machine.state.join("yard/yard.sock").display().to_string();
    let unix: Vec<String> = sockets(&["-U"])
        .into_iter()
        .filter(|line| line.contains('/'))
        .collect();
    assert!(!unix.is_empty(), "no unix socket");
    for line in &unix {
        assert!(line.contains(&socket), "{unix:?}");
    }

    hold.release();
}

/// `status --watch` returns as soon as an attention item is open, whatever
/// its age: an approval already open when it starts returns at once. After
/// the last attempt ends with nothing open or ready, a watch armed with
/// `--since` past an open approval returns with no items. One armed on that
/// idle board keeps waiting, and returns the next item raised. With
/// `--since` naming the seq of an open approval it keeps waiting until a
/// further item is raised, then prints both.
///
/// Sabotage: key `--watch` on events after start (follow the audit stream
/// from the current seq). The already-open approval never raises again, so
/// the first `status --watch` never returns and the test fails at the
/// deadline. Ignore `--since`; the drain watch returns Y-1's approval at
/// once, and the last watch prints Y-1 alone. Count an `attempt.ended` at
/// any seq as the drain; the watch armed on the idle board returns no items.
/// Never count an idle board under `--since`; the drain watch never returns.
#[test]
fn status_watch_wakes_on_attention_and_the_final_drain() {
    let machine = Machine::new("g7watch", |request| {
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut history = project.watch(0);

    // Y-1's approval is open before the watch starts.
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create feature.txt",
    ]);
    history.until("approval raised", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });

    // The watch returns the open item at once.
    let items = attention_items(watch_attention(&project, None));
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["kind"], "approval");
    assert_eq!(items[0]["ticket"], "Y-1");

    // A drain watch armed past the open approval. Clear the item: park so
    // the scheduler does not start it again, then abandon. The last attempt
    // has ended with nothing open or ready, and the drain returns no items.
    let seq = project.json(&["status"])["seq"]
        .as_i64()
        .expect("status seq");
    let draining = watch_attention(&project, Some(seq));
    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);
    let ended = history.find("Y-1 ended", |event| {
        event["event"] == "attempt.ended" && event["ticket"] == "Y-1"
    });
    let drained = watch_answer(draining);
    assert_eq!(drained["attention"], json!([]));
    assert!(drained["seq"].as_i64().unwrap() >= ended["seq"].as_i64().unwrap());

    // The next watch starts on the idle board, after that `attempt.ended`;
    // `--since` keeps it from returning at once.
    assert!(
        project.json(&["status"])["attention"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
    let armed = project.json(&["status"])["seq"]
        .as_i64()
        .expect("status seq");
    let watching = watch_attention(&project, Some(armed));
    project.json(&["ticket", "unpark", "Y-1"]);

    // The scheduler starts Y-1 again; the watch returns its new approval.
    let items = attention_items(watching);
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["kind"], "approval");
    assert_eq!(items[0]["ticket"], "Y-1");

    // `--since` waits past an item already open. The answer's seq is the
    // stream position: Y-1's approval is open and was raised at or before it.
    let answered = watch_answer(watch_attention(&project, None));
    let seq = answered["seq"].as_i64().expect("watch prints its seq");
    let waiting = watch_attention(&project, Some(seq));

    // Raise the next item through the fixture: the watch returns both open
    // items and a seq at or after the new item's event.
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Second",
        "--body",
        "Create feature.txt",
    ]);
    let second = history.until("Y-2 approval", |event| {
        event["event"] == "attention.raised" && event["ticket"] == "Y-2"
    });
    let answer = watch_answer(waiting);
    let mut tickets: Vec<&str> = answer["attention"]
        .as_array()
        .expect("attention list")
        .iter()
        .map(|item| item["ticket"].as_str().unwrap_or_default())
        .collect();
    tickets.sort();
    assert_eq!(tickets, ["Y-1", "Y-2"], "{answer}");
    assert!(answer["seq"].as_i64().unwrap() >= second["seq"].as_i64().unwrap());
}
