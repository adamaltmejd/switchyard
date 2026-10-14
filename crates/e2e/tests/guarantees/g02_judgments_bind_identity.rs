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
    machine.wrapper(
        "pinfold",
        &format!(
            "if [ \"$1 $2\" = 'box exec' ]; then
  case \"$*\" in
    *reuse-closed*) printf 'closed\\n' >> '{}' ;;
    *reuse-other*) printf 'other\\n' >> '{}' ;;
    *reuse-unknown*) printf 'unknown\\n' >> '{}' ;;
  esac
fi",
            commands.display(),
            commands.display(),
            commands.display()
        ),
    );
    machine.start();
    let configuration = unreviewed(
        "[gates.closed]\ncommand = 'test -f feature.txt # reuse-closed'\nstage = 'candidate'\nclosed_inputs = 'runtime-v1'\n\
         [gates.other]\ncommand = 'test -f feature.txt # reuse-other'\nstage = 'candidate'\nclosed_inputs = 'runtime-v1'\n\
         [gates.unknown]\ncommand = 'test -f feature.txt # reuse-unknown'\nstage = 'candidate'\n",
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
        "test -f feature.txt # reuse-closed",
        "test -f feature.txt && test -s feature.txt # reuse-closed",
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
        "test -f feature.txt && test -s feature.txt # reuse-closed",
        "exit 7 # reuse-closed",
    );
    project.reconfigure(&failed_definition);
    let source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let mut graph = project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"]["plan"].clone();
    graph["nodes"]
        .as_object_mut()
        .unwrap()
        .remove("candidate-gate-unknown");
    graph["nodes"]["candidate-gate-other"]["routes"]
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
        assert_eq!(
            approved["payload"]["skipped"],
            json!(["candidate-gate-unknown"])
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
    // current approval. Sabotage: drop references from queue qualification.
    let current = project.json(&["attempt", "show", "Y-1"]);
    let approved = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path,
        "ticket": "Y-1", "target": "main", "base": final_item["payload"]["base"],
        "head": final_item["payload"]["head"], "proof": final_item["payload"]["proof"],
        "revision": final_item["payload"]["revision"], "plan_revision": plan_revision,
        "attention": current["attention"][0]["attention"],
        "accept_skipped": ["candidate-gate-unknown"], "text": "Accept this exact omitted gate"}),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    watch.event("landing.recorded", &[]);
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
    machine.kill();
    machine.start();
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

/// G2: equivalent consumers on one candidate, then changed declared
/// configuration. Existing adoption cases cannot show stable runtime
/// identity across distinct launches. Expected inputs come from fixture
/// literals, Git and real Pinfold/runtime. Observe store and operator MCP.
/// Sabotage: accept stdout claims, omit the declaration, include box names,
/// or overwrite launch/result/check history. Dropping the exact base source
/// must fail even when host Git gives two base commits the same tree.
/// Sabotage: bind a gate to the producer's input proof instead of its published
/// candidate proof. Their original public records must agree on publication.
#[test]
fn closed_box_gate_inputs_are_declared_stable_and_immutable() {
    let machine = Machine::new("g2-closed-inputs", worker(vec![]));
    let observed = machine.root.join("ready.jsonl");
    machine.wrapper(
        "pinfold",
        &format!(
            "if [ \"$1 $2\" = 'box up' ]; then
  fifo='{}.'$$
  mkfifo \"$fifo\" || exit $?
  trap 'rm -f \"$fifo\"' 0
  while IFS= read -r line; do
    printf '%s\\n' \"$line\" >> '{}'
    printf '%s\\n' \"$line\"
  done < \"$fifo\" &
  reader=$!
  \"$REAL\" \"$@\" > \"$fifo\"
  result=$?
  wait \"$reader\"
  exit \"$result\"
fi",
            machine.root.join("closed-input-images.fifo").display(),
            observed.display()
        ),
    );
    machine.start();
    let command = "test -f feature.txt && echo '{\"reuse\":\"eligible\",\"runtime\":\"forged\"}'";
    let configuration = unreviewed(&format!(
        "[gates.closed]\ncommand = {}\nstage = 'candidate'\nclosed_inputs = 'operator-runtime-v1'\n\n[gates.again]\ncommand = {}\nstage = 'candidate'\nclosed_inputs = 'operator-runtime-v1'\n\n[gates.unknown]\ncommand = {}\nstage = 'candidate'\n",
        serde_json::to_string(command).unwrap(),
        serde_json::to_string(command).unwrap(),
        serde_json::to_string(command).unwrap()
    ));
    let project = Project::new(&machine, "p", &configuration);
    let admitted_source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.git(&["commit", "--quiet", "--allow-empty", "-m", "Same-tree base"]);
    let base_source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    project.json(&["sync"]);
    let base_tree = project
        .git(&["rev-parse", &format!("{base_source}^{{tree}}")])
        .trim()
        .to_string();
    assert_ne!(base_source, admitted_source);
    assert_eq!(
        base_tree,
        project
            .git(&["rev-parse", &format!("{admitted_source}^{{tree}}")])
            .trim()
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Feature"]);
    approval(&mut watch);
    let original = project.rows("SELECT * FROM execution WHERE kind = 'gate' ORDER BY id");
    let checks = project.rows("SELECT * FROM \"check\" ORDER BY id");
    let closed = original
        .iter()
        .find(|r| r["node"] == "candidate-gate-closed")
        .unwrap();
    let evidence: Value = serde_json::from_str(closed["verification"].as_str().unwrap()).unwrap();
    let unknown = original
        .iter()
        .find(|r| r["node"] == "candidate-gate-unknown")
        .unwrap();
    let unknown_evidence: Value =
        serde_json::from_str(unknown["verification"].as_str().unwrap()).unwrap();
    assert_eq!(unknown_evidence["reuse"], "disabled");
    assert_eq!(unknown["outcome"], "pass");
    assert_eq!(closed["outcome"], "pass");
    assert_eq!(evidence["reuse"], "eligible");
    let again = original
        .iter()
        .find(|r| r["node"] == "candidate-gate-again")
        .unwrap();
    let again_evidence: Value =
        serde_json::from_str(again["verification"].as_str().unwrap()).unwrap();
    assert_ne!(again["handle"], closed["handle"]);
    assert_eq!(again_evidence["reuse"], "eligible");
    assert_eq!(again_evidence["identity"], evidence["identity"]);
    assert_eq!(evidence["identity"]["declaration"], "operator-runtime-v1");
    assert_eq!(evidence["launch"]["command"], command);
    let tree = git(
        &project.canonical(),
        &["rev-parse", "refs/yard/candidates/1^{tree}"],
    )
    .trim()
    .to_string();
    assert_eq!(evidence["identity"]["candidate"]["tree"], tree);
    assert_eq!(evidence["identity"]["candidate"]["base"], base_source);
    assert_ne!(evidence["identity"]["candidate"]["base"], admitted_source);
    assert_eq!(evidence["identity"]["candidate"]["base_tree"], base_tree);
    let supplier = project.rows(
        "SELECT id, result FROM execution WHERE kind = 'implementation' AND outcome = 'candidate'",
    )[0]
    .clone();
    let producer_result: Value =
        serde_json::from_str(supplier["result"].as_str().unwrap()).unwrap();
    let published_proof = &producer_result["outputs"]["result"]["value"]["proof"];
    assert_eq!(
        evidence["identity"]["inputs"]["candidate"]["source"]["execution"],
        supplier["id"]
    );
    assert_eq!(evidence["identity"]["candidate"]["proof"], *published_proof);
    assert_eq!(
        std::fs::read_to_string(project.path.join(format!(
            ".yard/local/attempts/1/proof-snapshots/{}/evidence.txt",
            published_proof.as_str().unwrap()
        )))
        .unwrap(),
        "first"
    );
    assert_eq!(
        evidence["identity"]["definition"]["build_context_tree"],
        project
            .git(&["rev-parse", &format!("{admitted_source}^{{tree}}")])
            .trim()
    );
    assert_eq!(
        evidence["identity"]["definition"]["dockerfile_blob"],
        project
            .git(&["rev-parse", &format!("{admitted_source}:.yard/Dockerfile")])
            .trim()
    );
    assert_eq!(
        evidence["runtime"]["environment"]["definition"],
        "yard-box-gate-v1"
    );
    assert_eq!(evidence["external_services"]["status"], "none");
    let ready: Vec<Value> = std::fs::read_to_string(&observed)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|line| line["event"] == "ready")
        .collect();
    assert!(
        ready
            .iter()
            .any(|line| line["image"]["id"] == evidence["image"]["id"])
    );
    let program = if cfg!(target_os = "macos") {
        "container"
    } else {
        "podman"
    };
    let runtime = std::process::Command::new(program)
        .arg("--version")
        .output()
        .unwrap();
    assert!(runtime.status.success());
    assert_eq!(
        evidence["runtime"]["supported"]["version"],
        String::from_utf8(runtime.stdout).unwrap().trim()
    );
    let pinfold = machine.pinfold(&["--version"]);
    assert!(pinfold.status.success());
    assert_eq!(
        evidence["runtime"]["supported"]["pinfold"],
        String::from_utf8(pinfold.stdout).unwrap().trim()
    );
    let result: Value = serde_json::from_str(closed["result"].as_str().unwrap()).unwrap();
    assert_eq!(result["verification"], evidence);
    let mut operator = machine.operator();
    operator.initialize();
    {
        let (revision, declaration) = (1, "operator-runtime-v2");
        project.reconfigure(&configuration.replace("operator-runtime-v1", declaration));
        let source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
        let revised = operator.call(
            "yard_attempt_revise",
            json!({"project": project.path,
            "ticket": "Y-1", "plan_revision": revision, "definition_commit": source}),
        );
        assert_eq!(
            revised["result"]["plan_revision"],
            revision + 1,
            "{revised}"
        );
        approval(&mut watch);
        let rows = project
            .rows("SELECT * FROM execution WHERE node = 'candidate-gate-closed' ORDER BY id");
        let next = rows.last().unwrap();
        assert_ne!(next["handle"], closed["handle"]);
        let next: Value = serde_json::from_str(next["verification"].as_str().unwrap()).unwrap();
        assert_eq!(next["identity"]["declaration"], declaration);
        assert_ne!(next["identity"], evidence["identity"]);
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM execution WHERE id IN ({}, {}, {}) ORDER BY id",
                original[0]["id"], original[1]["id"], original[2]["id"]
            )),
            original
        );
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM \"check\" WHERE id <= {} ORDER BY id",
                checks.last().unwrap()["id"]
            )),
            checks
        );
        let shown = operator.call(
            "yard_attempt_show",
            json!({"project": project.path, "ticket": "Y-1"}),
        );
        assert!(shown.get("error").is_none(), "{shown}");
        let old = shown["result"]["executions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["execution"] == closed["id"])
            .unwrap();
        assert_eq!(old["verification"], evidence);
        assert_eq!(old["result"], result);
        let producer = shown["result"]["executions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["execution"] == supplier["id"])
            .unwrap();
        assert_eq!(producer["result"], producer_result);
    }
    machine.kill();
    machine.start();
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id IN ({}, {}, {}) ORDER BY id",
            original[0]["id"], original[1]["id"], original[2]["id"]
        )),
        original
    );
}

