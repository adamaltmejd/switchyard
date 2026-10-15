//! G2: Judgments bind exact identity.

use e2e::*;
use serde_json::{Value, json};
use std::io::Write;

const GATE: &str = "[gates.check]\ncommand = \"test -f feature.txt\"\nstage = \"candidate\"\n";

/// G2/G3: a second producer publishes the same Git tree and proof after a ticket
/// edit. Fixture command/model counts distinguish reference reuse from replay;
/// immutable rows and MCP distinguish it from a copied verdict or a skip.
/// Existing definition-adoption cases keep the same supplier and cannot show
/// this boundary. Git supplies trees; the fixture supplies counts and proof.
/// Sabotage: rerun every gate, qualify by head alone, accept unknown inputs,
/// copy a passing check, or carry an old operator identity to approval.
#[test]
fn candidate_reuses_original_pass_after_bookkeeping_with_unknown_rerun() {
    let implementations = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = implementations.clone();
    let machine = Machine::new("g2-reference-reuse", move |request| {
        if request.opens() {
            counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        let contents = if request.last_user().contains("change-tree") {
            "printf changed > feature.txt"
        } else {
            "test -f feature.txt || printf feature > feature.txt"
        };
        let proof = if request.last_user().contains("change-") {
            "printf second > /yard/proof/evidence.txt"
        } else {
            "test -f /yard/proof/evidence.txt || printf first > /yard/proof/evidence.txt"
        };
        act(
            request,
            vec![bash(&format!(
                "cd /workspace && ({contents}) && git add feature.txt \
             && git commit --allow-empty -qm Revision && ({proof})"
            ))],
        )
    });
    let commands = machine.root.join("gate-commands");
    // Revision prompts contain gate definitions. Count only literal lines from
    // real command stdout, never argv or marker mentions inside SDK frames.
    machine.wrapper(
        "pinfold",
        &format!(
            r#"if [ "$1 $2" = 'box exec' ]; then
  fifo='{}.'$$
  mkfifo "$fifo" || exit $?
  trap 'rm -f "$fifo"' 0
  {{
    while IFS= read -r line; do
      case "$line" in
        reuse-closed) printf 'closed\n' >> '{}' ;;
        reuse-other) printf 'other\n' >> '{}' ;;
        reuse-unknown) printf 'unknown\n' >> '{}' ;;
      esac
      printf '%s\n' "$line"
    done
    printf '%s' "$line"
  }} < "$fifo" &
  reader=$!
  "$REAL" "$@" > "$fifo"
  result=$?
  wait "$reader"
  exit "$result"
fi"#,
            machine.root.join("gate-commands.fifo").display(),
            commands.display(),
            commands.display(),
            commands.display()
        ),
    );
    machine.start();
    let configuration = unreviewed(
        "[gates.closed]\ncommand = 'echo reuse-closed; test -f feature.txt'\nstage = 'candidate'\nclosed_inputs = 'runtime-v1'\n\
         [gates.other]\ncommand = 'echo reuse-other; test -f feature.txt'\nstage = 'candidate'\nclosed_inputs = 'runtime-v1'\n\
         [gates.unknown]\ncommand = 'echo reuse-unknown; test -f feature.txt'\nstage = 'candidate'\n",
    );
    let project = Project::new(&machine, "p", &configuration);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Feature"]);
    let original_item = approval(&mut watch);
    let original = project.rows("SELECT * FROM execution WHERE kind = 'gate' ORDER BY id");
    let original_checks = project.rows("SELECT * FROM \"check\" ORDER BY id");
    let initial_tree = git(
        &project.canonical(),
        &["rev-parse", &format!("{}^{{tree}}", head(&original_item))],
    )
    .trim()
    .to_string();
    let mut operator = machine.operator();
    operator.initialize();
    for (body, expected_known, expected_unknown) in [
        ("Bookkeeping only", 1, 2),
        ("change-proof", 2, 3),
        ("change-tree", 3, 4),
    ] {
        let revision = project.json(&["ticket", "show", "Y-1"])["revision"]
            .as_i64()
            .unwrap();
        project.json(&[
            "ticket",
            "edit",
            "Y-1",
            "--revision",
            &revision.to_string(),
            "--body",
            body,
        ]);
        // approval consumes future events. Searching retained history here would
        // return the old approval before the replacement executions finish.
        let item = approval(&mut watch);
        let log = std::fs::read_to_string(&commands).unwrap();
        for (name, count) in [
            ("closed", expected_known),
            ("other", expected_known),
            ("unknown", expected_unknown),
        ] {
            assert_eq!(
                log.lines().filter(|line| *line == name).count(),
                count,
                "{body}: {log}"
            );
        }
        assert_eq!(
            implementations.load(std::sync::atomic::Ordering::SeqCst),
            expected_unknown
        );
        assert_eq!(item["payload"]["revision"], revision + 1);
        assert_ne!(head(&item), head(&original_item));
        let tree = git(
            &project.canonical(),
            &["rev-parse", &format!("{}^{{tree}}", head(&item))],
        )
        .trim()
        .to_string();
        if body == "Bookkeeping only" {
            assert_eq!(tree, initial_tree);
            assert_eq!(proof(&item), proof(&original_item));
            let shown = operator.call(
                "yard_attempt_show",
                json!({"project": project.path, "ticket": "Y-1"}),
            );
            assert!(shown.get("error").is_none(), "{shown}");
            let executions = shown["result"]["executions"].as_array().unwrap();
            for name in ["closed", "other"] {
                let old = original.iter().find(|row| row["name"] == name).unwrap();
                let check = original_checks
                    .iter()
                    .find(|row| row["execution"] == old["id"])
                    .unwrap();
                let current = executions
                    .iter()
                    .rev()
                    .find(|row| row["name"] == name)
                    .unwrap();
                assert_eq!(current["outcome"], "reused");
                assert_eq!(current["ticket_revision"], revision + 1);
                assert_eq!(current["head"], head(&item));
                assert_eq!(
                    current["result"]["reference"],
                    json!({"execution": old["id"], "check": check["id"], "result": {"execution": old["id"]}})
                );
                assert_eq!(current["result"]["judgment"], Value::Null);
                assert_eq!(current["result"]["outputs"], json!({}));
                assert!(
                    item["payload"]["checks"]
                        .as_array()
                        .unwrap()
                        .contains(&check["id"])
                );
            }
            assert!(project.rows("SELECT * FROM \"check\" WHERE execution IN (SELECT id FROM execution WHERE outcome = 'reused')").is_empty());
        } else if body == "change-proof" {
            assert_eq!(tree, initial_tree);
            assert_ne!(proof(&item), proof(&original_item));
        } else {
            assert_ne!(tree, initial_tree);
        }
    }
    // Explicit definition adoption changes only the consuming gate. A runtime
    // declaration changes its declared tool/configuration identity separately.
    // Sabotage: omit command, declaration or image from matching, or rerun
    // unrelated consumers despite identical consumed definitions.
    let changed_command = configuration.replace(
        "echo reuse-closed; test -f feature.txt",
        "echo reuse-closed; test -f feature.txt && test -s feature.txt",
    );
    let changed_runtime = changed_command.replacen("runtime-v1", "runtime-v2", 1);
    let mut plan_revision = 1;
    for (definition, expected_closed) in [(&changed_command, 4), (&changed_runtime, 5)] {
        let retained = project.rows("SELECT * FROM \"check\" WHERE execution = (SELECT max(id) FROM execution WHERE name = 'other' AND outcome = 'pass')")[0].clone();
        project.reconfigure(definition);
        let source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
        let revised = operator.call(
            "yard_attempt_revise",
            json!({"project": project.path,
            "ticket": "Y-1", "plan_revision": plan_revision, "definition_commit": source}),
        );
        plan_revision += 1;
        assert_eq!(
            revised["result"]["plan_revision"], plan_revision,
            "{revised}"
        );
        let item = approval(&mut watch);
        let log = std::fs::read_to_string(&commands).unwrap();
        for (name, count) in [("closed", expected_closed), ("other", 3), ("unknown", 4)] {
            assert_eq!(
                log.lines().filter(|line| *line == name).count(),
                count,
                "{log}"
            );
        }
        assert!(
            item["payload"]["checks"]
                .as_array()
                .unwrap()
                .contains(&retained["id"])
        );
        let references = project.rows("SELECT verification FROM execution WHERE name = 'other' AND outcome = 'reused' ORDER BY id");
        let reference: Value =
            serde_json::from_str(references.last().unwrap()["verification"].as_str().unwrap())
                .unwrap();
        assert_eq!(reference["reference"]["check"], retained["id"]);
    }
    // A changed observed image and its declared tool source invalidates every
    // boxed consumer, while preserving the exact frozen command definitions.
    let dockerfile = project.git(&["show", "HEAD:.yard/Dockerfile"]);
    project.write(
        ".yard/Dockerfile",
        &format!("{dockerfile}\nENV Y150_IMAGE_REVISION=second\n"),
    );
    project.git(&["commit", "--quiet", "-am", "Change declared image tools"]);
    let image = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let blob = project
        .git(&["rev-parse", "HEAD:.yard/Dockerfile"])
        .trim()
        .to_string();
    project.json(&["sync"]);
    let revised = operator.call(
        "yard_attempt_revise",
        json!({"project": project.path,
        "ticket": "Y-1", "plan_revision": plan_revision, "image_commit": image}),
    );
    plan_revision += 1;
    assert_eq!(
        revised["result"]["plan_revision"], plan_revision,
        "{revised}"
    );
    approval(&mut watch);
    let log = std::fs::read_to_string(&commands).unwrap();
    for (name, count) in [("closed", 6), ("other", 4), ("unknown", 5)] {
        // Failure-only, bounded identities distinguish work between sync and
        // adoption from replay on the same frozen inputs. Never dump results,
        // commands, launch environment or operator/worker connection data.
        assert_eq!(
            log.lines().filter(|line| *line == name).count(),
            count,
            "{log}\nimage adoption execution identities (latest 48): {}",
            json!(project.rows(
                "SELECT e.id, e.kind, e.name, e.node, e.reason, e.status,
                    e.outcome, e.head, e.base, e.proof, e.plan_revision,
                    e.ticket_revision, e.image_id,
                    json_extract(e.inputs, '$.candidate.source.execution') AS candidate_producer,
                    json_extract(e.verification, '$.reference') AS reference,
                    json_extract(e.verification, '$.candidate.tree') AS candidate_tree,
                    json_extract(e.verification, '$.candidate.base_tree') AS base_tree,
                    json_extract(e.verification, '$.policy.node.operator_failure') AS operator_failure,
                    json_extract(e.verification, '$.policy.definition.dockerfile_blob') AS dockerfile_blob,
                    json_extract(e.verification, '$.policy.definition.build_context_tree') AS build_context_tree,
                    json_extract(COALESCE(p.evidence, a.admission), '$.inputs.definition_commit') AS definition_commit,
                    json_extract(COALESCE(p.evidence, a.admission), '$.inputs.staging_commit') AS staging_commit,
                    json_extract(COALESCE(p.evidence, a.admission), '$.inputs.image_commit') AS image_commit
                 FROM execution e JOIN attempt a ON a.id = e.attempt
                 LEFT JOIN plan_revision p ON p.attempt = e.attempt AND p.revision = e.plan_revision
                 ORDER BY e.id DESC LIMIT 48"
            ))
        );
    }
    let observed =
        project.rows("SELECT verification FROM execution WHERE name = 'closed' ORDER BY id");
    let observed: Value =
        serde_json::from_str(observed.last().unwrap()["verification"].as_str().unwrap()).unwrap();
    assert_eq!(observed["identity"]["definition"]["dockerfile_blob"], blob);

    // G2: change only the frozen node policy on an already verified candidate.
    // Sabotage: compare consumed definitions but omit the original node policy
    // when matching, then qualify only the replacement execution's policy.
    let policy_original = project
        .rows("SELECT * FROM execution WHERE name = 'closed' ORDER BY id DESC LIMIT 1")[0]
        .clone();
    let policy_check = project.rows(&format!(
        "SELECT * FROM \"check\" WHERE execution = {}",
        policy_original["id"]
    ))[0]
        .clone();
    let retained = project.rows("SELECT * FROM \"check\" WHERE execution = (SELECT max(id) FROM execution WHERE name = 'other' AND outcome = 'pass')")[0].clone();
    let before_policy = project.json(&["attempt", "show", "Y-1"])["attempt"].clone();
    let revised = operator.call(
        "yard_attempt_revise",
        json!({"project": project.path,
        "ticket": "Y-1", "plan_revision": plan_revision,
        "overrides": {"candidate-gate-closed": {"operator_failure": true}}}),
    );
    plan_revision += 1;
    assert_eq!(
        revised["result"]["plan_revision"], plan_revision,
        "{revised}"
    );
    let item = approval(&mut watch);
    let log = std::fs::read_to_string(&commands).unwrap();
    for (name, count) in [("closed", 7), ("other", 4), ("unknown", 5)] {
        assert_eq!(
            log.lines().filter(|line| *line == name).count(),
            count,
            "{log}"
        );
    }
    for key in ["base", "head", "proof"] {
        assert_eq!(item["payload"][key], before_policy[key]);
    }
    assert_eq!(item["payload"]["plan_revision"], plan_revision);
    let shown = operator.call(
        "yard_attempt_show",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert!(shown.get("error").is_none(), "{shown}");
    let executions = shown["result"]["executions"].as_array().unwrap();
    let changed = executions
        .iter()
        .rev()
        .find(|row| row["name"] == "closed")
        .unwrap();
    assert_ne!(changed["execution"], policy_original["id"]);
    assert_eq!(changed["outcome"], "pass");
    assert_eq!(
        changed["verification"]["policy"]["node"]["operator_failure"],
        true
    );
    assert_eq!(changed["result"]["reference"], Value::Null);
    assert!(
        !item["payload"]["checks"]
            .as_array()
            .unwrap()
            .contains(&policy_check["id"])
    );
    let unrelated = executions
        .iter()
        .rev()
        .find(|row| row["name"] == "other")
        .unwrap();
    assert_eq!(unrelated["outcome"], "reused");
    assert_eq!(unrelated["result"]["reference"]["check"], retained["id"]);
    assert!(
        item["payload"]["checks"]
            .as_array()
            .unwrap()
            .contains(&retained["id"])
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id = {}",
            policy_original["id"]
        )),
        vec![policy_original]
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id = {}",
            policy_check["id"]
        )),
        vec![policy_check]
    );

    // A failure is never a reusable pass. Omit the undeclared gate at this
    // boundary to observe skip, reference and accepted failure together.
    // Sabotage: reference failed results, carry acceptance to a new identity,
    // or manufacture a check for the omitted node.
    let failed_definition = changed_runtime.replace(
        "echo reuse-closed; test -f feature.txt && test -s feature.txt",
        "echo reuse-closed; exit 7",
    );
    project.reconfigure(&failed_definition);
    let source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let mut graph = project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"]["plan"].clone();
    for omitted in [
        "candidate-gate-unknown",
        "landing-gate-closed",
        "landing-gate-unknown",
    ] {
        graph["nodes"].as_object_mut().unwrap().remove(omitted);
    }
    // Ordinary reduced selections are one reachable linear chain. Candidate
    // failure acceptance cannot authorize the failing command on merged input.
    graph["nodes"]["candidate-gate-other"]["routes"]["complete"] =
        json!([{"to": "landing-gate-other"}]);
    graph["nodes"]["landing-gate-other"]["routes"]
        .as_object_mut()
        .unwrap()
        .remove("complete");
    graph["nodes"]["candidate-gate-closed"]["operator_failure"] = json!(true);
    let revised = operator.call("yard_attempt_revise", json!({"project": project.path,
        "ticket": "Y-1", "plan_revision": plan_revision, "definition_commit": source, "graph": graph}));
    plan_revision += 1;
    assert_eq!(
        revised["result"]["plan_revision"], plan_revision,
        "{revised}"
    );
    let mut accepted = Vec::new();
    let mut failed_executions = Vec::new();
    let mut decisions = Vec::new();
    let mut final_item = Value::Null;
    for (body, expected_closed) in [(None, 8), (Some("Bookkeeping after acceptance"), 9)] {
        if let Some(body) = body {
            let revision = project.json(&["ticket", "show", "Y-1"])["revision"]
                .as_i64()
                .unwrap();
            project.json(&[
                "ticket",
                "edit",
                "Y-1",
                "--revision",
                &revision.to_string(),
                "--body",
                body,
            ]);
        }
        let raised = watch.attention();
        let item = raised["data"].clone();
        assert_eq!(item["kind"], "candidate", "{item}");
        let check = &item["payload"]["evidence"]["check"];
        assert_eq!(check["verdict"], "fail");
        let answer = operator.call(
            "yard_attempt_accept",
            json!({"project": project.path, "ticket": "Y-1",
            "attention": raised["attention"], "check": check["check"], "plan_revision": plan_revision,
            "target": "main", "head": item["payload"]["head"], "proof": item["payload"]["proof"],
            "text": "Keep this exact known failure"}),
        );
        assert!(answer.get("error").is_none(), "{answer}");
        let approved = approval(&mut watch);
        // Approval exposes the whole selection; omission decisions name only
        // their candidate or prepared-merge nodes.
        assert_eq!(
            approved["payload"]["skipped"],
            json!([
                "candidate-gate-unknown",
                "landing-gate-closed",
                "landing-gate-unknown"
            ])
        );
        assert!(
            approved["payload"]["checks"]
                .as_array()
                .unwrap()
                .contains(&check["check"])
        );
        let log = std::fs::read_to_string(&commands).unwrap();
        assert_eq!(
            log.lines().filter(|line| *line == "closed").count(),
            expected_closed
        );
        assert_eq!(log.lines().filter(|line| *line == "unknown").count(), 5);
        assert_eq!(log.lines().filter(|line| *line == "other").count(), 4);
        accepted.push(
            project.rows(&format!(
                "SELECT * FROM \"check\" WHERE id = {}",
                check["check"]
            ))[0]
                .clone(),
        );
        failed_executions.push(
            project.rows(&format!(
                "SELECT * FROM execution WHERE id = {}",
                check["execution"]
            ))[0]
                .clone(),
        );
        decisions.push(
            project.rows(&format!(
                "SELECT * FROM attention WHERE id = {}",
                raised["attention"]
            ))[0]
                .clone(),
        );
        final_item = approved;
    }
    assert_ne!(accepted[0]["id"], accepted[1]["id"]);
    let shown = operator.call(
        "yard_attempt_show",
        json!({"project": project.path, "ticket": "Y-1"}),
    );
    assert!(shown.get("error").is_none(), "{shown}");
    let retained = shown["result"]["executions"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|row| row["name"] == "other")
        .unwrap();
    assert_eq!(retained["outcome"], "reused");
    assert!(
        final_item["payload"]["checks"]
            .as_array()
            .unwrap()
            .contains(&retained["result"]["reference"]["check"])
    );
    assert!(project.rows(&format!("SELECT id FROM execution WHERE node = 'candidate-gate-unknown' AND plan_revision = {plan_revision}")).is_empty());
    for failed in &accepted {
        let execution = shown["result"]["executions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["execution"] == failed["execution"])
            .unwrap();
        assert_eq!(execution["outcome"], "fail");
        assert_eq!(execution["result"]["judgment"]["verdict"], "fail");
        assert_eq!(execution["result"]["reference"], Value::Null);
    }
    assert_eq!(
        project.rows(
            "SELECT count(*) AS n FROM attention WHERE kind = 'candidate' AND resolution = 'accept'"
        ),
        vec![json!({"n": 2})]
    );
    // G3: the queue rechecks the original selected reference under the exact
    // current approval. Selected merged verification still runs. Sabotage:
    // drop references from queue qualification, carry candidate acceptance to
    // landing, or manufacture results for explicitly omitted landing nodes.
    let current = project.json(&["attempt", "show", "Y-1"]);
    let approved = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path,
        "ticket": "Y-1", "target": "main", "base": final_item["payload"]["base"],
        "head": final_item["payload"]["head"], "proof": final_item["payload"]["proof"],
        "revision": final_item["payload"]["revision"], "plan_revision": plan_revision,
        "attention": current["attention"][0]["attention"],
        "accept_skipped": ["candidate-gate-unknown"],
        "observe_landing_skipped": ["landing-gate-closed", "landing-gate-unknown"],
        "text": "Accept this exact omitted gate"}),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    let attempt = current["attempt"]["attempt"].clone();
    // find searches retained events too: preparation and checked release may
    // already have arrived by the time the omission attention is observed.
    let preparation = watch.find("reuse selection preparation", |event| {
        event["event"] == "landing.prepared" && event["attempt"] == attempt
    });
    let omission = watch.find("reuse selection landing omissions", |event| {
        event["event"] == "attention.raised"
            && event["attempt"] == attempt
            && event["data"]["reason"] == "landing"
            && event["data"]["payload"]["evidence"]["landing"]["execution"]
                == preparation["execution"]
            && event["seq"].as_i64().unwrap() > preparation["seq"].as_i64().unwrap()
    });
    let evidence = &omission["data"]["payload"]["evidence"];
    let landing = &evidence["landing"];
    watch.find("reuse omission parent released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == landing["execution"]
            && event["seq"].as_i64().unwrap() > omission["seq"].as_i64().unwrap()
    });
    let merged = git(
        &project.canonical(),
        &[
            "rev-parse",
            &format!("refs/yard/landings/{}", landing["execution"]),
        ],
    )
    .trim()
    .to_string();
    assert_eq!(landing["prepared"]["merged"], merged);
    assert_eq!(
        evidence["identity"],
        json!({"attempt": attempt, "target": "main",
        "base": final_item["payload"]["base"], "head": final_item["payload"]["head"],
        "proof": final_item["payload"]["proof"], "revision": final_item["payload"]["revision"],
        "plan_revision": plan_revision, "skipped": ["landing-gate-closed", "landing-gate-unknown"]})
    );
    let answer = operator.call(
        "yard_attempt_accept",
        json!({"project": project.path,
        "ticket": "Y-1", "attention": omission["attention"], "target": "main",
        "base": final_item["payload"]["base"], "head": final_item["payload"]["head"],
        "proof": final_item["payload"]["proof"], "revision": final_item["payload"]["revision"],
        "plan_revision": plan_revision, "landing": landing["execution"],
        "old": landing["prepared"]["old"], "merged": merged,
        "accept_skipped": ["landing-gate-closed", "landing-gate-unknown"],
        "text": "Accept these exact merged omissions"}),
    );
    assert!(answer.get("error").is_none(), "{answer}");
    decisions.push(
        project.rows(&format!(
            "SELECT * FROM attention WHERE id = {}",
            omission["attention"]
        ))[0]
            .clone(),
    );
    let recorded = watch.event("landing.recorded", &[]);
    assert_eq!(project.canonical_head(), merged);
    assert_eq!(project.rows(&format!(
        "SELECT node, head, outcome FROM execution WHERE attempt = {attempt} AND reason = 'landing' AND kind = 'gate' ORDER BY id"
    )), vec![json!({"node": "landing-gate-other", "head": merged, "outcome": "pass"})]);
    let log = std::fs::read_to_string(&commands).unwrap();
    for (name, count) in [("closed", 9), ("other", 5), ("unknown", 5)] {
        assert_eq!(log.lines().filter(|line| *line == name).count(), count);
    }
    git(
        &project.canonical(),
        &[
            "merge-base",
            "--is-ancestor",
            final_item["payload"]["head"].as_str().unwrap(),
            "refs/heads/main",
        ],
    );
    assert_eq!(
        project.rows("SELECT checks FROM approval ORDER BY id DESC LIMIT 1"),
        vec![json!({"checks": final_item["payload"]["checks"].to_string()})]
    );
    let approval_rows = project.rows("SELECT * FROM approval ORDER BY id");
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE kind = 'gate' AND id <= {} ORDER BY id",
            original.last().unwrap()["id"]
        )),
        original
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id <= {} ORDER BY id",
            original_checks.last().unwrap()["id"]
        )),
        original_checks
    );
    let cleanup = watch.find("reuse selection cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == attempt
            && event["seq"].as_i64().unwrap() > recorded["seq"].as_i64().unwrap()
    });
    watch.find("reuse selection cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    machine.kill();
    machine.start();
    assert_eq!(
        project.rows("SELECT * FROM approval ORDER BY id"),
        approval_rows
    );
    for failed in &accepted {
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM \"check\" WHERE id = {}",
                failed["id"]
            )),
            vec![failed.clone()]
        );
    }
    for execution in failed_executions {
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM execution WHERE id = {}",
                execution["id"]
            )),
            vec![execution]
        );
    }
    for decision in decisions {
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM attention WHERE id = {}",
                decision["id"]
            )),
            vec![decision]
        );
    }
}

