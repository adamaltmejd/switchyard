//! G2/G8: a choice consumes publications and grants no implementation authority.
use e2e::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};

fn rows(project: &Project<'_>, sql: &str) -> Vec<Value> {
    project
        .rows(sql)
        .into_iter()
        .map(|mut row| {
            for key in ["inputs", "result", "settings", "report"] {
                if let Some(text) = row[key].as_str() {
                    row[key] = serde_json::from_str(text).unwrap();
                }
            }
            row
        })
        .collect()
}

fn approval(watch: &mut Watch) -> Value {
    watch.until("selected candidate approval", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    })["data"]["payload"]
        .clone()
}

/// G2/G8: five distinct model producers publish different candidates with
/// their own frozen reviews. B publishes before the last source, then one
/// replacement invalidates its old choice and earns fresh evidence and landing.
/// Git and fixture bytes supply heads/content; durable FKs bind findings.
/// Sabotage: choose newest, copy losing reviews, grant picker writer/approval
/// authority, omit choice identity or mutate historical evidence.
/// Admit reference reuse for a writer-graph review despite its typed output
/// consumers; the recorded eligibility assertion fails before a no-op retry
/// could loop on reference-only reviews.
#[test]
fn only_the_declared_choice_is_checked_and_landed() {
    let holds: Vec<_> = (0..5).map(|_| Latch::new()).collect();
    let writer_holds = holds.clone();
    let replacement = Latch::new();
    let replacement_hold = replacement.clone();
    let turns: Vec<_> = (0..5).map(|_| AtomicUsize::new(0)).collect();
    let machine = Machine::new("g8-select", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        if request.has_tool("yard_publish_selection") {
            assert!(!request.has_tool("yard_publish_review"));
            assert!(!request.has_tool("yard_propose"));
            assert!(!request.has_tool("yard_attempt_approve"));
            if request.opens() {
                return Reply::Tools(vec![tool(
                    "yard_publish_selection",
                    json!({"candidate": "foreign"}),
                )]);
            }
            if request
                .tool_results()
                .last()
                .is_some_and(|(_, text)| text.contains("selection_input"))
            {
                return Reply::Tools(vec![bash(
                    "test -d /yard/candidates/6331/source/.git && case \"$(cat /yard/candidates/6331/source/same.txt)\" in writer-1-*) test \"$(cat /yard/candidates/6331/source/same.txt)\" = \"$(cat /yard/candidates/6331/proof/evidence.txt)\";; *) exit 92;; esac && ! (echo bad > /yard/candidates/6331/source/same.txt) 2>/dev/null && ! (echo bad > /yard/candidates/6331/proof/evidence.txt) 2>/dev/null && echo picker-read-only",
                )]);
            }
            if request
                .tool_results()
                .last()
                .is_some_and(|(_, text)| text.contains("picker-read-only"))
            {
                assert!(request.prompt().contains("Same frozen reviewer."));
                return Reply::Tools(vec![tool(
                    "yard_publish_selection",
                    json!({"candidate": "c1"}),
                )]);
            }
            assert!(
                request
                    .tool_results()
                    .last()
                    .unwrap()
                    .1
                    .contains("published")
            );
            return Reply::Text("selected".into());
        }
        if request.has_tool("yard_publish_review") {
            assert!(request.prompt().contains("Same frozen reviewer."));
            if request.opens() {
                let prompt = request.prompt();
                let seat = prompt
                    .split_once("You are the r")
                    .unwrap()
                    .1
                    .split_whitespace()
                    .next()
                    .unwrap();
                let index: usize = seat.parse().unwrap();
                let n = if index == 5 { 0 } else { index };
                return Reply::Tools(vec![bash(&format!(
                    "case \"$(cat same.txt)\" in writer-{n}-*) test \"$(cat same.txt)\" = \"$(cat /yard/proof/evidence.txt)\";; *) exit 93;; esac && ! (echo bad > same.txt) 2>/dev/null && echo exact-review-$(cat same.txt)",
                ))]);
            }
            if request
                .tool_results()
                .last()
                .is_some_and(|(_, text)| text.contains("exact-review"))
            {
                let priority = "P3";
                return Reply::Tools(vec![publish(json!([
                    { "priority": priority, "body": "Fixture review evidence." },
                    { "priority": priority, "body": "Fixture review evidence." }
                ]))]);
            }
            return Reply::Text("reviewed".into());
        }
        let n: usize = request.body["model"]
            .as_str()
            .unwrap()
            .strip_prefix("producer-")
            .unwrap()
            .parse()
            .unwrap();
        let reply = if request.opens() {
            let version = turns[n].fetch_add(1, Ordering::SeqCst);
            Reply::Tools(vec![bash(&format!(
                "printf 'writer-{n}-{version}\\n' > same.txt && printf 'writer-{n}-{version}\\n' > /yard/proof/evidence.txt && git add same.txt && git commit -qm Independent && echo producer-ready"
            ))])
        } else if request
            .tool_results()
            .last()
            .is_some_and(|(_, text)| text.contains("producer-ready"))
        {
            let hold = if turns[n].load(Ordering::SeqCst) == 1 {
                writer_holds[n].clone()
            } else {
                replacement_hold.clone()
            };
            Reply::Hold(hold, Box::new(Reply::Text("done".into())))
        } else {
            Reply::Text("done".into())
        };
        Reply::ChatUsage {
            input: if n == 2 { 0 } else { PROMPT_TOKENS },
            output: if n == 2 { 0 } else { COMPLETION_TOKENS },
            cache: None,
            reply: Box::new(reply),
        }
    });
    // Remove measurement fields only. Real Pi still runs and publishes normally.
    machine.wrapper(
        "pinfold",
        r#"if [ "$1 $2" = 'box exec' ]; then
  out=$(mktemp -d)
  mkfifo "$out/frames"
  "$REAL" "$@" >"$out/frames" &
  child=$!
  while IFS= read -r line; do
    case "$line" in
      *'"type":"message_end"'*'"model":"producer-3"'*)
        line=$(printf '%s\n' "$line" | sed -E 's/,"cost":\{[^}]*\}//g; s/,"usage":\{[^}]*\}//g') ;;
    esac
    printf '%s\n' "$line"
  done <"$out/frames"
  wait "$child"
  code=$?
  rm -rf "$out"
  exit "$code"