/// G2/G5/G9: revise the final policy at manual approval, then change its
/// producer policy. The first edit retains the completed unrelated seat;
/// the second selects that profile for the non-final producer and invalidates
/// its typed consumers. Routing then omits the middle seat: the completed
/// first seat keeps its evidence despite a changed complete successor, while
/// the final seat must bind that actual producer instead of the omitted seat.
/// Hold the revised final worker through the real
/// host model route to prove stale edits refuse, with quiescent edits in the
/// same project as controls. Revisions/policies come from fixture literals;
/// candidate identity comes from host git. Store snapshots prove old rows
/// unchanged and stale edits/decisions write nothing.
/// Sabotage: overwrite admission, remove plan CAS, qualify by revision alone,
/// omit node/policy identity, run the old seat instead of the selected profile,
/// keep the ordinary-seat digest for a different runner, bind complete routing
/// into completed evidence, or select an obsolete producer.
/// Existing G2 edits change ticket/head/proof; none revises a plan on the same
/// candidate or binds a new consumer to an eligible older-revision producer.
/// Declared agents have different model settings. Committed extension and
/// skill markers reach the host model, whose opening requests count consumers.
/// Independent committed gate, reviewer setting, extension and skill adoption
/// follows completed checks. Host Git supplies blobs and the fixed image tree.
/// Sabotage: resolved overwrites adopted definitions, stale tool/skill digest,
/// or whole-panel invalidation. Host counters prove actual consumers.
/// Gate-only and agent-only adoption each retain the fixture's exact image,
/// whole context and staging source. Sabotage: adopt an omitted source in
/// either revision, then restore it on the later staging revision; checking
/// only the later revision misses that fault. Agent-only adoption must change
/// both affected reviewer digests, not merely their launched model settings.
/// Sabotage: omit agent settings from review consumer digests while still
/// launching the selected model.
/// Gate-only adoption must also change the actual gate consumer digest.
/// Sabotage original-gate-digests: check_digest reads original_admission for
/// gates instead of the current selected admission. The committed command
/// and host counter change on the same candidate with fixed image/context;
/// neither invocation counts nor immutable history alone catch that fault.
/// All three selected Pi agents consume these paths. Each staging adoption
/// must rerun the implementer and both reviews; its fresh candidate supplier
/// also invalidates both host gates. Sabotage: ignore staged inputs for a
/// completed implementer merely because its old candidate tuple still matches.
/// Image adoption is observed through the image ENV marker and COPY bytes in
/// real consumers, not optional runtime image IDs. Sabotage: build current HEAD
/// instead of the selected image context, or reuse the prior consumer image.
/// G2 real launch evidence: source/image adoption leaves newer templates
/// unselected. Compare verification trees and source blobs with host Git,
/// source bytes and the daemon executable with host SHA-256, and runtime
/// images with the real pinfold ready stream. Host identities stay unknown.
/// Sabotage: record head as tree, read current templates at launch, substitute
/// admitted image labels for observed IDs, or fingerprint host environment.
/// Earlier blocks observe admission and behavior, not launch/result evidence.
#[test]
fn quiescent_plan_cas_retains_only_exact_node_and_producer_evidence() {
    let hold = Latch::new();
    let held = hold.clone();
    let final_calls = std::sync::Mutex::new(0);
    let correctness_calls = std::sync::Mutex::new(0);
    let capacity = Latch::new();
    let capacity_held = capacity.clone();
    let machine = Machine::new("g2-plan-cas", move |request| {
        let future = request.system().contains("consumer-source-v2");
        let image = request.system().contains("consumer-image-v2");
        assert!(request.system().contains("consumer-skill-v"));
        assert!(request.system().contains(if future {
            "consumer-source-v2"
        } else {
            "consumer-source-v1"
        }));
        if request.has_tool("yard_publish_review") {
            if future {
                assert_eq!(request.body["model"], "adopted-model");
            }
            if !future
                && request.opens()
                && request
                    .prompt()
                    .contains("You are the correctness reviewer")
            {
                let mut calls = correctness_calls.lock().unwrap();
                *calls += 1;
                let instructions = if *calls == 1 {
                    "Review for correctness."
                } else {
                    "Alternate final review."
                };
                // G2: selecting the old runner instead of the fixture profile
                // loses these instructions, regardless of prompt framing.
                assert!(request.prompt().contains(instructions));
                assert_eq!(
                    request.body["model"],
                    if *calls == 1 {
                        "fake-model"
                    } else if *calls == 2 {
                        "selected-model"
                    } else {
                        "adopted-model"
                    }
                );
            }
            if !future && request.opens() && request.prompt().contains("You are the final reviewer")
            {
                let mut calls = final_calls.lock().unwrap();
                *calls += 1;
                let instructions = if *calls == 1 {
                    "Final review."
                } else {
                    "Alternate final review."
                };
                assert!(request.prompt().contains(instructions));
                assert_eq!(
                    request.body["model"],
                    if *calls == 1 {
                        "fake-model"
                    } else if *calls <= 4 {
                        "selected-model"
                    } else {
                        "adopted-model"
                    }
                );
                if *calls == 2 {
                    return Reply::Hold(
                        held.clone(),
                        Box::new(act(request, vec![publish(json!([]))])),
                    );
                }
            }
            if image {
                return match request.tool_results().len() {
                    0 => Reply::Tools(vec![bash("cat /image-input")]),
                    1 => {
                        assert_eq!(request.last_tool_result().unwrap().1.trim(), "image-v2");
                        Reply::Tools(vec![publish(json!([]))])
                    }
                    _ => Reply::Text("done".into()),
                };
            }
            return act(request, vec![publish(json!([]))]);
        }
        if request.opens() && request.prompt().contains("Capacity holder") {
            return Reply::Hold(capacity_held.clone(), Box::new(Reply::Text("Held".into())));
        }
        let feature = if image {
            "feature-image-v2"
        } else if request.system().contains("consumer-skill-v2") {
            "feature-skill-v2"
        } else if future {
            "feature-tool-v2"
        } else {
            "feature-v1"
        };
        act(
            request,
            vec![bash(&format!(
                "{}cd /workspace && printf '{feature}\\n' > feature.txt && git add -A && git commit -qm Feature && printf 'proof' > /yard/proof/evidence.txt",
                if image {
                    "test \"$(cat /image-input)\" = image-v2 && "
                } else {
                    ""
                },
            ))],
        )
    });
    let observed_images = machine.root.join("observed-images.jsonl");
    machine.wrapper(
        "pinfold",
        &format!(
            "if [ \"$1 $2\" = 'box up' ]; then
  fifo='{}.'$$
  mkfifo \"$fifo\" || exit $?
  trap 'rm -f \"$fifo\"' 0
  while IFS= read -r line; do
    printf '%s\\n' \"$line\" >> '{}'
    printf '%s\\n' \"$line\"
  done < \"$fifo\" &
  reader=$!
  \"$REAL\" \"$@\" > \"$fifo\"
  result=$?
  wait \"$reader\"
  exit \"$result\"
fi",
            machine.root.join("observed-images.fifo").display(),
            observed_images.display(),
        ),
    );
    let sha256 = |bytes: &[u8]| {
        let mut child = std::process::Command::new("shasum")
            .args(["-a", "256"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_string()
    };
    let yard_sha256 = sha256(&std::fs::read(yard()).unwrap());
    machine.start();
    let configuration = config(&format!("{GATE}\n[review.seats.middle]\nagent = \"worker\"\ninstructions = \"Middle review.\"\n\n[review.seats.final]\nagent = \"worker\"\ninstructions = \"Final review.\"\n\n[profiles.final]\nrunner = {{kind = \"agent\", agent = \"worker\", instructions = \"Alternate final review.\"}}\naccess = \"read-only\"\noutputs = {{result = \"verdict\"}}\noutcomes = [\"complete\", \"blocked\"]\n"))
        .replace("review = [\"correctness\"]", "review = [\"correctness\", \"middle\", \"final\"]");
    let configuration = configuration.replace(
        "agent = \"worker\", instructions = \"Alternate final review.\"",
        "agent = \"selected\", instructions = \"Alternate final review.\"",
    ) + "\n[agents.selected]\nharness = \"pi\"\nprovider = \"openrouter\"\nmodel = \"selected-model\"\n";
    let gate_counter = machine.root.join("gate-count");
    let keep_counter = machine.root.join("keep-count");
    let configuration = configuration.replace(GATE, &format!(
        "[gates.check]\ncommand = 'test -f feature.txt && echo original >> {}'\nstage = 'candidate'\nruns_in = 'host'\n",
        gate_counter.display()
    )) + &format!("\n[gates.keep]\ncommand = 'echo keep >> {}'\nstage = 'candidate'\nruns_in = 'host'\n", keep_counter.display());
    let configuration = configuration.replace("max_lanes = 2", "max_lanes = 2\nmax_executions = 1");
    let stage = |project: &Project, tool_version: &str, skill_version: &str| {
        project.write(".pi/extensions/consumer/index.ts", &format!(
            "export default function (pi) {{\n  pi.on(\"before_agent_start\", async (event) => ({{systemPrompt: event.systemPrompt + '\\nconsumer-source-{tool_version}\\nconsumer-image-' + (process.env.YARD_IMAGE_SOURCE || 'v1')}}));\n}}\n"
        ));
        project.write(
            ".agents/skills/consumer/SKILL.md",
            &format!(
                "---\nname: consumer\ndescription: consumer-skill-{skill_version}\n---\nFixture.\n"
            ),
        );
        project.git(&["add", "-A"]);
        project.git(&["commit", "--quiet", "-m", "Consumer inputs"]);
        project.json(&["sync"]);
    };
    let project = Project::new(&machine, "p", &configuration);
    stage(&project, "v1", "v1");
    let mut image_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let mut image_tree = project
        .git(&["rev-parse", "HEAD^{tree}"])
        .trim()
        .to_string();
    let mut dockerfile_blob = project
        .git(&["rev-parse", "HEAD:.yard/Dockerfile"])
        .trim()
        .to_string();
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Plan CAS"]);
    let first = approval(&mut watch);
    let candidate = git(
        &project.path.join(".yard/local/attempts/1/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    let original_admission = project.rows("SELECT admission FROM attempt WHERE id = 1");
    let original_ticket = project.rows("SELECT * FROM ticket WHERE id = 1");
    let original_executions = project.rows("SELECT * FROM execution WHERE attempt = 1 ORDER BY id");
    let original_checks = project.rows("SELECT * FROM \"check\" ORDER BY id");
    let original_ids: Vec<_> = original_executions
        .iter()
        .map(|row| row["id"].as_i64().unwrap().to_string())
        .collect();
    let check_ids: Vec<_> = original_checks
        .iter()
        .map(|row| row["id"].as_i64().unwrap().to_string())
        .collect();
    let unrelated = original_executions
        .iter()
        .find(|row| row["node"] == "review-0")
        .unwrap();
    let old_final = original_executions
        .iter()
        .find(|row| row["node"] == "review-2")
        .unwrap();
    let middle = original_executions
        .iter()
        .find(|row| row["node"] == "review-1")
        .unwrap();
    let mut operator = machine.operator();
    operator.initialize();
    let changed = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 1,
            "overrides": {"review-2": {"profile": "final", "operator_failure": true}}
        }),
    );
    assert_eq!(changed["result"]["plan_revision"], 2, "{changed}");
    hold.wait_held();
    let held_verification = project
        .rows("SELECT verification FROM execution WHERE node = 'review-2' AND status = 'running'");
    let held_verification: Value =
        serde_json::from_str(held_verification[0]["verification"].as_str().unwrap()).unwrap();
    let ready: Value = std::fs::read_to_string(&observed_images)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .rfind(|line| line["event"] == "ready")
        .unwrap();
    assert_eq!(
        held_verification["image"],
        json!({"status": "observed", "id": ready["image"]["id"]})
    );
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let revisions = project.rows("SELECT * FROM plan_revision ORDER BY revision");
    let before_attempt = project.rows("SELECT * FROM attempt ORDER BY id");
    let before_executions = project.rows("SELECT * FROM execution ORDER BY id");
    let before_checks = project.rows("SELECT * FROM \"check\" ORDER BY id");
    let before_attention = project.rows("SELECT * FROM attention ORDER BY id");
    let refused = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 1,
            "overrides": {"review-0": {"profile": "final", "operator_failure": true}}
        }),
    );
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "plan_revision", "expected": 1, "current": 2})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows("SELECT * FROM plan_revision ORDER BY revision"),
        revisions
    );
    assert_eq!(
        project.rows("SELECT * FROM attempt ORDER BY id"),
        before_attempt
    );
    assert_eq!(
        project.rows("SELECT * FROM execution ORDER BY id"),
        before_executions
    );
    assert_eq!(
        project.rows("SELECT * FROM \"check\" ORDER BY id"),
        before_checks
    );
    assert_eq!(
        project.rows("SELECT * FROM attention ORDER BY id"),
        before_attention
    );
    hold.release();
    let second = approval(&mut watch);
    assert_eq!(second["payload"]["head"], candidate);
    assert_eq!(second["payload"]["proof"], first["payload"]["proof"]);
    assert_eq!(second["payload"]["revision"], 1);
    assert_eq!(second["payload"]["plan_revision"], 2);
    let revised = project.rows("SELECT id, plan_revision, inputs, digest FROM execution WHERE node = 'review-2' ORDER BY id DESC LIMIT 1");
    assert_ne!(revised[0]["id"], old_final["id"]);
    assert_eq!(revised[0]["plan_revision"], 2);
    assert_ne!(revised[0]["digest"], old_final["digest"]);
    assert_eq!(
        project.rows(
            "SELECT agent, model FROM execution WHERE node = 'review-2' AND plan_revision = 2"
        ),
        vec![json!({"agent": "selected", "model": "selected-model"})]
    );
    let inputs: Value = serde_json::from_str(revised[0]["inputs"].as_str().unwrap()).unwrap();
    assert_eq!(inputs["panel"]["source"]["execution"], middle["id"]);
    assert_eq!(
        project.rows("SELECT plan_revision FROM execution WHERE node = 'review-0' ORDER BY id"),
        vec![json!({"plan_revision": 1})]
    );
    let unrelated_check = original_checks
        .iter()
        .find(|row| row["execution"] == unrelated["id"])
        .unwrap();
    assert!(
        second["payload"]["checks"]
            .as_array()
            .unwrap()
            .contains(&unrelated_check["id"])
    );
    let old_final_check = original_checks
        .iter()
        .find(|row| row["execution"] == old_final["id"])
        .unwrap();
    let revised_check = project.rows(&format!(
        "SELECT id FROM \"check\" WHERE execution = {}",
        revised[0]["id"].as_i64().unwrap()
    ));
    let selected = second["payload"]["checks"].as_array().unwrap();
    assert!(!selected.contains(&old_final_check["id"]));
    assert!(selected.contains(&revised_check[0]["id"]));

    let changed = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 2,
            "overrides": {"review-0": {"profile": "final", "operator_failure": true}}
        }),
    );
    assert_eq!(changed["result"]["plan_revision"], 3, "{changed}");
    let third = approval(&mut watch);
    assert_eq!(third["payload"]["head"], candidate);
    assert_eq!(third["payload"]["proof"], first["payload"]["proof"]);
    let producer = project
        .rows("SELECT id, digest FROM execution WHERE node = 'review-0' AND plan_revision = 3");
    assert_ne!(producer[0]["digest"], unrelated["digest"]);
    let consumer = project
        .rows("SELECT id, inputs FROM execution WHERE node = 'review-1' AND plan_revision = 3");
    let inputs: Value = serde_json::from_str(consumer[0]["inputs"].as_str().unwrap()).unwrap();
    assert_eq!(inputs["panel"]["source"]["execution"], producer[0]["id"]);
    assert_ne!(inputs["panel"]["source"]["execution"], unrelated["id"]);
    let final_consumer = project.rows(
        "SELECT id, inputs, digest FROM execution WHERE node = 'review-2' AND plan_revision = 3",
    );
    let inputs: Value =
        serde_json::from_str(final_consumer[0]["inputs"].as_str().unwrap()).unwrap();
    assert_eq!(inputs["panel"]["source"]["execution"], consumer[0]["id"]);
    assert_eq!(final_consumer[0]["digest"], revised[0]["digest"]);
    let affected = project.rows(&format!(
        "SELECT id FROM \"check\" WHERE execution = {}",
        producer[0]["id"].as_i64().unwrap()
    ));
    let selected = third["payload"]["checks"].as_array().unwrap();
    assert!(selected.contains(&affected[0]["id"]));
    assert!(!selected.contains(&unrelated_check["id"]));
    // Fixture routing removes only the middle seat. The first reviewer keeps
    // its policy and inputs; the final reviewer now consumes it directly.
    let mut graph = json!({"inputs": {"feedback": "verdict", "merged": "candidate"}, "nodes": {
        "implementer": {
            "runner": {"kind": "agent", "agent": "worker", "instructions": ""},
            "access": "write-allowed", "inputs": {"feedback": {"output": "feedback", "kind": "verdict"}},
            "outputs": {"result": "candidate"}, "outcomes": ["complete"],
            "routes": {"complete": [{"to": "candidate-gate-check"}]}
        },
        "candidate-gate-check": {
            "runner": {"kind": "command", "gate": "check"}, "access": "write-allowed",
            "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "failed"], "requires": ["candidate"],
            "routes": {"complete": [{"to": "review-0"}], "failed": [{"to": "implementer", "repeat": 1}]}
        },
        "review-0": {
            "profile": "final", "operator_failure": true,
            "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "requires": ["candidate"], "routes": {"complete": [{"to": "review-2"}]}
        },
        "review-2": {
            "profile": "final", "operator_failure": true,
            "inputs": {
                "candidate": {"node": "implementer", "output": "result", "kind": "candidate"},
                "panel": {"node": "review-0", "output": "result", "kind": "verdict"}
            },
            "requires": ["candidate", "panel"],
            "routes": {"complete": [{"to": "landing-gate-check"}], "blocked": [{"to": "implementer", "repeat": 3}]}
        },
        "landing-gate-check": {
            "runner": {"kind": "command", "gate": "check"}, "access": "write-allowed",
            "inputs": {"merged": {"output": "merged", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"], "requires": ["merged"]
        }
    }});
    graph["nodes"]["candidate-gate-keep"] = graph["nodes"]["candidate-gate-check"].clone();
    graph["nodes"]["candidate-gate-keep"]["runner"]["gate"] = json!("keep");
    graph["nodes"]["candidate-gate-check"]["routes"]["complete"] =
        json!([{"to": "candidate-gate-keep"}]);
    graph["nodes"]["landing-gate-keep"] = graph["nodes"]["landing-gate-check"].clone();
    graph["nodes"]["landing-gate-keep"]["runner"]["gate"] = json!("keep");
    graph["nodes"]["landing-gate-check"]["routes"]["complete"] =
        json!([{"to": "landing-gate-keep"}]);
    let before_routing = project.rows("SELECT * FROM execution WHERE attempt = 1 ORDER BY id");
    let routed = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 3, "graph": graph
        }),
    );
    assert_eq!(routed["result"]["plan_revision"], 4, "{routed}");
    let fourth = approval(&mut watch);
    let selected = fourth["payload"]["checks"].as_array().unwrap();
    assert!(selected.contains(&affected[0]["id"]));
    let final_check = project.rows(&format!(
        "SELECT id FROM \"check\" WHERE execution = {}",
        final_consumer[0]["id"].as_i64().unwrap()
    ));
    assert!(!selected.contains(&final_check[0]["id"]));
    let routed_consumer = project.rows(
        "SELECT id, inputs, digest FROM execution WHERE node = 'review-2' AND plan_revision = 4",
    );
    let inputs: Value =
        serde_json::from_str(routed_consumer[0]["inputs"].as_str().unwrap()).unwrap();
    assert_eq!(inputs["panel"]["source"]["execution"], producer[0]["id"]);
    assert_ne!(inputs["panel"]["source"]["execution"], consumer[0]["id"]);
    assert_eq!(routed_consumer[0]["digest"], final_consumer[0]["digest"]);
    let routed_check = project.rows(&format!(
        "SELECT id FROM \"check\" WHERE execution = {}",
        routed_consumer[0]["id"].as_i64().unwrap()
    ));
    assert!(selected.contains(&routed_check[0]["id"]));
    let old_ids: Vec<_> = before_routing
        .iter()
        .map(|row| row["id"].as_i64().unwrap().to_string())
        .collect();
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id IN ({}) ORDER BY id",
            old_ids.join(",")
        )),
        before_routing
    );
    assert_eq!(
        project.rows("SELECT node, plan_revision FROM execution WHERE kind = 'review' ORDER BY id"),
        vec![
            json!({"node": "review-0", "plan_revision": 1}),
            json!({"node": "review-1", "plan_revision": 1}),
            json!({"node": "review-2", "plan_revision": 1}),
            json!({"node": "review-2", "plan_revision": 2}),
            json!({"node": "review-0", "plan_revision": 3}),
            json!({"node": "review-1", "plan_revision": 3}),
            json!({"node": "review-2", "plan_revision": 3}),
            json!({"node": "review-2", "plan_revision": 4}),
        ]
    );
    assert_eq!(
        project.rows("SELECT * FROM ticket WHERE id = 1"),
        original_ticket
    );
    assert_eq!(
        project.rows("SELECT admission FROM attempt WHERE id = 1"),
        original_admission
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id IN ({}) ORDER BY id",
            original_ids.join(",")
        )),
        original_executions
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id IN ({}) ORDER BY id",
            check_ids.join(",")
        )),
        original_checks
    );
    // Hold the sole effect slot before approving. The queued landing has no
    // intent and cannot move canonical before adoption withdraws approval.
    let adoption_executions = project.rows("SELECT * FROM execution WHERE attempt = 1 ORDER BY id");
    let prior_gate_digest = adoption_executions
        .iter()
        .rev()
        .find(|row| row["node"] == "candidate-gate-check")
        .unwrap()["digest"]
        .as_str()
        .unwrap()
        .to_string();
    let adoption_ids = adoption_executions
        .iter()
        .map(|row| row["id"].as_i64().unwrap().to_string())
        .collect::<Vec<_>>();
    let adoption_checks = project.rows("SELECT * FROM \"check\" WHERE execution IN (SELECT id FROM execution WHERE attempt = 1) ORDER BY id");
    let adoption_check_ids = adoption_checks
        .iter()
        .map(|row| row["id"].as_i64().unwrap().to_string())
        .collect::<Vec<_>>();
    let original_revisions =
        project.rows("SELECT * FROM plan_revision WHERE attempt = 1 ORDER BY revision");
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Capacity holder",
        "--workflow",
        "plan",
    ]);
    capacity.wait_held();
    let accepted = operator.call(
        "yard_attempt_approve",
        json!({
            "project": project.path, "ticket": "Y-1", "target": "main", "head": candidate,
            "proof": fourth["payload"]["proof"], "plan_revision": 4
        }),
    );
    assert!(accepted.get("error").is_none(), "{accepted}");
    assert_eq!(
        project.rows("SELECT state FROM approval"),
        vec![json!({"state": "active"})]
    );
    // G2: sync alone keeps completed work's plan, including staged sources.
    // Sabotage: sync overwrites active evidence before explicit adoption.
    let frozen_plan = project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"].clone();
    let gate_definition = configuration.replace("echo original >>", "echo adopted >>");
    project.write(".yard/config.toml", &gate_definition);
    project.git(&["add", "-A"]);
    project.git(&["commit", "--quiet", "-m", "Adopt candidate gate"]);
    project.json(&["sync"]);
    assert_eq!(
        project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"],
        frozen_plan
    );
    let definition_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let frozen_sources = |definition: &str| {
        let shown = project.json(&["attempt", "show", "Y-1"]);
        let inputs = &shown["attempt"]["plan"]["inputs"];
        assert_eq!(inputs["definition_commit"], definition);
        assert_eq!(inputs["staging_commit"], image_commit);
        assert_eq!(inputs["image_commit"], image_commit);
        assert_eq!(inputs["build_context_tree"], image_tree);
    };
    let adopt = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 4,
            "definition_commit": definition_commit
        }),
    );
    assert_eq!(adopt["result"]["plan_revision"], 5, "{adopt}");
    assert_eq!(
        project.rows("SELECT state FROM approval"),
        vec![json!({"state": "withdrawn"})]
    );
    project.json(&["attempt", "abandon", "Y-2"]);
    capacity.release();
    let fifth = approval(&mut watch);
    assert_eq!(fifth["payload"]["plan_revision"], 5);
    frozen_sources(&definition_commit);
    assert_eq!(
        std::fs::read_to_string(&gate_counter).unwrap(),
        "original\nadopted\n"
    );
    assert_eq!(std::fs::read_to_string(&keep_counter).unwrap(), "keep\n");
    let gate_consumers =
        project.rows("SELECT node, digest FROM execution WHERE attempt = 1 AND plan_revision = 5");
    assert_eq!(gate_consumers.len(), 1);
    assert_eq!(gate_consumers[0]["node"], "candidate-gate-check");
    assert_ne!(
        gate_consumers[0]["digest"].as_str().unwrap(),
        prior_gate_digest
    );
    assert!(
        fifth["payload"]["checks"]
            .as_array()
            .unwrap()
            .contains(&routed_check[0]["id"])
    );

    let settings_definition =
        gate_definition.replace("model = \"selected-model\"", "model = \"adopted-model\"");
    project.write(".yard/config.toml", &settings_definition);
    project.git(&["add", "-A"]);
    project.git(&[
        "commit",
        "--quiet",
        "-m",
        "Adopt declared reviewer settings",
    ]);
    project.json(&["sync"]);
    let settings_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let adopt = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 5,
            "definition_commit": settings_commit
        }),
    );
    assert_eq!(adopt["result"]["plan_revision"], 6, "{adopt}");
    let sixth = approval(&mut watch);
    assert_eq!(sixth["payload"]["plan_revision"], 6);
    frozen_sources(&settings_commit);
    assert_eq!(
        project.rows(
            "SELECT node, model FROM execution WHERE attempt = 1 AND plan_revision = 6 ORDER BY id"
        ),
        vec![
            json!({"node": "review-0", "model": "adopted-model"}),
            json!({"node": "review-2", "model": "adopted-model"})
        ]
    );
    for row in project.rows(
        "SELECT node, digest FROM execution WHERE attempt = 1 AND plan_revision = 6 ORDER BY id",
    ) {
        let previous = adoption_executions
            .iter()
            .rev()
            .find(|previous| previous["node"] == row["node"])
            .unwrap();
        assert_ne!(
            row["digest"].as_str().unwrap(),
            previous["digest"].as_str().unwrap()
        );
    }

    // G2: image-only adoption uses an older exact context while current
    // definitions and staging differ from both retained sources.
    // Sabotage: ignore image_commit, build HEAD, or adopt an omitted source.
    // Real consumers read COPY bytes and ENV; host Git supplies identities.
    // Host gate definitions stay fixed; their suppliers change independently.
    let mut current_candidate = candidate.clone();
    let mut staging_commit = image_commit.clone();
    for (revision, skill, feature) in [
        (7, "v1", "feature-tool-v2\n"),
        (8, "v2", "feature-skill-v2\n"),
        (9, "v2", "feature-image-v2\n"),
    ] {
        let frozen_plan = project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"].clone();
        let prior_executions =
            project.rows("SELECT * FROM execution WHERE attempt = 1 ORDER BY id");
        let prior_ids = prior_executions
            .iter()
            .map(|row| row["id"].as_i64().unwrap().to_string())
            .collect::<Vec<_>>();
        let prior_checks = project.rows("SELECT * FROM \"check\" WHERE execution IN (SELECT id FROM execution WHERE attempt = 1) ORDER BY id");
        let prior_check_ids = prior_checks
            .iter()
            .map(|row| row["id"].as_i64().unwrap().to_string())
            .collect::<Vec<_>>();
        let prior_revisions =
            project.rows("SELECT * FROM plan_revision WHERE attempt = 1 ORDER BY revision");
        let (source_key, source_commit) = if revision == 9 {
            let dockerfile = project.git(&["show", "HEAD:.yard/Dockerfile"]);
            project.write("image-input", "image-v2\n");
            project.write(
                ".yard/Dockerfile",
                &format!("{dockerfile}\nCOPY image-input /image-input\nENV YARD_IMAGE_SOURCE=v2\n"),
            );
            project.write(
                ".yard/config.toml",
                &settings_definition.replace("adopted-model", "unselected-model"),
            );
            stage(&project, "unselected", "unselected");
            image_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
            image_tree = project
                .git(&["rev-parse", "HEAD^{tree}"])
                .trim()
                .to_string();
            dockerfile_blob = project
                .git(&["rev-parse", "HEAD:.yard/Dockerfile"])
                .trim()
                .to_string();
            project.write("image-input", "unselected-image\n");
            project.git(&["commit", "--quiet", "-am", "Later unselected context"]);
            project.json(&["sync"]);
            ("image_commit", image_commit.clone())
        } else {
            stage(&project, "v2", skill);
            staging_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
            ("staging_commit", staging_commit.clone())
        };
        assert_eq!(
            project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"],
            frozen_plan
        );
        let before = project.rows("SELECT * FROM audit ORDER BY seq");
        let stale = operator.call(
            "yard_attempt_revise",
            json!({
                "project": project.path, "ticket": "Y-1", "plan_revision": revision - 2,
                (source_key): source_commit
            }),
        );
        assert_eq!(stale["error"]["code"], "stale");
        assert_eq!(
            stale["error"]["data"],
            json!({"identity": "plan_revision", "expected": revision - 2, "current": revision - 1})
        );
        assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
        let adopt = operator.call(
            "yard_attempt_revise",
            json!({
                "project": project.path, "ticket": "Y-1", "plan_revision": revision - 1,
                (source_key): source_commit
            }),
        );
        assert_eq!(adopt["result"]["plan_revision"], revision, "{adopt}");
        let item = approval(&mut watch);
        assert_eq!(item["payload"]["plan_revision"], revision);
        let rows = project.rows(&format!("SELECT id, node, digest, inputs, result, teardown, reserved, verification FROM execution WHERE attempt = 1 AND plan_revision = {revision} ORDER BY id"));
        assert_eq!(
            rows.iter()
                .map(|row| row["node"].clone())
                .collect::<Vec<_>>(),
            json!([
                "implementer",
                "candidate-gate-check",
                "candidate-gate-keep",
                "review-0",
                "review-2"
            ])
            .as_array()
            .unwrap()
            .clone()
        );
        for row in &rows {
            let old = prior_executions
                .iter()
                .rev()
                .find(|old| old["node"] == row["node"])
                .unwrap();
            assert_ne!(row["id"], old["id"]);
            assert_eq!(row["teardown"], 1);
            assert_eq!(row["reserved"], 0);
            if revision == 9 && row["node"].as_str().unwrap().starts_with("candidate-gate-") {
                assert_eq!(row["digest"], old["digest"]);
            }
            if row["node"].as_str().unwrap().starts_with("review-") {
                assert_ne!(row["digest"], old["digest"]);
            }
        }
        let clone = project.path.join(".yard/local/attempts/1/clone");
        let produced = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
        assert_ne!(produced, current_candidate);
        git(
            &clone,
            &["merge-base", "--is-ancestor", &current_candidate, &produced],
        );
        assert_eq!(
            std::fs::read_to_string(clone.join("feature.txt")).unwrap(),
            feature
        );
        assert_eq!(item["payload"]["head"], produced);
        assert_eq!(item["payload"]["proof"], first["payload"]["proof"]);
        let base = git(
            &project.canonical(),
            &["merge-base", "refs/heads/main", &produced],
        )
        .trim()
        .to_string();
        let candidate_value =
            json!({"base": base, "head": produced, "proof": first["payload"]["proof"]});
        let producer_result: Value =
            serde_json::from_str(rows[0]["result"].as_str().unwrap()).unwrap();
        assert_eq!(
            producer_result["outputs"]["result"],
            json!({"kind": "candidate", "value": candidate_value})
        );
        for row in &rows[1..] {
            let inputs: Value = serde_json::from_str(row["inputs"].as_str().unwrap()).unwrap();
            assert_eq!(
                inputs["candidate"],
                json!({"kind": "candidate",
                "source": {"execution": rows[0]["id"], "node": "implementer", "output": "result"},
                "value": candidate_value})
            );
        }
        let inputs: Value = serde_json::from_str(rows[4]["inputs"].as_str().unwrap()).unwrap();
        let supplier = &rows[3];
        assert_eq!(
            inputs["panel"]["source"],
            json!({"execution": supplier["id"], "node": "review-0", "output": "result"})
        );
        let result: Value = serde_json::from_str(supplier["result"].as_str().unwrap()).unwrap();
        assert_eq!(inputs["panel"]["kind"], "verdict");
        assert_eq!(
            inputs["panel"]["value"],
            result["outputs"]["result"]["value"]
        );
        let show = project.json(&["attempt", "show", "Y-1"]);
        let evidence = &show["attempt"]["plan"];
        assert_eq!(evidence["inputs"]["image_commit"], image_commit);
        assert_eq!(evidence["inputs"]["build_context_tree"], image_tree);
        assert_eq!(evidence["inputs"]["dockerfile_blob"], dockerfile_blob);
        assert_eq!(evidence["inputs"]["definition_commit"], settings_commit);
        assert_eq!(evidence["inputs"]["staging_commit"], staging_commit);
        for path in [
            ".pi/extensions/consumer/index.ts",
            ".agents/skills/consumer/SKILL.md",
        ] {
            let blob = project
                .git(&["rev-parse", &format!("{staging_commit}:{path}")])
                .trim()
                .to_string();
            let source = evidence["inputs"]["trusted_staged_sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|source| source["path"] == path)
                .unwrap();
            assert_eq!(source["blob"], blob);
        }
        // G2: the immutable launch and its result describe the actual frozen
        // consumer, not the newer canonical template or a provenance commit.
        let tree = git(
            &project.canonical(),
            &["rev-parse", &format!("{produced}^{{tree}}")],
        )
        .trim()
        .to_string();
        let base_tree = git(
            &project.canonical(),
            &["rev-parse", &format!("{base}^{{tree}}")],
        )
        .trim()
        .to_string();
        for row in &rows[1..] {
            let verification: Value =
                serde_json::from_str(row["verification"].as_str().unwrap()).unwrap();
            assert_eq!(verification["candidate"]["tree"], tree);
            assert_eq!(verification["candidate"]["base_tree"], base_tree);
            assert_eq!(
                verification["candidate"]["proof"],
                first["payload"]["proof"]
            );
            assert_eq!(verification["tools"]["yard"]["sha256"], yard_sha256);
            assert_eq!(verification["tools"]["executed"]["status"], "unknown");
            assert_eq!(verification["runtime"]["environment"]["status"], "unknown");
            assert_eq!(verification["reuse"], "disabled");
            let result: Value = serde_json::from_str(row["result"].as_str().unwrap()).unwrap();
            assert_eq!(result["verification"], verification);
            if row["node"].as_str().unwrap().starts_with("candidate-gate-") {
                assert_eq!(verification["image"]["status"], "unknown");
                let command = if row["node"] == "candidate-gate-check" {
                    format!(
                        "test -f feature.txt && echo adopted >> {}",
                        gate_counter.display()
                    )
                } else {
                    format!("echo keep >> {}", keep_counter.display())
                };
                assert_eq!(verification["launch"]["command"], command);
            } else {
                assert_eq!(
                    verification["policy"]["definition"]["build_context_tree"],
                    image_tree
                );
                assert_eq!(
                    verification["policy"]["definition"]["dockerfile_blob"],
                    dockerfile_blob
                );
                for path in [
                    ".pi/extensions/consumer/index.ts",
                    ".agents/skills/consumer/SKILL.md",
                ] {
                    let blob = project
                        .git(&["rev-parse", &format!("{staging_commit}:{path}")])
                        .trim()
                        .to_string();
                    let bytes = project.git(&["show", &format!("{staging_commit}:{path}")]);
                    let sources = verification["policy"]["definition"]["trusted_staged_sources"]
                        .as_array()
                        .unwrap();
                    assert_eq!(
                        sources
                            .iter()
                            .find(|source| source["path"] == path)
                            .unwrap()["blob"],
                        blob
                    );
                    let sources = verification["launch"]["trusted_bytes"].as_array().unwrap();
                    assert_eq!(
                        sources
                            .iter()
                            .find(|source| source["path"] == path)
                            .unwrap()["sha256"],
                        sha256(bytes.as_bytes())
                    );
                }
            }
        }
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM execution WHERE id IN ({}) ORDER BY id",
                prior_ids.join(",")
            )),
            prior_executions
        );
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM \"check\" WHERE id IN ({}) ORDER BY id",
                prior_check_ids.join(",")
            )),
            prior_checks
        );
        assert_eq!(project.rows(&format!("SELECT * FROM plan_revision WHERE attempt = 1 AND revision < {revision} ORDER BY revision")), prior_revisions);
        current_candidate = produced;
    }
    assert_eq!(
        std::fs::read_to_string(&gate_counter).unwrap(),
        "original\nadopted\nadopted\nadopted\nadopted\n"
    );
    assert_eq!(
        std::fs::read_to_string(&keep_counter).unwrap(),
        "keep\nkeep\nkeep\nkeep\n"
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id IN ({}) ORDER BY id",
            old_ids.join(",")
        )),
        before_routing
    );
    assert_eq!(
        project.rows("SELECT admission FROM attempt WHERE id = 1"),
        original_admission
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id IN ({}) ORDER BY id",
            check_ids.join(",")
        )),
        original_checks
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id IN ({}) ORDER BY id",
            adoption_ids.join(",")
        )),
        adoption_executions
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id IN ({}) ORDER BY id",
            adoption_check_ids.join(",")
        )),
        adoption_checks
    );
    assert_eq!(
        project.rows(
            "SELECT * FROM plan_revision WHERE attempt = 1 AND revision <= 4 ORDER BY revision"
        ),
        original_revisions
    );
    let history = project.json(&["attempt", "show", "Y-1"])["plan_revisions"].clone();
    assert_eq!(
        history
            .as_array()
            .unwrap()
            .iter()
            .map(|revision| revision["revision"].as_i64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
    assert_eq!(history[0]["ticket_revision"], 1);
    assert_eq!(history[1]["ticket_revision"], 1);
    assert_eq!(history[2]["ticket_revision"], 1);
    assert_eq!(history[3]["ticket_revision"], 1);
    assert_eq!(
        history[0]["plan"]["nodes"]["review-2"].get("operator_failure"),
        None
    );
    assert_eq!(
        history[1]["plan"]["nodes"]["review-2"]["operator_failure"],
        true
    );
    assert_eq!(
        history[2]["plan"]["nodes"]["review-0"]["outcomes"],
        json!(["complete"])
    );
    assert_eq!(
        history[2]["plan"]["nodes"]["review-0"]["routes"]["complete"],
        json!([{"to": "review-1", "repeat": null}])
    );
    assert_eq!(
        history[3]["plan"]["nodes"]["review-0"]["routes"]["complete"],
        json!([{"to": "review-2", "repeat": null}])
    );
    assert_eq!(
        history[2]["plan"]["nodes"]["review-0"]["operator_failure"],
        true
    );
    let seq = last_seq(&project);
    drop(operator);
    drop(watch);
    machine.kill();
    machine.start();
    let mut watch = project.watch(seq);
    assert_eq!(
        project.json(&["attempt", "show", "Y-1"])["plan_revisions"],
        history
    );
    let mut operator = machine.operator();
    operator.initialize();
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let decision = json!({"project": project.path, "ticket": "Y-1", "target": "main", "head": candidate,
        "proof": fourth["payload"]["proof"], "plan_revision": 1});
    let stale = operator.call("yard_attempt_approve", decision.clone());
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(
        stale["error"]["data"],
        json!({"identity": "plan_revision", "expected": 1, "current": 9})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows("SELECT state FROM approval"),
        vec![json!({"state": "withdrawn"})]
    );
    let mut decision = decision;
    decision["plan_revision"] = json!(9);
    decision["head"] = json!(current_candidate);
    let accepted = operator.call("yard_attempt_approve", decision);
    assert!(accepted.get("error").is_none(), "{accepted}");
    watch.event("landing.recorded", &[]);
    git(
        &project.canonical(),
        &[
            "merge-base",
            "--is-ancestor",
            &current_candidate,
            "refs/heads/main",
        ],
    );
    // Future landing consumes the selected definitions, on the prepared input.
    assert_eq!(
        std::fs::read_to_string(&gate_counter).unwrap(),
        "original\nadopted\nadopted\nadopted\nadopted\nadopted\n"
    );
    assert_eq!(
        std::fs::read_to_string(&keep_counter).unwrap(),
        "keep\nkeep\nkeep\nkeep\nkeep\n"
    );
    let requests = machine.model.requests();
    let openings = |marker: &str, reviewer: bool| {
        requests
            .iter()
            .filter(|request| {
                request.opens()
                    && request.has_tool("yard_publish_review") == reviewer
                    && !request.has_tool("yard_publish_report")
                    && request.system().contains(marker)
            })
            .count()
    };
    // Reviews: 3 initial + 1 profile + 3 supplier/profile + 1 routing +
    // 2 settings, then 2 extension + 2 skill + 2 image. Implementation:
    // initial, extension, skill, image. The capacity holder is excluded.
    assert_eq!(openings("consumer-source-v1", true), 10);
    assert_eq!(openings("consumer-source-v2", true), 6);
    assert_eq!(openings("consumer-skill-v1", true), 12);
    assert_eq!(openings("consumer-skill-v2", true), 4);
    assert_eq!(openings("consumer-source-v1", false), 1);
    assert_eq!(openings("consumer-source-v2", false), 3);
    assert_eq!(openings("consumer-skill-v1", false), 2);
    assert_eq!(openings("consumer-skill-v2", false), 2);
    assert_eq!(openings("consumer-image-v2", true), 2);
    assert_eq!(openings("consumer-image-v2", false), 1);
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

/// A seat held mid-review, then a ticket edit: the seat stops and its box
/// is gone before the implementer runs for the edit, and the round count
/// is unchanged. Control: the seat's box is listed while the fixture holds
/// it.
///
/// Sabotage: make `admit::steer` return no executions to stop; the seat
/// stays held, its box stays listed and no implementer starts.
#[test]
fn an_edit_stops_a_held_seat_and_costs_no_round() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-held", move |request| {
        if request.has_tool("yard_publish_review") {
            return Reply::Hold(
                held.clone(),
                Box::new(Reply::Tools(vec![publish(json!([]))])),
            );
        }
        if request.opens() && request.last_user().contains("The operator edited") {
            return Reply::Tools(vec![bash(
                "cd /workspace && git commit -q --allow-empty -m Again && echo committed",
            )]);
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &config(""));
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Add feature",
        "--body",
        "Create it",
    ]);
    hold.wait_held();
    let handle =
        project.rows("SELECT handle FROM execution WHERE kind = 'review'")[0]["handle"].clone();
    let listed = |machine: &Machine| {
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|listed| listed["name"] == handle)
    };
    assert!(
        listed(&machine),
        "the held seat's box {handle} is not listed"
    );

    let revision = project.json(&["ticket", "show", "Y-1"])["revision"].to_string();
    project.json(&[
        "ticket",
        "edit",
        "Y-1",
        "--revision",
        &revision,
        "--body",
        "Create it, twice",
    ]);
    watch.until("the edit's implementer", |event| {
        event["event"] == "execution.started"
            && event["ticket"] == "Y-1"
            && event["data"]["reason"] == "edit"
    });
    assert!(!listed(&machine), "the seat's box {handle} is still up");
    assert_eq!(
        project.rows("SELECT outcome FROM execution WHERE kind = 'review' ORDER BY id")[0]["outcome"],
        "stopped"
    );
    assert_eq!(
        project.rows("SELECT rounds FROM attempt")[0]["rounds"],
        json!(0)
    );
    hold.release();
}

