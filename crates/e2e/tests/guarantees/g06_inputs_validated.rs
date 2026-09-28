//! G6: Inputs are validated at the boundary.

use e2e::*;
use serde_json::json;

fn last_seq(project: &Project) -> i64 {
    project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap()
}

/// A worker's call straight to the MCP route, past its harness's schema
/// check.
fn propose_directly(arguments: serde_json::Value) -> ToolCall {
    let body = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "yard_propose", "arguments": arguments },
    });
    bash(&format!(
        "curl -s -H \"Authorization: Bearer $YARD_MCP_BEARER\" -H 'content-type: application/json' \
         \"$YARD_MCP_ENDPOINT\" -d '{body}'"
    ))
}

/// A malformed tool payload is refused by name, nothing written: a
/// proposal carrying one unknown key beside valid ones, sent straight to
/// the route so no client schema stops it first. Control: the same proposal
/// without it is raised.
///
/// Sabotage: make `mcp::strict` accept unknown keys; the first proposal is
/// raised and there are two.
#[test]
fn a_malformed_tool_payload_is_refused_by_name() {
    let machine = Machine::new("g6-tool", |request| match request.tool_results().len() {
        _ if request.opens() => Reply::Tools(vec![propose_directly(
            json!({ "kind": "ticket", "title": "Split", "colour": "red" }),
        )]),
        1 => Reply::Tools(vec![propose_directly(
            json!({ "kind": "ticket", "title": "Split" }),
        )]),
        _ => Reply::Text("done".into()),
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Too large"]);
    watch.until("the worker stopped", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "stopped"
    });

    let answers: Vec<String> = machine
        .model
        .requests()
        .last()
        .unwrap()
        .tool_results()
        .into_iter()
        .map(|(_, text)| text)
        .collect();
    assert_eq!(answers.len(), 2, "{answers:?}");
    let refused: serde_json::Value = serde_json::from_str(answers[0].trim()).unwrap();
    assert_eq!(refused["result"]["isError"], true, "{refused}");
    assert!(answers[0].contains("colour"), "{refused}");
    let refusals = project.rows("SELECT data FROM audit WHERE event = 'tool.refused'");
    assert_eq!(refusals.len(), 1);
    let data: serde_json::Value =
        serde_json::from_str(refusals[0]["data"].as_str().unwrap()).unwrap();
    assert_eq!(data, json!({ "tool": "yard_propose", "code": "invalid" }));
    let proposals = project.rows("SELECT payload FROM attention WHERE kind = 'proposal'");
    assert_eq!(proposals.len(), 1);
    assert!(!proposals[0]["payload"].as_str().unwrap().contains("colour"));
}

/// A TOML with an unknown key is refused by name, nothing written. The key
/// is inside a gate, a table read by name. Control: the corrected
/// configuration syncs.
///
/// Sabotage: drop `deny_unknown_fields` from the gate's table; the sync
/// imports the misspelt gate.
#[test]
fn a_config_with_an_unknown_key_is_refused_by_name() {
    let machine = Machine::new("g6-toml", |_| Reply::Text("unused".into()));
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let canonical = project.canonical_head();
    let seq = last_seq(&project);

    project.write(
        ".yard/config.toml",
        &config("[gates.check]\ncomand = \"true\"\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Misspelt gate"]);
    let refused = project.refused(&["sync"]);
    assert_eq!(refused["code"], "invalid", "{refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("comand"),
        "{refused}"
    );
    assert_eq!(project.canonical_head(), canonical);
    assert_eq!(last_seq(&project), seq);
    assert!(git(&project.canonical(), &["for-each-ref", "refs/yard"]).is_empty());

    project.write(
        ".yard/config.toml",
        &config("[gates.check]\ncommand = \"true\"\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Fix the gate"]);
    assert_eq!(project.json(&["sync"])["sync"], "imported");
}

/// An unknown workflow name is refused by name, nothing written, on filing
/// and on an edit. Control: a configured workflow is accepted by both.
///
/// Sabotage: make `admit::ticket_edit` skip its workflow lookup; the edit
/// names a workflow that does not exist.
#[test]
fn an_unknown_workflow_name_is_refused_by_name() {
    let machine = Machine::new("g6-workflow", |_| Reply::Text("unused".into()));
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let seq = last_seq(&project);

    let refused = project.refused(&[
        "ticket",
        "new",
        "--title",
        "Plan",
        "--parked",
        "--workflow",
        "nope",
    ]);
    assert_eq!(refused["code"], "invalid", "{refused}");
    assert!(refused["message"].as_str().unwrap().contains("nope"));
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM ticket").is_empty());

    project.json(&[
        "ticket",
        "new",
        "--title",
        "Plan",
        "--parked",
        "--workflow",
        "plan",
    ]);
    let seq = last_seq(&project);
    let refused = project.refused(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        "1",
        "--workflow",
        "nope",
    ]);
    assert_eq!(refused["code"], "invalid", "{refused}");
    assert!(refused["message"].as_str().unwrap().contains("nope"));
    assert_eq!(last_seq(&project), seq);
    let ticket = project.json(&["ticket", "show", "Y-1"]);
    assert_eq!(
        (ticket["workflow"].as_str(), ticket["revision"].as_i64()),
        (Some("plan"), Some(1))
    );

    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        "1",
        "--workflow",
        "default",
    ]);
    assert_eq!(
        project.json(&["ticket", "show", "Y-1"])["workflow"],
        "default"
    );
}

/// A sync that removes a workflow an open ticket names is refused, naming
/// the ticket, nothing written. The ticket is parked, so nothing runs on
/// it. Control: once the ticket is abandoned the same sync imports.
///
/// Sabotage: make `admit::sync` skip parked tickets in its check; the sync
/// imports and the ticket names a workflow that is gone.
#[test]
fn a_sync_removing_a_named_workflow_is_refused() {
    let machine = Machine::new("g6-removal", |_| Reply::Text("unused".into()));
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Plan",
        "--parked",
        "--workflow",
        "plan",
    ]);
    let canonical = project.canonical_head();
    let seq = last_seq(&project);

    let without = config("").replace(
        "[workflows.plan]\naccess = \"read-only\"\nreview = \"none\"\n",
        "",
    );
    assert!(!without.contains("workflows.plan"));
    project.write(".yard/config.toml", &without);
    project.git(&["commit", "--quiet", "-am", "Drop plan"]);
    let refused = project.refused(&["sync"]);
    assert_eq!(refused["code"], "refused", "{refused}");
    assert_eq!(refused["data"]["tickets"], json!(["Y-1"]));
    assert_eq!(project.canonical_head(), canonical);
    assert_eq!(last_seq(&project), seq);
    assert!(git(&project.canonical(), &["for-each-ref", "refs/yard"]).is_empty());

    project.json(&["ticket", "abandon", "Y-1", "--reason", "Not needed"]);
    assert_eq!(project.json(&["sync"])["sync"], "imported");
}