fi"#,
    );
    machine.start();
    let mut extra = String::from(
        "\n[agents.worker.judgment_inputs]\nconfiguration = 'fixture-pinfold-v1'\nmodel = 'scripted-fixture-v1'\n\
         [profiles.picker]\nrunner = {kind = 'agent', agent = 'worker', instructions = 'Compare frozen inputs.'}\naccess = 'read-only'\noutputs = {result = 'selection'}\noutcomes = ['complete']\n\
         [profiles.reader]\nrunner = {kind = 'agent', agent = 'worker', instructions = 'Same frozen reviewer.'}\naccess = 'read-only'\noutputs = {result = 'verdict'}\noutcomes = ['complete', 'blocked']\n\
         [gates.check]\ncommand = 'case \"$(cat same.txt)\" in writer-1-*) test \"$(cat same.txt)\" = \"$(cat /yard/proof/evidence.txt)\" || exit 91;; *) exit 91;; esac'\nstage = 'candidate'\n",
    );
    extra.push_str("\n[gates.other]\ncommand = 'true'\nstage = 'landing'\n");
    let mut nodes = serde_json::Map::new();
    let mut inputs = serde_json::Map::new();
    let mut seats = Vec::new();
    for n in 0..5 {
        extra.push_str(&format!(
            "\n[agents.p{n}]\nharness = 'pi'\nprovider = 'openrouter'\nmodel = 'producer-{n}'\n\
             [profiles.p{n}]\nrunner = {{kind = 'agent', agent = 'p{n}', instructions = 'Same frozen instructions.'}}\naccess = 'write-allowed'\noutputs = {{result = 'candidate'}}\noutcomes = ['complete']\n\
             [review.seats.r{n}]\nagent = 'worker'\ninstructions = 'Same frozen reviewer.'\n"
        ));
        seats.push(format!("r{n}"));
        let writer = match n {
            0 => "implementer".into(),
            2 => "build-gate-model".into(),
            3 => "review-0".into(),
            4 => "landing-gate-other".into(),
            _ => format!("writer-{n}"),
        };
        let reviewer = format!("review-{}", if n == 0 { 5 } else { n });
        nodes.insert(
            writer.clone(),
            json!({"profile": format!("p{n}"), "routes": {"complete": [
            {"to": reviewer}, {"to": "selector"}]}}),
        );
        nodes.insert(
            reviewer.clone(),
            json!({"profile": "reader",
            "inputs": {"candidate": {"node": writer, "output": "result", "kind": "candidate"}},
            "requires": ["candidate"], "routes": {"complete": [{"to": "selector"}]}}),
        );
        inputs.insert(
            format!("c{n}"),
            json!({"node": writer, "output": "result", "kind": "candidate"}),
        );
        inputs.insert(
            format!("r{n}"),
            json!({"node": reviewer, "output": "result", "kind": "verdict"}),
        );
    }
    seats.push("r5".into());
    extra.push_str(
        "\n[review.seats.r5]\nagent = 'worker'\ninstructions = 'Same frozen reviewer.'\n",
    );
    nodes.insert("selector".into(), json!({"profile": "picker", "inputs": inputs,
        "requires": inputs.keys().collect::<Vec<_>>(), "routes": {"complete": [{"to": "candidate-gate-check"}]}}));
    nodes.insert("candidate-gate-check".into(), json!({
        "runner": {"kind": "command", "gate": "check"}, "access": "write-allowed",
        "inputs": {"candidate": {"node": "selector", "output": "result", "kind": "selection"}},
        "requires": ["candidate"], "outputs": {"result": "verdict"}, "outcomes": ["complete", "failed"],
        "routes": {"complete": [{"to": "landing-gate-check"}], "failed": [{"to": "implementer", "repeat": 1}]}
    }));
    nodes.insert("landing-gate-check".into(), json!({
        "runner": {"kind": "command", "gate": "check"}, "access": "write-allowed",
        "inputs": {"merged": {"node": null, "output": "merged", "kind": "candidate"}},
        "requires": ["merged"], "outputs": {"result": "verdict"}, "outcomes": ["complete"], "routes": {}
    }));
    let configuration = config(&extra)
        .replace("max_rounds = 3", "max_rounds = 1")
        .replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 5")
        .replace(
            "review = [\"correctness\"]",
            &format!("review = {}", json!(seats)),
        );
    let project = Project::new(&machine, "p", &configuration);
    let main = project.canonical_head();
    project.git(&["checkout", "--quiet", "-b", "feature/a"]);
    project.json(&["sync", "feature/a"]);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Five candidate choice",
        "--target",
        "feature/a",
        "--workflow",
        "plan",
    ]);
    let planner = watch.find("planner finished", |event| {
        event["event"] == "attempt.ended"
    });
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner["attempt"]
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let target = project.canonical_head();
    let mut operator = machine.operator();
    operator.initialize();
    let before_start = std::time::Instant::now();
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1",
        "graph": {"inputs": {"merged": "candidate"}, "nodes": nodes}}),
    );
    assert!(started.get("error").is_none(), "{started}");
    let attempt = started["result"]["attempt"].as_i64().unwrap();
    for hold in &holds {
        hold.wait_held();
    }
    let all_held = std::time::Instant::now();
    // The completed planner also has a workspace. Count every writer in the
    // admitted graph attempt, without filtering to expected producer names.
    // Sabotage: schedule a sixth implementation workspace in this attempt.
    let writers = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE attempt = {attempt} AND kind = 'implementation' AND workspace IS NOT NULL ORDER BY id"
        ),
    );
    assert_eq!(writers.len(), 5);
    let mut heads = Vec::new();
    for n in 0..5 {
        let node = match n {
            0 => "implementer".into(),
            2 => "build-gate-model".into(),
            3 => "review-0".into(),
            4 => "landing-gate-other".into(),
            _ => format!("writer-{n}"),
        };
        let row = writers.iter().find(|row| row["workspace"] == node).unwrap();
        assert_eq!(row["settings"]["model"], format!("producer-{n}"));
        assert_eq!(row["base"], target);
        let hex = node.bytes().map(|b| format!("{b:02x}")).collect::<String>();
        let clone = project.path.join(format!(
            ".yard/local/attempts/{attempt}/writers/{hex}/clone"
        ));
        heads.push(git(&clone, &["rev-parse", "HEAD"]).trim().to_owned());
        assert_eq!(
            git(&clone, &["show", "HEAD:same.txt"]).trim(),
            format!("writer-{n}-0")
        );
    }
    // Force B's publication before the final source. Newest-head selection loses.
    let mut durations = Vec::new();
    for n in [1, 0, 2, 3, 4] {
        let lower = all_held.elapsed().as_secs_f64();
        holds[n].release();
        let id = &writers
            .iter()
            .find(|row| row["model"] == format!("producer-{n}"))
            .unwrap()["id"];
        watch.find("writer released", |event| {
            event["event"] == "execution.released" && &event["execution"] == id
        });
        durations.push((id.clone(), lower, before_start.elapsed().as_secs_f64()));
    }
    let first = approval(&mut watch);
    assert_eq!(first["head"], heads[1]);
    // G7: attribute to the producer, not the picker; use eligible source IDs,
    // not edges, downstream checks or the newest workspace. Fixture tokens and
    // host hold intervals supply expectations; missing usage cannot become zero.
    let stats = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    let trials = &stats["result"]["trials"];
    let models = trials["models"].as_array().unwrap();
    for n in 0..5 {
        let model = models
            .iter()
            .find(|model| model["identity"]["requested"]["model"] == format!("producer-{n}"))
            .unwrap();
        assert_eq!(model["publication_count"], 1);
        assert_eq!(model["eligible_count"], 1);
        assert_eq!(model["selected_count"], usize::from(n == 1));
        assert_eq!(model["selected_rate"], if n == 1 { 1.0 } else { 0.0 });
        assert_eq!(model["outcomes"]["completed"], 1);
        assert_eq!(model["identity"]["observed"]["status"], "unknown");
        let input = &model["measurements"]["input"];
        if n == 3 {
            assert_eq!(input["known_sum"], Value::Null);
            assert_eq!(input["unknown_count"], 1);
        } else {
            assert_eq!(
                input["known_sum"],
                if n == 2 { 0 } else { 2 * PROMPT_TOKENS }
            );
            assert_eq!(input["complete_count"], 1);
        }
    }
    let chosen_source = &trials["choices"][0]["selected_sources"][0];
    assert_eq!(
        chosen_source["execution"],
        writers
            .iter()
            .find(|row| row["model"] == "producer-1")
            .unwrap()["id"]
    );
    // Concurrent execution durations cannot be summed as graph wall time.
    let span = trials["cohorts"][0]["timing"]["elapsed_seconds"]
        .as_f64()
        .unwrap();
    assert!(
        span + 0.01 >= durations.last().unwrap().1
            && span <= before_start.elapsed().as_secs_f64() + 0.01
    );
    for (id, lower, upper) in durations {
        let row = stats["result"]["executions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["execution"] == id)
            .unwrap();
        let elapsed = row["elapsed_seconds"].as_f64().unwrap();
        assert!(elapsed + 0.01 >= lower && elapsed <= upper + 0.01, "{row}");
    }
    let skipped = first["skipped"].as_array().unwrap();
    assert_eq!(skipped.len(), 2);
    assert!(skipped.contains(&json!("landing-gate-other")));
    assert!(skipped.contains(&json!("review-0")));
    let history = rows(
        &project,
        "SELECT * FROM execution WHERE result IS NOT NULL ORDER BY id",
    );
    let picker = history
        .iter()
        .find(|row| row["node"] == "selector" && row["outcome"] == "selected")
        .unwrap();
    assert_eq!(picker["inputs"].as_object().unwrap().len(), 10);
    let inspected = project.json(&["attempt", "show", "Y-1"]);
    for n in 0..5 {
        assert_eq!(picker["inputs"][format!("c{n}")]["value"]["head"], heads[n]);
        assert_eq!(
            picker["inputs"][format!("c{n}")]["value"]["producer"]["settings"]["model"],
            format!("producer-{n}")
        );
        assert_eq!(
            picker["inputs"][format!("r{n}")]["source"]["node"],
            format!("review-{}", if n == 0 { 5 } else { n })
        );
        let review = &picker["inputs"][format!("r{n}")]["value"]["review"];
        assert_eq!(
            review["node"]["runner"]["instructions"],
            "Same frozen reviewer."
        );
        assert_eq!(review["findings"][0]["body"], "Fixture review evidence.");
        let execution = history
            .iter()
            .find(|row| row["id"] == picker["inputs"][format!("r{n}")]["source"]["execution"])
            .unwrap();
        assert_eq!(execution["settings"]["model"], "fake-model");
        assert_eq!(execution["inputs"]["candidate"]["value"]["head"], heads[n]);
        // G2: identical prose in multiple rows and reviews is not identity.
        // Sabotage: derive IDs from text/index or use the newest check.
        // The durable FK join supplies expectations outside the binary.
        let sources = project.rows(&format!(
            "SELECT f.id AS finding, f.\"check\" AS \"check\", e.id AS execution,
             e.attempt, 'Y-' || a.ticket AS ticket, e.base, e.head, e.proof,
             e.ticket_revision, e.plan_revision, e.digest
             FROM finding f JOIN \"check\" c ON c.id = f.\"check\"
             JOIN execution e ON e.id = c.execution JOIN attempt a ON a.id = e.attempt
             WHERE e.id = {} ORDER BY f.priority, f.id",
            execution["id"]
        ));
        assert_eq!(sources.len(), 2);
        let raw = inspected["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["execution"] == execution["id"])
            .unwrap();
        assert_eq!(review["findings"], raw["findings"]);
        assert_eq!(review["findings"].as_array().unwrap().len(), 2);
        for (finding, source) in review["findings"].as_array().unwrap().iter().zip(sources) {
            assert_eq!(
                *finding,
                json!({
                    "source": source, "priority": "P3", "body": "Fixture review evidence.",
                    "file": null, "line": null, "category": null
                })
            );
        }
        assert_eq!(review["inputs"], execution["inputs"]);
    }
    assert_eq!(
        picker["result"]["outputs"]["result"]["value"]["candidate"],
        picker["inputs"]["c1"]
    );
    assert_eq!(
        picker["result"]["outputs"]["result"]["value"]["picker"]["execution"],
        picker["id"]
    );
    assert_eq!(first["checks"].as_array().unwrap().len(), 2);
    assert!(
        first["checks"]
            .as_array()
            .unwrap()
            .contains(&picker["inputs"]["r1"]["value"]["publication"])
    );
    let mut direct = json!({"project": project.path, "ticket": "Y-1", "attempt": attempt, "revision": 1,
        "plan_revision": 1, "target": "feature/a", "choice": first["choice"], "inputs": picker["inputs"], "candidate": "foreign"});
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_select", direct.clone());
    assert_eq!(refused["error"]["code"], "refused");
    assert_eq!(
        refused["error"]["data"]["reason"], "selection_input",
        "{refused}"
    );
    assert_eq!(refused["error"]["data"]["identity"], "candidate");
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    direct["candidate"] = json!("c1");
    let chosen = operator.call("yard_attempt_select", direct.clone());
    assert!(chosen.get("error").is_none(), "{chosen}");
    let second = approval(&mut watch);
    assert_eq!(second["head"], heads[1]);
    assert_ne!(second["choice"], first["choice"]);
    let old_approve = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path, "ticket": "Y-1",
        "target": "feature/a", "head": heads[1], "proof": first["proof"], "choice": first["choice"]}),
    );
    assert_eq!(
        old_approve["error"]["data"]["identity"], "choice",
        "{old_approve}"
    );
    assert_eq!(old_approve["error"]["code"], "stale");
    assert_eq!(old_approve["error"]["data"]["expected"], first["choice"]);
    assert_eq!(old_approve["error"]["data"]["current"], second["choice"]);
    let retained = rows(
        &project,
        "SELECT * FROM execution WHERE status = 'ended' ORDER BY id",
    );
    let raw_reviews = project.rows("SELECT * FROM finding ORDER BY id");
    assert_eq!(raw_reviews.len(), 10);
    let raw_checks = project.rows("SELECT * FROM \"check\" ORDER BY id");
    let retry = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "writer-1"}),
    );
    assert!(retry.get("error").is_none(), "{retry}");
    replacement.wait_held();
    direct["choice"] = second["choice"].clone();
    let before_stale = project.rows("SELECT * FROM audit ORDER BY seq");
    let stale = operator.call("yard_attempt_select", direct);
    assert_eq!(stale["error"]["data"]["identity"], "choice", "{stale}");
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(stale["error"]["data"]["expected"], second["choice"]);
    assert_eq!(stale["error"]["data"]["current"], 0);
    assert_eq!(
        project.rows("SELECT * FROM audit ORDER BY seq"),
        before_stale
    );
    let hex = "writer-1"
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let clone = project.path.join(format!(
        ".yard/local/attempts/{attempt}/writers/{hex}/clone"
    ));
    let replacement_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_ne!(replacement_head, heads[1]);
    replacement.release();
    let third = approval(&mut watch);
    assert_eq!(third["head"], replacement_head);
    assert_ne!(third["choice"], second["choice"]);
    let choices = rows(
        &project,
        "SELECT * FROM execution WHERE node = 'selector' AND outcome = 'selected' ORDER BY id",
    );
    let current = choices.last().unwrap();
    // Repeated application of c1 and reused losing inputs are single source
    // opportunities. An obsolete choice must not be reported as current.
    let stats = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    let trials = &stats["result"]["trials"];
    let producer = trials["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|model| model["identity"]["requested"]["model"] == "producer-1")
        .unwrap();
    assert_eq!(producer["eligible_count"], 2);
    assert_eq!(producer["selected_count"], 2);
    assert_eq!(producer["selected_rate"], 1.0);
    assert_eq!(producer["current_selected_count"], 1);
    for choice in trials["choices"].as_array().unwrap() {
        assert_eq!(
            choice["application"]["status"],
            if choice["execution"] == third["choice"] {
                "applied"
            } else {
                "superseded"
            }
        );
        assert_eq!(choice["selected_sources"].as_array().unwrap().len(), 1);
    }
    assert_eq!(current["inputs"]["c1"]["value"]["head"], replacement_head);
    assert_eq!(
        current["inputs"]["c1"]["value"]["producer"]["settings"]["model"],
        "producer-1"
    );
    assert!(
        third["checks"]
            .as_array()
            .unwrap()
            .contains(&current["inputs"]["r1"]["value"]["publication"])
    );
    let fresh_reviews = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE kind = 'review' AND node = 'review-1' AND head = '{replacement_head}'"
        ),
    );
    assert_eq!(fresh_reviews.len(), 1);
    let verification: Value =
        serde_json::from_str(fresh_reviews[0]["verification"].as_str().unwrap()).unwrap();
    assert_eq!(verification["reuse"], "disabled");
    assert_eq!(fresh_reviews[0]["result"]["reference"], Value::Null);
    assert_eq!(fresh_reviews[0]["settings"]["model"], "fake-model");
    assert_eq!(
        fresh_reviews[0]["inputs"]["candidate"]["value"]["head"],
        replacement_head
    );
    for previous in &retained {
        assert_eq!(
            rows(
                &project,
                &format!("SELECT * FROM execution WHERE id = {}", previous["id"])
            ),
            vec![previous.clone()]
        );
    }
    for previous in &raw_reviews {
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM finding WHERE id = {}",
                previous["id"]
            )),
            vec![previous.clone()]
        );
    }
    for previous in &raw_checks {
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM \"check\" WHERE id = {}",
                previous["id"]
            )),
            vec![previous.clone()]
        );
    }
    let approved = operator.call("yard_attempt_approve", json!({"project": project.path, "ticket": "Y-1", "target": "feature/a", "head": replacement_head, "proof": third["proof"], "choice": third["choice"]}));
    assert!(approved.get("error").is_none(), "{approved}");
    watch.find("replacement candidate landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    assert_eq!(
        git(&project.canonical(), &["show", "feature/a:same.txt"]).trim(),
        "writer-1-1"
    );
    assert_eq!(project.canonical_head(), main);
    let gates = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'gate' AND parent IS NULL ORDER BY id",
    );
    assert!(gates.iter().any(|row| row["head"] == heads[1]));
    assert!(gates.iter().any(|row| row["head"] == replacement_head));
    assert!(
        gates
            .iter()
            .all(|row| row["head"] == heads[1] || row["head"] == replacement_head)
    );
    let cleanup = watch.find("selected trial cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == attempt
    });
    watch.find("trial cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let before = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    )["result"]["trials"]
        .clone();
    let producer = before["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|model| model["identity"]["requested"]["model"] == "producer-1")
        .unwrap();
    assert_eq!(producer["landings"].as_array().unwrap().len(), 1);
    drop(watch);
    machine.kill();
    machine.start();
    let after = operator.call(
        "yard_attempt_stats",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert_eq!(after["result"]["trials"], before);
}
