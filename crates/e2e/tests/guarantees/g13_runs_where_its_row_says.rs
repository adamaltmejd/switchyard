//! G13: Every gate and seat runs where its row says.

use e2e::*;
use serde_json::{Value, json};
use std::io::Write;
use std::sync::{Arc, Mutex};

fn seat(request: &ModelRequest) -> bool {
    request.has_tool("yard_publish_review")
}

/// The detail each gate execution recorded: its output's tail.
fn gate_details(project: &Project) -> Vec<Value> {
    project.rows(
        "SELECT name, reason, head, outcome, detail FROM execution WHERE kind = 'gate' ORDER BY id",
    )
}

/// The `write=<status>` a probe echoed: non-zero when the write was refused.
fn write_failed(text: &str) -> bool {
    text.lines()
        .find_map(|line| line.strip_prefix("write="))
        .and_then(|status| status.trim().parse::<i32>().ok())
        .is_some_and(|status| status != 0)
}

/// A box gate, a host gate and a seat read the candidate's proof snapshot.
/// The box gate and the seat cannot write it; the seat cannot write its
/// workspace; the box gate receives neither worker route and no ignored
/// worker file. The host candidate gate reads the head, base, only its named
/// env and the snapshot `$YARD_PROOF` names before review, then reads it
/// again at landing. Controls: the implementer writes both live directories
/// and reaches the model route in the same project. While the seat is held
/// the live proof changes, and the landing execution still reads the approved
/// candidate's snapshot, not the changed live directory or the merged
/// checkout's same-named file.
///
/// Sabotage: mount the seat checkout or either proof mount writable; its
/// write succeeds. Mount the live proof for the seat; it writes there. Give
/// the box gate the worker routes; its call reaches the fixture or MCP
/// listener. Build the box gate checkout from the clone; its ignored file
/// survives. Set `YARD_PROOF` to the live proof directory in
/// `supervise::host_gate`; the landing execution reads the changed live
/// file. Or leave it unset; `proof=` names nothing. Inherit the daemon's
/// environment; the unnamed variable reaches the host gate. Check out the
/// base instead of the head; `head=` names the base. Run review alongside the
/// candidate gates; the audit order changes. Omit `YARD_BASE`, or set it to
/// the candidate head; `base=` names the wrong commit.
#[test]
fn gates_and_a_seat_judge_the_proof_snapshot_where_their_row_says() {
    let hold = Latch::new();
    let seat_hold = hold.clone();
    let seat_probe = Arc::new(Mutex::new(String::new()));
    let captured = seat_probe.clone();
    let committed = Arc::new(Mutex::new(String::new()));
    let head = committed.clone();
    let mut machine = Machine::new("g13-proof", move |request| {
        if seat(request) {
            return match request.tool_results().len() {
                // Held on open, so the test changes the live proof first.
                0 => Reply::Hold(
                    seat_hold.clone(),
                    Box::new(Reply::Tools(vec![bash(
                        "cd /workspace; touch planted 2>/dev/null; echo \"file=$?\"; \
                         git commit -q --allow-empty -m planted 2>/dev/null; echo \"commit=$?\"; \
                         git update-ref refs/heads/planted HEAD 2>/dev/null; echo \"ref=$?\"; \
                         v=$(cat /yard/proof/x 2>/dev/null); echo \"read=$v\"; \
                         echo seat > /yard/proof/seat 2>/dev/null; echo \"write=$?\"",
                    )])),
                ),
                1 => {
                    *captured.lock().unwrap() = request.last_tool_result().unwrap().1;
                    Reply::Tools(vec![publish(json!([]))])
                }
                _ => Reply::Text("done".into()),
            };
        }
        if request.opens() {
            return Reply::Tools(vec![bash(
                "cd /workspace && curl -s -m 5 -o /dev/null http://openrouter.yard/api/v1/from-worker; \
                 printf '*.log\\n' > .gitignore && echo left > ignored.log \
                 && printf 'snapshot-payload' > /yard/proof/x \
                 && printf 'snapshot-evidence' > /yard/proof/evidence.txt \
                 && printf 'checkout-evidence' > evidence.txt \
                 && printf 'feature\\n' > feature.txt && git add -A \
                 && git commit -q -m 'Add feature' && git rev-parse HEAD",
            )]);
        }
        *head.lock().unwrap() = request.last_tool_result().unwrap().1;
        Reply::Text("done".into())
    });
    machine.env.push(("GATE_NAMED".into(), "named".into()));
    machine.env.push(("GATE_UNNAMED".into(), "unnamed".into()));
    machine.start();
    let gates = "[gates.boxed]\ncommand = \"v=$(cat /yard/proof/x 2>/dev/null); echo read=$v; \
                echo gate > /yard/proof/gate 2>/dev/null; echo write=$?; \
                curl -sf -m 5 http://openrouter.yard/api/v1/from-gate >/dev/null; \
                echo mcp=$(curl -s -m 5 -o /dev/null -w '%{http_code}' -X POST http://yard.mcp/mcp); \
                echo none=$(curl -s -m 5 -o /dev/null -w '%{http_code}' -X POST http://unrouted.invalid/mcp); \
                test -e ignored.log; echo ignored=$?\"\nstage = \"candidate\"\n\
                [gates.hosted]\ncommand = \"echo head=$(git rev-parse HEAD); echo base=${YARD_BASE:-unset}; \
                echo named=${GATE_NAMED:-unset}; echo unnamed=${GATE_UNNAMED:-unset}; \
                echo proof=$(cat \\\"$YARD_PROOF/evidence.txt\\\" 2>/dev/null); \
                echo checkout=$(cat evidence.txt 2>/dev/null)\"\nstage = \"candidate\"\nruns_in = \"host\"\nenv = [\"GATE_NAMED\"]\n";
    let project = Project::new(&machine, "p", &config(gates));
    let base = project.canonical_head();
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "Add feature"]);
    // The candidate execution ends before the seat starts; the seat is held.
    hold.wait_held();
    let candidate = git(
        &project.path.join(".yard/local/attempts/1/clone"),
        &["rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
    let gates = gate_details(&project);
    assert_eq!(gates.len(), 2, "{gates:?}");
    assert_eq!(gates[1]["name"], "hosted", "{gates:?}");
    let detail = gates[1]["detail"].as_str().unwrap();
    assert!(detail.contains(&format!("head={candidate}")), "{detail}");
    assert!(detail.contains(&format!("base={base}")), "{detail}");
    assert!(detail.contains("named=named"), "{detail}");
    assert!(detail.contains("unnamed=unset"), "{detail}");
    // Every gate, in declared order, before any review.
    let order: Vec<Value> = project
        .rows(
            "SELECT audit.event, execution.kind FROM audit JOIN execution ON execution.id = audit.execution
             WHERE audit.event IN ('execution.started', 'execution.ended')
             AND execution.kind IN ('gate', 'review') ORDER BY audit.seq",
        )
        .into_iter()
        .map(|row| json!([row["kind"], row["event"]]))
        .collect();
    assert_eq!(
        order,
        [
            json!(["gate", "execution.started"]),
            json!(["gate", "execution.ended"]),
            json!(["gate", "execution.started"]),
            json!(["gate", "execution.ended"]),
            json!(["review", "execution.started"]),
        ]
    );
    // The live proof changes after the snapshot; only the snapshot is judged.
    let live = project.path.join(".yard/local/attempts/1/proof");
    std::fs::write(live.join("evidence.txt"), "live-mutated").unwrap();
    hold.release();

    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");

    let clone = project.path.join(".yard/local/attempts/1/clone");
    let committed = committed.lock().unwrap().trim().to_string();
    assert_eq!(git(&clone, &["rev-parse", "HEAD"]).trim(), committed);
    assert!(!clone.join("planted").exists());
    assert!(clone.join("ignored.log").is_file());
    assert_eq!(
        std::fs::read_to_string(live.join("x")).unwrap(),
        "snapshot-payload"
    );

    assert_eq!(gates[0]["name"], "boxed", "{gates:?}");
    let detail = gates[0]["detail"].as_str().unwrap();
    assert!(detail.contains("read=snapshot-payload"), "{detail}");
    assert!(write_failed(detail), "{detail}");
    let answer = |host: &str| {
        detail
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{host}=")))
            .unwrap_or_else(|| panic!("no {host}= in {detail}"))
            .to_string()
    };
    assert_eq!(answer("mcp"), answer("none"), "{detail}");
    assert!(detail.contains("ignored=1"), "{detail}");
    let paths: Vec<String> = machine
        .model
        .requests()
        .iter()
        .map(|request| request.path.clone())
        .collect();
    assert!(
        paths.iter().any(|path| path.ends_with("/from-worker")),
        "{paths:?}"
    );
    assert!(
        !paths.iter().any(|path| path.ends_with("/from-gate")),
        "{paths:?}"
    );

    let seat = seat_probe.lock().unwrap().clone();
    for probe in ["file=", "commit=", "ref="] {
        assert!(seat.contains(probe), "{seat}");
        assert!(!seat.contains(&format!("{probe}0")), "{seat}");
    }
    assert!(seat.contains("read=snapshot-payload"), "{seat}");
    assert!(write_failed(&seat), "{seat}");

    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    let proof = approval["data"]["payload"]["proof"].as_str().unwrap();
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", head, "--proof", proof,
    ]);
    watch.event("landing.recorded", &[]);

    // The host gate's candidate execution, then the same gate again on the
    // merged ref.
    let hosted: Vec<Value> = gate_details(&project)
        .into_iter()
        .filter(|gate| gate["name"] == "hosted")
        .collect();
    assert_eq!(hosted.len(), 2, "{hosted:?}");
    assert_eq!(hosted[0]["reason"], "candidate", "{hosted:?}");
    assert_eq!(hosted[1]["reason"], "landing", "{hosted:?}");
    for gate in &hosted {
        let detail = gate["detail"].as_str().unwrap();
        assert!(detail.contains("proof=snapshot-evidence"), "{detail}");
        // The checkout carries a same-named file with other content.
        assert!(detail.contains("checkout=checkout-evidence"), "{detail}");
    }
}

