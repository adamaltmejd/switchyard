//! G7: Claude estimates remain optional and local to each execution.
use e2e::*;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Real Claude: missing cost, a failed resumed result, an interrupted
/// resumed tool turn, then an explicit-zero resumed completion. Host counters
/// and pinned frames establish invocation totals, including cache reads/writes.
/// Sabotage: default absent cost to zero, add result totals to observations,
/// count resumed history, or save usage only at result. Pi/Codex cannot catch
/// Claude's result replacement or its API/login accounting differences.
#[test]
fn claude_api_usage_is_optional_and_execution_local() {
    scenario(false);
}

/// G7 through subscription login rather than API-key accounting.
/// Sabotage: fabricate absent cost or count resumed session totals again.
#[test]
fn claude_login_usage_is_optional_and_execution_local() {
    scenario(true);
}

fn scenario(login: bool) {
    let phase = Arc::new(AtomicUsize::new(0));
    let replies = Arc::new(std::array::from_fn::<_, 4, _>(|_| AtomicUsize::new(0)));
    let hold = Latch::new();
    let (round, counts, held) = (phase.clone(), replies.clone(), hold.clone());
    let machine = Machine::new(
        if login { "g7-cl-login" } else { "g7-cl-api" },
        move |request| {
            let n = round.load(Ordering::SeqCst);
            let marker = format!("y76-round-{n}");
            let done = request
                .tool_results()
                .iter()
                .any(|(_, text)| text.contains(&marker));
            if done && n == 1 {
                return Reply::AnthropicError;
            }
            if done && n == 2 {
                return Reply::Hold(held.clone(), Box::new(Reply::Text("never".into())));
            }
            counts[n].fetch_add(1, Ordering::SeqCst);
            let reply = if done {
                Reply::Text("done".into())
            } else {
                Reply::Tools(vec![
                    claude_bash(&format!(
                        "printf 'feature\\n' > round-{n}.txt && git add round-{n}.txt && git commit -qm 'Round {n}' && echo {marker}"
                    )),
                    claude_bash("echo second-content-block"),
                ])
            };
            Reply::AnthropicUsage {
                input: PROMPT_TOKENS,
                output: COMPLETION_TOKENS,
                cache_read: 70,
                cache_write: 30,
                reply: Box::new(reply),
            }
        },
    );
    if login {
        machine.write_claude_env();
    } else {
        machine.write_operator_env(&format!(
            "ANTHROPIC_API_KEY={SECRET}\nYARD_ORIGIN_ANTHROPIC={}\n",
            machine.model.origin()
        ));
    }
    // Alter only measurement availability on real frames. Raw frames remain
    // host evidence; outcomes, sessions, registration and exit status survive.
    machine.wrapper("pinfold", &format!(r#"if [ "$1 $2" = 'box exec' ]; then
  out=$(mktemp -d)
  mkfifo "$out/frames"
  "$REAL" "$@" >"$out/frames" &
  child=$!
  while IFS= read -r line; do
    printf '%s\n' "$line" >> '{}/raw-'"$3"'.jsonl'
    case "$line" in
      *'"type":"result"'*)
        case "$3" in
          *-1) line=$(printf '%s\n' "$line" | sed -E 's/,"total_cost_usd":[0-9.eE+-]+//g; s/"total_cost_usd":[0-9.eE+-]+,//g') ;;
          *-4) line=$(printf '%s\n' "$line" | sed -E 's/"total_cost_usd":[0-9.eE+-]+/"total_cost_usd":0/g') ;;
        esac ;;
    esac
    printf '%s\n' "$line"
  done <"$out/frames"
  wait "$child"
  code=$?
  rm -rf "$out"
  exit "$code"