/// Seats publish no findings; the implementer commits feature.txt and writes
/// its proof, or runs a repair once when its execution's prompt carries that
/// repair's key. Keys are tried in order, so the newest prompt's key comes
/// first: a fresh session's brief repeats the ticket body.
fn worker(repairs: Vec<(&'static str, ToolCall)>) -> impl Fn(&ModelRequest) -> Reply + Send + Sync {
    let repairs = std::sync::Mutex::new(
        repairs
            .into_iter()
            .map(|(key, call)| (key, Some(call)))
            .collect::<Vec<_>>(),
    );
    move |request| {
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() {
            let prompt = request.last_user();
            let mut repairs = repairs.lock().unwrap();
            if let Some((_, call)) = repairs.iter_mut().find(|(key, _)| prompt.contains(key)) {
                return Reply::Tools(vec![call.take().expect("one repair per key")]);
            }
        }
        act(
            request,
            vec![bash(
                "cd /workspace && printf 'feature\\n' > feature.txt && git add -A \
                 && git commit -q -m 'Add feature' && printf 'first' > /yard/proof/evidence.txt \
                 && echo committed",
            )],
        )
    }
}

fn unreviewed(extra: &str) -> String {
    config(extra).replace("review = [\"correctness\"]", "review = \"none\"")
}

fn auto(config: String) -> String {
    config.replace("approve = \"manual\"", "approve = \"auto\"")
}

/// The next attention item, which must be an approval: its data.
#[track_caller]
fn approval(watch: &mut Watch) -> Value {
    let raised = watch.attention();
    assert_eq!(raised["data"]["kind"], "approval", "{raised}");
    raised["data"].clone()
}

fn head(item: &Value) -> String {
    item["payload"]["head"].as_str().unwrap().to_string()
}

fn proof(item: &Value) -> String {
    item["payload"]["proof"].as_str().unwrap().to_string()
}

/// Gate and review executions on `head`, in order: (kind, ticket revision).
fn judged(project: &Project, head: &str) -> Vec<(String, i64)> {
    project
        .rows(&format!(
            "SELECT kind, ticket_revision FROM execution
             WHERE kind IN ('gate', 'review') AND head = '{head}' ORDER BY id"
        ))
        .iter()
        .map(|row| {
            (
                row["kind"].as_str().unwrap().to_string(),
                row["ticket_revision"].as_i64().unwrap(),
            )
        })
        .collect()
}

fn kinds(project: &Project) -> Vec<String> {
    project
        .rows("SELECT kind FROM execution WHERE kind IN ('gate', 'review') ORDER BY id")
        .iter()
        .map(|row| row["kind"].as_str().unwrap().to_string())
        .collect()
}

fn last_seq(project: &Project) -> i64 {
    project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap()
}

/// A passed gate and review, then a new commit, a ticket edit, two synced
/// template changes and a proof-only repair. Only candidate changes rejudge.
///
/// The new commit keeps the tree, so only the head tells the two candidates
/// apart. The edit's implementer run makes no commit: the same head is
/// judged afresh at the new revision, without `stopped:unchanged`, so the
/// revision binding shows in the revision each judging execution records.
/// Gate, workflow and seat sync keeps the admitted judgments. The proof-only repair
/// reruns the gate on the same head under the new proof digest, and the old
/// check does not count. Delayed approves naming the old proof or the old
/// head write nothing; approving the current head and proof succeeds.
///
/// Sabotage: drop `head` from `checks::current`'s match; the empty commit
/// reaches approval on the first commit's gate and review. Make
/// `admit::steer` set nothing; no implementer runs for the edit and no
/// approval follows it. Keep `stop("unchanged")` for edit runs; the attempt
/// stops. Record the old revision on the gate and review executions; `judged`
/// no longer reads `revision + 1`. Make admit::approval_item accept an old
/// head; the stale approve writes.
#[test]
fn a_new_head_revision_or_proof_rejudges_but_template_sync_keeps_checks() {
    let machine = Machine::new(
        "g2-identity",
        worker(vec![
            (
                "Change the proof",
                bash("printf 'second' > /yard/proof/evidence.txt"),
            ),
            ("nothing else", bash("cd /workspace && echo unchanged")),
            (
                "Once more",
                bash("cd /workspace && git commit -q --allow-empty -m Again && echo committed"),
            ),
        ]),
    );
    machine.start();
    let admitted = config(GATE).replace(
        "[workflows.default]\n",
        "[workflows.default]\ninstructions = \"Admitted workflow instructions\"\nmax_session_executions = 1\n",
    );
    let project = Project::new(&machine, "p", &admitted);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create feature.txt",
    ]);
    let first_item = approval(&mut watch);
    let first = head(&first_item);
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
        "Y-1",
        "--head",
        &first,
        "--proof",
        &proof(&first_item),
        "--text",
        "Once more",
    ]);
    let second = approval(&mut watch);
    let candidate = head(&second);

    let canonical = project.canonical();
    let tree = |commit: &str| git(&canonical, &["rev-parse", &format!("{commit}^{{tree}}")]);
    assert_ne!(candidate, first);
    assert_eq!(tree(&candidate), tree(&first));
    let once = vec![("gate".to_string(), 1), ("review".to_string(), 1)];
    assert_eq!(judged(&project, &first), once);
    assert_eq!(judged(&project, &candidate), once);

    // A ticket edit supersedes the approval and the implementer runs.
    let revision = project.json(&["ticket", "show", "Y-1"])["revision"]
        .as_i64()
        .unwrap();
    assert_eq!(second["payload"]["revision"], revision);
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision.to_string(),
        "--body",
        "Create feature.txt and nothing else",
    ]);
    let third = approval(&mut watch);

    assert_eq!(head(&third), candidate);
    assert_eq!(third["payload"]["revision"], revision + 1);
    let superseded =
        project.rows("SELECT resolution FROM attention WHERE kind = 'approval' ORDER BY id");
    assert_eq!(superseded[1]["resolution"], "superseded");
    let reasons =
        project.rows("SELECT reason FROM execution WHERE kind = 'implementation' ORDER BY id");
    assert_eq!(reasons.last().unwrap()["reason"], "edit");
    assert_eq!(
        judged(&project, &candidate),
        vec![
            ("gate".to_string(), revision),
            ("review".to_string(), revision),
            ("gate".to_string(), revision + 1),
            ("review".to_string(), revision + 1),
        ]
    );

    // A delayed edit on the old revision cannot overwrite the accepted edit.
    // Sabotage: drop admit::edit's revision comparison; this edit writes.
    let seq = last_seq(&project);
    let refused = project.refused(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision.to_string(),
        "--body",
        "Overwrite the accepted edit",
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], revision);
    assert_eq!(refused["data"]["current"], revision + 1);
    assert_eq!(last_seq(&project), seq);
    assert_eq!(
        project.json(&["ticket", "show", "Y-1"])["body"],
        "Create feature.txt and nothing else"
    );

    // G2: late template lookup or digest invalidation would add judgments.
    // The subsequent proof repair and its attention form the observation barrier.
    project.reconfigure(&config(&GATE.replace("test -f", "test -s")));
    project.reconfigure(
        &config(&GATE.replace("test -f", "test -s"))
            .replace(
                "Review for correctness.",
                "Review for correctness and naming.",
            )
            .replace(
                "[workflows.default]\n",
                "[workflows.default]\ninstructions = \"New workflow instructions\"\nmax_session_executions = 1\n",
            ),
    );
    let fifth = third.clone();

    // A repair that changes only `/yard/proof` is a new candidate.
    // Sabotage: match only base and head in `checks::current`; the old gate
    // counts for the new proof and no gate reruns.
    let old_proof = proof(&fifth);
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
        "Y-1",
        "--head",
        &candidate,
        "--proof",
        &old_proof,
        "--text",
        "Change the proof",
    ]);
    let sixth = approval(&mut watch);
    assert_eq!(head(&sixth), candidate);
    let repair_prompt = machine
        .model
        .requests()
        .iter()
        .rev()
        .find(|request| request.opens() && !request.has_tool("yard_publish_review"))
        .unwrap()
        .prompt();
    assert!(
        repair_prompt.contains("Admitted workflow instructions"),
        "{repair_prompt}"
    );
    assert!(
        !repair_prompt.contains("New workflow instructions"),
        "{repair_prompt}"
    );
    assert_eq!(
        kinds(&project),
        [
            "gate", "review", "gate", "review", "gate", "review", "gate", "review"
        ]
    );
    assert_eq!(
        project
            .rows("SELECT DISTINCT plan_revision FROM execution WHERE kind IN ('gate', 'review')"),
        vec![json!({"plan_revision": 1})]
    );
    let gates = project.rows(
        "SELECT c.id AS id, e.head AS head, e.proof AS proof FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.kind = 'gate' ORDER BY c.id DESC LIMIT 2",
    );
    assert_eq!(gates[0]["head"], candidate.as_str());
    assert_eq!(gates[1]["head"], candidate.as_str());
    assert_ne!(gates[0]["proof"], gates[1]["proof"]);
    assert_eq!(sixth["payload"]["checks"][0], gates[0]["id"]);
    assert_eq!(sixth["payload"]["proof"], gates[0]["proof"]);

    // A delayed approve naming the old proof writes nothing.
    // Sabotage: drop the proof comparison in `admit::approval_item`; the
    // delayed approve names the old proof and is given anyway.
    let seq = last_seq(&project);
    let refused = project.refused(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &candidate, "--proof",
        &old_proof,
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], old_proof.as_str());
    assert_eq!(refused["data"]["current"], sixth["payload"]["proof"]);
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    let refused = project.refused(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &first,
    ]);
    assert_eq!(refused["code"], "stale", "{refused}");
    assert_eq!(refused["data"]["expected"], first.as_str());
    assert_eq!(refused["data"]["current"], candidate.as_str());
    assert_eq!(last_seq(&project), seq);
    assert!(project.rows("SELECT id FROM approval").is_empty());

    let new_proof = proof(&sixth);
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &candidate, "--proof",
        &new_proof,
    ]);
    assert_eq!(
        project.rows("SELECT head, proof FROM approval"),
        vec![json!({ "head": candidate, "proof": new_proof })]
    );
}

