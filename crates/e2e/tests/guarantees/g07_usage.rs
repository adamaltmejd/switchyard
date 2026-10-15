//! G7: optional usage is measured data with per-execution provenance.
use e2e::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Real Pi: no usage, then a resumed turn interrupted after one reported
/// cache-inclusive message, then an explicit-zero resumed completion. The host
/// wrapper removes fields from real frames; Pi otherwise supplies catalog cost.
/// Sabotage: drop cacheRead or cacheWrite, unwrap_or(0), save only terminal
/// usage, or count resumed history.
/// Fresh Codex and the ordinary Pi landing cannot expose these mechanisms.
#[test]
fn pi_unknowns_and_interrupted_resume_usage_remain_execution_local() {
    let hold = Latch::new();
    let held = hold.clone();
    let openings = AtomicUsize::new(0);
    let machine = Machine::new("g7-pi-usage", move |request| {
        let round = if request.opens() {
            openings.fetch_add(1, Ordering::SeqCst)
        } else {
            openings.load(Ordering::SeqCst) - 1
        };
        let reply = if round == 1 {
            if request.opens() {
                Reply::Tools(vec![bash("echo observed")])
            } else {
                Reply::Hold(held.clone(), Box::new(Reply::Text("never".into())))
            }
        } else {
            act(
                request,
                vec![commit_file(
                    if round == 0 {
                        "feature.txt"
                    } else {
                        "zero.txt"
                    },
                    "feature\n",
                    "Feature",
                )],
            )
        };
        Reply::ChatUsage {
            input: if round == 2 { 0 } else { PROMPT_TOKENS },
            output: if round == 2 { 0 } else { COMPLETION_TOKENS },
            // Pi reports input 700, cacheRead 200 and cacheWrite 100.
            cache: (round == 1).then_some((200, 100)),
            reply: Box::new(reply),
        }
    });
    // Stream the real child unchanged except for absent measurement fields.
    // Keep stderr, exit status, registration and terminal outcome intact.
    machine.wrapper(
        "pinfold",
        r#"if [ "$1 $2" = 'box exec' ]; then
  case "$3" in
    *-1|*-2)
      out=$(mktemp -d)
      mkfifo "$out/frames"
      "$REAL" "$@" >"$out/frames" &
      child=$!
      while IFS= read -r line; do
        case "$line" in
          *'"type":"message_end"'*'"role":"assistant"'*)
            line=$(printf '%s\n' "$line" | sed -E 's/,"cost":\{[^}]*\}//g')
            case "$3" in
              *-1) line=$(printf '%s\n' "$line" | sed -E 's/,"usage":\{[^}]*\}//g') ;;
            esac ;;
        esac
        printf '%s\n' "$line"
      done <"$out/frames"
      wait "$child"
      code=$?
      rm -rf "$out"
      exit "$code" ;;
  esac
fi"#,
    );
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("").replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Usage"]);
    let approval = history.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    let first = pi_usage(&project, 1);
    let unknown = json!({"completeness": "unknown", "provenance": "unreported"});
    assert_eq!(
        first["usage"],
        json!({
            "input": null, "output": null, "cost": null,
            "fields": {"input": unknown, "output": unknown, "cost": unknown}
        })
    );
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
        "Y-1",
        "--head",
        approval["data"]["payload"]["head"].as_str().unwrap(),
        "--text",
        "Observe usage",
    ]);
    hold.wait_held();
    // End the real box while the next model request is held. EOF drains
    // already reported frames; operator stop could race their pipe delivery.
    let handle = project.rows("SELECT handle FROM execution WHERE id = 2")[0]["handle"]
        .as_str()
        .unwrap()
        .to_string();
    let down = machine.pinfold(&["box", "down", &handle]);
    assert!(
        down.status.success(),
        "{}",
        String::from_utf8_lossy(&down.stderr)
    );
    let stopped = history.attention();
    assert_eq!(
        (&stopped["data"]["kind"], &stopped["data"]["reason"]),
        (&json!("stopped"), &json!("failed"))
    );
    assert!(hold.is_held());
    let partial = json!({"completeness": "partial", "provenance": "harness"});
    let second = pi_usage(&project, 2);
    assert_eq!(second["outcome"], "failed");
    assert_eq!(second["resumed"], first["execution"]);
    assert_eq!(
        second["usage"],
        json!({
            "input": PROMPT_TOKENS, "output": COMPLETION_TOKENS, "cost": null,
            "fields": {"input": partial, "output": partial, "cost": unknown}
        })
    );
    // The interrupted record survives reopening; the fixture is still held.
    drop(history);
    machine.kill();
    machine.start();
    // G7 preserves usage/resume identity; checked cleanup may finish on restart.
    let reopened = pi_usage(&project, 2);
    for field in [
        "execution",
        "harness",
        "harness_version",
        "model",
        "session_id",
        "resumed",
        "outcome",
        "usage",
    ] {
        assert_eq!(reopened[field], second[field], "{field}");
    }
    let mut history = project.watch(stopped["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-1"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    let third = pi_usage(&project, 3);
    assert_eq!(third["resumed"], second["execution"]);
    let complete = json!({"completeness": "complete", "provenance": "harness"});
    assert_eq!(
        third["usage"],
        json!({
            "input": 0, "output": 0, "cost": 0.0,
            "fields": {"input": complete, "output": complete, "cost": complete}
        })
    );
    hold.release();
}

fn pi_usage(project: &Project, id: i64) -> Value {
    let shown = project.json(&["attempt", "show", "Y-1"]);
    shown["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|execution| execution["execution"] == id)
        .unwrap()
        .clone()
}