/// G2/G5: revise a published seat while its next real model request is held.
/// Hold real box-up stdin EOF during teardown. The old box and host owner
/// remain, with no replacement. The accepted edit controls the stale CAS.
/// Fixture policy and verdict, pinfold and the host lock are external oracles.
/// Ticket edits change candidate identity; this changes policy after publication.
/// Sabotage: count superseded judgment, load startup from latest revision,
/// or release reservation before checked teardown.
#[test]
fn a_live_plan_revision_preserves_publication_and_waits_for_checked_teardown() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-live-plan", move |request| {
        if request.has_tool("yard_publish_review") {
            if request.prompt().contains("Replacement review.") {
                return act(request, vec![publish(json!([]))]);
            }
            if request.opens() {
                return Reply::Tools(vec![publish(
                    json!([{"priority": "P1", "body": "old policy block"}]),
                )]);
            }
            return Reply::Hold(held.clone(), Box::new(Reply::Text("done".into())));
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Feature")],
        )
    });
    let said = machine.root.join("down-said");
    let release = machine.root.join("down-release");
    let armed = machine.root.join("hold-review-down");
    for path in [&said, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }
    let mut writer = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&release)
        .unwrap();
    std::fs::write(&armed, "").unwrap();
    machine.wrapper("pinfold", &format!(
        "if [ \"$1 $2\" = 'box up' ]; then\nread -r spec\ncase \"$spec\" in *'/reviews/'*)\nif [ -f '{}' ]; then\n{{ printf '%s\\n' \"$spec\"; cat; echo held > '{}'; read released < '{}'; }} | \"$REAL\" \"$@\"\nexit $?\nfi;; esac\n{{ printf '%s\\n' \"$spec\"; cat; }} | \"$REAL\" \"$@\"\nexit $?\nfi",
        armed.display(), said.display(), release.display()));
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(
            "[profiles.replacement]\nrunner = {kind = \"agent\", agent = \"worker\", instructions = \"Replacement review.\"}\naccess = \"read-only\"\noutputs = {result = \"verdict\"}\noutcomes = [\"complete\", \"blocked\"]\n",
        ),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Feature"]);
    hold.wait_held();
    let old = project.rows(
        "SELECT id, attempt, handle, plan_revision, inputs FROM execution WHERE kind = 'review'",
    )[0]
    .clone();
    let publication = project.rows("SELECT * FROM \"check\" WHERE kind = 'review'");
    assert_eq!(publication[0]["verdict"], "fail");
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .open(project.path.join(format!(
            ".yard/local/attempts/{}/effects/{}/lock",
            old["attempt"], old["id"]
        )))
        .unwrap();
    let mut operator = machine.operator();
    operator.initialize();
    let adjusted = operator.call(
        "yard_attempt_revise",
        json!({
        "project": project.path, "ticket": "Y-1", "plan_revision": 1,
        "overrides": {"review-0": {"profile": "replacement"}}}),
    );
    assert_eq!(adjusted["result"]["plan_revision"], 2, "{adjusted}");
    assert_eq!(watch.said(&said), "held\n");
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == old["handle"])
    );
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    assert_eq!(
        project.rows(
            "SELECT id, attempt, handle, plan_revision, inputs FROM execution WHERE kind = 'review'"
        ),
        vec![old.clone()]
    );
    assert_eq!(
        project.rows("SELECT reserved, teardown, result FROM execution WHERE kind = 'review'"),
        vec![json!({"reserved": 1, "teardown": 0, "result": null})]
    );
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let revisions = project.rows("SELECT * FROM plan_revision ORDER BY revision");
    let stale = operator.call(
        "yard_attempt_revise",
        json!({
        "project": project.path, "ticket": "Y-1", "plan_revision": 1,
        "overrides": {"review-0": {"profile": "replacement"}}}),
    );
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(
        stale["error"]["data"],
        json!({"identity": "plan_revision", "expected": 1, "current": 2})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows("SELECT * FROM plan_revision ORDER BY revision"),
        revisions
    );
    std::fs::remove_file(&armed).unwrap();
    writeln!(writer, "go").unwrap();
    watch.until("replacement seat", |event| {
        event["event"] == "execution.started"
            && event["data"]["kind"] == "review"
            && event["execution"] != old["id"]
    });
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|row| row["name"] == old["handle"])
    );
    lock.try_lock().unwrap();
    lock.unlock().unwrap();
    let ended = project.rows(&format!(
        "SELECT plan_revision, inputs, result, reserved, teardown FROM execution WHERE id = {}",
        old["id"]
    ))[0]
        .clone();
    assert_eq!(ended["plan_revision"], 1);
    assert_eq!(ended["inputs"], old["inputs"]);
    assert_eq!(ended["reserved"], 0);
    assert_eq!(ended["teardown"], 1);
    let result: Value = serde_json::from_str(ended["result"].as_str().unwrap()).unwrap();
    assert_eq!(result["judgment"]["publication"], publication[0]["id"]);
    let approved = approval(&mut watch);
    assert_eq!(approved["payload"]["plan_revision"], 2);
    assert_eq!(project.rows("SELECT rounds FROM attempt")[0]["rounds"], 0);
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id = {}",
            publication[0]["id"]
        )),
        publication
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT plan_revision, inputs, result, reserved, teardown FROM execution WHERE id = {}",
            old["id"]
        ))[0],
        ended
    );
    hold.release();
}