/// An automatic approval, a landing held in its host gate, and a sync that
/// protects its path: the landing withdraws the approval when it records its
/// intent, raises `approval`, and reruns no check. Control: the operator's
/// approval of the same head lands.
///
/// Sabotage: make `queue::land` re-read the approval at its intent against
/// the configuration it started with; the landing records an intent and
/// retires on the moved target instead of withdrawing. Hash
/// `protected_paths` into the gate digest; the sync reruns the candidate
/// gate.
#[test]
fn a_sync_that_protects_an_auto_approved_path_withdraws_it() {
    let machine = notes_machine("g2-protect");
    let (project, mut watch, mut release, head) = held_after_policy_change(
        &machine,
        "[gates.check]\ncommand = \"test -f docs/notes.md\"\nstage = \"candidate\"\n\n",
        |config| config.replace("\".pi/\"]", "\".pi/\", \"docs/\"]"),
    );
    release.write_all(b"go\n").unwrap();
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn", "{ended}");
    let raised = approval(&mut watch);
    assert_eq!(raised["payload"]["head"], head.as_str());
    assert_eq!(raised["payload"]["plan_revision"], 1);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|request| request.opens())
            .count(),
        1
    );
    assert_eq!(
        project.rows("SELECT actor, state FROM approval"),
        vec![json!({ "actor": "auto", "state": "withdrawn" })]
    );
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'gate' AND reason = 'candidate'")
            .len(),
        1,
        "a check reran"
    );

    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    release.write_all(b"go\n").unwrap();
    watch.event("landing.recorded", &[]);
    assert_eq!(project.json(&["ticket", "show", "Y-1"])["state"], "done");
}

