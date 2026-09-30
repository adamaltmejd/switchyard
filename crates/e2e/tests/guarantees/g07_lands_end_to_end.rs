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
    let head = raised["data"]["payload"]["head"]
        .as_str()
        .unwrap()
        .to_string();
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
    let proof = raised["data"]["payload"]["proof"].as_str().unwrap();
    let snapshot = project.path.join(format!(
        ".yard/local/attempts/1/proof-snapshots/{proof}/evidence.txt"
    ));
    assert_eq!(std::fs::read(&snapshot).unwrap(), b"evidence");

    project.json(&[
        "attempt", "approve", "Y-1", "--head", &head, "--proof", proof, "--text", "ship it",
    ]);
    watch.event("landing.recorded", &[]);
    watch.until("cleanup", |event| {
        event["event"] == "execution.ended" && event["data"]["kind"] == "cleanup"
    });

    // Canonical is a fast-forward to the candidate, and the ticket is done.
    assert_eq!(project.canonical_head(), head);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");

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
    let spec = [
        "sync.imported",
        "sync.consumed",
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
        "check.recorded",
        "approval.given",
        "approval.ended",
        "landing.intent",
        "landing.recorded",
        "attention.raised",
        "attention.resolved",
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