/// Under `auto` a protected path still raises `approval`. The candidate
/// renames `AGENTS.md` away, so only its old path is protected. Control: an
/// unprotected candidate in the same project is approved automatically and
/// lands.
///
/// Sabotage: drop `--no-renames` from `git::changed_paths`; the rename shows
/// only its new path and the candidate is approved automatically.
#[test]
fn auto_approval_raises_approval_on_a_protected_path() {
    let machine = Machine::new("g2-protected", |request| {
        if request.prompt().contains("Move the guidance") {
            return act(
                request,
                vec![bash(
                    "cd /workspace && mkdir -p docs && git mv AGENTS.md docs/AGENTS.md \
                     && git commit -q -m Move && echo committed",
                )],
            );
        }
        act(
            request,
            vec![commit_file("feature.txt", "feature\n", "Add feature")],
        )
    });
    machine.start();
    let project = Project::new(&machine, "p", &auto(unreviewed("")));
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Move the guidance"]);
    project.json(&["ticket", "new", "--title", "Add feature"]);

    let decided = watch.find("Y-1 decided", |event| {
        event["ticket"] == "Y-1"
            && (event["event"] == "attention.raised" || event["event"] == "approval.given")
    });
    assert_eq!(decided["event"], "attention.raised", "{decided}");
    assert_eq!(decided["data"]["kind"], "approval");
    assert_eq!(
        decided["data"]["payload"]["protected"],
        json!(["AGENTS.md"])
    );

    watch.find("Y-2 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-2"
    });
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