fn notes_machine(test: &str) -> Machine {
    Machine::new(test, |request| {
        act(
            request,
            vec![commit_file("docs/notes.md", "notes\n", "Add notes")],
        )
    })
}

/// An automatic approval whose landing is held in its host gate, after a
/// sync that changes policy: the project, its watch, the gate's release and
/// the approved head. `gates` goes before the held landing gate.
fn held_after_policy_change<'m>(
    machine: &'m Machine,
    gates: &str,
    change: impl Fn(String) -> String,
) -> (Project<'m>, Watch, std::fs::File, String) {
    let hold = machine.root.join("hold");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&hold)
            .status()
            .unwrap()
            .success()
    );
    // Held open for writing, so each line released stays until a gate reads it.
    let release = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&hold)
        .unwrap();
    let gates = format!(
        "{gates}[gates.held]\ncommand = \"read line < {}\"\nruns_in = \"host\"\n",
        hold.display()
    );
    machine.start();
    let base = auto(unreviewed(&gates));
    let project = Project::new(machine, "p", &base);
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add notes"]);
    let approved = watch.event("approval.given", &[("actor", "auto")]);
    let head = approved["data"]["head"].as_str().unwrap().to_string();
    watch.event("execution.started", &[("name", "held")]);

    project.reconfigure(&change(base));
    (project, watch, release, head)
}

