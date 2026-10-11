//! G7: Claude estimates remain optional and local to each execution.
use e2e::*;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Real Claude: missing cost, a failed resumed result, an interrupted
/// resumed tool turn, then a resumed completion without a cost baseline.
/// Fresh nonzero and zero reports control conservative resumed accounting.
/// Host counters
/// and pinned frames establish invocation totals, including cache reads/writes.
/// Sabotage: default absent cost to zero, add result totals to observations,
/// count resumed history, or save usage only at result. Pi/Codex cannot catch
/// Claude's result replacement or repeated content-block observations. API
/// and login use this same reader; G8 independently exercises both routes.
/// Ignoring or overwriting a nonzero output update also fails: the interrupted
/// oracle checks every raw output report for its single provider response.
/// Saving resumed session cost as local or subtracting across missing cost
/// and interruption fails the null-cost blocks; discarding fresh estimates
/// fails the nonzero control.
#[test]
fn claude_usage_is_optional_and_execution_local() {
    let phase = Arc::new(AtomicUsize::new(0));
    let replies = Arc::new(std::array::from_fn::<_, 6, _>(|_| AtomicUsize::new(0)));
    let hold = Latch::new();
    let (round, counts, held) = (phase.clone(), replies.clone(), hold.clone());
    let machine = Machine::new("g7-cl-api", move |request| {
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
    });
    machine.write_operator_env(&format!(
        "ANTHROPIC_API_KEY={SECRET}\nYARD_ORIGIN_ANTHROPIC={}\n",
        machine.model.origin()
    ));
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
          *-8) line=$(printf '%s\n' "$line" | sed -E 's/"total_cost_usd":[0-9.eE+-]+/"total_cost_usd":0/g') ;;
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
    let config = claude_config("")
        .replace("review = [\"correctness\"]", "review = \"none\"")
        .replace("login = true", "provider = \"anthropic\"");
    let project = Project::new(&machine, "p", &config);
    let mut history = project.watch(0);
    project.json(&["ticket", "new", "--title", "Claude usage"]);
    let approval = history.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    check(&project, 1, replies[0].load(Ordering::SeqCst), "complete");
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
    check(&project, 2, replies[1].load(Ordering::SeqCst), "partial");
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
    check(&project, 3, replies[2].load(Ordering::SeqCst), "partial");
    let before = execution(&project, "Y-1", 3);
    drop(history);
    machine.kill();
    machine.start();
    assert_eq!(execution(&project, "Y-1", 3), before);
    phase.store(3, Ordering::SeqCst);
    let mut history = project.watch(stopped["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-1"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    check(&project, 4, replies[3].load(Ordering::SeqCst), "complete");
    // Real resumed cost is session-cumulative even though result tokens are
    // local. It is evidence for withholding cost, not a local price oracle.
    let fresh = raw_cost(&machine, 1);
    assert!(fresh > 0.0);
    let failed_session = raw_cost(&machine, 2);
    assert!(failed_session > fresh);
    assert!(raw_cost(&machine, 4) > failed_session);
    hold.release();
    retire_fixture(&project, &mut history, "Y-1");

    // Fresh sessions supply execution-local cost evidence. These focused
    // controls distinguish withholding ambiguous totals from discarding all
    // estimates, while preserving the original missing-baseline resume chain.
    phase.store(4, Ordering::SeqCst);
    project.json(&["ticket", "new", "--title", "Fresh estimate"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    assert!(fresh_estimate(&project, "Y-2", 2, 6) > 0.0);
    retire_fixture(&project, &mut history, "Y-2");
    phase.store(5, Ordering::SeqCst);
    project.json(&["ticket", "new", "--title", "Explicit zero estimate"]);
    assert_eq!(history.attention()["data"]["kind"], "approval");
    assert_eq!(fresh_estimate(&project, "Y-3", 3, 8), 0.0);
    retire_fixture(&project, &mut history, "Y-3");
}

// Observe transcripts before retirement removes them. Park first to prevent
// readmission; await cleanup so its execution cannot race the next control's ID.
fn retire_fixture(project: &Project, history: &mut Watch, ticket: &str) {
    project.json(&["ticket", "park", ticket]);
    project.json(&["attempt", "abandon", ticket]);
    project.json(&["ticket", "abandon", ticket, "--reason", "Usage observed"]);
    history.event("execution.ended", &[("kind", "cleanup")]);
}

fn raw_frames(machine: &Machine, id: i64) -> String {
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
    std::fs::read_to_string(path).unwrap()
}

fn raw_cost(machine: &Machine, id: i64) -> f64 {
    raw_frames(machine, id)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|frame| frame["type"] == "result")
        .unwrap()["total_cost_usd"]
        .as_f64()
        .unwrap()
}

fn execution(project: &Project, ticket: &str, id: i64) -> Value {
    project.json(&["attempt", "show", ticket])["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["execution"] == id)
        .unwrap()
        .clone()
}

fn fresh_estimate(project: &Project, ticket: &str, attempt: i64, id: i64) -> f64 {
    let execution = execution(project, ticket, id);
    assert_eq!(execution["resumed"], Value::Null);
    let frames = std::fs::read_to_string(project.path.join(format!(
        ".yard/local/attempts/{attempt}/transcripts/{id}.jsonl"
    )))
    .unwrap();
    let result = frames
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|frame| frame["type"] == "result")
        .unwrap();
    let expected = result["total_cost_usd"]
        .as_f64()
        .expect("numeric fresh harness cost");
    let cost = execution["usage"]["cost"]
        .as_f64()
        .expect("numeric fresh execution cost");
    assert_eq!(cost, expected);
    assert_eq!(
        execution["usage"]["fields"]["cost"],
        json!({
            "completeness": "complete", "provenance": "harness"
        })
    );
    cost
}

// This fixture has one response before interruption. Inspect every raw token
// report, including updates outside assistant snapshots, so a missed or later
// overwritten nonzero report cannot be hidden by selecting the last snapshot.
fn output_reports(value: &Value, reports: &mut Vec<u64>) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if key == "output_tokens" {
                    reports.push(value.as_u64().expect("a raw output counter"));
                } else {
                    output_reports(value, reports);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                output_reports(value, reports);
            }
        }
        _ => {}
    }
}

fn check(project: &Project, id: i64, replies: usize, completeness: &str) {
    assert!(replies > 0);
    let execution = execution(project, "Y-1", id);
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
    let output = if id == 3 {
        assert!(result.is_none());
        assert_eq!(
            replies, 1,
            "the interrupted oracle covers one provider response"
        );
        let raw = raw_frames(project.machine, id);
        eprintln!("Y-76 interrupted execution {id}, raw Claude stream-json:\n{raw}");
        let mut reports = Vec::new();
        for line in raw.lines() {
            let frame: Value = serde_json::from_str(line).expect("a real Claude frame");
            output_reports(&frame, &mut reports);
        }
        // The provider's message_delta is not itself a harness observation.
        // Retain any larger raw report rather than inferring it from the reply.
        reports
            .into_iter()
            .max()
            .expect("Claude reported output before interruption")
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
        COMPLETION_TOKENS * replies as u64
    };
    let reported = json!({"completeness": completeness, "provenance": "harness"});
    let unknown = json!({"completeness": "unknown", "provenance": "unreported"});
    assert_eq!(
        execution["usage"],
        json!({
            "input": (PROMPT_TOKENS + 70 + 30) * replies as u64,
            "output": output, "cost": null,
            "fields": {"input": reported, "output": reported, "cost": unknown}
        })
    );
}
