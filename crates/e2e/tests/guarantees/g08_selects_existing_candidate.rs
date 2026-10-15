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

/// G2/G8: operator failure policy on a chosen source uses the same exact
/// failed-check accept/repair path as ordinary verification. Two producers and
/// their same-profile reviewers disagree; only B's failures can be answered.
/// Git supplies heads and version bytes. Existing ordinary failure controls
/// have no selector, and the five-model trial uses automatic repair/limit.
/// Sabotage: reject operator_failure on admitted verification, skip selected
/// review attention, omit the landing failed outcome, copy A's review, treat
/// repair as acceptance, or rewrite failed evidence after acceptance.
#[test]
fn selected_failures_keep_exact_operator_accept_and_repair() {
    let versions = AtomicUsize::new(0);
    let machine = Machine::new("g8-selected-failure-policy", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        if request.has_tool("yard_publish_selection") {
            return act(
                request,
                vec![tool("yard_publish_selection", json!({"candidate": "b"}))],
            );
        }
        if request.has_tool("yard_publish_review") {
            if request.opens() {
                return Reply::Tools(vec![bash("printf 'review-version-'; cat version.txt")]);
            }
            if let Some((_, text)) = request.tool_results().last()
                && text.contains("review-version-")
            {
                let passing =
                    text.contains("review-version-3") || text.contains("review-version-10");
                return Reply::Tools(vec![publish(if passing {
                    json!([])
                } else {
                    json!([{ "priority": "P1", "body": "selected-review-policy-failure" }])
                })]);
            }
            return Reply::Text("reviewed".into());
        }
        if !request.opens() {
            return Reply::Text("done".into());
        }
        let selected = request.body["model"] == "selected-producer";
        let version = if selected {
            versions.fetch_add(1, Ordering::SeqCst)
        } else {
            10
        };
        Reply::Tools(vec![bash(&format!(
            "cd /workspace && printf '{}\\n' > source.txt && printf '{version}\\n' > version.txt && git add source.txt version.txt && git commit -qm Publication-{version}",
            if selected { "B" } else { "A" }
        ))])
    });
    machine.start();
    let configuration = config(
        "\n[agents.a]\nharness = 'pi'\nprovider = 'openrouter'\nmodel = 'losing-producer'\n\
         [agents.b]\nharness = 'pi'\nprovider = 'openrouter'\nmodel = 'selected-producer'\n\
         [profiles.a]\nrunner = {kind = 'agent', agent = 'a', instructions = 'Implement independently.'}\naccess = 'write-allowed'\noutputs = {result = 'candidate'}\noutcomes = ['complete']\n\
         [profiles.b]\nrunner = {kind = 'agent', agent = 'b', instructions = 'Implement independently.'}\naccess = 'write-allowed'\noutputs = {result = 'candidate'}\noutcomes = ['complete']\n\
         [profiles.reader]\nrunner = {kind = 'agent', agent = 'worker', instructions = 'Same frozen reviewer.'}\naccess = 'read-only'\noutputs = {result = 'verdict'}\noutcomes = ['complete', 'blocked']\n\
         [profiles.picker]\nrunner = {kind = 'agent', agent = 'worker', instructions = 'Choose declared candidate b.'}\naccess = 'read-only'\noutputs = {result = 'selection'}\noutcomes = ['complete']\n\
         [review.seats.selected]\nagent = 'worker'\ninstructions = 'Same frozen reviewer.'\n\
         [gates.check]\ncommand = 'test \"$(cat source.txt)\" = B && { test \"$(cat version.txt)\" -ge 2 || { echo candidate-policy-failure; exit 1; }; }'\nstage = 'candidate'\n\
         [gates.merged]\ncommand = 'test \"$(cat version.txt)\" -ge 4 || { echo merged-policy-failure; exit 1; }'\nstage = 'landing'\n"
    ).replace("review = [\"correctness\"]", "review = [\"correctness\", \"selected\"]");
    let project = Project::new(&machine, "p", &configuration);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Selected operator failures",
        "--workflow",
        "plan",
    ]);
    let planner = watch.event("attempt.ended", &[]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner["attempt"]
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let mut nodes = json!({
        "implementer": {"profile": "a", "routes": {"complete": [{"to": "review-0"}, {"to": "selector"}]}},
        "writer-b": {"profile": "b", "routes": {"complete": [{"to": "review-1"}, {"to": "selector"}]}},
        "review-0": {"profile": "reader", "operator_failure": true, "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}}, "requires": ["candidate"], "routes": {"complete": [{"to": "selector"}]}},
        "review-1": {"profile": "reader", "operator_failure": true, "inputs": {"candidate": {"node": "writer-b", "output": "result", "kind": "candidate"}}, "requires": ["candidate"], "routes": {"complete": [{"to": "selector"}]}},
        "selector": {"profile": "picker", "inputs": {
            "a": {"node": "implementer", "output": "result", "kind": "candidate"},
            "b": {"node": "writer-b", "output": "result", "kind": "candidate"},
            "ra": {"node": "review-0", "output": "result", "kind": "verdict"},
            "rb": {"node": "review-1", "output": "result", "kind": "verdict"}
        }, "requires": ["a", "b", "ra", "rb"], "routes": {"complete": [{"to": "candidate-gate-check"}]}}
    });
    for (name, gate, next, landing) in [
        (
            "candidate-gate-check",
            "check",
            Some("landing-gate-check"),
            false,
        ),
        (
            "landing-gate-check",
            "check",
            Some("landing-gate-merged"),
            true,
        ),
        ("landing-gate-merged", "merged", None, true),
    ] {
        let mut node = json!({"runner": {"kind": "command", "gate": gate},
            "access": "write-allowed", "operator_failure": true, "inputs": {},
            "requires": [if landing {"merged"} else {"candidate"}],
            "outputs": {"result": "verdict"}, "outcomes": ["complete", "failed"], "routes": {}});
        node["inputs"][if landing { "merged" } else { "candidate" }] = if landing {
            json!({"node": null, "output": "merged", "kind": "candidate"})
        } else {
            json!({"node": "selector", "output": "result", "kind": "selection"})
        };
        if let Some(next) = next {
            node["routes"]["complete"] = json!([{"to": next}]);
        }
        if !landing {
            node["routes"]["failed"] = json!([{"to": "implementer", "repeat": 3}]);
        }
        nodes[name] = node;
    }
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1",
        "graph": {"inputs": {"merged": "candidate"}, "nodes": nodes}}),
    );
    assert!(started.get("error").is_none(), "{started}");
    let answer = |item: &Value| {
        let payload = &item["data"]["payload"];
        let evidence = &payload["evidence"];
        let mut params = json!({"project": project.path, "ticket": "Y-1", "attention": item["attention"],
            "check": evidence["check"]["check"], "plan_revision": payload["plan_revision"],
            "target": "main", "head": payload["head"], "proof": payload["proof"], "text": "Decide this exact failed result"});
        if let Some(landing) = evidence.get("landing") {
            for (key, value) in [
                ("landing", &landing["execution"]),
                ("child", &evidence["check"]["execution"]),
                ("node", &evidence["node"]),
                ("base", &landing["base"]),
                ("revision", &landing["revision"]),
                ("old", &landing["prepared"]["old"]),
                ("merged", &landing["prepared"]["merged"]),
            ] {
                params[key] = value.clone();
            }
        }
        params
    };
    let mut old = Vec::new();
    let mut old_checks = Vec::new();
    let mut repair_source = None;
    let mut preceding_gate = Value::Null;
    let mut accepted_review = Value::Null;
    for (reason, accept, version) in [
        ("failed", false, 0),
        ("failed", true, 1),
        ("blocked", false, 1),
        ("blocked", true, 2),
    ] {
        let item = watch.attention();
        assert_eq!(item["data"]["kind"], "candidate");
        assert_eq!(item["data"]["reason"], reason);
        let payload = &item["data"]["payload"];
        assert!(payload["choice"].is_i64());
        let attempt = item["attempt"].as_i64().unwrap();
        let clone = project.path.join(format!(
            ".yard/local/attempts/{attempt}/writers/7772697465722d62/clone"
        ));
        assert_eq!(
            git(&clone, &["show", "HEAD:version.txt"]).trim(),
            version.to_string()
        );
        assert_eq!(payload["head"], git(&clone, &["rev-parse", "HEAD"]).trim());
        let evidence = &payload["evidence"];
        assert_eq!(evidence["check"]["verdict"], "fail");
        let choice = rows(
            &project,
            &format!("SELECT * FROM execution WHERE id = {}", payload["choice"]),
        );
        let chosen = &choice[0]["inputs"]["b"];
        assert_eq!(chosen["source"]["node"], "writer-b");
        assert_eq!(chosen["value"]["head"], payload["head"]);
        assert_eq!(choice[0]["inputs"]["ra"]["value"]["verdict"], "pass");
        if reason == "blocked" {
            assert_eq!(evidence["inputs"]["candidate"], *chosen);
            let mut foreign = answer(&item);
            foreign["check"] = preceding_gate.clone();
            let before = project.rows("SELECT * FROM audit ORDER BY seq");
            let refused = operator.call("yard_attempt_accept", foreign);
            assert_eq!(refused["error"]["code"], "stale");
            assert_eq!(refused["error"]["data"]["identity"], "selected_result");
            assert_eq!(refused["error"]["data"]["expected"], preceding_gate);
            assert_eq!(
                refused["error"]["data"]["current"],
                evidence["check"]["check"]
            );
            assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
        } else {
            preceding_gate = evidence["check"]["check"].clone();
        }
        if let Some(source) = repair_source.take() {
            let producer = rows(
                &project,
                &format!(
                    "SELECT * FROM execution WHERE id = {}",
                    chosen["source"]["execution"]
                ),
            );
            assert_eq!(producer[0]["workspace"], "writer-b");
            assert_eq!(producer[0]["reason"], "repair");
            assert_eq!(
                producer[0]["inputs"]["feedback"]["source"]["execution"],
                source
            );
        }
        let snapshot = rows(
            &project,
            &format!(
                "SELECT * FROM execution WHERE id = {}",
                evidence["check"]["execution"]
            ),
        );
        old.extend(snapshot);
        old_checks.extend(project.rows(&format!(
            "SELECT * FROM \"check\" WHERE id = {}",
            evidence["check"]["check"]
        )));
        if reason == "blocked" && accept {
            accepted_review = evidence.clone();
        }
        let result = operator.call(
            if accept {
                "yard_attempt_accept"
            } else {
                "yard_attempt_repair"
            },
            answer(&item),
        );
        assert_eq!(result["result"]["accepted"], accept, "{result}");
        if !accept {
            repair_source = Some(evidence["check"]["execution"].clone());
        }
        old.extend(choice);
    }
    let first = approval(&mut watch);
    assert_eq!(first["accepted"], json!([accepted_review]));
    assert_eq!(first["checks"].as_array().unwrap().len(), 2);
    assert!(
        first["checks"]
            .as_array()
            .unwrap()
            .contains(&accepted_review["check"]["check"])
    );
    let approved = operator.call("yard_attempt_approve", json!({"project": project.path, "ticket": "Y-1",
        "target": "main", "head": first["head"], "proof": first["proof"], "choice": first["choice"], "plan_revision": first["plan_revision"]}));
    assert!(approved.get("error").is_none(), "{approved}");
    let failed = watch.attention();
    assert_eq!(failed["data"]["kind"], "candidate");
    assert_eq!(failed["data"]["reason"], "landing");
    let evidence = &failed["data"]["payload"]["evidence"];
    assert_eq!(evidence["result"]["outcome"], "failed");
    assert_eq!(
        evidence["check"]["head"],
        git(
            &project.canonical(),
            &[
                "rev-parse",
                &format!("refs/yard/landings/{}", evidence["landing"]["execution"])
            ]
        )
        .trim()
    );
    old.extend(rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE id = {}",
            evidence["check"]["execution"]
        ),
    ));
    old_checks.extend(project.rows(&format!(
        "SELECT * FROM \"check\" WHERE id = {}",
        evidence["check"]["check"]
    )));
    watch.find("selected landing repair parent released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == evidence["landing"]["execution"]
            && event["seq"].as_i64().unwrap() > failed["seq"].as_i64().unwrap()
    });
    let repaired = operator.call("yard_attempt_repair", answer(&failed));
    assert_eq!(repaired["result"]["accepted"], false, "{repaired}");
    let fresh = approval(&mut watch);
    assert_ne!(fresh["head"], first["head"]);
    assert_ne!(fresh["choice"], first["choice"]);
    assert_eq!(fresh["accepted"], json!([]));
    assert_eq!(fresh["checks"].as_array().unwrap().len(), 2);
    for check in fresh["checks"].as_array().unwrap() {
        assert_eq!(
            project.rows(&format!(
                "SELECT e.head, c.verdict FROM \"check\" c JOIN execution e ON e.id = c.execution WHERE c.id = {check}"
            )),
            vec![json!({"head": fresh["head"], "verdict": "pass"})]
        );
    }
    let repairs = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'implementation' AND reason = 'repair' ORDER BY id",
    );
    assert_eq!(repairs.len(), 3);
    let producer = repairs.last().unwrap();
    assert_eq!(producer["workspace"], "writer-b");
    assert_eq!(
        producer["inputs"]["feedback"]["value"]["evidence"],
        *evidence
    );
    assert_eq!(producer["head"], first["head"]);
    let approved = operator.call("yard_attempt_approve", json!({"project": project.path, "ticket": "Y-1",
        "target": "main", "head": fresh["head"], "proof": fresh["proof"], "choice": fresh["choice"], "plan_revision": fresh["plan_revision"]}));
    assert!(approved.get("error").is_none(), "{approved}");
    let failed = watch.attention();
    assert_eq!(failed["data"]["kind"], "candidate");
    assert_eq!(failed["data"]["reason"], "landing");
    assert_eq!(failed["data"]["payload"]["choice"], fresh["choice"]);
    old.extend(rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE id = {}",
            failed["data"]["payload"]["evidence"]["check"]["execution"]
        ),
    ));
    old_checks.extend(project.rows(&format!(
        "SELECT * FROM \"check\" WHERE id = {}",
        failed["data"]["payload"]["evidence"]["check"]["check"]
    )));
    watch.find("selected landing acceptance parent released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == failed["data"]["payload"]["evidence"]["landing"]["execution"]
            && event["seq"].as_i64().unwrap() > failed["seq"].as_i64().unwrap()
    });
    let accepted = operator.call("yard_attempt_accept", answer(&failed));
    assert_eq!(accepted["result"]["accepted"], true, "{accepted}");
    watch.event("landing.recorded", &[]);
    assert_eq!(
        git(&project.canonical(), &["show", "main:source.txt"]).trim(),
        "B"
    );
    assert_eq!(
        git(&project.canonical(), &["show", "main:version.txt"]).trim(),
        "3"
    );
    for original in old {
        assert_eq!(
            rows(
                &project,
                &format!("SELECT * FROM execution WHERE id = {}", original["id"])
            ),
            vec![original]
        );
    }
    for original in old_checks {
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM \"check\" WHERE id = {}",
                original["id"]
            )),
            vec![original]
        );
    }
}