/// G2: manual policy synced while the auto-approved candidate's landing is
/// held withdraws approval on the same plan, with no new model turn or check.
/// Sabotage: read approve from the admitted plan; the held landing continues.
/// The protected-path scenario exercises the other policy input.
#[test]
fn manual_policy_withdraws_auto_approval_without_rejudgment() {
    let machine = notes_machine("g2-manual");
    let (project, mut watch, mut release, head) = held_after_policy_change(
        &machine,
        "[gates.check]\ncommand = \"test -f docs/notes.md\"\nstage = \"candidate\"\n\n",
        |config| config.replace("approve = \"auto\"", "approve = \"manual\""),
    );
    release.write_all(b"go\n").unwrap();
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn");
    let raised = approval(&mut watch);
    assert_eq!(raised["payload"]["head"], head);
    assert_eq!(raised["payload"]["plan_revision"], 1);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|request| request.opens())
            .count(),
        1
    );
    assert_eq!(
        project.rows("SELECT actor, state FROM approval"),
        vec![json!({"actor": "auto", "state": "withdrawn"})]
    );
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE kind = 'gate' AND reason = 'candidate'")
            .len(),
        1
    );
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &head,
    ]);
    release.write_all(b"go\n").unwrap();
    watch.event("landing.recorded", &[]);
}

