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
    assert_eq!(pi_usage(&project, 2), second);
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

fn configuration() -> String {
    codex_config("[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n")
        .replace("review = [\"correctness\"]", "review = \"none\"")
}

fn usage_rows(project: &Project) -> Vec<Value> {
    project.rows("SELECT id, kind, tokens_in, tokens_out, cost FROM execution ORDER BY id")
}

/// Two fresh, completed real Codex runs use nonzero and explicit zero
/// Responses totals. Expected totals come from host requests and real harness
/// frames, not Yard's reader. MCP must expose the same persisted raw record.
/// Sabotage: keep Codex cost=0, coalesce nulls, or discard numeric zero.
/// Existing G7 Pi usage cannot catch Codex's unreported monetary cost.
#[test]
fn codex_completed_usage_preserves_unknown_cost_and_explicit_zero_tokens() {
    for (name, input, output) in [("g7-usage", 1000, 100), ("g7-zero", 0, 0)] {
        let mut machine = Machine::new(name, move |request| Reply::ResponsesUsage {
            input,
            output,
            reply: Box::new(act(
                request,
                vec![codex_commit_file("feature.txt", "feature\n", "Feature")],
            )),
        });
        let account = "acct-g7-usage";
        machine.write_codex_env(&codex_jwt(account, 3600), account);
        machine.start();
        let project = Project::new(&machine, "p", &configuration());
        let mut history = project.watch(0);
        project.json(&["ticket", "new", "--title", "Usage"]);
        assert_eq!(history.attention()["data"]["kind"], "approval");
        let rows = usage_rows(&project);
        let worker = rows
            .iter()
            .find(|row| row["kind"] == "implementation")
            .unwrap();
        let id = worker["id"].as_i64().unwrap();
        let frames = std::fs::read_to_string(
            project
                .path
                .join(format!(".yard/local/attempts/1/transcripts/{id}.jsonl")),
        )
        .unwrap();
        let terminal = frames
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|frame| frame["type"] == "turn.completed")
            .unwrap();
        let turns = machine.model.requests().len() as u64;
        assert!(turns > 1, "the real harness must complete its tool turn");
        assert_eq!(terminal["usage"]["input_tokens"], input * turns);
        assert_eq!(terminal["usage"]["output_tokens"], output * turns);
        assert_eq!(worker["tokens_in"], terminal["usage"]["input_tokens"]);
        assert_eq!(worker["tokens_out"], terminal["usage"]["output_tokens"]);
        assert_eq!(worker["cost"], Value::Null);

        let mut operator = machine.operator();
        operator.initialize();
        let answer = operator.call(
            "yard_attempt_show",
            json!({"project": project.path, "ticket": "Y-1"}),
        );
        assert!(answer.get("error").is_none(), "{answer}");
        let execution = answer["result"]["executions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["execution"] == id)
            .unwrap();
        let fields = json!({
            "input": {"completeness": "complete", "provenance": "harness"},
            "output": {"completeness": "complete", "provenance": "harness"},
            "cost": {"completeness": "unknown", "provenance": "unreported"}
        });
        assert_eq!(
            execution["usage"],
            json!({
                "input": input * turns, "output": output * turns, "cost": null, "fields": fields
            })
        );
        let stored = project.rows(&format!(
            "SELECT usage_fields FROM execution WHERE id = {id}"
        ));
        assert_eq!(
            serde_json::from_str::<Value>(stored[0]["usage_fields"].as_str().unwrap()).unwrap(),
            fields
        );
    }
}