/// A host landing gate runs on the merged ref and never for a candidate
/// whose approval was superseded. The first ticket's landing is a real
/// merge and is held in its gate; the second is approved behind it, and
/// edited while its own landing is held in its `git merge-tree`, after the
/// queue has taken it and before any gate.
///
/// Sabotage: make `queue::land` start its gates from the rows it read
/// before merging, without `admit`; the edited ticket's landing runs the
/// host gate. Or run the landing gate on the candidate head instead of the
/// merged ref; its head is not a merge of the target and the first head. Or
/// set `YARD_BASE` to the candidate head; `base=` names it instead of the
/// target.
///
/// Sabotage: in `queue::admit` (crates/yard/src/jobs/queue.rs), which the
/// landing calls after the held merge, skip the reread by replacing `let
/// lapse = holds(...).await?;` with `let lapse = None;` and `unchanged` with
/// `true`; the superseded Y-2 goes on to a gate and the decisive-event
/// assertion fails on `execution.started` instead of the landing's
/// `execution.ended`.
#[test]
fn a_host_landing_gate_runs_on_the_merged_ref_only_when_approved() {
    let machine = Machine::new("g13-host-landing", |request| {
        let file = if request.prompt().contains("Second") {
            "second.txt"
        } else {
            "first.txt"
        };
        act(request, vec![commit_file(file, "text\n", "Add a file")])
    });
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
    let hold = fifo("hold");
    let (said, resume) = (fifo("said"), fifo("resume"));
    let mut release = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&hold)
        .unwrap();
    machine.start();
    let gate = format!(
        "[gates.held]\ncommand = \"echo base=${{YARD_BASE:-unset}}; echo head=$(git rev-parse HEAD); read line < {}\"\nruns_in = \"host\"\n",
        hold.display()
    );
    let project = Project::new(
        &machine,
        "p",
        &config(&gate).replace("review = [\"correctness\"]", "review = \"none\""),
    );
    let mut watch = project.watch(0);
    project.json(&["ticket", "new", "--title", "First"]);
    project.json(&["ticket", "new", "--title", "Second"]);
    let approval = |watch: &mut Watch, ticket: &str| {
        watch.find(ticket, |event| {
            event["event"] == "attention.raised"
                && event["ticket"] == ticket
                && event["data"]["kind"] == "approval"
        })["data"]["payload"]["head"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let first = approval(&mut watch, "Y-1");
    let second = approval(&mut watch, "Y-2");
    // The operator moves the target, so the first landing is a merge.
    project.write("other.txt", "other\n");
    project.git(&["add", "other.txt"]);
    project.git(&["commit", "--quiet", "-m", "Operator change"]);
    project.json(&["sync"]);
    let target = project.canonical_head();

    // Only the second landing merges the second head.
    machine.wrapper(
        "git",
        &format!(
            "case \" $* \" in *' merge-tree '*'{second}'*) echo held > '{}'; read line < '{}';; esac",
            said.display(),
            resume.display()
        ),
    );

    project.json(&[
        "attempt", "approve", "--target", "main", "Y-1", "--head", &first,
    ]);
    watch.event("execution.started", &[("name", "held")]);
    project.json(&[
        "attempt", "approve", "--target", "main", "Y-2", "--head", &second,
    ]);
    // One line per landing gate that could run, so a second gate never
    // blocks and the count below is what fails.
    release.write_all(b"go\ngo\n").unwrap();
    watch.find("Y-1 landed", |event| {
        event["event"] == "landing.recorded" && event["ticket"] == "Y-1"
    });
    assert_eq!(watch.said(&said), "held\n");
    let revision = project.json(&["ticket", "show", "Y-2"])["revision"].to_string();
    project.json(&[
        "ticket",
        "edit",
        "Y-2",
        "--revision",
        &revision,
        "--body",
        "Changed",
    ]);
    std::fs::write(&resume, "go\n").unwrap();
    // The first Y-2 event that decides the outcome: its landing ends with no
    // gate, or goes ahead on the superseded candidate.
    let decided = watch.until("Y-2 landing ended or a gate", |event| {
        event["ticket"] == "Y-2"
            && (event["event"] == "landing.recorded"
                || (event["event"] == "execution.started" && event["data"]["kind"] == "gate")
                || (event["event"] == "execution.ended" && event["data"]["kind"] == "landing"))
    });
    assert_eq!(decided["event"], "execution.ended", "{decided}");

    let gates = gate_details(&project);
    assert_eq!(gates.len(), 1, "{gates:?}");
    let merged = gates[0]["head"].as_str().unwrap();
    assert!(
        gates[0]["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("head={merged}"))
    );
    assert!(
        gates[0]["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("base={target}")),
        "{:?}",
        gates[0]["detail"]
    );
    let parents = git(
        &project.canonical(),
        &["rev-list", "--parents", "-n", "1", merged],
    );
    assert_eq!(
        parents.split_whitespace().skip(1).collect::<Vec<_>>(),
        [target.as_str(), first.as_str()]
    );
    assert_eq!(
        project.rows("SELECT state FROM approval WHERE attempt = 2"),
        vec![json!({ "state": "withdrawn" })]
    );
}

/// G13/G2: A writable ordinary worker's second launch reads admitted blobs
/// after both source and canonical edits. Explicit staging adoption changes
/// its consumed definition and starts a fresh session on the same attempt.
/// Sabotage: stage selected inputs from the live candidate or canonical head;
/// the second probe reads changed bytes. Mount /yard/input writable; the
/// trusted write succeeds while the same-project source write controls it.
/// Omit selected blobs from consumed_definition/session comparison; adoption
/// resumes the old session. Stage all guidance as selected input; the
/// unselected file appears in the trusted mount.
#[test]
fn ordinary_writers_read_frozen_selected_inputs_and_adoption_starts_fresh() {
    let hold = Latch::new();
    let second_hold = hold.clone();
    let probes = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured = probes.clone();
    let round = Mutex::new(0);
    let machine = Machine::new("g13-selected-ordinary", move |request| {
        if request.has_tool("yard_publish_report") {
            return planned(request);
        }
        let mut round = round.lock().unwrap();
        if request.opens() {
            *round += 1;
            return Reply::Tools(vec![bash(&format!(
                "cd /workspace; \
                 printf 'candidate-skill\\n' > selected.md; echo source_write=$?; \
                 printf 'candidate-reference\\n' > reference.txt; \
                 echo skill=$(cat /yard/input/trusted/selected.md); \
                 echo reference=$(cat /yard/input/trusted/reference.txt); \
                 echo blocked > /yard/input/trusted/selected.md 2>/dev/null; echo write=$?; \
                 test -e /yard/input/trusted/unselected.md; echo unselected=$?; \
                 printf '{round}\\n' > feature.txt; \
                 git add selected.md reference.txt feature.txt && git commit -q -m 'Candidate edit'"
            ))]);
        }
        captured
            .lock()
            .unwrap()
            .push(request.last_tool_result().unwrap().1);
        if *round == 2 {
            Reply::Hold(second_hold.clone(), Box::new(Reply::Text("done".into())))
        } else {
            Reply::Text("done".into())
        }
    });
    machine.start();
    let configuration = config(
        r#"
[profiles.selected]
access = "write-allowed"
outcomes = ["complete"]
[profiles.selected.outputs]
result = "candidate"
[profiles.selected.runner]
kind = "agent"
agent = "worker"
instructions = "Use the selected skill and reference."
skills = ["selected.md"]
references = ["reference.txt"]
"#,
    )
    .replace("review = [\"correctness\"]", "review = \"none\"");
    let project = Project::new(&machine, "p", &configuration);
    project.write("selected.md", "admitted-skill\n");
    project.write("reference.txt", "admitted-reference\n");
    project.write("unselected.md", "unselected-guidance\n");
    project.git(&["add", "-A"]);
    project.git(&["commit", "-qm", "Selected fixture inputs"]);
    project.json(&["sync"]);
    let identities = || {
        json!([
            {"path": "selected.md", "blob": project.git(&["rev-parse", "HEAD:selected.md"]).trim(), "executable": false},
            {"path": "reference.txt", "blob": project.git(&["rev-parse", "HEAD:reference.txt"]).trim(), "executable": false}
        ])
    };
    let original = identities();
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Use selected guidance",
        "--workflow",
        "plan",
    ]);
    let planner = watch.event("attempt.ended", &[("outcome", "planned")]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner["attempt"]
    });
    watch.find("planner released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({
            "project": project.path, "ticket": "Y-1",
            "workflow": "default",
            "overrides": {"implementer": {"profile": "selected"}}
        }),
    );
    assert!(started.get("error").is_none(), "{started}");
    let attempt = started["result"]["attempt"].as_i64().unwrap();
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let admitted = project.json(&["attempt", "show", "Y-1"])["attempt"]["admission"].clone();
    assert_eq!(
        admitted["inputs"]["selected_agent_sources"]["implementer"],
        original
    );

    // The candidate has already committed conflicting selected bytes. Sync
    // changes canonical too, before the repair's staging boundary.
    project.write("selected.md", "adopted-skill\n");
    project.write("reference.txt", "adopted-reference\n");
    project.git(&["add", "selected.md", "reference.txt"]);
    project.git(&["commit", "-qm", "Operator changes selected inputs"]);
    project.json(&["sync"]);
    let adopted = identities();
    let staging = project.canonical_head();
    project.json(&[
        "attempt",
        "reject",
        "--target",
        "main",
        "Y-1",
        "--head",
        approval["data"]["payload"]["head"].as_str().unwrap(),
        "--text",
        "Read the inputs again",
    ]);
    hold.wait_held();
    let probe = probes.lock().unwrap().last().unwrap().clone();
    assert!(probe.contains("skill=admitted-skill"), "{probe}");
    assert!(probe.contains("reference=admitted-reference"), "{probe}");
    assert!(probe.contains("source_write=0"), "{probe}");
    assert!(write_failed(&probe), "{probe}");
    assert!(probe.contains("unselected=1"), "{probe}");
    let sessions = project.rows(&format!("SELECT id, resumed FROM execution WHERE kind = 'implementation' AND attempt = {attempt} ORDER BY id"));
    assert_eq!(sessions[1]["resumed"], sessions[0]["id"], "{sessions:?}");
    let revised = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-1", "plan_revision": 1,
            "staging_commit": staging
        }),
    );
    assert_eq!(revised["result"]["plan_revision"], 2, "{revised}");
    hold.release();
    let approval = watch.attention();
    assert_eq!(approval["data"]["kind"], "approval", "{approval}");
    let probe = probes.lock().unwrap().last().unwrap().clone();
    assert!(probe.contains("skill=adopted-skill"), "{probe}");
    assert!(probe.contains("reference=adopted-reference"), "{probe}");
    let revised_admission = project.json(&["attempt", "show", "Y-1"])["attempt"]["plan"].clone();
    assert_eq!(
        revised_admission["inputs"]["selected_agent_sources"]["implementer"],
        adopted
    );
    assert_eq!(project.rows(&format!("SELECT plan_revision, resumed FROM execution WHERE kind = 'implementation' AND attempt = {attempt} ORDER BY id")).last().unwrap(),
        &json!({"plan_revision": 2, "resumed": null}));
    let requests: Vec<_> = machine
        .model
        .requests()
        .into_iter()
        .filter(|request| request.opens() && !request.has_tool("yard_publish_report"))
        .collect();
    let fresh = requests.last().unwrap();
    assert_eq!(
        fresh.body["messages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|message| message["role"] == "user")
            .count(),
        1
    );
    assert!(
        fresh
            .last_user()
            .contains("/yard/input/trusted/selected.md")
    );
    assert!(
        fresh
            .last_user()
            .contains("/yard/input/trusted/reference.txt")
    );
    assert!(
        !fresh
            .last_user()
            .contains("/yard/input/trusted/unselected.md")
    );
}