/// G2: a delayed acceptance cannot cross an edit, even before the held edit
/// worker changes the head. After release it cannot cross the new head either.
/// Refusals name the changed identity and write no rows; the current item's
/// repair exit and its completed approval supply the same-project control.
/// Expected revisions are the two CLI edits; heads come from host git.
/// Sabotage: omit ticket revision or head from candidate decisions, or let a
/// repaired decision carry to the next candidate's check.
#[test]
fn delayed_candidate_acceptance_names_the_changed_identity_and_writes_nothing() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-accept-stale", move |request| {
        if request
            .prompt()
            .contains("Your workspace is read-only. Plan the work")
        {
            return planned(request);
        }
        if request.has_tool("yard_publish_review") {
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.last_user().contains("The operator edited") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![bash(
                    "cd /workspace && git commit --allow-empty -qm Edited",
                )])),
            );
        }
        let command = if request.last_user().contains("The gate check failed") {
            "cd /workspace && touch ready.txt && git add -A && git commit -qm Ready"
        } else {
            "cd /workspace && touch feature.txt && git add -A && git commit -qm Feature"
        };
        act(request, vec![bash(command)])
    });
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config("[gates.check]\ncommand = \"test -f ready.txt\"\nstage = \"candidate\"\n"),
    );
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Delayed acceptance",
        "--workflow",
        "plan",
    ]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == 1
    });
    watch.find("cleanup reservation released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let mut operator = machine.operator();
    operator.initialize();
    let admitted = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workflow": "default",
            "overrides": {"candidate-gate-check": {"operator_failure": true}}
        }),
    );
    assert!(admitted.get("error").is_none(), "{admitted}");
    let first = watch.attention();
    assert_eq!(first["data"]["kind"], "candidate");
    let first_item = project.json(&["attempt", "show", "Y-1"])["attention"][0].clone();
    let first_head = git(
        &project.path.join(".yard/local/attempts/2/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    let delayed = json!({
        "project": project.path, "ticket": "Y-1", "attention": first_item["attention"], "check": first_item["payload"]["evidence"]["check"]["check"], "plan_revision": 1,
        "target": "main", "head": first_head, "proof": first_item["payload"]["proof"], "text": "Accept the old failure"
    });
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        "1",
        "--body",
        "New brief",
    ]);
    hold.wait_held();
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_accept", delayed.clone());
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "revision", "expected": 1, "current": 2})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    hold.release();
    let second = watch.attention();
    assert_eq!(second["data"]["kind"], "candidate");
    let second_head = git(
        &project.path.join(".yard/local/attempts/2/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    assert_ne!(first_head, second_head);
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_accept", delayed);
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "head", "expected": first_head, "current": second_head})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    let item = project.json(&["attempt", "show", "Y-1"])["attention"][0].clone();
    let repaired = operator.call("yard_attempt_repair", json!({
        "project": project.path, "ticket": "Y-1", "attention": item["attention"], "check": item["payload"]["evidence"]["check"]["check"], "plan_revision": 1,
        "target": "main", "head": second_head, "proof": item["payload"]["proof"], "text": "Use the admitted repair route"
    }));
    assert_eq!(repaired["result"]["accepted"], false, "{repaired}");
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval");
    let payload = &approval["data"]["payload"];
    let head = payload["head"].as_str().unwrap();
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{head}:ready.txt")]
        ),
        ""
    );
    project.json(&[
        "attempt", "approve", "Y-1", "--target", "main", "--head", head,
    ]);
    watch.event("landing.recorded", &[]);
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", head, "refs/heads/main"],
    );
}