/// An automatic approval, a sync that sets `approve = "manual"`, and the
/// attempt abandoned while the landing reads the candidate's protected paths
/// at its intent: the landing withdraws, and no `approval` is raised for the
/// ended attempt. After checked cleanup, a live candidate in this same
/// project loses automatic approval under the same manual policy change.
/// Its existing checks survive, and operator approval lands it.
#[test]
fn an_attempt_abandoned_while_its_landing_rereads_policy_raises_nothing() {
    let machine = Machine::new("g2-abandon", |request| {
        let content = if request.context().contains("Live notes") {
            "live notes\n"
        } else {
            "notes\n"
        };
        act(
            request,
            vec![commit_file("docs/notes.md", content, "Add notes")],
        )
    });
    let candidate_gate =
        "[gates.check]\ncommand = \"test -f docs/notes.md\"\nstage = \"candidate\"\n\n";
    let (project, mut watch, mut release, _) =
        held_after_policy_change(&machine, candidate_gate, |config| {
            config.replace("approve = \"auto\"", "approve = \"manual\"")
        });
    let first = project.json(&["attempt", "show", "Y-1"]);
    let first_ticket = first["attempt"]["ticket"].as_str().unwrap();
    let abandoned = first["attempt"]["attempt"].clone();
    let fifo = |name: &str| {
        let path = machine.root.join(name);
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        path
    };
    let (said, resume) = (fifo("said"), fifo("resume"));
    let armed = machine.root.join("armed");
    std::fs::write(&armed, "").unwrap();
    // Armed after the sync, so the next protected-path read is the intent's.
    machine.wrapper(
        "git",
        &format!(
            "case \" $* \" in *' diff --name-only '*) if rm '{}' 2>/dev/null; then echo held > '{}'; read line < '{}'; fi;; esac",
            armed.display(),
            said.display(),
            resume.display()
        ),
    );
    release.write_all(b"go\n").unwrap();
    assert_eq!(watch.said(&said), "held\n");
    project.json(&["ticket", "park", first_ticket]);
    project.json(&["attempt", "abandon", first_ticket]);
    std::fs::write(&resume, "go\n").unwrap();
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn", "{ended}");
    // G2: remove queue::admit's post-protected-path ended-state refusal and
    // queue::lapsed's live-state eligibility together; an open approval then
    // survives for the abandoned candidate after the protected-path read.
    assert_eq!(
        project.rows("SELECT kind FROM attention WHERE state = 'open'"),
        Vec::<Value>::new()
    );
    let cleanup = watch.find("abandoned candidate cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == abandoned
    });
    assert_eq!(cleanup["data"]["outcome"], "removed");
    watch.find("abandoned cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });

    let base = auto(unreviewed(&format!(
        "{candidate_gate}[gates.held]\ncommand = \"read line < {}\"\nruns_in = \"host\"\n",
        machine.root.join("hold").display()
    )));
    project.reconfigure(&base);
    let candidate_base = project.canonical_head();
    let created = project.json(&["ticket", "new", "--title", "Live notes"]);
    let ticket = created["ticket"].as_str().unwrap();
    let approved = watch.until("live ticket auto approval", |event| {
        event["event"] == "approval.given"
            && event["data"]["actor"] == "auto"
            && event["ticket"] == ticket
    });
    let attempt = approved["attempt"].as_i64().unwrap();
    let clone = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    assert_eq!(approved["data"]["head"], head);
    watch.event("execution.started", &[("name", "held")]);
    let judgments = format!(
        "SELECT e.*, c.id AS check_id, c.execution AS check_execution, c.kind AS check_kind, c.verdict, c.created_at AS check_created_at FROM execution e LEFT JOIN \"check\" c ON c.execution = e.id WHERE e.attempt = {attempt} AND e.kind IN ('gate', 'review') AND e.reason = 'candidate' ORDER BY e.id"
    );
    let checks = project.rows(&judgments);
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0]["node"], "candidate-gate-check");
    assert_eq!(checks[0]["verdict"], "pass");
    let turns = machine
        .model
        .requests()
        .iter()
        .filter(|request| request.opens())
        .count();
    project.reconfigure(&base.replace("approve = \"auto\"", "approve = \"manual\""));
    std::fs::write(&armed, "").unwrap();
    release.write_all(b"go\n").unwrap();
    assert_eq!(watch.said(&said), "held\n");
    std::fs::write(&resume, "go\n").unwrap();
    // G2: read approval policy in queue::holds from the prior Loaded instead
    // of current policy; the landing retires instead of withdrawing.
    let ended = watch.event("execution.ended", &[("kind", "landing")]);
    assert_eq!(ended["data"]["outcome"], "withdrawn", "{ended}");
    let raised = watch.attention();
    assert_eq!(raised["attempt"], attempt);
    assert_eq!(raised["data"]["kind"], "approval");
    let payload = &raised["data"]["payload"];
    assert_eq!(payload["head"], head);
    assert_eq!(payload["base"], candidate_base);
    assert_eq!(payload["proof"], approved["data"]["proof"]);
    assert_eq!(payload["target"], "main");
    assert_eq!(payload["revision"], 1);
    assert_eq!(payload["plan_revision"], 1);
    assert_eq!(payload["protected"], json!([]));
    assert_eq!(payload["checks"], json!([checks[0]["check_id"]]));
    assert_eq!(
        project.rows(&format!(
            "SELECT actor, state FROM approval WHERE attempt = {attempt}"
        )),
        vec![json!({"actor": "auto", "state": "withdrawn"})]
    );
    assert_eq!(project.rows(&judgments), checks);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|request| request.opens())
            .count(),
        turns
    );
    project.json(&[
        "attempt", "approve", "--target", "main", ticket, "--head", &head,
    ]);
    release.write_all(b"go\n").unwrap();
    let landed = watch.event("landing.recorded", &[]);
    assert_eq!(landed["attempt"], attempt);
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", &head, "refs/heads/main"],
    );
    assert_eq!(project.json(&["ticket", "show", ticket])["state"], "done");
    assert_eq!(project.rows(&judgments), checks);
    assert_eq!(
        machine
            .model
            .requests()
            .iter()
            .filter(|request| request.opens())
            .count(),
        turns
    );
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

