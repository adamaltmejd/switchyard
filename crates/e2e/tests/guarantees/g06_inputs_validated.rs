//! G6: Inputs are validated at the boundary.

use e2e::*;
use serde_json::json;

/// A Claude worker's call through its own MCP session. Claude passes the
/// model's arguments to the server unchecked, so no client schema stops a
/// malformed one first.
fn propose_directly(arguments: serde_json::Value) -> ToolCall {
    tool("mcp__yard__yard_propose", arguments)
}

/// A malformed tool payload is refused by name, nothing written: a
/// proposal carrying one unknown key beside valid ones, sent through a
/// Claude worker's session so no client schema stops it first. Control: the
/// same proposal without it, in the same turn, is raised.
///
/// Sabotage: make `mcp::strict` accept unknown keys; the first proposal is
/// raised and there are two.
#[test]
fn inputs_are_refused_by_name_and_write_nothing() {
    let machine = Machine::new("g6", |request| {
        act(
            request,
            vec![
                propose_directly(json!({ "kind": "ticket", "title": "Split", "colour": "red" })),
                propose_directly(json!({ "kind": "ticket", "title": "Split" })),
            ],
        )
    });
    machine.write_claude_env();
    machine.start();
    let project = Project::new(&machine, "p", &claude_config(""));
    let mut watch = project.watch(0);
    project.ok("ticket.new", json!({"title": "Too large"}));
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
    assert!(answers[0].contains("colour"), "{}", answers[0]);
    let proposals = project.rows("SELECT payload FROM attention WHERE kind = 'proposal'");
    assert_eq!(proposals.len(), 1);
    assert!(!proposals[0]["payload"].as_str().unwrap().contains("colour"));
}