/// G2/G9/G13: selected ordinary and packaged reports consume different immutable
/// publications around the existing cleanup handoff. Command-only reuse never
/// enters either report harness, their selected sources or their model policy.
/// Sabotage: reuse changed/unknown inputs, invalidate every result, copy a report
/// as a new pass, retain old approval, or infer publication/landing from writes.
/// One machine uses the existing package image fixture without a debugger build.
/// Git and fixture bytes supply identities; host model counts expose replay.
#[test]
fn selected_cleanup_reuses_only_unchanged_actual_judgment_inputs() {
    use std::sync::{Arc, Mutex};
    let counts = Arc::new(Mutex::new(
        std::collections::BTreeMap::<String, usize>::new(),
    ));
    let observed = counts.clone();
    let machine = Machine::new("g2-cleanup-inputs", move |request| {
        let model = request.body["model"].as_str().unwrap();
        if request.opens() {
            *observed.lock().unwrap().entry(model.into()).or_default() += 1;
        }
        let report = || {
            tool(
                "yard_publish_report",
                json!({"outcome": "complete",
            "report": {"summary": "Fixture code and proof observed", "findings": []}}),
            )
        };
        if request
            .last_tool_result()
            .is_some_and(|(name, _)| name == "yard_publish_report")
        {
            return Reply::Text("done".into());
        }
        match model {
            "fake-model" => {
                if request.opens() {
                    Reply::Tools(vec![report()])
                } else {
                    Reply::Text("planned".into())
                }
            }
            "implementer-selected" => {
                if request.opens() {
                    Reply::Tools(vec![bash(
                        "printf 'before\n' > chain.txt && git add chain.txt && git commit -qm Implement && printf 'before\n' > /yard/proof/chain.txt",
                    )])
                } else {
                    Reply::Text("done".into())
                }
            }
            "cleanup-selected" => {
                if request.opens() {
                    // The same operation is a no-op on its second invocation.
                    Reply::Tools(vec![bash(
                        "printf 'after\n' > chain.txt && git add chain.txt && (git diff --cached --quiet || git commit -qm Cleanup)",
                    )])
                } else {
                    Reply::Tools(vec![report()])
                }
            }
            "repair-selected" => {
                if request.opens() {
                    Reply::Tools(vec![bash(
                        "test \"$(cat chain.txt)\" = after && printf 'after\n' > /yard/proof/chain.txt",
                    )])
                } else {
                    Reply::Text("done".into())
                }
            }
            "audit-current" | "audit-unknown" => {
                if request.opens() {
                    Reply::Tools(vec![bash(
                        "test \"$(cat chain.txt)\" = \"$(cat /yard/proof/chain.txt)\" && printf 'audit-ok\n'",
                    )])
                } else {
                    assert!(request.last_tool_result().unwrap().1.contains("audit-ok"));
                    Reply::Tools(vec![report()])
                }
            }
            "audit-original" => {
                assert!(!request.has_tool("bash"));
                if request.opens() {
                    Reply::Tools(vec![tool(
                        "read_declared",
                        json!({"file": "source:chain.txt"}),
                    )])
                } else if request.last_tool_result().unwrap().0 == "read_declared" {
                    assert!(request.last_tool_result().unwrap().1.contains("before"));
                    Reply::Tools(vec![report()])
                } else {
                    Reply::Text("done".into())
                }
            }
            other => panic!("unexpected model {other}"),
        }
    });
    machine.start();
    let project = super::g08_durable_package::fixture(&machine);
    machine.wrapper(
        "pinfold",
        &format!(
            r#"
if [ "$1 $2" = 'image build' ]; then
    previous=''
    for arg do
        if [ "$previous" = '--context' ]; then cp -R '{root}/build-inputs' "$arg/build-inputs"; fi
        previous="$arg"
    done
fi
"#,
            root = machine.root.display()
        ),
    );
    super::g08_durable_package::image(&project, "");
    let mut configuration = String::from(
        "\n[gates.verify]\ncommand = 'test \"$(cat chain.txt)\" = after'\nclosed_inputs = 'fixture-runtime-v1'\n",
    );
    for (role, output) in [
        ("implementer", "candidate"),
        ("cleanup", "report"),
        ("repair", "candidate"),
    ] {
        configuration.push_str(&format!(
            r#"
[agents.{role}]
harness = "pi"
provider = "openrouter"
model = "{role}-selected"
[profiles.{role}]
access = "write-allowed"
outcomes = ["complete"]
[profiles.{role}.outputs]
result = "{output}"
[profiles.{role}.runner]
kind = "agent"
agent = "{role}"
instructions = "Run the selected fixture role."
skills = ["trusted/selected.md"]
"#
        ));
    }
    for name in ["current", "original", "unknown"] {
        configuration.push_str(&format!(
            r#"
[agents.audit-{name}]
harness = "pi"
provider = "openrouter"
model = "audit-{name}"
"#
        ));
        if name != "unknown" {
            configuration.push_str(&format!(
                r#"[agents.audit-{name}.judgment_inputs]
configuration = "fixture-pinfold-v1"
model = "scripted-fixture-v1"
"#
            ));
        }
        configuration.push_str(&format!(
            r#"
[profiles.audit-{name}]
access = "read-only"
outcomes = ["complete"]
[profiles.audit-{name}.outputs]
result = "report"
[profiles.audit-{name}.runner]
kind = "{}"
agent = "audit-{name}"
instructions = "Judge only the declared fixture input."
"#,
            if name == "original" {
                "packaged"
            } else {
                "agent"
            }
        ));
        configuration.push_str(if name == "original" {
            "package = 'durable-audit'\nskill = 'trusted/selected.md'\nreferences = ['trusted/reference.txt']\nfiles = [{root = 'source', path = 'chain.txt'}]\n"
        } else { "skills = ['trusted/selected.md']\n" });
    }
    project.reconfigure(&unreviewed(&configuration));
    let mut graph = super::g13_runs_where_its_row_says::sequential_handoff_graph();
    graph["inputs"]["candidate"] = json!("candidate");
    for name in ["current", "original", "unknown"] {
        let node = format!("cleanup-audit-{name}");
        let candidate = if name == "original" {
            json!({"node": "implementer", "output": "result", "kind": "candidate"})
        } else {
            json!({"output": "candidate", "kind": "candidate"})
        };
        graph["nodes"][&node] = json!({"profile": format!("audit-{name}"),
            "inputs": {"candidate": candidate},
            "requires": ["candidate"]});
        graph["nodes"]["implementer"]["routes"]["complete"]
            .as_array_mut()
            .unwrap()
            .push(json!({"to": node}));
    }
    // This fixture selects the native landing gate only.
    graph["nodes"]
        .as_object_mut()
        .unwrap()
        .remove("candidate-gate-verify");
    graph["nodes"]["repair"]["routes"]["complete"] = json!([{"to": "landing-gate-verify"}]);
    let mut watch = project.watch(0);
    let mut operator = machine.operator();
    operator.initialize();
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Cleanup actual inputs",
        "--workflow",
        "plan",
    ]);
    let planned = watch.event("attempt.ended", &[("outcome", "planned")]);
    let removed = watch.find("planning cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planned["attempt"]
    });
    watch.find("planning clone released", |event| {
        event["event"] == "execution.released" && event["execution"] == removed["execution"]
    });
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "graph": graph}),
    );
    assert!(started.get("error").is_none(), "{started}");
    let attempt = started["result"]["attempt"].as_i64().unwrap();
    let first = approval(&mut watch);
    let rows =
        project.rows("SELECT * FROM execution WHERE node LIKE 'cleanup-audit-%' ORDER BY id");
    let original = |node: &str| rows.iter().find(|row| row["node"] == node).unwrap();
    let before_head = original("cleanup-audit-current")["head"]
        .as_str()
        .unwrap()
        .to_string();
    let before_proof = original("cleanup-audit-current")["proof"]
        .as_str()
        .unwrap()
        .to_string();
    let after_head = head(&first);
    let after_proof = proof(&first);
    let consumed: Value = serde_json::from_str(
        original("cleanup-audit-current")["inputs"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let producer = consumed["candidate"]["source"]["execution"]
        .as_i64()
        .unwrap();
    assert_eq!(
        git(
            &project.canonical(),
            &[
                "rev-parse",
                &format!("refs/yard/candidates/{attempt}/{producer}")
            ]
        )
        .trim(),
        before_head
    );
    let snapshots = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/proof-snapshots"));
    for (digest, contents) in [(&before_proof, "before\n"), (&after_proof, "after\n")] {
        assert_eq!(
            std::fs::read_to_string(snapshots.join(digest).join("chain.txt")).unwrap(),
            contents
        );
    }
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{before_head}:chain.txt")]
        ),
        "before\n"
    );
    assert_eq!(
        git(
            &project.canonical(),
            &["show", &format!("{after_head}:chain.txt")]
        ),
        "after\n"
    );
    assert_ne!(before_proof, after_proof);
    let changed = rows
        .iter()
        .rev()
        .find(|row| row["node"] == "cleanup-audit-current")
        .unwrap();
    assert_eq!(changed["head"], after_head);
    assert_eq!(changed["proof"], after_proof);
    assert_eq!(changed["outcome"], "complete");
    let report: Value = serde_json::from_str(changed["report"].as_str().unwrap()).unwrap();
    assert_eq!(report["head"], after_head);
    assert_eq!(report["proof"], after_proof);
    let assert_reference = |row: &Value, earlier: &Value| {
        assert_eq!(row["outcome"], "reused");
        let result: Value = serde_json::from_str(row["result"].as_str().unwrap()).unwrap();
        assert_eq!(
            result["reference"],
            json!({"execution": earlier["id"], "result": {"execution": earlier["id"]}})
        );
        assert_eq!(result["outputs"], json!({}));
        assert_eq!(result["judgment"], Value::Null);
        assert_eq!(row["report"], Value::Null);
    };
    assert_reference(
        rows.iter()
            .rev()
            .find(|row| row["node"] == "cleanup-audit-original")
            .unwrap(),
        original("cleanup-audit-original"),
    );
    let before = project.rows("SELECT * FROM approval");
    let stale = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path,
        "ticket": "Y-1", "target": "main", "head": before_head, "proof": before_proof}),
    );
    assert_eq!(stale["error"]["code"], "stale", "{stale}");
    assert_eq!(
        stale["error"]["data"],
        json!({"identity": "head", "expected": before_head, "current": after_head})
    );
    assert_eq!(project.rows("SELECT * FROM approval"), before);
    project.json(&[
        "attempt",
        "reject",
        "Y-1",
        "--target",
        "main",
        "--head",
        &after_head,
        "--proof",
        &after_proof,
        "--text",
        "Run the no-op selected cleanup",
    ]);
    let second = approval(&mut watch);
    assert_eq!(head(&second), after_head);
    assert_eq!(proof(&second), after_proof);
    let publications = project.rows("SELECT id, result FROM execution WHERE node = 'repair' AND outcome = 'candidate' ORDER BY id");
    assert_eq!(publications.len(), 2);
    assert_ne!(publications[0]["id"], publications[1]["id"]);
    for publication in &publications {
        assert_eq!(
            git(
                &project.canonical(),
                &[
                    "rev-parse",
                    &format!("refs/yard/candidates/{attempt}/{}", publication["id"])
                ]
            )
            .trim(),
            after_head
        );
    }
    let current =
        project.rows("SELECT * FROM execution WHERE node LIKE 'cleanup-audit-%' ORDER BY id");
    assert_reference(
        current
            .iter()
            .rev()
            .find(|row| row["node"] == "cleanup-audit-current")
            .unwrap(),
        changed,
    );
    assert_reference(
        current
            .iter()
            .rev()
            .find(|row| row["node"] == "cleanup-audit-original")
            .unwrap(),
        original("cleanup-audit-original"),
    );
    for old in &rows {
        assert_eq!(
            project.rows(&format!("SELECT * FROM execution WHERE id = {}", old["id"]))[0],
            *old
        );
    }
    assert!(project.rows("SELECT * FROM \"check\" WHERE execution IN (SELECT id FROM execution WHERE outcome = 'reused')").is_empty());
    assert_eq!(
        *counts.lock().unwrap(),
        std::collections::BTreeMap::from([
            ("fake-model".into(), 1),
            ("implementer-selected".into(), 1),
            ("cleanup-selected".into(), 2),
            ("repair-selected".into(), 2),
            ("audit-current".into(), 2),
            ("audit-original".into(), 1),
            ("audit-unknown".into(), 3)
        ])
    );
    project.json(&[
        "attempt",
        "approve",
        "Y-1",
        "--target",
        "main",
        "--head",
        &after_head,
        "--proof",
        &after_proof,
    ]);
    watch.event("landing.recorded", &[]);
    assert_eq!(
        git(&project.canonical(), &["show", "refs/heads/main:chain.txt"]),
        "after\n"
    );
}