/// G2/G8: explicit skipped acceptance follows a full default landing in the
/// same project. A reduced plan retains one candidate gate and no seats.
/// An edit holds the new worker before changing head: delayed decisions name
/// the old revision, then old head, and write nothing. Wrong plan and retained
/// node identities also refuse; current identities record the operator's text.
/// A real worker proof makes omitted and mismatched proof decisions stale;
/// malformed proof remains invalid. The current proof controls both refusals.
/// Expected nodes and revisions are fixture literals; base/head come from git.
/// The pre-decision execution/check snapshot must survive acceptance, landing
/// and a quiescent reopen. Existing omission coverage has no explicit decision.
/// Sabotage: manufacture a passing check/result for an accepted omission, drop
/// skipped-node or plan binding, drop revision/head binding, or mark the empty
/// panel reviewed on acceptance. Remove approval evidence persistence to lose
/// the decision at landing or reopen. Accept blank text to lose the required
/// operator explanation.
/// Sabotage: classify missing required proof as invalid or ignore proof binding.
#[test]
fn selected_skipped_decisions_bind_identity_without_manufacturing_results() {
    let hold = Latch::new();
    let held = hold.clone();
    let machine = Machine::new("g2-skipped", move |request| {
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
        let path = if request.context().contains("Explicit empty") {
            "explicit-empty.txt"
        } else if request.context().contains("Ordinary empty") {
            "ordinary-empty.txt"
        } else if request.context().contains("Reduced") {
            "reduced.txt"
        } else {
            "full.txt"
        };
        let mut calls = vec![commit_file(path, "feature\n", "Feature")];
        if path == "reduced.txt" {
            calls.push(bash(
                "printf 'selected evidence' > /yard/proof/evidence.txt",
            ));
        }
        act(request, calls)
    });
    let armed = machine.root.join("landing-armed");
    let said = machine.root.join("landing-said");
    let release = machine.root.join("landing-release");
    for fifo in [&said, &release] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    let _writer = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&release)
        .unwrap();
    machine.start();
    let project = Project::new(
        &machine,
        "p",
        &config(&format!(
            "[gates.check]\ncommand = \"git rev-parse HEAD\"\nstage = \"candidate\"\n\
         [gates.keep]\ncommand = 'if test -f {}; then rm {}; git rev-parse HEAD > {}; read released < {}; fi; git rev-parse HEAD'\nstage = \"candidate\"\nruns_in = \"host\"\n\
         [gates.other]\ncommand = \"git rev-parse HEAD\"\nstage = \"candidate\"\n",
            armed.display(),
            armed.display(),
            said.display(),
            release.display(),
        )),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Full"]);
    let full = approval(&mut watch);
    project.json(&[
        "attempt",
        "approve",
        "Y-1",
        "--target",
        "main",
        "--head",
        full["payload"]["head"].as_str().unwrap(),
    ]);
    watch.event("landing.recorded", &[]);
    // Ordinary approval must not silently become explicit skipped acceptance.
    assert_eq!(
        project.rows("SELECT accepted_skipped FROM approval"),
        vec![json!({"accepted_skipped": null})]
    );
    let base = project.canonical_head();
    project.json(&["ticket", "new", "--title", "Reduced", "--workflow", "plan"]);
    watch.event("attempt.ended", &[("outcome", "planned")]);
    let planner = project.json(&["attempt", "show", "Y-2"])["attempt"]["attempt"].clone();
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    // Retained gates keep their ordinary failed repair edge even without a successor.
    let graph = json!({"inputs": {"feedback": "verdict", "merged": "candidate"}, "nodes": {
        "implementer": {
            "runner": {"kind": "agent", "agent": "worker", "instructions": ""},
            "access": "write-allowed", "inputs": {"feedback": {"output": "feedback", "kind": "verdict"}},
            "outputs": {"result": "candidate"}, "outcomes": ["complete"], "requires": [],
            "routes": {"complete": [{"to": "candidate-gate-keep"}]}
        },
        "candidate-gate-keep": {
            "runner": {"kind": "command", "gate": "keep"}, "access": "write-allowed",
            "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "failed"], "requires": ["candidate"],
            "routes": {"complete": [{"to": "landing-gate-keep"}], "failed": [{"to": "implementer", "repeat": 1}]}
        },
        "landing-gate-keep": {
            "runner": {"kind": "command", "gate": "keep"}, "access": "write-allowed",
            "inputs": {"merged": {"output": "merged", "kind": "candidate"}},
            "outputs": {"result": "verdict"}, "outcomes": ["complete"], "requires": ["merged"],
            "routes": {}
        }
    }});
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-2", "graph": graph.clone()}),
    );
    assert!(started.get("error").is_none(), "{started}");
    let first = approval(&mut watch);
    let shown = project.json(&["attempt", "show", "Y-2"]);
    let first_attention = shown["attention"][0]["attention"].clone();
    let attempt = shown["attempt"]["attempt"].as_i64().unwrap();
    let clone = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let first_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    let delayed = json!({"project": project.path, "ticket": "Y-2", "target": "main", "base": base,
        "head": first_head, "proof": first["payload"]["proof"], "revision": 1, "plan_revision": 1,
        "attention": first_attention,
        "accept_skipped": ["candidate-gate-check", "review-0"], "text": "Accept the selected candidate omissions"});
    project.json(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        "1",
        "--body",
        "Reduced, edited",
    ]);
    hold.wait_held();
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_approve", delayed.clone());
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "revision", "expected": 1, "current": 2})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    hold.release();
    let current = approval(&mut watch);
    let current_attention =
        project.json(&["attempt", "show", "Y-2"])["attention"][0]["attention"].clone();
    let head = git(&clone, &["rev-parse", "HEAD"]).trim().to_string();
    assert_ne!(head, first_head);
    let current_proof = proof(&current);
    let snapshot = project.path.join(format!(
        ".yard/local/attempts/{attempt}/proof-snapshots/{current_proof}/evidence.txt"
    ));
    assert_eq!(std::fs::read(snapshot).unwrap(), b"selected evidence");
    let mut decision = delayed;
    decision["head"] = json!(head);
    decision["revision"] = json!(2);
    decision["attention"] = current_attention.clone();
    decision["proof"] = json!(current_proof);
    for (key, old, identity, expected) in [
        ("head", json!(first_head), "head", json!(head)),
        (
            "attention",
            first_attention.clone(),
            "attention",
            current_attention.clone(),
        ),
        ("plan_revision", json!(0), "plan_revision", json!(1)),
        ("proof", json!("wrong-proof"), "proof", json!(current_proof)),
        (
            "accept_skipped",
            json!(["candidate-gate-keep"]),
            "skipped",
            json!(["candidate-gate-check", "candidate-gate-other", "review-0"]),
        ),
    ] {
        let mut stale = decision.clone();
        stale[key] = old.clone();
        let before = project.rows("SELECT * FROM audit ORDER BY seq");
        let approvals = project.rows("SELECT * FROM approval ORDER BY id");
        let refused = operator.call("yard_attempt_approve", stale);
        assert_eq!(refused["error"]["code"], "stale");
        assert_eq!(
            refused["error"]["data"],
            json!({"identity": identity, "expected": old, "current": expected})
        );
        assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
        assert_eq!(
            project.rows("SELECT * FROM approval ORDER BY id"),
            approvals
        );
    }
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let approvals = project.rows("SELECT * FROM approval ORDER BY id");
    let mut omitted = decision.clone();
    omitted.as_object_mut().unwrap().remove("proof");
    let refused = operator.call("yard_attempt_approve", omitted);
    assert_eq!(refused["error"]["code"], "stale");
    assert_eq!(
        refused["error"]["data"],
        json!({"identity": "proof", "expected": "", "current": current_proof})
    );
    let mut malformed = decision.clone();
    malformed["proof"] = json!(42);
    let refused = operator.call("yard_attempt_approve", malformed);
    assert_eq!(refused["error"]["code"], "invalid");
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows("SELECT * FROM approval ORDER BY id"),
        approvals
    );
    // This snapshot proves the acceptance adds no execution, result or check.
    // The retained gate already finished; omissions have no rows to accept.
    let judgments = format!(
        "SELECT e.node, e.head, e.proof, e.result, c.verdict FROM execution e LEFT JOIN \"check\" c ON c.execution = e.id WHERE e.attempt = {attempt} AND e.kind IN ('gate', 'review') AND (e.reason IS NULL OR e.reason != 'landing') ORDER BY e.id"
    );
    let original = project.rows(&judgments);
    assert_eq!(original.len(), 2); // first candidate, then the edited candidate
    assert!(
        original
            .iter()
            .all(|row| row["node"] == "candidate-gate-keep"
                && row["verdict"] == "pass"
                && row["proof"] == current_proof)
    );
    assert_eq!(current["payload"]["unreviewed"], true);
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let mut blank = decision.clone();
    blank["text"] = json!("   ");
    let refused = operator.call("yard_attempt_approve", blank);
    assert_eq!(refused["error"]["code"], "invalid");
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    decision["observe_landing_skipped"] = json!(["landing-gate-check", "landing-gate-other"]);
    let accepted = operator.call("yard_attempt_approve", decision.clone());
    assert!(accepted.get("error").is_none(), "{accepted}");
    let expected = json!({"identity": {"attempt": attempt, "target": "main", "base": base, "head": head, "proof": current["payload"]["proof"], "revision": 2, "plan_revision": 1,
        "skipped": ["candidate-gate-check", "candidate-gate-other", "review-0"]}, "attention": current_attention, "nodes": ["candidate-gate-check", "review-0"], "actor": "operator", "text": decision["text"]});
    let evidence =
        format!("SELECT accepted_skipped, actor, overrode FROM approval WHERE attempt = {attempt}");
    let rows =
        vec![json!({"accepted_skipped": expected.to_string(), "actor": "operator", "overrode": 1})];
    assert_eq!(project.rows(&evidence), rows);
    // G2/G3/G8: sabotage by fabricating an omitted pass, inheriting an
    // answer on a new merge, or consuming before checked ownership release.
    let preparation = watch.find("omission preparation", |event| {
        event["event"] == "landing.prepared" && event["attempt"] == attempt
    });
    let omission = watch.find("prepared omissions", |event| {
        event["event"] == "attention.raised"
            && event["attempt"] == attempt
            && event["data"]["reason"] == "landing"
            && event["data"]["payload"]["evidence"]["landing"]["execution"]
                == preparation["execution"]
            && event["seq"].as_i64().unwrap() > preparation["seq"].as_i64().unwrap()
    });
    let omission_evidence = omission["data"]["payload"]["evidence"].clone();
    let identity = &omission_evidence["landing"];
    let parent = identity["execution"].as_i64().unwrap();
    watch.find("omission parent released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == parent
            && event["seq"].as_i64().unwrap() > omission["seq"].as_i64().unwrap()
    });
    let merged = git(
        &project.canonical(),
        &["rev-parse", &format!("refs/yard/landings/{parent}")],
    )
    .trim()
    .to_string();
    assert_eq!(identity["prepared"]["merged"], merged);
    assert_eq!(
        omission_evidence["identity"],
        json!({"attempt": attempt, "target": "main",
        "base": base, "head": head, "proof": current_proof, "revision": 2, "plan_revision": 1,
        "skipped": ["landing-gate-check", "landing-gate-other"]})
    );
    let mut merged_decision = json!({"project": project.path, "ticket": "Y-2", "attention": omission["attention"],
        "plan_revision": 1, "target": "main", "base": base, "head": head, "proof": current_proof,
        "revision": 2, "landing": parent, "old": identity["prepared"]["old"], "merged": merged,
        "accept_skipped": ["landing-gate-check", "landing-gate-other"], "text": "Accept these merged omissions"});
    let parent_lock = std::fs::OpenOptions::new()
        .read(true)
        .open(
            project
                .path
                .join(format!(".yard/local/landings/{parent}/lock")),
        )
        .unwrap();
    parent_lock.try_lock().unwrap();
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let questions = project.rows("SELECT * FROM attention ORDER BY id");
    let refused = operator.call("yard_attempt_accept", merged_decision.clone());
    assert_eq!(refused["error"]["code"], "stale");
    let data = &refused["error"]["data"];
    assert_eq!(data["identity"], "ownership");
    assert_eq!(
        data["expected"],
        json!({"execution": parent, "released": true})
    );
    assert_eq!(data["current"]["execution"], parent);
    assert_eq!(data["current"]["status"], "ended");
    assert_eq!(data["current"]["reserved"], false);
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows("SELECT * FROM attention ORDER BY id"),
        questions
    );
    parent_lock.unlock().unwrap();
    std::fs::write(&armed, "").unwrap();
    let answer = operator.call("yard_attempt_accept", merged_decision.clone());
    assert!(answer.get("error").is_none(), "{answer}");
    let accepted_omission = format!(
        "SELECT * FROM attention WHERE id = {}",
        omission["attention"]
    );
    let omission_snapshot = project.rows(&accepted_omission);
    assert_eq!(watch.said(&said).trim(), merged);
    let omissions = format!(
        "SELECT id FROM execution WHERE attempt = {attempt} AND node IN ('landing-gate-check', 'landing-gate-other')"
    );
    assert!(project.rows(&omissions).is_empty());
    // The host child holds a real point while canonical advances. Restart
    // must retire the old input and ask again, with no inherited decision.
    let old = project.canonical_head();
    let tree = git(
        &project.canonical(),
        &["rev-parse", &format!("{old}^{{tree}}")],
    )
    .trim()
    .to_string();
    let moved = git(
        &project.canonical(),
        &["commit-tree", &tree, "-p", &old, "-m", "Operator advance"],
    )
    .trim()
    .to_string();
    git(
        &project.canonical(),
        &["update-ref", "refs/heads/main", &moved, &old],
    );
    assert_eq!(project.rows(&format!("SELECT node, head, status FROM execution WHERE attempt = {attempt} AND reason = 'landing' AND status = 'running'")),
        vec![json!({"node": "landing-gate-keep", "head": merged, "status": "running"})]);
    // Resume at the consumed cursor so queued pre-crash events are replayed.
    let seen = std::mem::take(&mut watch.seen);
    let since = seen.last().unwrap()["seq"].as_i64().unwrap();
    drop(watch);
    machine.kill();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&release)
        .unwrap()
        .write_all(b"released\n")
        .unwrap();
    machine.start();
    watch = project.watch(since);
    watch.seen = seen;
    let fresh_preparation = watch.find("fresh merged preparation", |event| {
        event["event"] == "landing.prepared"
            && event["attempt"] == attempt
            && event["execution"] != parent
            && event["data"]["old"] == moved
            && event["seq"].as_i64().unwrap() > omission["seq"].as_i64().unwrap()
    });
    let fresh = watch.find("fresh omission preparation", |event| {
        event["event"] == "attention.raised"
            && event["attempt"] == attempt
            && event["data"]["reason"] == "landing"
            && event["data"]["payload"]["evidence"]["landing"]["execution"]
                == fresh_preparation["execution"]
            && event["attention"] != omission["attention"]
            && event["seq"].as_i64().unwrap() > fresh_preparation["seq"].as_i64().unwrap()
    });
    let fresh_identity = &fresh["data"]["payload"]["evidence"]["landing"];
    let fresh_parent = fresh_identity["execution"].as_i64().unwrap();
    watch.find("fresh omission release", |event| {
        event["event"] == "execution.released"
            && event["execution"] == fresh_parent
            && event["seq"].as_i64().unwrap() > fresh["seq"].as_i64().unwrap()
    });
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let decisions_before = project.rows("SELECT * FROM attention ORDER BY id");
    let stale = operator.call("yard_attempt_accept", merged_decision.clone());
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(
        stale["error"]["data"],
        json!({"identity": "old", "expected": old, "current": moved})
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows("SELECT * FROM attention ORDER BY id"),
        decisions_before
    );
    assert_eq!(project.rows(&accepted_omission), omission_snapshot);
    merged_decision["attention"] = fresh["attention"].clone();
    merged_decision["landing"] = json!(fresh_parent);
    merged_decision["old"] = json!(moved);
    let fresh_head = git(
        &project.canonical(),
        &["rev-parse", &format!("refs/yard/landings/{fresh_parent}")],
    )
    .trim()
    .to_string();
    merged_decision["merged"] = json!(fresh_head);
    let answer = operator.call("yard_attempt_accept", merged_decision);
    assert!(answer.get("error").is_none(), "{answer}");
    watch.event("landing.recorded", &[]);
    assert_eq!(project.canonical_head(), fresh_head);
    assert_eq!(project.rows(&format!("SELECT node, head, outcome FROM execution WHERE attempt = {attempt} AND reason = 'landing' AND head = '{fresh_head}'")),
        vec![json!({"node": "landing-gate-keep", "head": fresh_head, "outcome": "pass"})]);
    // G3/G8: sabotage by landing with an ineligible retained gate result.
    let shown = project.json(&["attempt", "show", "Y-2"]);
    let fresh_gates: Vec<_> = shown["executions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|execution| {
            execution["kind"] == "gate"
                && execution["reason"] == "landing"
                && execution["head"] == fresh_head
        })
        .map(|execution| {
            json!({"node": execution["binding"]["node"],
            "head": execution["head"], "outcome": execution["outcome"],
            "result_eligible": execution["result_eligible"]})
        })
        .collect();
    assert_eq!(
        fresh_gates,
        vec![json!({"node": "landing-gate-keep", "head": fresh_head,
        "outcome": "pass", "result_eligible": true})]
    );
    assert!(project.rows(&omissions).is_empty());
    let cleanup = watch.find("accepted candidate cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == attempt
    });
    watch.find("cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    git(
        &project.canonical(),
        &["merge-base", "--is-ancestor", &head, "refs/heads/main"],
    );
    assert_eq!(project.rows(&evidence), rows);
    assert_eq!(project.rows(&judgments), original);
    let admission = format!(
        "SELECT json_extract(admission, '$.unreviewed') AS unreviewed FROM attempt WHERE id = {attempt}"
    );
    assert_eq!(project.rows(&admission), vec![json!({"unreviewed": 1})]);
    let seen = std::mem::take(&mut watch.seen);
    let since = seen.last().unwrap()["seq"].as_i64().unwrap();
    drop(watch);
    machine.kill();
    machine.start();
    watch = project.watch(since);
    watch.seen = seen;
    project.json(&["status"]);
    assert_eq!(project.rows(&evidence), rows);
    assert_eq!(project.rows(&judgments), original);
    assert_eq!(project.rows(&admission), vec![json!({"unreviewed": 1})]);
    assert!(project.rows(&omissions).is_empty());
    // Ordinary empty selection observes no omission decision; G1 covers reduced landing.
    // Sabotage: synthesize omission evidence on ordinary approval without a request.
    assert_eq!(project.rows(&accepted_omission), omission_snapshot);
    // Explicit empty selection must expose the host-Git merge before intent.
    let mut empty = graph.clone();
    empty["inputs"].as_object_mut().unwrap().remove("merged");
    empty["nodes"]
        .as_object_mut()
        .unwrap()
        .retain(|name, _| name == "implementer");
    empty["nodes"]["implementer"]["routes"] = json!({});
    for (ticket, title, selected, explicit) in [
        ("Y-3", "Ordinary empty", empty.clone(), false),
        ("Y-4", "Explicit empty", empty, true),
    ] {
        project.json(&["ticket", "new", "--title", title, "--workflow", "plan"]);
        let planned = watch.event("attempt.ended", &[("outcome", "planned")]);
        let cleanup = watch.find("selection planner cleanup", |event| {
            event["event"] == "execution.ended"
                && event["data"]["kind"] == "cleanup"
                && event["attempt"] == planned["attempt"]
        });
        watch.find("selection planner release", |event| {
            event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
        });
        let start = operator.call(
            "yard_attempt_start",
            json!({"project": project.path, "ticket": ticket, "graph": selected}),
        );
        assert!(start.get("error").is_none(), "{start}");
        let approved = approval(&mut watch);
        let mut approve = json!({"project": project.path, "ticket": ticket, "target": "main", "head": approved["payload"]["head"]});
        if explicit {
            approve["observe_landing_skipped"] = json!(["landing-gate-check"]);
        }
        let old = project.canonical_head();
        let result = operator.call("yard_attempt_approve", approve);
        assert!(result.get("error").is_none(), "{result}");
        let attempt = project.json(&["attempt", "show", ticket])["attempt"]["attempt"]
            .as_i64()
            .unwrap();
        let omission_rows = format!(
            "SELECT * FROM attention WHERE attempt = {attempt} AND json_type(payload, '$.evidence.nodes') = 'array' ORDER BY id"
        );
        let selected_rows = format!(
            "SELECT e.node, e.result, c.id AS check_id FROM execution e LEFT JOIN \"check\" c ON c.execution = e.id WHERE e.attempt = {attempt} AND e.kind IN ('gate', 'review') ORDER BY e.id"
        );
        let mut empty_snapshot = None;
        if explicit {
            let preparation = watch.find("empty merged preparation", |event| {
                event["event"] == "landing.prepared"
                    && event["attempt"] == attempt
                    && event["data"]["old"] == old
                    && event["seq"].as_i64().unwrap() > fresh["seq"].as_i64().unwrap()
            });
            let item = watch.find("empty prepared attention", |event| {
                event["event"] == "attention.raised"
                    && event["attempt"] == attempt
                    && event["data"]["reason"] == "landing"
                    && event["data"]["payload"]["evidence"]["landing"]["execution"]
                        == preparation["execution"]
                    && event["attention"] != fresh["attention"]
                    && event["seq"].as_i64().unwrap() > preparation["seq"].as_i64().unwrap()
            });
            let identity = &item["data"]["payload"]["evidence"]["landing"];
            let parent = identity["execution"].as_i64().unwrap();
            watch.find("empty preparation released", |event| {
                event["event"] == "execution.released"
                    && event["execution"] == parent
                    && event["seq"].as_i64().unwrap() > item["seq"].as_i64().unwrap()
            });
            let merged = git(
                &project.canonical(),
                &["rev-parse", &format!("refs/yard/landings/{parent}")],
            )
            .trim()
            .to_string();
            assert_eq!(project.canonical_head(), old);
            assert_eq!(identity["prepared"]["merged"], merged);
            assert_eq!(
                project.rows(&format!(
                    "SELECT intent_state FROM execution WHERE id = {parent}"
                )),
                vec![json!({"intent_state": null})]
            );
            assert!(project.rows(&selected_rows).is_empty());
            assert_eq!(project.rows(&format!("SELECT json_extract(admission, '$.unreviewed') AS unreviewed FROM attempt WHERE id = {attempt}")), vec![json!({"unreviewed": 1})]);
            let pending = project.rows(&omission_rows);
            let seen = std::mem::take(&mut watch.seen);
            let since = seen.last().unwrap()["seq"].as_i64().unwrap();
            drop(watch);
            machine.kill();
            machine.start();
            watch = project.watch(since);
            watch.seen = seen;
            project.json(&["status"]);
            assert_eq!(project.rows(&omission_rows), pending);
            let accepted = operator.call("yard_attempt_accept", json!({"project": project.path, "ticket": ticket,
                "attention": item["attention"], "plan_revision": 1, "target": "main", "head": identity["head"],
                "base": identity["base"], "revision": 1, "landing": parent, "old": old, "merged": merged,
                "accept_skipped": ["landing-gate-check"], "text": "Accept named empty landing omission"}));
            assert!(accepted.get("error").is_none(), "{accepted}");
            empty_snapshot = Some((merged, project.rows(&omission_rows)));
        }
        watch.event("landing.recorded", &[]);
        if let Some((merged, snapshot)) = empty_snapshot {
            assert_eq!(project.canonical_head(), merged);
            assert!(project.rows(&selected_rows).is_empty());
            assert_eq!(project.rows(&omission_rows), snapshot);
            let cleanup = watch.find("empty landing cleanup", |event| {
                event["event"] == "execution.ended"
                    && event["data"]["kind"] == "cleanup"
                    && event["attempt"] == attempt
            });
            watch.find("empty cleanup release", |event| {
                event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
            });
            let seen = std::mem::take(&mut watch.seen);
            let since = seen.last().unwrap()["seq"].as_i64().unwrap();
            drop(watch);
            machine.kill();
            machine.start();
            watch = project.watch(since);
            watch.seen = seen;
            project.json(&["status"]);
            assert_eq!(project.rows(&omission_rows), snapshot);
            assert!(project.rows(&selected_rows).is_empty());
        } else {
            assert!(project.rows(&omission_rows).is_empty());
        }
    }
}