/// G2/G9/G13: one ordinary three-role path retains its clone while changing
/// selected settings and grants. One machine, planner and three role boxes
/// also reach native verification and landing.
/// Expected bytes, inodes and ancestry come from the fixture, boxed Git and
/// host Git; settings come from requests received by the host model.
/// Sabotage: allocate a second clone for a sequential role; inode/ancestry or
/// retained bytes differ. Bypass workspace ownership; the held retry starts.
/// Infer publication/operator authority from source access; cleanup publishes
/// a candidate or its landing tool succeeds.
#[test]
fn selected_cleanup_hands_one_exclusive_clone_to_separately_configured_repair() {
    use std::os::unix::fs::MetadataExt;

    let hold = Latch::new();
    let cleanup_hold = hold.clone();
    let probes = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let captured = probes.clone();
    let machine = Machine::new("g13-handoff", move |request| {
        let model = request.body["model"].as_str().unwrap();
        if model == "fake-model" {
            return planned(request);
        }
        if model == "cleanup-selected" {
            return match request.tool_results().len() {
                0 => Reply::Tools(vec![tool("yard_context", json!({}))]),
                1 => Reply::Tools(vec![bash(
                    "test \"$(cat chain.txt)\" = implemented && \
                     test \"$(cat /yard/input/trusted/cleanup.md)\" = trusted-cleanup && \
                     printf 'cleaned\\n' > chain.txt && git add chain.txt && git commit -qm Cleanup && \
                     echo inode=$(stat -c %i .git) && echo source_write=0 && \
                     cat /tmp/landing-probe.json",
                )]),
                2 => {
                    captured
                        .lock()
                        .unwrap()
                        .push((model.into(), request.last_tool_result().unwrap().1));
                    Reply::Hold(
                        cleanup_hold.clone(),
                        Box::new(Reply::Tools(vec![publish_report()])),
                    )
                }
                _ => Reply::Text("done".into()),
            };
        }
        if request.opens() {
            let command = if model == "implementer-selected" {
                "printf 'implemented\\n' > chain.txt && git add chain.txt && git commit -qm Implement && echo inode=$(stat -c %i .git)"
            } else {
                "test \"$(cat chain.txt)\" = cleaned && test \"$(cat /yard/input/trusted/repair.md)\" = trusted-repair && printf 'repaired\\n' > chain.txt && git add chain.txt && git commit -qm Repair && echo inode=$(stat -c %i .git)"
            };
            return Reply::Tools(vec![bash(command)]);
        }
        captured
            .lock()
            .unwrap()
            .push((model.into(), request.last_tool_result().unwrap().1));
        Reply::Text("done".into())
    });
    machine.start();
    let mut extra = String::from(
        "\n[gates.verify]\ncommand = 'test \"$(cat chain.txt)\" = repaired'\nstage = 'candidate'\nruns_in = 'host'\n",
    );
    for (role, output) in [
        ("implementer", "candidate"),
        ("cleanup", "report"),
        ("repair", "candidate"),
    ] {
        extra.push_str(&format!(
            "\n[agents.{role}]\nharness = 'pi'\nprovider = 'openrouter'\nmodel = '{role}-selected'\n\n[profiles.{role}]\naccess = 'write-allowed'\noutcomes = ['complete']\n[profiles.{role}.outputs]\nresult = '{output}'\n[profiles.{role}.runner]\nkind = 'agent'\nagent = '{role}'\ninstructions = 'Selected {role} role.'\n"
        ));
        if role != "implementer" {
            extra.push_str(&format!("skills = ['{role}.md']\n"));
        }
    }
    let project = Project::new(
        &machine,
        "p",
        &config(&extra).replace("review = [\"correctness\"]", "review = \"none\""),
    );
    project.write("cleanup.md", "trusted-cleanup\n");
    project.write("repair.md", "trusted-repair\n");
    // Probe through the harness's actual MCP session, from a trusted fixture
    // extension. No second session or bearer enters the assertion.
    project.write(
        ".pi/extensions/probe/index.ts",
        r#"import { writeFileSync } from 'node:fs';
export default function () {
  const original = globalThis.fetch;
  globalThis.fetch = async function (url, options) {
    const message = JSON.parse(options?.body || '{}');
    if (message.method === 'tools/call' && message.params?.name === 'yard_context') {
      const probe = await original(url, { ...options, body: JSON.stringify({
        jsonrpc: '2.0', id: 9876, method: 'tools/call', params: {
          name: 'yard_attempt_approve', arguments: {}
        }
      }) });
      writeFileSync('/tmp/landing-probe.json', await probe.text());
    }
    return original(url, options);
  };
}
"#,
    );
    project.git(&["add", "-A"]);
    project.git(&["commit", "-qm", "Trusted handoff inputs"]);
    project.json(&["sync"]);
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Sequential fixture",
        "--workflow",
        "plan",
    ]);
    let planner = watch.event("attempt.ended", &[("outcome", "planned")]);
    let cleanup = watch.find("planner cleanup", |event| {
        event["event"] == "execution.ended"
            && event["data"]["kind"] == "cleanup"
            && event["attempt"] == planner["attempt"]
    });
    watch.find("planner release", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let graph = sequential_handoff_graph();
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "graph": graph}),
    );
    assert!(started.get("error").is_none(), "{started}");
    let attempt = started["result"]["attempt"].as_i64().unwrap();
    hold.wait_held();
    let owner = project.rows(&format!("SELECT id FROM execution WHERE attempt = {attempt} AND node = 'cleanup' AND status = 'running'"))[0]["id"].as_i64().unwrap();
    let clone = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let inode = std::fs::metadata(clone.join(".git")).unwrap().ino();
    assert_eq!(
        std::fs::read_to_string(clone.join("chain.txt")).unwrap(),
        "cleaned\n"
    );
    let refused = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "implementer"}),
    );
    assert_eq!(
        refused["error"]["data"],
        json!({"reason": "workspace_owned", "workspace": "implementer", "execution": owner}),
        "{refused}"
    );
    let probe = probes.lock().unwrap().last().unwrap().1.clone();
    assert!(probe.contains("source_write=0"), "{probe}");
    let reply: Value =
        serde_json::from_str(probe.lines().find(|line| line.starts_with('{')).unwrap()).unwrap();
    let failure = &reply["result"]["structuredContent"]["error"];
    assert_eq!(failure["data"]["execution"], owner, "{failure}");
    assert_eq!(failure["message"], "operator_only", "{failure}");
    hold.release();
    let approval = watch.until("handoff approval", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });
    let first = approval["data"]["payload"].clone();
    let rows = project.rows(&format!("SELECT id, node, workspace, resumed, result, teardown, reserved FROM execution WHERE attempt = {attempt} AND kind = 'implementation' ORDER BY id"));
    assert_eq!(
        rows.iter()
            .map(|row| row["node"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["implementer", "cleanup", "repair"]
    );
    for row in &rows {
        assert_eq!(row["workspace"], "implementer");
        assert_eq!(row["teardown"], 1);
        assert_eq!(row["reserved"], 0);
        assert_eq!(row["resumed"], Value::Null);
    }
    for pair in rows.windows(2) {
        let release = project.rows(&format!(
            "SELECT seq FROM audit WHERE event = 'execution.released' AND execution = {}",
            pair[0]["id"]
        ))[0]["seq"]
            .as_i64()
            .unwrap();
        let start = project.rows(&format!(
            "SELECT seq FROM audit WHERE event = 'execution.started' AND execution = {}",
            pair[1]["id"]
        ))[0]["seq"]
            .as_i64()
            .unwrap();
        assert!(release < start);
    }
    let result: Value = serde_json::from_str(rows[1]["result"].as_str().unwrap()).unwrap();
    assert_eq!(result["outputs"]["result"]["kind"], "report");
    assert_eq!(project.rows(&format!("SELECT json_extract(data, '$.candidate') AS execution FROM audit WHERE event = 'attempt.candidate' AND attempt = {attempt} ORDER BY seq")), vec![json!({"execution": rows[0]["id"]}), json!({"execution": rows[2]["id"]})]);
    assert_eq!(std::fs::metadata(clone.join(".git")).unwrap().ino(), inode);
    let probes = probes.lock().unwrap();
    let inodes: Vec<_> = probes
        .iter()
        .map(|(_, probe)| {
            probe
                .lines()
                .find(|line| line.starts_with("inode="))
                .unwrap()
        })
        .collect();
    assert_eq!(inodes.len(), 3);
    assert!(inodes.iter().all(|value| *value == inodes[0]), "{inodes:?}");
    for role in ["implementer", "cleanup", "repair"] {
        let request = machine
            .model
            .requests()
            .into_iter()
            .find(|request| request.opens() && request.body["model"] == format!("{role}-selected"))
            .unwrap();
        assert!(
            request
                .last_user()
                .contains(&format!("Selected {role} role."))
        );
        assert_eq!(
            request.body["messages"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|message| message["role"] == "user")
                .count(),
            1
        );
    }
    let ancestry = git(
        &project.canonical(),
        &[
            "rev-list",
            "--format=%s",
            "--no-commit-header",
            first["head"].as_str().unwrap(),
            "-3",
        ],
    );
    assert_eq!(
        ancestry.lines().collect::<Vec<_>>(),
        ["Repair", "Cleanup", "Implement"]
    );
    project.json(&[
        "attempt",
        "approve",
        "--target",
        "main",
        "Y-1",
        "--head",
        first["head"].as_str().unwrap(),
    ]);
    watch.event("landing.recorded", &[]);
    assert_eq!(
        git(&project.canonical(), &["show", "refs/heads/main:chain.txt"]),
        "repaired\n"
    );
}

// Shared by the ordinary and packaged cleanup host scenarios.
pub(super) fn sequential_handoff_graph() -> Value {
    json!({"inputs": {"merged": "candidate", "feedback": "verdict"}, "nodes": {
        "implementer": {"profile": "implementer", "workspace": "implementer",
            "routes": {"complete": [{"to": "cleanup"}]}},
        "cleanup": {"profile": "cleanup", "workspace": "implementer",
            "inputs": {"candidate": {"node": "implementer", "output": "result", "kind": "candidate"}},
            "requires": ["candidate"], "routes": {"complete": [{"to": "repair"}]}},
        "repair": {"profile": "repair", "workspace": "implementer",
            "inputs": {"cleanup": {"node": "cleanup", "output": "result", "kind": "report"},
                "feedback": {"output": "feedback", "kind": "verdict"}},
            "requires": ["cleanup"], "routes": {"complete": [{"to": "candidate-gate-verify"}]}},
        "candidate-gate-verify": {"runner": {"kind": "command", "gate": "verify"},
            "access": "write-allowed", "inputs": {"candidate": {"node": "repair", "output": "result", "kind": "candidate"}},
            "requires": ["candidate"], "outputs": {"result": "verdict"}, "outcomes": ["complete", "failed"],
            "routes": {"complete": [{"to": "landing-gate-verify"}], "failed": [{"to": "repair", "repeat": 1}]}},
        "landing-gate-verify": {"runner": {"kind": "command", "gate": "verify"},
            "access": "write-allowed", "inputs": {"merged": {"output": "merged", "kind": "candidate"}},
            "requires": ["merged"], "outputs": {"result": "verdict"}, "outcomes": ["complete"]}
    }})
}
