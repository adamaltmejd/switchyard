//! G1/G2/G6/G7 through trusted stdio, while real worker HTTP stays restricted.
use e2e::*;
use serde_json::{Value, json};

fn args(project: &Project, extra: Value) -> Value {
    let mut result = extra;
    result["project"] = json!(project.path);
    result
}

fn call(operator: &mut Operator, project: &Project, tool: &str, extra: Value) -> Value {
    let answer = operator.call(tool, args(project, extra));
    assert!(answer.get("error").is_none(), "{answer}");
    answer["result"].clone()
}

pub(super) fn scoped_node(report: bool) -> Value {
    json!({"runner": if report {
        json!({"kind": "agent", "agent": "worker", "instructions": "Audit the exact input."})
    } else { json!({"kind": "command", "gate": "standalone"}) },
        "access": if report { "read-only" } else { "write-allowed" },
        "outputs": {"result": if report { "report" } else { "verdict" }},
        "outcomes": ["complete"]})
}

pub(super) fn scoped_start(
    operator: &mut Operator,
    project: &Project,
    input: &str,
    report: bool,
) -> i64 {
    call(operator, project, "yard_execution_start",
        json!({"target": "main", "input": input, "node": scoped_node(report), "timeout_seconds": 300}))
        ["execution"].as_i64().unwrap()
}