fi"#, machine.root.display()));
    machine.start();
    let config = claude_config("").replace("review = [\"correctness\"]", "review = \"none\"");
    let config = if login {
        config
    } else {
        config.replace("login = true", "provider = \"anthropic\"")
    };
    let project = Project::new(&machine, "p", &config);
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Claude usage"]);
    let approval = history.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    check(
        &project,
        1,
        replies[0].load(Ordering::SeqCst),
        "complete",
        false,
    );
    phase.store(1, Ordering::SeqCst);
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
    check(
        &project,
        2,
        replies[1].load(Ordering::SeqCst),
        "partial",
        true,
    );
    phase.store(2, Ordering::SeqCst);
    project.json(&["attempt", "start", "Y-1"]);
    hold.wait_held();
    let handle = project.rows("SELECT handle FROM execution WHERE id = 3")[0]["handle"]
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
    check(
        &project,
        3,
        replies[2].load(Ordering::SeqCst),
        "partial",
        false,
    );
    let before = execution(&project, 3);
    drop(history);
    machine.kill();
    machine.start();
    assert_eq!(execution(&project, 3), before);
    phase.store(3, Ordering::SeqCst);
    let mut history = project.watch(stopped["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-1"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    check(
        &project,
        4,
        replies[3].load(Ordering::SeqCst),
        "complete",
        true,
    );
    assert_eq!(execution(&project, 4)["usage"]["cost"], 0.0);
    // Compare real resumed estimates with the fresh harness estimate for the
    // same fixture counters. No Yard pricing table supplies an expected bill.
    let fresh = raw_cost(&machine, 1);
    assert!(fresh > 0.0);
    for (id, n) in [(2, 1), (4, 3)] {
        let expected = fresh * replies[n].load(Ordering::SeqCst) as f64
            / replies[0].load(Ordering::SeqCst) as f64;
        assert!((raw_cost(&machine, id) - expected).abs() < 1e-9);
    }
    hold.release();
}

fn raw_cost(machine: &Machine, id: i64) -> f64 {
    let path = std::fs::read_dir(&machine.root)
        .unwrap()
        .map(Result::unwrap)
        .find(|entry| {
            let name = entry.file_name();
            let name = name.to_str().unwrap();
            name.starts_with("raw-") && name.ends_with(&format!("-{id}.jsonl"))
        })
        .unwrap()
        .path();
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|frame| frame["type"] == "result")
        .unwrap()["total_cost_usd"]
        .as_f64()
        .unwrap()
}

fn execution(project: &Project, id: i64) -> Value {
    project.json(&["attempt", "show", "Y-1"])["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["execution"] == id)
        .unwrap()
        .clone()
}

fn check(project: &Project, id: i64, replies: usize, completeness: &str, cost_reported: bool) {
    assert!(replies > 0);
    let execution = execution(project, id);
    if id > 1 {
        assert_eq!(execution["resumed"], id - 1);
    }
    let frames = std::fs::read_to_string(
        project
            .path
            .join(format!(".yard/local/attempts/1/transcripts/{id}.jsonl")),
    )
    .unwrap();
    let frames: Vec<Value> = frames
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let result = frames.iter().find(|frame| frame["type"] == "result");
    if id == 3 {
        assert!(result.is_none());
    } else {
        let result = result.unwrap();
        assert_eq!(result["is_error"], id == 2);
        // Pinned result totals must equal this invocation's fixture counters,
        // even when the resumed session already contains earlier usage.
        assert_eq!(
            result["usage"]["input_tokens"],
            PROMPT_TOKENS * replies as u64
        );
        assert_eq!(
            result["usage"]["cache_read_input_tokens"],
            70 * replies as u64
        );
        assert_eq!(
            result["usage"]["cache_creation_input_tokens"],
            30 * replies as u64
        );
        assert_eq!(
            result["usage"]["output_tokens"],
            COMPLETION_TOKENS * replies as u64
        );
    }
    let reported = json!({"completeness": completeness, "provenance": "harness"});
    let unknown = json!({"completeness": "unknown", "provenance": "unreported"});
    let cost = if cost_reported {
        result.unwrap()["total_cost_usd"].clone()
    } else {
        Value::Null
    };
    if cost_reported {
        assert!(cost.is_number());
    }
    assert_eq!(
        execution["usage"],
        json!({
            "input": (PROMPT_TOKENS + 70 + 30) * replies as u64,
            "output": COMPLETION_TOKENS * replies as u64, "cost": cost,
            "fields": {"input": reported, "output": reported,
                "cost": if cost_reported { &reported } else { &unknown }}
        })
    );
}
