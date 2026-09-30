//! G6: Inputs are validated at the boundary.

use e2e::*;
use serde_json::json;

fn last_seq(project: &Project) -> i64 {
    project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap()
}

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
/// On the same machine, in a second project: an unknown workflow name is
/// refused by name, nothing written, on filing and on an edit. Control: a
/// configured workflow is accepted by both. Removing the named workflow
/// refuses until that ticket is abandoned. An unknown TOML key refuses until
/// corrected; each refusal writes nothing.
///
/// Sabotage: make `mcp::strict` accept unknown keys; the first proposal is
/// raised and there are two. Make `admit::ticket_new` skip its workflow
/// lookup; the filing is accepted and a ticket exists. Make
/// `admit::ticket_edit` skip its workflow lookup; the edit names a workflow
/// that does not exist.
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
    assert!(answers[0].contains("colour"), "{}", answers[0]);
    let proposals = project.rows("SELECT payload FROM attention WHERE kind = 'proposal'");
    assert_eq!(proposals.len(), 1);
    assert!(!proposals[0]["payload"].as_str().unwrap().contains("colour"));

    // No worker runs in this project: its one ticket stays parked.
    let project = Project::new(&machine, "q", &claude_config(""));
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

    // The accepted workflow can be edited back, but cannot disappear while
    // this parked ticket names it. Sabotage: skip parked tickets in sync.
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        "2",
        "--workflow",
        "plan",
    ]);
    let canonical = project.canonical_head();
    let seq = last_seq(&project);

    let without = claude_config("").replace(
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

    project.json(&["ticket", "abandon", "Y-1", "--reason", "Not needed"]);
    assert_eq!(project.json(&["sync"])["sync"], "imported");

    // A misspelt optional gate key must not import using its default.
    // Sabotage: remove Gate's deny_unknown_fields.
    let canonical = project.canonical_head();
    let seq = last_seq(&project);

    project.write(
        ".yard/config.toml",
        &claude_config("[gates.check]\ncommand = \"true\"\ntimout_minutes = 5\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Misspelt gate"]);
    let refused = project.refused(&["sync"]);
    assert_eq!(refused["code"], "invalid", "{refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("timout_minutes"),
        "{refused}"
    );
    assert_eq!(project.canonical_head(), canonical);
    assert_eq!(last_seq(&project), seq);
    assert!(git(&project.canonical(), &["for-each-ref", "refs/yard"]).is_empty());

    project.write(
        ".yard/config.toml",
        &claude_config("[gates.check]\ncommand = \"true\"\ntimeout_minutes = 5\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Fix the gate"]);
    assert_eq!(project.json(&["sync"])["sync"], "imported");
}