/// G5: cancellation during shared image preparation with spare capacity.
/// The wrapper delegates real Git, then holds its inherited execution lock
/// and the actual context until fixture release. Host inode, bytes and flock
/// are the oracles. Sabotage: release the image mutex on cancellation without
/// retaining the preparation association through checked execution release.
/// Existing down/exec holds cannot expose a successor deleting a Git context.
#[test]
fn cancelled_image_preparation_retains_its_context_until_checked_owner_release() {
    use std::os::unix::fs::MetadataExt;
    let machine = Machine::new("g68-image-owner", |_| Reply::Text("unused".into()));
    let held = machine.root.join("image-held");
    let release = machine.root.join("image-release");
    let armed = machine.root.join("image-armed");
    let command_done = machine.root.join("boxed-command-done");
    for path in [&held, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(&armed, "").unwrap();
    machine.wrapper(
        "git",
        &format!(
            r#"
clone=0
destination=''
for argument in "$@"; do
    if [ "$argument" = clone ]; then clone=1; fi
    destination="$argument"
done
case "$destination" in
    */.yard/local/build/*)
        if [ "$clone" = 1 ] && [ -e '{armed}' ]; then
            rm '{armed}'
            "$REAL" "$@"
            result=$?
            printf 'original-context\n' > "$destination/owner-marker"
            printf '%s\n%s\n%s\n' "$$" "$destination" "$result" > '{held}'
            read line < '{release}'
            printf 'last-owner-write\n' >> "$destination/owner-marker"
            exit "$result"
        fi
        ;;
esac
"#,
            armed = armed.display(),
            held = held.display(),
            release = release.display()
        ),
    );
    machine.wrapper("pinfold", &format!(
        "if [ \"$1 $2\" = 'box exec' ]; then \"$REAL\" \"$@\"; result=$?; printf '%s\\n' \"$result\" > '{}'; exit \"$result\"; fi",
        command_done.display()));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(
            r#"
[gates.standalone]
command = "printf 'successor-command\n'"
runs_in = "box"
"#,
        )
        .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 2"),
    );
    let input = project.canonical_head();
    let mut operator = machine.operator();
    operator.initialize();
    let mut history = project.watch(0);
    let original = scoped_start(&mut operator, &project, &input, false);
    let proof = history.said_before_end(&held, Some(original));
    let lines: Vec<_> = proof.lines().collect();
    let pid: u32 = lines[0].parse().unwrap();
    let context = std::path::PathBuf::from(lines[1]);
    assert_eq!(
        lines[2], "0",
        "the held child must delegate successful real Git"
    );
    assert_eq!(context, project.path.join(".yard/local/build").join(&input));
    let marker = context.join("owner-marker");
    let inode = std::fs::metadata(&context).unwrap().ino();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(
            project
                .path
                .join(format!(".yard/local/effects/{original}/lock")),
        )
        .unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    call(
        &mut operator,
        &project,
        "yard_execution_stop",
        json!({"execution": original}),
    );
    history.find("cancelled preparation ended", |event| {
        event["event"] == "execution.ended" && event["execution"] == original
    });
    history.find("held preparation ownership", |event| {
        event["event"] == "attention.raised"
            && event["execution"] == original
            && event["data"]["reason"] == "ownership"
    });
    assert_eq!(
        project.rows(&format!(
            "SELECT outcome, reserved, teardown FROM execution WHERE id = {original}"
        )),
        vec![json!({"outcome": "stopped", "reserved": 1, "teardown": 0})]
    );
    // Restart discards the in-memory image mutex. External ownership still
    // protects the context; the successor has its own available capacity.
    drop(history);
    drop(operator);
    machine.kill();
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    machine.start();
    let mut operator = machine.operator();
    operator.initialize();
    let mut history = project.watch(0);
    let successor = scoped_start(&mut operator, &project, &input, false);
    history.find("successor waits for image preparation owner", |event| {
        event["event"] == "execution.waiting"
            && event["execution"] == successor
            && event["data"]["reason"] == "image_preparation"
            && event["data"]["owner"] == original
    });
    assert_eq!(project.rows(&format!("SELECT id, reserved, handle FROM execution WHERE id IN ({original}, {successor}) ORDER BY id")),
        vec![json!({"id": original, "reserved": 1, "handle": null}), json!({"id": successor, "reserved": 1, "handle": null})]);
    assert_eq!(std::fs::metadata(&context).unwrap().ino(), inode);
    assert!(!command_done.exists());
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "original-context\n"
    );
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert!(
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(&release, "go\n").unwrap();
    history.find("original preparation checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == original
    });
    history.find("successor checked completion", |event| {
        event["event"] == "execution.released" && event["execution"] == successor
    });
    let result = call(
        &mut operator,
        &project,
        "yard_execution_show",
        json!({"execution": successor}),
    );
    assert_eq!(result["outcome"], "complete");
    assert_eq!(result["result"]["infrastructure"]["outcome"], "completed");
    assert_eq!(std::fs::read_to_string(&command_done).unwrap(), "0\n");
    assert!(!context.exists());
}

/// G5/G6/G13: no tickets, real boxed command and report over operator stdio.
/// Hold the report after publication, before its final host model reply.
/// Oracles are the empty fixture, host Git, real pinfold exec and model tokens.
/// Sabotage: manufacture an attempt; discard scope; forward worker creation;
/// mount report source writable; publish a check/approval or release early.
/// Accept standalone selected skills/references without staging them; the
/// selected-input refusal fails. Existing command/report starts are the
/// same-project controls; fixture paths and unchanged store are the oracles.
/// Attempt-backed reports cannot expose the absent association or authority.
#[test]
fn standalone_command_and_report_need_no_ticket_and_grant_no_landing_authority() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g68-scoped", move |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if !request.prompt().contains("Frozen scope:") {
            return act(
                request,
                vec![commit_file("feature.txt", "feature\n", "Feature")],
            );
        }
        match request.turn() {
            0 => Reply::Tools(vec![tool("yard_context", json!({}))]),
            1 => Reply::Tools(vec![bash("git rev-parse HEAD")]),
            2 => Reply::Tools(vec![publish_report()]),
            _ => Reply::Hold(held.clone(), Box::new(Reply::Text("done".into()))),
        }
    });
    let command_held = machine.root.join("command-held");
    let command_release = machine.root.join("command-release");
    let armed = machine.root.join("command-armed");
    for path in [&command_held, &command_release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(&armed, "").unwrap();
    // Delegate the real command unchanged, retaining its child and box until
    // the host has observed the actual effect. Preserve its exit status.
    machine.wrapper("pinfold", &format!(
        "if [ -e '{}' ] && [ \"$1 $2\" = 'box exec' ]; then \"$REAL\" \"$@\"; result=$?; echo held > '{}'; read line < '{}'; exit \"$result\"; fi",
        armed.display(), command_held.display(), command_release.display()));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(
            r#"
[gates.standalone]
command = "printf 'command-effect\n' > command-only.txt; git rev-parse HEAD"
runs_in = "box"
"#,
        ),
    );
    // Trusted fixture extension probes the same worker session, not a new bearer.
    let extension = r#"import { writeFileSync } from 'node:fs';
export default function () {
  const original = globalThis.fetch;
  globalThis.fetch = async function (url, options) {
    const message = JSON.parse(options?.body || '{}');
    if (message.method === 'tools/call' && message.params?.name === 'yard_context') {
      const results = [];
      for (const name of ['yard_execution_start', 'yard_publish_review']) {
        const response = await original(url, { ...options, body: JSON.stringify({
          jsonrpc: '2.0', id: 9876, method: 'tools/call',
          params: { name, arguments: REQUEST }
        }) });
        results.push(await response.json());
      }
      writeFileSync('/tmp/standalone-probe.json', JSON.stringify(results));
    }
    return original(url, options);
  };
}
"#
    .replace(
        "REQUEST",
        &args(
            &project,
            json!({"target": "main", "input": project.canonical_head(),
        "node": scoped_node(false), "timeout_seconds": 300}),
        )
        .to_string(),
    );
    project.write(".pi/extensions/probe/index.ts", &extension);
    project.git(&["add", "-A"]);
    project.git(&["commit", "-qm", "Trusted standalone probe"]);
    project.json(&["sync"]);
    let input = project.canonical_head();
    let tree = git(
        &project.canonical(),
        &["rev-parse", &format!("{input}^{{tree}}")],
    )
    .trim()
    .to_string();
    let config_blob = git(
        &project.canonical(),
        &["rev-parse", &format!("{input}:.yard/config.toml")],
    )
    .trim()
    .to_string();
    let mut operator = machine.operator();
    operator.initialize();
    let mut history = project.watch(0);
    let mut candidate = scoped_node(false);
    candidate["outputs"]["result"] = json!("candidate");
    let refused = operator.call(
        "yard_execution_start",
        args(
            &project,
            json!({"target": "main", "input": input, "node": candidate, "timeout_seconds": 300}),
        ),
    );
    assert_eq!(refused["error"]["code"], "refused");
    assert_eq!(refused["error"]["message"], "standalone_authority");
    let before = project.rows("SELECT * FROM execution");
    let audit = project.rows("SELECT * FROM audit");
    for (field, path) in [("skills", "README.md"), ("references", "missing.txt")] {
        let mut selected = scoped_node(true);
        selected["runner"][field] = json!([path]);
        let refused = operator.call(
            "yard_execution_start",
            args(
                &project,
                json!({"target": "main", "input": input, "node": selected, "timeout_seconds": 300}),
            ),
        );
        assert_eq!(refused["error"]["code"], "refused", "{refused}");
        assert_eq!(
            refused["error"]["data"]["reason"], "standalone_selected_inputs",
            "{refused}"
        );
        assert_eq!(
            refused["error"]["data"]["input"],
            format!("node.runner.{field}")
        );
        assert_eq!(refused["error"]["data"]["selected"], json!([path]));
    }
    assert_eq!(project.rows("SELECT * FROM execution"), before);
    assert_eq!(project.rows("SELECT * FROM audit"), audit);
    assert!(machine.model.requests().is_empty());
    let command = scoped_start(&mut operator, &project, &input, false);
    assert_eq!(
        history.said_before_end(&command_held, Some(command)),
        "held\n"
    );
    let command_box = project.rows(&format!(
        "SELECT handle FROM execution WHERE id = {command}"
    ))[0]["handle"]
        .as_str()
        .unwrap()
        .to_string();
    let actual = machine.pinfold(&[
        "box",
        "exec",
        &command_box,
        "--",
        "cat",
        "/workspace/command-only.txt",
    ]);
    assert!(actual.status.success(), "{actual:?}");
    assert_eq!(actual.stdout, b"command-effect\n");
    std::fs::remove_file(&armed).unwrap();
    std::fs::write(&command_release, "go\n").unwrap();
    history.find("command checked release", |e| {
        e["event"] == "execution.released" && e["execution"] == command
    });
    assert_eq!(
        project.rows(&format!(
            "SELECT outcome, exit_code, detail FROM execution WHERE id = {command}"
        )),
        vec![json!({"outcome": "complete", "exit_code": 0, "detail": format!("{input}\n")})]
    );
    let report = scoped_start(&mut operator, &project, &input, true);
    hold.wait_held();
    let held_row = project.rows(&format!(
        "SELECT handle, status, reserved, result, report FROM execution WHERE id = {report}"
    ))[0]
        .clone();
    assert_eq!(held_row["status"], "running");
    assert_eq!(held_row["reserved"], 1);
    assert_eq!(held_row["result"], Value::Null);
    assert!(!held_row["report"].is_null());
    let name = held_row["handle"].as_str().unwrap();
    let actual_input = machine.pinfold(&[
        "box",
        "exec",
        name,
        "--",
        "git",
        "-C",
        "/workspace",
        "rev-parse",
        "HEAD",
    ]);
    assert!(actual_input.status.success(), "{actual_input:?}");
    assert_eq!(
        String::from_utf8(actual_input.stdout).unwrap(),
        format!("{input}\n")
    );
    let probe = machine.pinfold(&[
        "box",
        "exec",
        name,
        "--",
        "cat",
        "/tmp/standalone-probe.json",
    ]);
    assert!(probe.status.success(), "{probe:?}");
    let probes: Value = serde_json::from_slice(&probe.stdout).unwrap();
    assert_eq!(probes[0]["result"]["isError"], true);
    assert!(
        probes[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("operator_only")
    );
    assert_eq!(probes[1]["result"]["isError"], true);
    let write = machine.pinfold(&[
        "box",
        "exec",
        name,
        "--",
        "sh",
        "-c",
        "printf forbidden > /workspace/report-write.txt",
    ]);
    assert!(!write.status.success(), "{write:?}");
    assert!(
        !project
            .path
            .join(format!(
                ".yard/local/effects/{report}/checkout/report-write.txt"
            ))
            .exists()
    );
    hold.release();
    history.find("report checked release", |e| {
        e["event"] == "execution.released" && e["execution"] == report
    });
    for (id, kind) in [(command, "command"), (report, "report")] {
        let row = call(
            &mut operator,
            &project,
            "yard_execution_show",
            json!({"execution": id}),
        );
        assert_eq!(row["attempt"], Value::Null);
        assert_eq!(row["scope"]["project"], json!(project.path));
        assert_eq!(row["scope"]["target"], "main");
        assert_eq!(
            row["scope"]["input"],
            json!({"commit": input, "tree": tree})
        );
        assert_eq!(
            row["scope"]["policy"]["definitions"]["config_blob"],
            config_blob
        );
        let mut node = scoped_node(kind == "report");
        node["inputs"] = json!({});
        node["requires"] = json!([]);
        node["routes"] = json!({});
        assert_eq!(row["scope"]["policy"]["node"], node);
        assert_eq!(row["scope"]["timeout_seconds"], 300);
        assert_eq!(row["result"]["scope"], row["scope"]);
        assert_eq!(row["result"]["judgment"], Value::Null);
        assert_eq!(
            row["result"]["outputs"]["result"]["kind"],
            if kind == "report" {
                "report"
            } else {
                "verdict"
            }
        );
        assert_eq!(row["outcome"], "complete");
        if kind == "report" {
            assert_eq!(row["usage"]["input"], 4 * PROMPT_TOKENS);
            assert_eq!(row["usage"]["output"], 4 * COMPLETION_TOKENS);
            assert_eq!(
                row["result"]["outputs"]["result"]["value"]["report"],
                json!({"summary": "Fixture audit", "findings": []})
            );
        } else {
            assert_eq!(row["usage"]["input"], Value::Null);
        }
    }
    assert_eq!(project.rows("SELECT (SELECT count(*) FROM ticket) AS tickets, (SELECT count(*) FROM attempt) AS attempts, (SELECT count(*) FROM execution) AS executions, (SELECT count(*) FROM \"check\") AS checks, (SELECT count(*) FROM approval) AS approvals"),
        vec![json!({"tickets": 0, "attempts": 0, "executions": 2, "checks": 0, "approvals": 0})]);
    assert_eq!(project.canonical_head(), input);
}

/// Cancel while a real implementer is held, then let it commit and land.
/// The base extension probes an operator tool with the harness's actual
/// worker session. Stale mutations cross stdio after a successful edit.
/// Sabotage: forward worker tools to operator dispatch; the park succeeds.
/// Stop the attempt on wait cancellation or stdin EOF; the held work cannot
/// reach approval. Drop revision/head from the stdio arguments or rewrite
/// daemon errors; stale identities or the externally observed rows differ.
/// Skip strict stdio schema validation; the unknown-field edit writes.
#[test]
fn operator_wait_cancellation_keeps_work_and_worker_grants_separate() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g7mcp", move |request| match request.turn() {
        0 => Reply::Hold(
            held.clone(),
            Box::new(Reply::Tools(vec![tool("yard_context", json!({}))])),
        ),
        1 => Reply::Tools(vec![commit_file("feature.txt", "feature\n", "Add feature")]),
        _ => Reply::Text("done".into()),
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    // This trusted fixture extension uses the worker's real HTTP session.
    // It changes no Yard code and routes no public model call.
    let extension = r#"import { writeFileSync } from 'node:fs';
export default function () {
  const original = globalThis.fetch;
  globalThis.fetch = async function (url, options) {
    const message = JSON.parse(options?.body || '{}');
    if (message.method === 'tools/call' && message.params?.name === 'yard_context') {
      const probe = await original(url, { ...options, body: JSON.stringify({
        jsonrpc: '2.0', id: 9876, method: 'tools/call', params: {
          name: 'yard_ticket_park', arguments: { project: PROJECT, ticket: 'Y-1' }
        }
      }) });
      writeFileSync('/workspace/operator-probe.json', await probe.text());
    }
    return original(url, options);
  };
}
"#
    .replace("PROJECT", &json!(project.path).to_string());
    project.write(".pi/extensions/probe/index.ts", &extension);
    project.git(&["add", "-A"]);
    project.git(&["commit", "-qm", "Install trusted fixture probe"]);
    project.json(&["sync"]);
    let mut operator = machine.operator();
    operator.initialize();
    call(
        &mut operator,
        &project,
        "yard_ticket_new",
        json!({"title": "Feature", "parked": true}),
    );
    call(
        &mut operator,
        &project,
        "yard_ticket_edit",
        json!({"ticket": "Y-1", "revision": 1, "body": "commit feature"}),
    );
    let before = project.rows("SELECT revision, body FROM ticket WHERE id = 1");
    let stale = operator.call(
        "yard_ticket_edit",
        args(
            &project,
            json!({"ticket": "Y-1", "revision": 1, "body": "stale"}),
        ),
    );
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(stale["error"]["data"], json!({"expected": 1, "current": 2}));
    assert_eq!(
        before,
        vec![json!({"revision": 2, "body": "commit feature"})]
    );
    assert_eq!(
        project.rows("SELECT revision, body FROM ticket WHERE id = 1"),
        before
    );
    // G6: schema-invalid stdio input writes nothing; its valid control above did.
    let invalid = operator.call(
        "yard_ticket_edit",
        args(
            &project,
            json!({"ticket": "Y-1", "revision": 2, "body": "invalid", "unknown": true}),
        ),
    );
    assert_eq!(invalid["error"]["code"], "invalid");
    assert_eq!(
        project.rows("SELECT revision, body FROM ticket WHERE id = 1"),
        before
    );

    let mut history = project.watch(0);
    let old = project.canonical_head();
    call(
        &mut operator,
        &project,
        "yard_ticket_unpark",
        json!({"ticket": "Y-1"}),
    );
    hold.wait_held();
    let waiting = operator.send(
        "tools/call",
        json!({"name": "yard_attention", "arguments": args(&project, json!({}))}),
    );
    let ping = operator.send("ping", json!({}));
    operator.receive(ping);
    operator.notify("notifications/cancelled", json!({"requestId": waiting}));
    assert_eq!(operator.receive(waiting)["error"]["code"], -32800);
    // The second client closes stdin normally while the same real box is held.
    let mut disconnected = machine.operator();
    disconnected.initialize();
    disconnected.send(
        "tools/call",
        json!({"name": "yard_attention", "arguments": args(&project, json!({}))}),
    );
    let ping = disconnected.send("ping", json!({}));
    disconnected.receive(ping);
    assert!(disconnected.close_input().success());
    hold.release();
    let approval = history.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    let probe: Value = serde_json::from_str(
        &std::fs::read_to_string(
            project
                .path
                .join(".yard/local/attempts/1/clone/operator-probe.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(probe["result"]["isError"], true);
    assert_eq!(
        project.rows(
            "SELECT json_extract(data, '$.code') AS code FROM audit WHERE event = 'tool.refused'"
        ),
        vec![json!({"code": "refused"})]
    );
    assert_eq!(
        project.rows("SELECT parked FROM ticket WHERE id = 1"),
        vec![json!({"parked": 0})]
    );
    // Positive control: the trusted operator performs the same park.
    call(
        &mut operator,
        &project,
        "yard_ticket_park",
        json!({"ticket": "Y-1"}),
    );
    assert_eq!(
        project.rows("SELECT parked FROM ticket WHERE id = 1"),
        vec![json!({"parked": 1})]
    );
    let candidate = git(
        &project.path.join(".yard/local/attempts/1/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    assert_eq!(approval["data"]["payload"]["head"], candidate);
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{candidate}:feature.txt")]
        ),
        "feature\n"
    );
    let stale = operator.call(
        "yard_attempt_approve",
        args(
            &project,
            json!({"ticket": "Y-1", "head": old, "target": "main"}),
        ),
    );
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(
        stale["error"]["data"],
        json!({"expected": old, "current": candidate})
    );
    assert!(project.rows("SELECT id FROM approval").is_empty());
    call(
        &mut operator,
        &project,
        "yard_attempt_approve",
        json!({"ticket": "Y-1", "head": candidate, "target": "main"}),
    );
    history.event("landing.recorded", &[]);
    assert_eq!(project.canonical_head(), candidate);
    assert_eq!(
        project.rows("SELECT state FROM ticket WHERE id = 1"),
        vec![json!({"state": "done"})]
    );
}