/// Five different real Pi producers race on the same file. B publishes before
/// the last writer, and is replaced after two choices and a passing check.
/// Git in the fixture clones supplies heads; fixture bytes supply proof/content.
/// Existing writer isolation tests stop before selection and cannot catch this.
/// Sabotage: use the newest attempt head, grant picker review/write authority,
/// copy a losing review, reuse a choice after replacement, overwrite old evidence,
/// or approve the same head without binding the exact choice.
/// A published picker then receives a nonretryable host-model error: only an
/// explicit start may retry it. Sabotage: let publication mask picker failure.
/// At the final nondefault-target CAS, real Git holds the open intent while a
/// replacement is refused. Sabotage: omit the intent fence in start_writer.
/// Gate- and review-shaped candidate names exercise selected admission and
/// current consumers. Sabotage: filter roots or reviewers by lexical name.
/// B's replacement fails the native candidate gate. Only its producer receives
/// the fixture failure marker and repairs it. Sabotage: drop root feedback or
/// clear choice before recording the repair execution's bound failure inputs.
/// Selected landing omissions require exact operator acceptance before CAS.
/// Sabotage: omit choice from landing attention, making valid acceptance stale.
/// G10: an external target conflicts with B after candidate-gate repair. Its
/// actual producer fetches the target bundle through its own input mount,
/// resolves the conflict and earns fresh review, choice, checks and approval.
/// Sabotage: stage target.bundle in the shared attempt input instead of B's.
/// B's candidate-gate repair receives a blocking review at the configured
/// review limit. Explicit exact approval keeps its own failed review while
/// a losing candidate root is named review-0. Sabotage: classify that writer
/// as a reviewer in limit_checks and index its absent candidate input.
/// A losing writer named landing-gate-other still omits that configured gate.
/// Sabotage: derive skipped verification by name alone, hiding the omission
/// and refusing the exact observe/accept control despite its supported graph.
/// After B merges an operator .yard change, another retained producer refuses
/// its own .yard edit, then publishes cleanly at its independent original base.
/// Sabotage: bind that producer or its publication to the selected attempt base.
/// B continues while a real merge-tree is held before intent. Its obsolete
/// conflict must withdraw without staging a bundle or overwriting continuation.
/// Sabotage: resolve the return producer from a cleared current choice or check
/// only ticket revision. A fresh conflict below must still repair B normally.
/// With B's repair pending, a real scheduler Git read holds lane reacquisition.
/// Choosing A refuses without a write; choosing the same exact B still repairs B.
/// Sabotage: activate A while retaining B's pending target-fetch instructions.
/// Explicit starts of A or B must leave automatic repair alone.
/// Sabotage: let an operator writer start consume or relabel pending repair.
/// A second held scheduler lets a body edit supersede pending B repair.
/// Sabotage: retain old next across the edit, or dispatch it after the lane wait.
/// Fresh revision-two candidates, own reviews and a new conflict still repair B.
#[test]
fn only_the_declared_choice_is_checked_and_landed() {
    let holds: Vec<_> = (0..5).map(|_| Latch::new()).collect();
    let writer_holds = holds.clone();
    let replacement = Latch::new();
    let replacement_hold = replacement.clone();
    let repair = Latch::new();
    let repair_hold = repair.clone();
    let landing_repair = Latch::new();
    let landing_repair_hold = landing_repair.clone();
    let edited_writer = Latch::new();
    let edited_writer_hold = edited_writer.clone();
    let fresh_repair = Latch::new();
    let fresh_repair_hold = fresh_repair.clone();
    let superseding = Latch::new();
    let superseding_hold = superseding.clone();
    let dirty_writer = Latch::new();
    let dirty_writer_hold = dirty_writer.clone();
    let clean_writer = Latch::new();
    let clean_writer_hold = clean_writer.clone();
    let turns: Vec<_> = (0..5).map(|_| AtomicUsize::new(0)).collect();
    let picker_finishes = AtomicUsize::new(0);
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
            if picker_finishes.fetch_add(1, Ordering::SeqCst) == 0 {
                return Reply::ResponsesError;
            }
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
                let priority = if request
                    .tool_results()
                    .last()
                    .unwrap()
                    .1
                    .contains("exact-review-writer-1-2")
                {
                    "P1"
                } else {
                    "P3"
                };
                return Reply::Tools(vec![publish(
                    json!([{ "priority": priority, "body": "Fixture review evidence." }]),
                )]);
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
        if request.opens() {
            let version = turns[n].fetch_add(1, Ordering::SeqCst);
            let fix = if n == 1 && version == 6 {
                assert!(
                    request
                        .last_user()
                        .contains("git fetch /yard/input/target.bundle refs/heads/feature/a")
                );
                "git fetch /yard/input/target.bundle refs/heads/feature/a && test \"$(git show FETCH_HEAD:same.txt)\" = operator-edited-target && (git merge --no-commit FETCH_HEAD || test -f .git/MERGE_HEAD) && "
            } else if n == 1 && version == 5 {
                assert!(
                    request
                        .last_user()
                        .contains("Edited pending repair instructions.")
                );
                ""
            } else if n == 1 && version == 4 {
                assert!(
                    request
                        .last_user()
                        .contains("git fetch /yard/input/target.bundle refs/heads/feature/a")
                );
                assert!(request.last_user().contains("same.txt"));
                "git fetch /yard/input/target.bundle refs/heads/feature/a && test \"$(git show FETCH_HEAD:same.txt)\" = operator-target-marker && (git merge --no-commit FETCH_HEAD || test -f .git/MERGE_HEAD) && "
            } else if n == 1 && version == 2 {
                assert!(
                    request
                        .last_user()
                        .contains("selected-B-needs-repair-marker")
                );
                "printf 'selected-B-repaired\\n' > repair.txt && git add repair.txt && "
            } else if n == 2 && version == 1 {
                "printf 'worker-owned\\n' > .yard/worker.txt && git add .yard/worker.txt && "
            } else if n == 2 && version == 2 {
                "git rm .yard/worker.txt && "
            } else {
                ""
            };
            return Reply::Tools(vec![bash(&format!(
                "{fix}printf 'writer-{n}-{version}\\n' > same.txt && printf 'writer-{n}-{version}\\n' > /yard/proof/evidence.txt && git add same.txt && git commit -qm Independent && echo producer-ready"
            ))]);
        }
        if request
            .tool_results()
            .last()
            .is_some_and(|(_, text)| text.contains("producer-ready"))
        {
            let hold = if turns[n].load(Ordering::SeqCst) == 1 {
                writer_holds[n].clone()
            } else if n == 1 && turns[n].load(Ordering::SeqCst) == 7 {
                fresh_repair_hold.clone()
            } else if n == 1 && turns[n].load(Ordering::SeqCst) == 6 {
                edited_writer_hold.clone()
            } else if n == 1 && turns[n].load(Ordering::SeqCst) == 5 {
                landing_repair_hold.clone()
            } else if n == 1 && turns[n].load(Ordering::SeqCst) == 4 {
                superseding_hold.clone()
            } else if n == 1 && turns[n].load(Ordering::SeqCst) == 3 {
                repair_hold.clone()
            } else if n == 2 && turns[n].load(Ordering::SeqCst) == 2 {
                dirty_writer_hold.clone()
            } else if n == 2 && turns[n].load(Ordering::SeqCst) == 3 {
                clean_writer_hold.clone()
            } else {
                replacement_hold.clone()
            };
            return Reply::Hold(hold, Box::new(Reply::Text("done".into())));
        }
        Reply::Text("done".into())
    });
    let armed = machine.root.join("landing-armed");
    let said = machine.root.join("landing-said");
    let release = machine.root.join("landing-release");
    let merge_armed = machine.root.join("merge-armed");
    let merge_said = machine.root.join("merge-said");
    let merge_release = machine.root.join("merge-release");
    let pending_armed = machine.root.join("pending-armed");
    let pending_ready = machine.root.join("pending-ready");
    let pending_said = machine.root.join("pending-said");
    let pending_release = machine.root.join("pending-release");
    for fifo in [
        &said,
        &release,
        &merge_said,
        &merge_release,
        &pending_said,
        &pending_release,
    ] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    machine.wrapper("git", &format!(
        "if [ -e '{}' ]; then case \" $* \" in *' update-ref --no-deref refs/heads/feature/a '*)\n\
         echo held > '{}'; read line < '{}';; esac; fi\n\
         if [ -e '{}' ]; then case \" $* \" in *' merge-tree --write-tree '*)\n\
         echo held > '{}'; read line < '{}';; esac; fi\n\
         if [ -e '{}' ]; then case \" $* \" in *' bundle create --quiet '*'/target.bundle '*)\n\
         \"$REAL\" \"$@\" || exit $?; touch '{}'; exit 0;; esac; fi\n\
         if [ -e '{}' ]; then case \" $* \" in *' symbolic-ref --quiet --short HEAD '*)\n\
         echo held > '{}'; read line < '{}';; esac; fi",
        armed.display(), said.display(), release.display(),
        merge_armed.display(), merge_said.display(), merge_release.display(),
        pending_armed.display(), pending_ready.display(), pending_ready.display(),
        pending_said.display(), pending_release.display()
    ));
    machine.start();
    let mut extra = String::from(
        "\n[profiles.picker]\nrunner = {kind = 'agent', agent = 'worker', instructions = 'Compare frozen inputs.'}\naccess = 'read-only'\noutputs = {result = 'selection'}\noutcomes = ['complete']\n\
         [profiles.reader]\nrunner = {kind = 'agent', agent = 'worker', instructions = 'Same frozen reviewer.'}\naccess = 'read-only'\noutputs = {result = 'verdict'}\noutcomes = ['complete', 'blocked']\n\
         [gates.check]\ncommand = 'case \"$(cat same.txt)\" in writer-1-*) test \"$(cat same.txt)\" = \"$(cat /yard/proof/evidence.txt)\" || exit 91;; *) exit 91;; esac; if test \"$(cat same.txt)\" != writer-1-0 && ! grep -qx selected-B-repaired repair.txt 2>/dev/null; then echo selected-B-needs-repair-marker; exit 94; fi'\nstage = 'candidate'\n",
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
    let independent_base = target.clone();
    let mut operator = machine.operator();
    operator.initialize();
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
    for n in [1, 0, 2, 3, 4] {
        holds[n].release();
        let id = &writers
            .iter()
            .find(|row| row["model"] == format!("producer-{n}"))
            .unwrap()["id"];
        watch.find("writer released", |event| {
            event["event"] == "execution.released" && &event["execution"] == id
        });
    }
    let failure = watch.find("published picker failed", |event| {
        event["event"] == "attention.raised"
            && event["data"]["kind"] == "red"
            && event["data"]["reason"] == "review"
    });
    assert_eq!(failure["data"]["payload"]["seat"], "selector");
    watch.find("failed picker released", |event| {
        event["event"] == "execution.released" && event["execution"] == failure["execution"]
    });
    let failed = rows(&project, "SELECT * FROM execution WHERE node = 'selector'");
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["id"], failure["execution"]);
    assert_eq!(failed[0]["outcome"], "error");
    assert_eq!(failed[0]["result"]["infrastructure"]["outcome"], "failed");
    assert!(failed[0]["result"]["outputs"]["result"].is_null());
    assert_eq!(failed[0]["report"]["candidate"], failed[0]["inputs"]["c1"]);
    assert!(project.rows("SELECT * FROM approval").is_empty());
    assert!(
        project
            .rows("SELECT * FROM execution WHERE kind = 'gate'")
            .is_empty()
    );
    let retry = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "attention": failure["attention"]
        }),
    );
    assert!(retry.get("error").is_none(), "{retry}");
    let first = approval(&mut watch);
    assert_eq!(first["head"], heads[1]);
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
    assert_eq!(raw_reviews.len(), 5);
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
    repair.wait_held();
    let repairing = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'implementation' AND reason = 'repair' ORDER BY id",
    );
    assert_eq!(repairing.len(), 1);
    let repairing = &repairing[0];
    assert_eq!(repairing["workspace"], "writer-1");
    assert_eq!(repairing["settings"]["model"], "producer-1");
    assert_eq!(repairing["head"], replacement_head);
    let failed_gates = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'gate' AND outcome = 'fail' ORDER BY id",
    );
    assert_eq!(failed_gates.len(), 1);
    let failed_gate = &failed_gates[0];
    assert_eq!(failed_gate["head"], replacement_head);
    assert_eq!(failed_gate["exit_code"], 94);
    let feedback = &repairing["inputs"]["feedback"];
    assert_eq!(feedback["source"]["execution"], failed_gate["id"]);
    assert_eq!(feedback["source"]["node"], "candidate-gate-check");
    assert_eq!(feedback["source"]["output"], "result");
    assert_eq!(feedback["value"]["verdict"], "fail");
    assert_eq!(feedback["value"]["gate"], "check");
    assert!(
        feedback["value"]["detail"]
            .as_str()
            .unwrap()
            .contains("selected-B-needs-repair-marker")
    );
    assert_eq!(
        project.rows("SELECT choice FROM attempt WHERE state = 'live'")[0]["choice"],
        Value::Null
    );
    let repaired_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_ne!(repaired_head, replacement_head);
    assert_eq!(
        git(&clone, &["show", "HEAD:repair.txt"]).trim(),
        "selected-B-repaired"
    );
    repair.release();
    let limited = watch.find("selected blocking review limit", |event| {
        event["event"] == "attention.raised"
            && event["data"]["kind"] == "stopped"
            && event["data"]["reason"] == "limit"
    });
    let third = limited["data"]["payload"].clone();
    assert_eq!(third["rounds"], 1);
    assert_eq!(third["findings"][0]["seat"], "r1");
    assert_eq!(third["findings"][0]["finding"]["priority"], "P1");
    assert_eq!(third["head"], repaired_head);
    assert_ne!(third["proof"], first["proof"]);
    let now = rows(
        &project,
        "SELECT * FROM execution WHERE status = 'ended' ORDER BY id",
    );
    for old in &retained {
        assert_eq!(now.iter().find(|row| row["id"] == old["id"]), Some(old));
    }
    assert_eq!(
        now.iter().find(|row| row["id"] == failed_gate["id"]),
        Some(failed_gate)
    );
    assert_eq!(
        now.iter().find(|row| row["id"] == failed[0]["id"]),
        Some(&failed[0])
    );
    let checks_now = project.rows("SELECT * FROM \"check\" ORDER BY id");
    for old in &raw_checks {
        assert_eq!(
            checks_now.iter().find(|row| row["id"] == old["id"]),
            Some(old)
        );
    }
    let findings_now = project.rows("SELECT * FROM finding ORDER BY id");
    for old in &raw_reviews {
        assert_eq!(
            findings_now.iter().find(|row| row["id"] == old["id"]),
            Some(old)
        );
    }
    assert_eq!(project.canonical_head(), target);
    let before_conflict = rows(
        &project,
        "SELECT * FROM execution WHERE status = 'ended' ORDER BY id",
    );
    project.write("same.txt", "operator-target-marker\n");
    project.write(
        ".yard/config.toml",
        &format!("{configuration}\n# operator-target-marker\n"),
    );
    project.git(&["add", "same.txt", ".yard/config.toml"]);
    project.git(&["commit", "-qm", "Advance the actual target against B"]);
    project.json(&["sync", "feature/a"]);
    let target = git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"])
        .trim()
        .to_owned();
    std::fs::write(&merge_armed, "").unwrap();
    let approved = operator.call(
        "yard_attempt_approve",
        json!({
            "project": project.path, "ticket": "Y-1", "target": "feature/a",
            "head": repaired_head, "proof": third["proof"], "choice": third["choice"]
        }),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    assert_eq!(watch.said(&merge_said), "held\n");
    let limited_picker = before_conflict
        .iter()
        .find(|row| row["id"] == third["choice"])
        .unwrap();
    let blocked_review = before_conflict
        .iter()
        .find(|row| row["id"] == limited_picker["inputs"]["r1"]["source"]["execution"])
        .unwrap();
    assert_eq!(
        blocked_review["inputs"]["candidate"],
        limited_picker["inputs"]["c1"]
    );
    let overridden = project.rows("SELECT * FROM approval ORDER BY id");
    assert_eq!(overridden.len(), 1);
    assert_eq!(overridden[0]["overrode"], 1);
    assert_eq!(overridden[0]["choice"], third["choice"]);
    let selected_checks: Value =
        serde_json::from_str(overridden[0]["checks"].as_str().unwrap()).unwrap();
    assert_eq!(selected_checks.as_array().unwrap().len(), 2);
    assert!(
        selected_checks
            .as_array()
            .unwrap()
            .contains(&limited_picker["inputs"]["r1"]["value"]["publication"])
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT verdict FROM \"check\" WHERE id = {}",
            limited_picker["inputs"]["r1"]["value"]["publication"]
        )),
        vec![json!({"verdict": "fail"})]
    );
    let obsolete = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE kind = 'landing' AND approval = {}",
            overridden[0]["id"]
        ),
    );
    assert_eq!(obsolete.len(), 1);
    let obsolete = &obsolete[0];
    assert_eq!(obsolete["status"], "running");
    assert!(obsolete["intent_state"].is_null());
    assert_eq!(obsolete["head"], repaired_head);
    let continued = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "writer-1"}),
    );
    assert!(continued.get("error").is_none(), "{continued}");
    let continuation = rows(
        &project,
        &format!(
            "SELECT id, workspace, settings, head, inputs, reason, plan_revision, ticket_revision FROM execution WHERE id = {}",
            continued["result"]["execution"]
        ),
    );
    assert_eq!(continuation[0]["workspace"], "writer-1");
    assert_eq!(continuation[0]["settings"]["model"], "producer-1");
    assert_eq!(continuation[0]["head"], repaired_head);
    let pending = project.rows(&format!(
        "SELECT next, target, landing_reds, choice FROM attempt WHERE id = {attempt}"
    ));
    assert!(pending[0]["choice"].is_null());
    assert!(pending[0]["next"].is_null());
    std::fs::remove_file(&merge_armed).unwrap();
    std::fs::write(&merge_release, "go\n").unwrap();
    let withdrawn = watch.find("obsolete selected landing withdrawn", |event| {
        event["event"] == "execution.ended" && event["execution"] == obsolete["id"]
    });
    assert_eq!(withdrawn["data"]["outcome"], "withdrawn");
    watch.find("obsolete selected landing released", |event| {
        event["event"] == "execution.released" && event["execution"] == obsolete["id"]
    });
    // Producer launch takes the same target lock as merge preparation. Its
    // supported continuation is already recorded before that lock releases.
    superseding.wait_held();
    assert_eq!(
        project.rows(&format!(
            "SELECT next, target, landing_reds, choice FROM attempt WHERE id = {attempt}"
        )),
        pending
    );
    assert_eq!(
        rows(
            &project,
            &format!(
                "SELECT id, workspace, settings, head, inputs, reason, plan_revision, ticket_revision FROM execution WHERE id = {}",
                continued["result"]["execution"]
            )
        ),
        continuation
    );
    for node in ["implementer", "writer-1"] {
        let hex = node.bytes().map(|b| format!("{b:02x}")).collect::<String>();
        assert!(
            !project
                .path
                .join(format!(
                    ".yard/local/attempts/{attempt}/writers/{hex}/input/target.bundle"
                ))
                .exists()
        );
    }
    assert!(
        !project
            .path
            .join(format!(
                ".yard/local/landings/{}/target.bundle",
                obsolete["id"]
            ))
            .exists()
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT state FROM approval WHERE id = {}",
            overridden[0]["id"]
        )),
        vec![json!({"state": "withdrawn"})]
    );
    let superseding_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_ne!(superseding_head, repaired_head);
    superseding.release();
    let fresh = approval(&mut watch);
    assert_eq!(fresh["head"], superseding_head);
    assert_ne!(fresh["choice"], third["choice"]);
    assert_eq!(fresh["skipped"], first["skipped"]);
    std::fs::write(&pending_armed, "").unwrap();
    let approved = operator.call(
        "yard_attempt_approve",
        json!({"project": project.path, "ticket": "Y-1", "target": "feature/a",
            "head": superseding_head, "proof": fresh["proof"], "choice": fresh["choice"]}),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    assert_eq!(watch.said(&pending_said), "held\n");
    let conflict_end = watch.find("selected conflict ended before pending repair", |event| {
        event["event"] == "execution.ended" && event["data"]["outcome"] == "conflict"
    });
    watch.find(
        "selected conflict released before pending repair",
        |event| {
            event["event"] == "execution.released"
                && event["execution"] == conflict_end["execution"]
                && event["seq"].as_i64().unwrap() > conflict_end["seq"].as_i64().unwrap()
        },
    );
    let pending = project.rows(&format!("SELECT * FROM attempt WHERE id = {attempt}"));
    assert_eq!(pending[0]["lane"], 0);
    assert_eq!(pending[0]["choice"], fresh["choice"]);
    let next: Value = serde_json::from_str(pending[0]["next"].as_str().unwrap()).unwrap();
    assert_eq!(next["reason"], "repair");
    assert_eq!(next["target"], target);
    assert_eq!(next["branch"], "feature/a");
    let bundle = project.path.join(format!(
        ".yard/local/attempts/{attempt}/writers/7772697465722d31/input/target.bundle"
    ));
    let bundle_bytes = std::fs::read(&bundle).unwrap();
    let choice = rows(
        &project,
        &format!("SELECT * FROM execution WHERE id = {}", fresh["choice"]),
    );
    let mut direct = json!({"project": project.path, "ticket": "Y-1", "attempt": attempt,
        "revision": 1, "plan_revision": 1, "target": "feature/a", "choice": fresh["choice"],
        "inputs": choice[0]["inputs"], "candidate": "c0"});
    let before = project.rows("SELECT * FROM audit ORDER BY seq");
    let refused = operator.call("yard_attempt_select", direct.clone());
    assert_eq!(refused["error"]["code"], "refused", "{refused}");
    assert_eq!(refused["error"]["data"]["reason"], "pending_continuation");
    assert_eq!(refused["error"]["data"]["identity"], "pending_continuation");
    assert_eq!(
        refused["error"]["data"]["expected"],
        choice[0]["inputs"]["c1"]
    );
    assert_eq!(
        refused["error"]["data"]["current"],
        choice[0]["inputs"]["c0"]
    );
    assert_eq!(project.rows("SELECT * FROM audit ORDER BY seq"), before);
    assert_eq!(
        project.rows(&format!("SELECT * FROM attempt WHERE id = {attempt}")),
        pending
    );
    assert_eq!(std::fs::read(&bundle).unwrap(), bundle_bytes);
    direct["candidate"] = json!("c1");
    let selected = operator.call("yard_attempt_select", direct);
    assert!(selected.get("error").is_none(), "{selected}");
    let current = project.rows(&format!(
        "SELECT choice, next FROM attempt WHERE id = {attempt}"
    ));
    assert_eq!(current[0]["choice"], selected["result"]["choice"]);
    assert_ne!(current[0]["choice"], fresh["choice"]);
    assert_eq!(current[0]["next"], pending[0]["next"]);
    assert_eq!(std::fs::read(&bundle).unwrap(), bundle_bytes);
    let reselected = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE id = {}",
            selected["result"]["choice"]
        ),
    );
    assert_eq!(
        reselected[0]["result"]["outputs"]["result"]["value"]["candidate"],
        choice[0]["inputs"]["c1"]
    );
    let before_attempts = project.rows("SELECT * FROM attempt ORDER BY id");
    let before_executions = project.rows("SELECT * FROM execution ORDER BY id");
    let before_approvals = project.rows("SELECT * FROM approval ORDER BY id");
    let before_attention = project.rows("SELECT * FROM attention ORDER BY id");
    let before_audit = project.rows("SELECT * FROM audit ORDER BY seq");
    for workspace in ["implementer", "writer-1"] {
        let refused = operator.call(
            "yard_attempt_start",
            json!({
                "project": project.path, "ticket": "Y-1", "workspace": workspace
            }),
        );
        assert_eq!(refused["error"]["code"], "refused", "{refused}");
        assert_eq!(refused["error"]["data"]["reason"], "pending_continuation");
        assert_eq!(refused["error"]["data"]["identity"], "pending_continuation");
        assert_eq!(refused["error"]["data"]["attempt"], attempt);
        assert_eq!(
            refused["error"]["data"]["choice"],
            selected["result"]["choice"]
        );
        assert_eq!(refused["error"]["data"]["current"], next);
        assert_eq!(
            project.rows("SELECT * FROM attempt ORDER BY id"),
            before_attempts
        );
        assert_eq!(
            project.rows("SELECT * FROM execution ORDER BY id"),
            before_executions
        );
        assert_eq!(
            project.rows("SELECT * FROM approval ORDER BY id"),
            before_approvals
        );
        assert_eq!(
            project.rows("SELECT * FROM attention ORDER BY id"),
            before_attention
        );
        assert_eq!(
            project.rows("SELECT * FROM audit ORDER BY seq"),
            before_audit
        );
        assert_eq!(std::fs::read(&bundle).unwrap(), bundle_bytes);
    }
    std::fs::remove_file(&pending_armed).unwrap();
    std::fs::remove_file(&pending_ready).unwrap();
    std::fs::write(&pending_release, "go\n").unwrap();
    landing_repair.wait_held();
    let repairs = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'implementation' AND reason = 'repair' ORDER BY id",
    );
    assert_eq!(repairs.len(), 2);
    let returned = &repairs[1];
    assert_eq!(returned["workspace"], "writer-1");
    assert_eq!(returned["settings"]["model"], "producer-1");
    assert_eq!(returned["head"], superseding_head);
    assert_eq!(returned["inputs"]["feedback"]["value"]["target"], target);
    assert_eq!(
        returned["inputs"]["feedback"]["value"]["branch"],
        "feature/a"
    );
    let conflict = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'landing' AND outcome = 'conflict'",
    );
    assert_eq!(conflict.len(), 1);
    assert_eq!(conflict[0]["head"], superseding_head);
    assert_eq!(git(&clone, &["rev-parse", "FETCH_HEAD"]).trim(), target);
    let landed_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_ne!(landed_head, superseding_head);
    assert_eq!(git(&clone, &["rev-parse", "HEAD^2"]).trim(), target);
    assert_eq!(git(&clone, &["show", "HEAD:same.txt"]).trim(), "writer-1-4");
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    landing_repair.release();
    let fourth = approval(&mut watch);
    assert_eq!(fourth["head"], landed_head);
    assert_eq!(fourth["skipped"], first["skipped"]);
    assert_ne!(fourth["choice"], third["choice"]);
    let after_conflict = rows(
        &project,
        "SELECT * FROM execution WHERE status = 'ended' ORDER BY id",
    );
    for old in before_conflict
        .iter()
        .chain(&conflict)
        .chain(&choice)
        .chain(&reselected)
    {
        assert_eq!(
            after_conflict.iter().find(|row| row["id"] == old["id"]),
            Some(old)
        );
    }
    let picker = after_conflict
        .iter()
        .find(|row| row["id"] == fourth["choice"])
        .unwrap();
    assert_eq!(
        picker["inputs"]["c1"]["source"]["execution"],
        returned["id"]
    );
    let review = after_conflict
        .iter()
        .find(|row| row["id"] == picker["inputs"]["r1"]["source"]["execution"])
        .unwrap();
    assert_eq!(review["inputs"]["candidate"], picker["inputs"]["c1"]);
    assert!(
        fourth["checks"]
            .as_array()
            .unwrap()
            .contains(&picker["inputs"]["r1"]["value"]["publication"])
    );
    // B's selected base advanced; this retained producer has never fetched it.
    // Observe its ancestry without resolving the absent target object locally.
    let hex = "build-gate-model"
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let independent_clone = project.path.join(format!(
        ".yard/local/attempts/{attempt}/writers/{hex}/clone"
    ));
    let retry = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workspace": "build-gate-model"
        }),
    );
    assert!(retry.get("error").is_none(), "{retry}");
    dirty_writer.wait_held();
    let continued = rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'build-gate-model' AND kind = 'implementation' ORDER BY id DESC LIMIT 1",
    );
    let dirty = &continued[0];
    assert_eq!(dirty["base"], independent_base);
    assert_eq!(dirty["head"], heads[2]);
    assert_eq!(dirty["settings"]["model"], "producer-2");
    let current = project.rows("SELECT base, choice FROM attempt WHERE state = 'live'");
    assert_eq!(current, vec![json!({"base": target, "choice": null})]);
    assert_eq!(
        git(
            &independent_clone,
            &["merge-base", "HEAD", &independent_base]
        )
        .trim(),
        independent_base
    );
    assert!(
        !git(&independent_clone, &["rev-list", "HEAD"])
            .lines()
            .any(|commit| commit == target)
    );
    assert_eq!(
        git(&independent_clone, &["show", "HEAD:.yard/worker.txt"]).trim(),
        "worker-owned"
    );
    dirty_writer.release();
    let refused = watch.find("independent .yard edit refused", |event| {
        event["event"] == "attention.raised"
            && event["execution"] == dirty["id"]
            && event["data"]["kind"] == "stopped"
            && event["data"]["reason"] == "failed"
    });
    assert_eq!(refused["attempt"], attempt);
    watch.find("independent refusal released", |event| {
        event["event"] == "execution.released" && event["execution"] == dirty["id"]
    });
    let refused_history = rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'build-gate-model' AND kind = 'implementation' ORDER BY id DESC LIMIT 1",
    );
    assert_eq!(refused_history[0]["outcome"], "refused");
    assert!(refused_history[0]["result"]["outputs"]["result"].is_null());
    let retry = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workspace": "build-gate-model"
        }),
    );
    assert!(retry.get("error").is_none(), "{retry}");
    clean_writer.wait_held();
    let continued = rows(
        &project,
        "SELECT * FROM execution WHERE workspace = 'build-gate-model' AND kind = 'implementation' ORDER BY id DESC LIMIT 1",
    );
    assert_eq!(continued[0]["base"], independent_base);
    assert_eq!(continued[0]["head"], heads[2]);
    let independent_head = git(&independent_clone, &["rev-parse", "HEAD"])
        .trim()
        .to_owned();
    assert_eq!(
        git(
            &independent_clone,
            &["merge-base", "HEAD", &independent_base]
        )
        .trim(),
        independent_base
    );
    assert!(
        !git(&independent_clone, &["rev-list", "HEAD"])
            .lines()
            .any(|commit| commit == target)
    );
    assert!(
        git(
            &independent_clone,
            &[
                "diff",
                "--name-only",
                &independent_base,
                "HEAD",
                "--",
                ".yard"
            ]
        )
        .trim()
        .is_empty()
    );
    clean_writer.release();
    let fifth = approval(&mut watch);
    assert_eq!(fifth["head"], landed_head);
    assert_eq!(fifth["skipped"], fourth["skipped"]);
    assert_ne!(fifth["choice"], fourth["choice"]);
    let after_continuation = rows(
        &project,
        "SELECT * FROM execution WHERE status = 'ended' ORDER BY id",
    );
    for old in after_conflict.iter().chain(&refused_history) {
        assert_eq!(
            after_continuation.iter().find(|row| row["id"] == old["id"]),
            Some(old)
        );
    }
    let picker = after_continuation
        .iter()
        .find(|row| row["id"] == fifth["choice"])
        .unwrap();
    let source = &picker["inputs"]["c2"];
    assert_eq!(source["source"]["execution"], continued[0]["id"]);
    assert_eq!(source["value"]["base"], independent_base);
    assert_eq!(source["value"]["head"], independent_head);
    assert_eq!(source["value"]["producer"]["execution"], continued[0]["id"]);
    assert_eq!(
        source["value"]["producer"]["settings"]["model"],
        "producer-2"
    );
    assert_eq!(
        picker["inputs"]["c1"]["source"]["execution"],
        returned["id"]
    );
    assert_eq!(picker["inputs"]["c1"]["value"]["base"], target);
    let review = after_continuation
        .iter()
        .find(|row| row["id"] == picker["inputs"]["r2"]["source"]["execution"])
        .unwrap();
    assert_eq!(review["inputs"]["candidate"], *source);
    assert_eq!(
        project.rows("SELECT base FROM attempt WHERE state = 'live'"),
        vec![json!({"base": target})]
    );
    // Hold the same real lane-reacquisition Git read again, now across an edit.
    project.write("same.txt", "operator-edited-target\n");
    project.git(&["add", "same.txt"]);
    project.git(&["commit", "-qm", "Advance target before pending repair edit"]);
    project.json(&["sync", "feature/a"]);
    let target = git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"])
        .trim()
        .to_owned();
    std::fs::write(&pending_armed, "").unwrap();
    let pending_since = project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap();
    let approved = operator.call(
        "yard_attempt_approve",
        json!({
            "project": project.path, "ticket": "Y-1", "target": "feature/a",
            "head": landed_head, "proof": fifth["proof"], "choice": fifth["choice"]
        }),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    assert_eq!(watch.said(&pending_said), "held\n");
    let conflict_end = watch.find("second pending conflict ended", |event| {
        event["event"] == "execution.ended"
            && event["data"]["outcome"] == "conflict"
            && event["seq"].as_i64().unwrap() > pending_since
    });
    watch.find("second pending conflict released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == conflict_end["execution"]
            && event["seq"].as_i64().unwrap() > conflict_end["seq"].as_i64().unwrap()
    });
    let pending = project.rows(&format!("SELECT * FROM attempt WHERE id = {attempt}"));
    assert_eq!(pending[0]["lane"], 0);
    assert_eq!(pending[0]["choice"], fifth["choice"]);
    let next: Value = serde_json::from_str(pending[0]["next"].as_str().unwrap()).unwrap();
    assert_eq!(next["reason"], "repair");
    assert_eq!(next["target"], target);
    let bundle_bytes = std::fs::read(&bundle).unwrap();
    let before_writers =
        project.rows("SELECT * FROM execution WHERE kind = 'implementation' ORDER BY id");
    let edited = operator.call(
        "yard_ticket_edit",
        json!({
            "project": project.path, "ticket": "Y-1", "revision": 1,
            "body": "Edited pending repair instructions."
        }),
    );
    assert!(edited.get("error").is_none(), "{edited}");
    let current = project.rows(&format!(
        "SELECT choice, next, lane, lane_since FROM attempt WHERE id = {attempt}"
    ));
    assert_eq!(
        current,
        vec![json!({"choice": null, "next": null, "lane": 0, "lane_since": null})]
    );
    assert_eq!(
        project.rows("SELECT * FROM execution WHERE kind = 'implementation' ORDER BY id"),
        before_writers
    );
    assert_eq!(std::fs::read(&bundle).unwrap(), bundle_bytes);
    std::fs::remove_file(&pending_armed).unwrap();
    std::fs::remove_file(&pending_ready).unwrap();
    std::fs::write(&pending_release, "go\n").unwrap();
    edited_writer.wait_held();
    let edited_source = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE attempt = {attempt} AND workspace = 'writer-1' AND kind = 'implementation' ORDER BY id DESC LIMIT 1"
        ),
    );
    assert_eq!(edited_source[0]["reason"], "edit");
    assert_eq!(edited_source[0]["ticket_revision"], 2);
    assert_eq!(
        edited_source[0]["body"],
        "Edited pending repair instructions."
    );
    assert!(edited_source[0]["inputs"]["feedback"].is_null());
    let edited_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_eq!(git(&clone, &["show", "HEAD:same.txt"]).trim(), "writer-1-5");
    edited_writer.release();
    let sixth = approval(&mut watch);
    assert_eq!(sixth["head"], edited_head);
    assert_eq!(sixth["skipped"], fifth["skipped"]);
    assert_ne!(sixth["choice"], fifth["choice"]);
    let edited_sources = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE attempt = {attempt} AND kind = 'implementation' AND ticket_revision = 2 ORDER BY id"
        ),
    );
    assert_eq!(edited_sources.len(), 5);
    assert!(edited_sources.iter().all(|row| row["reason"] == "edit"
        && row["body"] == "Edited pending repair instructions."
        && row["inputs"]["feedback"].is_null()));
    // The fresh revision still conflicts. Only its newly selected B repairs it.
    let approved = operator.call(
        "yard_attempt_approve",
        json!({
            "project": project.path, "ticket": "Y-1", "target": "feature/a",
            "head": edited_head, "proof": sixth["proof"], "choice": sixth["choice"]
        }),
    );
    assert!(approved.get("error").is_none(), "{approved}");
    fresh_repair.wait_held();
    let repaired_source = rows(
        &project,
        &format!(
            "SELECT * FROM execution WHERE attempt = {attempt} AND kind = 'implementation' ORDER BY id DESC LIMIT 1"
        ),
    );
    assert_eq!(repaired_source[0]["workspace"], "writer-1");
    assert_eq!(repaired_source[0]["reason"], "repair");
    assert_eq!(repaired_source[0]["ticket_revision"], 2);
    assert_eq!(repaired_source[0]["head"], edited_head);
    assert_eq!(
        repaired_source[0]["inputs"]["feedback"]["value"]["target"],
        target
    );
    assert_eq!(git(&clone, &["rev-parse", "FETCH_HEAD"]).trim(), target);
    assert_eq!(git(&clone, &["rev-parse", "HEAD^2"]).trim(), target);
    let fresh_landed_head = git(&clone, &["rev-parse", "HEAD"]).trim().to_owned();
    fresh_repair.release();
    let seventh = approval(&mut watch);
    assert_eq!(seventh["head"], fresh_landed_head);
    assert_eq!(seventh["skipped"], fifth["skipped"]);
    let history = rows(
        &project,
        "SELECT * FROM execution WHERE status = 'ended' ORDER BY id",
    );
    for old in &after_continuation {
        assert_eq!(history.iter().find(|row| row["id"] == old["id"]), Some(old));
    }
    let picker = history
        .iter()
        .find(|row| row["id"] == seventh["choice"])
        .unwrap();
    assert_eq!(
        picker["inputs"]["c1"]["source"]["execution"],
        repaired_source[0]["id"]
    );
    let review = history
        .iter()
        .find(|row| row["id"] == picker["inputs"]["r1"]["source"]["execution"])
        .unwrap();
    assert_eq!(review["inputs"]["candidate"], picker["inputs"]["c1"]);
    assert!(
        seventh["checks"]
            .as_array()
            .unwrap()
            .contains(&picker["inputs"]["r1"]["value"]["publication"])
    );
    // Reopening revalidates current suppliers without rewriting the immutable choice.
    machine.kill();
    machine.start();
    drop(watch);
    let since = project.rows("SELECT MAX(seq) AS seq FROM audit")[0]["seq"]
        .as_i64()
        .unwrap();
    let mut watch = project.watch(since);
    std::fs::write(&armed, "").unwrap();
    let approved = operator.call("yard_attempt_approve", json!({"project": project.path, "ticket": "Y-1",
        "target": "feature/a", "head": fresh_landed_head, "proof": seventh["proof"], "choice": seventh["choice"],
        "observe_landing_skipped": ["landing-gate-other"], "text": "Observe the omitted landing gate"}));
    assert!(approved.get("error").is_none(), "{approved}");
    let omission = watch.find("selected landing omission", |event| {
        event["event"] == "attention.raised"
            && event["data"]["kind"] == "candidate"
            && event["data"]["reason"] == "landing"
    });
    let payload = &omission["data"]["payload"];
    assert_eq!(payload["choice"], seventh["choice"]);
    assert_eq!(payload["evidence"]["nodes"], json!(["landing-gate-other"]));
    assert_eq!(
        payload["evidence"]["identity"]["skipped"],
        json!(["landing-gate-other"])
    );
    let identity = &payload["evidence"]["landing"];
    // Preparation already released this parent once; wait for this decision's release.
    watch.find("selected omission released", |event| {
        event["event"] == "execution.released"
            && event["execution"] == identity["execution"]
            && event["seq"].as_i64().unwrap() > omission["seq"].as_i64().unwrap()
    });
    let accepted = operator.call("yard_attempt_accept", json!({
        "project": project.path, "ticket": "Y-1", "attention": omission["attention"],
        "plan_revision": 1, "target": "feature/a", "head": fresh_landed_head, "proof": seventh["proof"],
        "base": identity["base"], "revision": identity["revision"], "landing": identity["execution"],
        "old": identity["prepared"]["old"], "merged": identity["prepared"]["merged"],
        "accept_skipped": ["landing-gate-other"], "text": "Accept this exact selected landing omission"
    }));
    assert_eq!(accepted["result"]["accepted"], true, "{accepted}");
    assert_eq!(watch.said(&said), "held\n");
    let intents = project.rows("SELECT * FROM execution WHERE intent_state = 'open'");
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0]["intent_ref"], "refs/heads/feature/a");
    assert_eq!(
        git(&project.canonical(), &["rev-parse", "refs/heads/feature/a"]).trim(),
        target
    );
    let before_attempt = project.rows("SELECT * FROM attempt ORDER BY id");
    let before_approval = project.rows("SELECT * FROM approval ORDER BY id");
    let before_writers =
        project.rows("SELECT * FROM execution WHERE kind = 'implementation' ORDER BY id");
    let refused = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1", "workspace": "writer-1"
        }),
    );
    assert_eq!(refused["error"]["code"], "refused", "{refused}");
    assert_eq!(refused["error"]["data"]["intent"], intents[0]["id"]);
    assert_eq!(
        project.rows("SELECT * FROM attempt ORDER BY id"),
        before_attempt
    );
    assert_eq!(
        project.rows("SELECT * FROM approval ORDER BY id"),
        before_approval
    );
    assert_eq!(
        project.rows("SELECT * FROM execution WHERE kind = 'implementation' ORDER BY id"),
        before_writers
    );
    std::fs::remove_file(&armed).unwrap();
    std::fs::write(&release, "go\n").unwrap();
    watch.find("selected candidate landed", |event| {
        event["event"] == "attempt.ended" && event["data"]["outcome"] == "landed"
    });
    assert_eq!(
        git(&project.canonical(), &["show", "feature/a:same.txt"]).trim(),
        "writer-1-6"
    );
    assert_eq!(project.canonical_head(), main);
    let gates = rows(
        &project,
        "SELECT * FROM execution WHERE kind = 'gate' ORDER BY id",
    );
    assert!(
        gates
            .iter()
            .any(|row| row["parent"].is_null() && row["head"] == heads[1])
    );
    assert!(
        gates
            .iter()
            .any(|row| row["parent"].is_null() && row["head"] == replacement_head)
    );
    assert!(
        gates
            .iter()
            .any(|row| row["parent"].is_null() && row["head"] == repaired_head)
    );
    assert!(
        gates
            .iter()
            .any(|row| row["parent"].is_null() && row["head"] == superseding_head)
    );
    assert!(
        gates
            .iter()
            .any(|row| row["parent"].is_null() && row["head"] == landed_head)
    );
    for head in [&edited_head, &fresh_landed_head] {
        assert!(
            gates
                .iter()
                .any(|row| row["parent"].is_null() && row["head"] == *head)
        );
    }
    assert!(
        gates
            .iter()
            .filter(|row| row["parent"].is_null())
            .all(|row| row["head"] == heads[1]
                || row["head"] == replacement_head
                || row["head"] == repaired_head
                || row["head"] == superseding_head
                || row["head"] == landed_head
                || row["head"] == edited_head
                || row["head"] == fresh_landed_head)
    );
    let approved = project.rows("SELECT * FROM approval ORDER BY id");
    assert_eq!(approved.len(), 5);
    assert_eq!(approved[0]["choice"], third["choice"]);
    assert_eq!(approved[0]["state"], "withdrawn");
    assert_eq!(approved[1]["choice"], fresh["choice"]);
    assert_eq!(approved[1]["state"], "retired");
    assert_eq!(approved[2]["choice"], fifth["choice"]);
    assert_eq!(approved[2]["state"], "retired");
    assert_eq!(approved[3]["choice"], sixth["choice"]);
    assert_eq!(approved[3]["state"], "retired");
    assert_eq!(approved[4]["choice"], seventh["choice"]);
    assert_eq!(approved[4]["state"], "landed");
}