/// The real old daemon writes a completed Codex zero-cost row and nullable
/// gate/prepare rows. Reopen twice to exercise migration and durable reads.
/// Expected values are the old store, not new Yard output. Old unsupported
/// zeros remain legacy observations, never complete measured bills.
/// Sabotage: coalesce missing usage or backfill harness/billed provenance.
/// Fresh-run assertions cannot catch the migration's historical classification.
#[test]
fn legacy_nullable_usage_survives_migration_and_reopen() {
    let mut machine = Machine::new("g7-legacy-usage", |request| {
        act(
            request,
            vec![codex_commit_file("feature.txt", "feature\n", "Feature")],
        )
    });
    let account = "acct-g7-legacy";
    machine.write_codex_env(&codex_jwt(account, 3600), account);
    machine.use_binary(Some(super::g15_plans_and_proposals::pre_target_binary()));
    machine.start();
    let project = Project::new(&machine, "p", &configuration());
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Legacy usage"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    let old = usage_rows(&project);
    assert!(old.iter().any(|row| row["kind"] == "gate"
        && row["tokens_in"].is_null()
        && row["tokens_out"].is_null()
        && row["cost"].is_null()));
    assert!(
        old.iter()
            .any(|row| row["kind"] == "implementation" && row["cost"] == 0.0)
    );
    machine.kill();
    machine.use_binary(None);
    for _ in 0..2 {
        machine.start();
        let shown = project.json(&["attempt", "show", "Y-1"]);
        assert_eq!(usage_rows(&project), old);
        for row in &old {
            let execution = shown["executions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|execution| execution["execution"] == row["id"])
                .unwrap();
            let field = json!({"completeness": "unknown", "provenance": "legacy"});
            assert_eq!(
                execution["usage"],
                json!({
                    "input": row["tokens_in"], "output": row["tokens_out"], "cost": row["cost"],
                    "fields": {"input": field, "output": field, "cost": field}
                })
            );
        }
        machine.kill();
    }
}

/// G7: real Codex fails and is interrupted on resumed turns after a tool
/// response with cache/reasoning subsets. The fixture holds the next request
/// until the real box ends; reopening then completes another resumed execution.
/// Sabotage: add cached/reasoning subsets, copy cumulative predecessor tokens,
/// discard nonterminal counters, or label interrupted usage complete.
/// Fresh Y-66 controls do not exercise session baselines or lost terminal frames.
#[test]
fn codex_failed_and_interrupted_resume_usage_remain_execution_local() {
    let hold = Latch::new();
    let held = hold.clone();
    let openings = AtomicUsize::new(0);
    let mut machine = Machine::new("g7-codex-local", move |request| {
        let round = if request.opens() {
            openings.fetch_add(1, Ordering::SeqCst)
        } else {
            openings.load(Ordering::SeqCst) - 1
        };
        match round {
            1 | 2 if request.opens() => Reply::Tools(vec![codex_shell("echo observed")]),
            1 => Reply::ResponsesError,
            2 => Reply::Hold(held.clone(), Box::new(Reply::Text("never".into()))),
            _ => act(
                request,
                vec![codex_commit_file(
                    if round == 0 {
                        "feature.txt"
                    } else {
                        "resumed.txt"
                    },
                    "feature\n",
                    "Feature",
                )],
            ),
        }
    });
    let account = "acct-g7-local";
    machine.write_codex_env(&codex_jwt(account, 3600), account);
    machine.start();
    let project = Project::new(&machine, "p", &configuration());
    // A different machine can retire a name between this build and box up.
    // Give this fixture distinct image content, not just a distinct name.
    let dockerfile = project.git(&["show", "HEAD:.yard/Dockerfile"]);
    project.write(
        ".yard/Dockerfile",
        &format!(
            "{dockerfile}\nLABEL dev.yard.e2e-machine=\"{}\"\n",
            machine.root.display()
        ),
    );
    project.git(&["add", ".yard/Dockerfile"]);
    project.git(&[
        "commit",
        "--quiet",
        "-m",
        "Isolate Codex usage fixture image",
    ]);
    project.json(&["sync"]);
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Local usage"]);
    let approval = history.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    let first = worker_usage(&project, 0);
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
        "Y-1",
        "--head",
        approval["data"]["payload"]["head"].as_str().unwrap(),
        "--text",
        "Observe failed usage",
    ]);
    let failed = history.attention();
    assert_eq!(
        (&failed["data"]["kind"], &failed["data"]["reason"]),
        (&json!("stopped"), &json!("failed"))
    );
    let second = worker_usage(&project, 1);
    assert_eq!(second["resumed"], first["execution"]);
    assert_eq!(second["outcome"], "failed");
    assert_codex_local(&second, PROMPT_TOKENS, COMPLETION_TOKENS, "partial");

    let mut history = project.watch(failed["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-1"]);
    hold.wait_held();
    let live = project
        .rows("SELECT handle FROM execution WHERE kind = 'implementation' AND status = 'running'");
    let down = machine.pinfold(&["box", "down", live[0]["handle"].as_str().unwrap()]);
    assert!(
        down.status.success(),
        "{}",
        String::from_utf8_lossy(&down.stderr)
    );
    let interrupted = history.attention();
    assert_eq!(
        (&interrupted["data"]["kind"], &interrupted["data"]["reason"]),
        (&json!("stopped"), &json!("failed"))
    );
    assert!(hold.is_held());
    let third = worker_usage(&project, 2);
    assert_eq!(third["resumed"], second["execution"]);
    assert_eq!(third["outcome"], "failed");
    assert_codex_local(&third, PROMPT_TOKENS, COMPLETION_TOKENS, "partial");

    drop(history);
    machine.kill();
    machine.start();
    assert_eq!(worker_usage(&project, 2), third);
    let mut history = project.watch(interrupted["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-1"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    let fourth = worker_usage(&project, 3);
    assert_eq!(fourth["resumed"], third["execution"]);
    // The resumed successful control has one tool response and one text
    // response. These literal totals come from the scripted host model.
    assert_codex_local(
        &fourth,
        2 * PROMPT_TOKENS,
        2 * COMPLETION_TOKENS,
        "complete",
    );
    hold.release();
}

fn worker_usage(project: &Project, index: usize) -> Value {
    project.json(&["attempt", "show", "Y-1"])["executions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["kind"] == "implementation")
        .nth(index)
        .unwrap()
        .clone()
}

fn assert_codex_local(execution: &Value, input: u64, output: u64, completeness: &str) {
    let reported = json!({"completeness": completeness, "provenance": "harness"});
    let unknown = json!({"completeness": "unknown", "provenance": "unreported"});
    assert_eq!(
        execution["usage"],
        json!({
            "input": input, "output": output, "cost": null,
            "fields": {"input": reported, "output": reported, "cost": unknown}
        })
    );
}
