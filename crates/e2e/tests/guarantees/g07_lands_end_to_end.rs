//! G7: A ticket lands end to end and leaves only rows.

use e2e::*;
use serde_json::json;

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
        "candidate.selected",
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

    // G7: the retained positive costs are summed once. The raw measured rows
    // are external evidence; fixture turns supply the token expectation.
    // Sabotage: sum predecessor totals or hide unknown gate/cleanup costs.
    let mut operator = machine.operator();
    let stats = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    let aggregate = &stats["result"]["aggregate"];
    assert_eq!(
        aggregate["input"]["known_sum"].as_f64(),
        Some((4 * PROMPT_TOKENS) as f64)
    );
    assert_eq!(
        aggregate["cost"]["reported"]["known_sum"].as_f64(),
        Some(
            workers
                .iter()
                .map(|row| row["cost"].as_f64().unwrap())
                .sum::<f64>()
        )
    );
    assert_eq!(aggregate["cost"]["reported"]["complete_count"], 2);
    assert!(
        aggregate["cost"]["reported"]["missing_count"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert_eq!(aggregate["cost"]["reported"]["completeness"], "partial");

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
