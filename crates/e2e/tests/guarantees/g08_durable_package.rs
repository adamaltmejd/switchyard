//! G8/G12/G13: explicit packaged selection and frozen instruction authority.

use e2e::*;
use serde_json::{Value, json};
use std::path::Path;

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            assert!(kind.is_file(), "trusted build inputs must be regular files");
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn fixture<'a>(machine: &'a Machine) -> Project<'a> {
    let inputs = std::env::var_os("YARD_E2E_DURABLE_INPUTS")
        .expect("YARD_E2E_DURABLE_INPUTS: public Node archive and packages/ locked archives");
    // Copy public build inputs only. No package or model code executes on host.
    copy_tree(Path::new(&inputs), &machine.root.join("build-inputs"));
    let project = Project::new(machine, "p", &config(""));
    let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runners/durable-audit");
    copy_tree(&package, &project.path.join("runners/durable-audit"));
    project.write("large.txt", &"x".repeat(65536));
    let sentinel = "require('node:fs').writeFileSync('/yard/state/sentinel', 'executed');\n";
    project.write("candidate.cjs", sentinel);
    project.write("candidate/.pi/extensions/candidate.js", sentinel);
    project.write(".pi/extensions/unselected.js", sentinel);
    project.write(
        "AGENTS.md",
        "UNSELECTED_GUIDANCE: load no selected audit instructions.\n",
    );
    project.write(".agents/skills/unselected/SKILL.md", "UNSELECTED_SKILL\n");
    project.write("candidate/trusted/selected.md", "CANDIDATE_SKILL\n");
    project.write(
        "audit.json",
        "CANDIDATE_LAUNCH: not trusted configuration\n",
    );
    project.write(
        "trusted/selected.md",
        "TRUSTED_SELECTED_SKILL_Y92: inspect declared files and publish.\n",
    );
    project.write("trusted/reference.txt", "TRUSTED_REFERENCE_Y92\n");
    project.git(&["add", "-A"]);
    project.git(&[
        "commit",
        "-qm",
        "Trusted packaged audit fixture and hostile candidate",
    ]);
    project.json(&["sync"]);
    project
}

fn image(project: &Project<'_>, sabotage: &str) {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runners/durable-audit/Dockerfile"),
    )
    .unwrap()
    .replace("\"$TARGETARCH\"", "\"$(uname -m)\"");
    project.write(
        ".yard/Dockerfile",
        &format!(
            "{source}\nLABEL dev.yard.e2e-machine=\"{}\"\n{sabotage}",
            project.machine.root.display()
        ),
    );
    project.git(&["add", ".yard/Dockerfile"]);
    project.git(&["commit", "-qm", "Configure isolated packaged image"]);
    project.json(&["sync"]);
}

fn launch(project: &Project<'_>, attempt: i64, prompt: &str) {
    launch_with_source(project, attempt, prompt, "large.txt");
}

fn launch_with_source(project: &Project<'_>, attempt: i64, prompt: &str, source: &str) {
    // Select canonical instructions through the operator config and MCP.
    // Candidate audit.json is hostile data, never daemon launch input.
    let selected = config(&format!(r#"
[profiles.durable]
access = "read-only"
outcomes = ["complete"]
[profiles.durable.outputs]
result = "report"
[profiles.durable.runner]
kind = "packaged"
package = "durable-audit"
agent = "worker"
instructions = {}
skill = "trusted/selected.md"
references = ["trusted/reference.txt"]
files = [{{root = "source", path = {}}}, {{root = "source", path = "candidate.cjs"}}, {{root = "source", path = "candidate/.pi/extensions/candidate.js"}}]
"#, json!(prompt), json!(source))).replace("[workflows.plan]", "[workflows.plan]\nprofile = \"durable\"");
    project.reconfigure(&selected);
    project.machine.wrapper("pinfold", &format!(r#"
if [ "$1 $2" = 'image build' ]; then
    previous=''
    for arg do
        if [ "$previous" = '--context' ]; then cp -R '{root}/build-inputs' "$arg/build-inputs"; fi
        previous="$arg"
    done
fi
if [ "$1 $2" = 'box up' ] && [ -f '{root}/hold-down-{attempt}' ]; then
    {{ cat; echo held > '{root}/down-said-{attempt}'; read released < '{root}/down-release-{attempt}'; }} | "$REAL" "$@"
    exit $?
fi
if [ "$1 $2" = 'box list' ] && [ -f '{root}/hold-listing' ]; then
    "$REAL" "$@"; result=$?
    echo "$result" > '{root}/listing-said'
    read released < '{root}/listing-release'
    exit "$result"
fi
if [ "$1 $2" = 'box exec' ]; then
    handle="$3"
    case " $* " in *' --mode json '*)
        "$REAL" box exec "$handle" -- sh -c '
            test -e /yard/state/audit-checkpoints/owned; echo predecessor_visible=$?
            test -d /yard/proof; echo proof_present=$?
            echo blocked > /yard/proof/intrusion 2>/dev/null; echo proof_write=$?
            cd /opt/yard/durable-audit && sha256sum --quiet --strict -c identity.sha256
            echo package_seal=$?
        ' >> '{root}/fresh-{attempt}'
        "$REAL" box exec "$handle" -- cat /opt/yard/durable-audit/runner.mjs > '{root}/asset-{attempt}'
    ;; *) exec "$REAL" "$@" ;; esac
    mkfifo '{root}/stdout-{attempt}.fifo' '{root}/stderr-{attempt}.fifo'
    tee -a '{root}/stdout-{attempt}' < '{root}/stdout-{attempt}.fifo' &
    stdout_observer=$!
    tee -a '{root}/stderr-{attempt}' < '{root}/stderr-{attempt}.fifo' >&2 &
    stderr_observer=$!
    "$REAL" "$@" > '{root}/stdout-{attempt}.fifo' 2> '{root}/stderr-{attempt}.fifo'
    result=$?
    wait "$stdout_observer" "$stderr_observer"
    rm '{root}/stdout-{attempt}.fifo' '{root}/stderr-{attempt}.fifo'
    "$REAL" box exec "$handle" -- sh -c '
        env | grep -q "sk-e2e-"; echo upstream_in_env=$?
        grep -R -a -q "sk-e2e-" /workspace /yard/input /yard/proof /yard/state /opt/yard/durable-audit; echo upstream_in_files=$?
        test -e /yard/state/sentinel; echo sentinel=$?
        test -e /yard/state/package-executed; echo package_executed=$?
        echo blocked > /workspace/intrusion 2>/dev/null; echo source_write=$?
        echo blocked > /yard/input/trusted/trusted/selected.md 2>/dev/null; echo trusted_write=$?
        test -e /yard/input/trusted/AGENTS.md; echo unselected_guidance=$?
        test -e /yard/state/agent/extensions/unselected.js; echo unselected_extension=$?
        test -e /yard/state/agent/skills/agents/unselected/SKILL.md; echo unselected_skill=$?
        test -e /yard/input/trusted/candidate/trusted/selected.md; echo candidate_skill=$?
        echo scratch > /yard/state/audit-scratch/probe; echo scratch_write=$?
        echo state > /yard/state/audit-checkpoints/probe; echo checkpoint_write=$?
    ' > '{root}/probe-{attempt}'
    if [ -f '{root}/hold-completed-{attempt}' ]; then
        rm '{root}/hold-completed-{attempt}'
        echo held > '{root}/said-{attempt}'
        read released < '{root}/release-{attempt}'
    fi
    exit "$result"
fi
"#, root = project.machine.root.display()));
}

fn selected_start(project: &Project<'_>, title: &str) {
    let mut operator = project.machine.operator();
    operator.initialize();
    let selected = operator.call(
        "yard_plan_validate",
        json!({"project": project.path,
        "workflow": "plan", "overrides": {"implementer": {"profile": "durable"}}}),
    );
    assert!(selected["error"].is_null(), "{selected}");
    assert_eq!(selected["result"]["shape"], "packaged-audit");
    project.json(&["ticket", "new", "--title", title, "--workflow", "plan"]);
}

fn audit_report() -> ToolCall {
    tool(
        "yard_publish_report",
        json!({"outcome": "complete", "report": expected_report()}),
    )
}

fn expected_report() -> Value {
    // Schema-valid test-audit content is supplied by the fixture model.
    json!({"summary": "Test audit: G8 native publication, G12 route, G13 selected inputs.",
        "findings": ["Sabotage: candidate launch or broad extensions bypass instruction selection.",
            "Oracle: fixture markers, Git blobs and boxed probes."]})
}

fn controller_requests(machine: &Machine) -> Vec<ModelRequest> {
    machine
        .model
        .requests()
        .into_iter()
        .filter(|r| r.has_tool("read_declared"))
        .collect()
}

fn ended(project: &Project<'_>, ticket: &str, prompt: &str) -> Value {
    let seq = project.json(&["status"])["seq"].as_i64().unwrap();
    let mut watch = project.watch(seq);
    selected_start(project, prompt);
    // Successful planners raise no attention. Bind the terminal observation to
    // the admitted execution, so cleanup or another ticket cannot satisfy it.
    let started = watch.until("admitted planner started", |event| {
        event["event"] == "execution.started"
            && event["ticket"] == ticket
            && event["data"]["kind"] == "implementation"
    });
    watch.until("admitted planner ended", |event| {
        event["event"] == "execution.ended"
            && event["ticket"] == ticket
            && event["attempt"] == started["attempt"]
            && event["execution"] == started["execution"]
            && event["data"]["kind"] == "implementation"
    });
    // Capacity release commits checked teardown after the work future ends.
    watch.until("admitted planner released", |event| {
        event["event"] == "execution.released"
            && event["attempt"] == started["attempt"]
            && event["execution"] == started["execution"]
    });
    let shown = project.json(&["attempt", "show", ticket]);
    let execution = shown["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|execution| execution["execution"] == started["execution"])
        .unwrap()
        .clone();
    assert_eq!(execution["teardown"], true);
    execution
}

fn transcript(project: &Project<'_>, attempt: i64) -> String {
    // Observe the real command's streams outside daemon-owned attempt data,
    // which successful planner cleanup may already have removed.
    ["stdout", "stderr"]
        .map(|stream| {
            std::fs::read_to_string(project.machine.root.join(format!("{stream}-{attempt}")))
                .unwrap()
        })
        .concat()
}

/// G8 pre-call packaging and G12/G13 isolation: remove a sealed Durable
/// entrypoint, then repair the image in the SAME project. The controller owns
/// no box; Yard selects its child through real Pi's native input hook. The fixture supplies
/// report text, selected skill/reference, key, hostile executable/extension
/// and an oversized source. Expected 32768-byte read comes from the spec.
/// Sabotage: skip identity checks, load CodingTools or candidate code, ignore
/// declared files/output bound, omit selected skill, persist the bearer/key,
/// count report characters instead of bytes, bypass the injecting route,
/// restore outer-model election, route native frames through Pi's stdout
/// handling or ignore the child's terminal status.
/// Stage broad base guidance/extensions, or stage selected files from the
/// current canonical/candidate tree instead of admitted blobs: excluded
/// markers/sentinels or the edited execution's old markers fail.
/// Existing Pi report tests prove publication,
/// not this packaged runtime, its preflight, or its bounded read surface.
/// Declared candidate/../large.txt normalizes to a real fixture file, so
/// bypassing the declared '..' component guard reaches model requests.
/// Sabotage: remove that guard. The undeclared traversal tool argument below
/// exercises a different mechanism: allowlist lookup, not path validation.
/// G13: a real pinfold exec wrapper holds after Pi and its child have published
/// and exited, while their box still exists and the daemon awaits exec end.
/// Edit there; keep the checkpoint while the replacement controller's model
/// request is held. The wrapper, not a binary barrier, holds that exact point.
/// Reuse
/// attempt state, enable resume, omit the proof mount or make it writable,
/// or follow checkpoint links during cleanup: fresh state or host sentinels
/// fail. G14's writable repair tests replacement of a staged models file;
/// this tests exclusive audit state and whole-attempt cleanup instead.
/// Mount the predecessor state at the replacement's `/yard/state`: its
/// `owned` checkpoint becomes visible and the fresh-state assertion fails.
#[test]
fn a_packaged_audit_refuses_missing_assets_then_publishes_in_a_corrected_box() {
    let second = Latch::new();
    let second_held = second.clone();
    let machine = Machine::new("g8-durable-package", move |request| {
        assert!(
            request.has_tool("read_declared"),
            "outer Pi made a model request"
        );
        assert!(request.context().contains("TRUSTED_SELECTED_SKILL_Y92"));
        for marker in [
            "UNSELECTED_GUIDANCE",
            "UNSELECTED_SKILL",
            "CANDIDATE_SKILL",
            "CANDIDATE_LAUNCH",
            "LATER_SELECTED_SKILL",
            "LATER_REFERENCE",
        ] {
            assert!(!request.context().contains(marker));
        }
        assert!(!request.has_tool("bash"));
        if request.opens() {
            let reply = Reply::Tools(vec![
                tool("read_declared", json!({"file": "source:large.txt"})),
                tool(
                    "read_declared",
                    json!({"file": "trusted:trusted/reference.txt"}),
                ),
                tool("read_declared", json!({"file": "source:candidate.cjs"})),
                tool(
                    "read_declared",
                    json!({"file": "source:candidate/.pi/extensions/candidate.js"}),
                ),
                tool("read_declared", json!({"file": "source:../operator.env"})),
                tool("bash", json!({"command": "node /workspace/candidate.cjs"})),
            ]);
            return if request.prompt().contains("Fresh replacement audit") {
                Reply::Hold(second_held.clone(), Box::new(reply))
            } else {
                reply
            };
        }
        let results = request.tool_results();
        if results
            .iter()
            .any(|(name, _)| name == "yard_publish_report")
        {
            assert!(results.iter().any(|(_, s)| s == "invalid_report"));
            return Reply::Tools(vec![audit_report()]);
        }
        let read = results
            .iter()
            .find(|(_, s)| s.contains("truncated"))
            .unwrap();
        let read: Value = serde_json::from_str(&read.1).unwrap();
        assert_eq!(read, json!({"value": "x".repeat(32768), "truncated": true}));
        assert!(
            results
                .iter()
                .any(|(_, s)| s.contains("TRUSTED_REFERENCE_Y92"))
        );
        assert!(results.iter().any(|(_, s)| s == "undeclared_file"));
        assert_eq!(
            results
                .iter()
                .filter(|(name, s)| name == "read_declared" && s.contains("sentinel"))
                .count(),
            2
        );
        Reply::Tools(vec![tool(
            "yard_publish_report",
            json!({
                "outcome": "complete", "report": {"summary": "é".repeat(4001), "findings": []}
            }),
        )])
    });
    machine.start();
    let project = fixture(&machine);
    image(
        &project,
        "RUN rm /opt/yard/durable-audit/node_modules/@earendil-works/pi-durable/dist/index.js\n",
    );
    launch(&project, 1, "Packaged audit");
    let refused = ended(&project, "Y-1", "Missing packaged asset");
    assert!(refused["report"].is_null());
    assert!(controller_requests(&machine).is_empty());
    assert!(
        machine.model.requests().is_empty(),
        "missing asset must precede every model call"
    );
    let log = transcript(&project, 1);
    assert!(log.contains("runner_unavailable"));
    assert!(log.contains("pi-durable/dist/index.js"));
    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);

    image(&project, "");
    launch_with_source(&project, 2, "Declared traversal", "candidate/../large.txt");
    let traversal = ended(&project, "Y-2", "Declared traversal");
    assert!(traversal["report"].is_null());
    assert!(controller_requests(&machine).is_empty());
    let log = transcript(&project, 2);
    assert!(log.contains("invalid_input"));
    assert!(log.contains("declared file"));
    project.json(&["ticket", "park", "Y-2"]);
    project.json(&["attempt", "abandon", "Y-2"]);

    launch(&project, 3, "Corrected packaged audit");
    // A packaged node with topology cannot fall back to ordinary admission.
    // Sabotage: send packaged shapes through the ordinary linear matcher.
    let mut operator = machine.operator();
    operator.initialize();
    let topology = operator.call(
        "yard_plan_validate",
        json!({"project": project.path,
        "workflow": "plan", "overrides": {"implementer": {"routes": {"complete": []}}}}),
    );
    assert_eq!(topology["error"]["code"], "refused");
    assert_eq!(topology["error"]["data"]["shape"], "custom-graph");
    let expected_sources = json!([
        {"path": "trusted/selected.md", "blob": project.git(&["rev-parse", "HEAD:trusted/selected.md"]).trim(), "executable": false},
        {"path": "trusted/reference.txt", "blob": project.git(&["rev-parse", "HEAD:trusted/reference.txt"]).trim(), "executable": false}
    ]);
    for name in ["said-3", "release-3"] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(machine.root.join(name))
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(machine.root.join("hold-completed-3"), "hold").unwrap();
    let sentinel = machine.root.join("host-sentinel");
    std::fs::create_dir(&sentinel).unwrap();
    std::fs::write(sentinel.join("value"), "host-fixture").unwrap();
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    selected_start(&project, "Corrected packaged asset");
    assert_eq!(watch.said(&machine.root.join("said-3")), "held\n");
    let held = project.json(&["attempt", "show", "Y-3"]);
    assert_eq!(held["attempt"]["admission"]["runtime"], "packaged-audit");
    assert_eq!(
        held["attempt"]["admission"]["inputs"]["trusted_staged_sources"],
        expected_sources
    );
    assert_eq!(held["executions"][0]["report"]["report"], expected_report());
    assert_eq!(held["executions"][0]["result_eligible"], false);
    assert_eq!(held["executions"][0]["reserved"], true);
    assert!(held["executions"][0]["result"].is_null());
    let owner = project
        .rows("SELECT id, handle FROM execution WHERE attempt = 3 AND kind = 'implementation'")[0]
        .clone();
    let old = owner["id"].as_i64().unwrap();
    // The real child has created its checkpoint directory. Plant through
    // the live box, without editing any daemon-owned launch input.
    let planted = machine.pinfold(&[
        "box", "exec", owner["handle"].as_str().unwrap(), "--", "sh", "-c",
        &format!("echo checkpoint-fixture > /yard/state/audit-checkpoints/owned && ln -s '{}' /yard/state/audit-checkpoints/host-link", sentinel.display()),
    ]);
    assert!(
        planted.status.success(),
        "{}",
        String::from_utf8_lossy(&planted.stderr)
    );
    let checkpoint = project.path.join(format!(
        ".yard/local/attempts/3/audits/{old}/state/audit-checkpoints/owned"
    ));
    assert_eq!(
        std::fs::read_to_string(&checkpoint).unwrap(),
        "checkpoint-fixture\n"
    );
    // Later canonical and candidate instructions differ from the admitted
    // blobs. The edited execution must still load the original selection.
    project.write("trusted/selected.md", "LATER_SELECTED_SKILL\n");
    project.write("trusted/reference.txt", "LATER_REFERENCE\n");
    project.write("candidate/trusted/selected.md", "LATER_CANDIDATE_SKILL\n");
    project.git(&["add", "-A"]);
    project.git(&[
        "commit",
        "-qm",
        "Change canonical and candidate instructions after admission",
    ]);
    project.json(&["sync"]);
    project.json(&[
        "ticket",
        "edit",
        "Y-3",
        "--revision",
        "1",
        "--body",
        "Fresh replacement audit",
    ]);
    std::fs::write(machine.root.join("release-3"), "go\n").unwrap();
    second.wait_held();
    assert_eq!(
        std::fs::read_to_string(&checkpoint).unwrap(),
        "checkpoint-fixture\n"
    );
    let fresh = std::fs::read_to_string(machine.root.join("fresh-3")).unwrap();
    assert_eq!(
        fresh
            .lines()
            .filter(|line| *line == "predecessor_visible=1")
            .count(),
        2,
        "{fresh}"
    );
    for status in fresh
        .lines()
        .filter_map(|line| line.strip_prefix("proof_write="))
    {
        assert_ne!(status, "0", "{fresh}");
    }
    assert!(fresh.contains("proof_write="), "{fresh}");
    assert_eq!(
        fresh
            .lines()
            .filter(|line| *line == "proof_present=0")
            .count(),
        2,
        "{fresh}"
    );
    let rows = project
        .rows("SELECT resumed, node FROM execution WHERE attempt = 3 AND kind = 'implementation'");
    assert_eq!(
        rows,
        vec![json!({"resumed": null, "node": "implementer"}); 2]
    );
    second.release();
    watch.until("replacement audit ended", |event| {
        event["event"] == "attempt.ended" && event["ticket"] == "Y-3"
    });
    watch.until("audit state cleaned", |event| {
        event["event"] == "execution.ended"
            && event["attempt"] == 3
            && event["data"]["kind"] == "cleanup"
    });
    assert_eq!(
        std::fs::read_to_string(sentinel.join("value")).unwrap(),
        "host-fixture"
    );
    let completed = project.json(&["attempt", "show", "Y-3"])["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == "implementation" && row["execution"] != old)
        .unwrap()
        .clone();
    assert_eq!(completed["teardown"], true);
    assert_eq!(completed["report"]["report"], expected_report());
    let native = project.rows(&format!(
        "SELECT session_id, mcp FROM execution WHERE id = {}",
        completed["execution"]
    ));
    assert!(
        native[0]["session_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert_eq!(native[0]["mcp"], "registered");
    assert_eq!(completed["result_eligible"], true);

    // G8: a selected-input refusal creates no work and cannot block other
    // admissions or cleanup. Sabotage: propagate inputs()'s refusal from
    // scheduled(), or reserve a lane before validating selected inputs.
    // Keep higher-priority malformed tickets ready while an independent valid
    // selection publishes and cleans up in this same project. Earlier package
    // refusals happen inside an admitted execution, not at this boundary.
    // Fixture paths/byte limits and host Git blobs are the external oracles.
    project.write(
        "trusted/selected.md",
        "TRUSTED_SELECTED_SKILL_Y92: inspect declared files and publish.\n",
    );
    project.write("trusted/reference.txt", "TRUSTED_REFERENCE_Y92\n");
    std::os::unix::fs::symlink("selected.md", project.path.join("trusted/not-regular.md")).unwrap();
    project.write("trusted/oversized.txt", &"x".repeat(1024 * 1024 + 1));
    project.git(&["add", "trusted"]);
    project.git(&["commit", "-qm", "Canonical malformed selected inputs"]);
    project.json(&["sync"]);
    launch(&project, 4, "Independent valid audit");
    let mut selected = std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
    let profile = selected
        .split_once("[profiles.durable]")
        .unwrap()
        .1
        .to_string();
    for (name, from, to) in [
        ("linked", "trusted/selected.md", "trusted/not-regular.md"),
        (
            "oversized",
            "trusted/reference.txt",
            "trusted/oversized.txt",
        ),
    ] {
        selected.push_str(&format!(
            "\n[workflows.{name}]\naccess = \"read-only\"\nreview = \"none\"\nprofile = \"{name}\"\n\n[profiles.{name}]{}",
            profile.replace("profiles.durable", &format!("profiles.{name}")).replace(from, to)
        ));
    }
    project.reconfigure(&selected);
    for (ticket, workflow, path, reason) in [
        (
            "Y-4",
            "linked",
            "trusted/not-regular.md",
            "selected_regular_file",
        ),
        (
            "Y-5",
            "oversized",
            "trusted/oversized.txt",
            "selected_input_bound",
        ),
    ] {
        project.json(&[
            "ticket",
            "new",
            "--title",
            "Malformed selected input",
            "--workflow",
            workflow,
            "--priority",
            "P0",
        ]);
        let refusal = machine.refused(&project.path, &["attempt", "start", ticket]);
        assert_eq!(refusal["code"], "refused");
        assert_eq!(refusal["message"], "invalid_input");
        assert_eq!(refusal["data"], json!({"path": path, "reason": reason}));
    }
    let expected_sources = json!([
        {"path": "trusted/selected.md", "blob": project.git(&["rev-parse", "HEAD:trusted/selected.md"]).trim(), "executable": false},
        {"path": "trusted/reference.txt", "blob": project.git(&["rev-parse", "HEAD:trusted/reference.txt"]).trim(), "executable": false}
    ]);
    let mut independent = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    let valid = ended(&project, "Y-6", "Independent valid audit");
    assert_eq!(valid["report"]["report"], expected_report());
    assert_eq!(valid["result_eligible"], true);
    independent.until(
        "independent valid audit cleaned with malformed tickets ready",
        |event| {
            event["event"] == "execution.ended"
                && event["attempt"] == 4
                && event["data"]["kind"] == "cleanup"
        },
    );
    assert_eq!(
        project.json(&["attempt", "show", "Y-6"])["attempt"]["admission"]["inputs"]["trusted_staged_sources"],
        expected_sources
    );
    for ticket in ["Y-4", "Y-5"] {
        let shown = project.json(&["ticket", "show", ticket]);
        assert_eq!(shown["state"], "open");
        assert!(shown["attempt"].is_null());
    }
    assert!(
        project
            .rows("SELECT id FROM attempt WHERE ticket IN (4, 5)")
            .is_empty()
    );
    assert!(project.rows("SELECT e.id FROM execution e JOIN attempt a ON a.id = e.attempt WHERE a.ticket IN (4, 5)").is_empty());
    // Admission and cleanup observations are complete. Close these tickets
    // before the next fixture config removes the workflows they name.
    for ticket in ["Y-4", "Y-5"] {
        project.json(&[
            "ticket",
            "abandon",
            ticket,
            "--reason",
            "Malformed-input admission and independent cleanup observed",
        ]);
    }
    // G8 admitted asset integrity: the image owns a changed controller whose
    // launcher seal is valid. Sabotage: remove runAudit's admitted hash loop.
    // The missing-module block still refuses through seal/import without that
    // loop. Here the real package would execute its marker and call the model.
    // Expected bytes are the fixture prefix plus repository package source;
    // the real boxed seal, marker probe and host request count are observers.
    const PREFIX: &str = "import { writeFileSync } from \"node:fs\"; writeFileSync(\"/yard/state/package-executed\", \"image-fixture\");";
    let original = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runners/durable-audit/runner.mjs"),
    )
    .unwrap();
    image(
        &project,
        &format!(
            "RUN cd /opt/yard/durable-audit && {{ printf '%s\\n' '{PREFIX}'; cat runner.mjs; }} > changed.mjs && mv changed.mjs runner.mjs && node/bin/node --no-addons seal.mjs && sha256sum --quiet --strict -c identity.sha256\n"
        ),
    );
    let before = machine.model.requests().len();
    launch(&project, 5, "Changed sealed controller");
    let changed = ended(&project, "Y-7", "Changed sealed controller");
    assert_eq!(
        std::fs::read_to_string(machine.root.join("asset-5")).unwrap(),
        format!("{PREFIX}\n{original}")
    );
    assert!(
        std::fs::read_to_string(machine.root.join("fresh-5"))
            .unwrap()
            .lines()
            .any(|line| line == "package_seal=0")
    );
    assert!(
        std::fs::read_to_string(machine.root.join("probe-5"))
            .unwrap()
            .lines()
            .any(|line| line == "package_executed=1")
    );
    assert_eq!(machine.model.requests().len(), before);
    assert!(changed["report"].is_null());
    assert_eq!(changed["result_eligible"], false);
    assert_eq!(changed["result"]["outputs"], json!({}));
    assert!(changed["result"]["outcome"].is_null());
    let terminal = std::fs::read_to_string(machine.root.join("stdout-5"))
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|frame| frame["type"] == "yard_audit_terminal")
        .unwrap();
    assert_eq!(terminal["exit_code"], 1);
    assert_eq!(
        terminal["detail"],
        "runner_unavailable:/opt/yard/durable-audit/runner.mjs"
    );
    project.json(&["ticket", "park", "Y-7"]);
    project.json(&["attempt", "abandon", "Y-7"]);

    image(&project, "");
    launch(&project, 6, "Unchanged sealed controller control");
    let corrected = ended(&project, "Y-8", "Unchanged sealed controller control");
    assert_eq!(
        std::fs::read_to_string(machine.root.join("asset-6")).unwrap(),
        original
    );
    assert!(
        std::fs::read_to_string(machine.root.join("fresh-6"))
            .unwrap()
            .lines()
            .any(|line| line == "package_seal=0")
    );
    assert!(machine.model.requests().len() > before);
    assert_eq!(corrected["report"]["report"], expected_report());
    assert_eq!(corrected["result_eligible"], true, "{corrected}");
    let requests = controller_requests(&machine);
    for request in requests {
        assert_eq!(
            request.header("authorization"),
            Some(format!("Bearer {SECRET}").as_str())
        );
        assert!(!request.body.to_string().contains(SECRET));
    }
    let probe = std::fs::read_to_string(machine.root.join("probe-3")).unwrap();
    for token in [
        "upstream_in_env=1",
        "upstream_in_files=1",
        "sentinel=1",
        "unselected_guidance=1",
        "unselected_extension=1",
        "unselected_skill=1",
        "candidate_skill=1",
        "scratch_write=0",
        "checkpoint_write=0",
    ] {
        assert!(probe.contains(token), "{probe}");
    }
    for key in ["source_write", "trusted_write"] {
        let status = probe
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .unwrap();
        assert_ne!(status, "0", "{probe}");
    }
    assert!(!transcript(&project, 3).contains(SECRET));
    assert!(
        !serde_json::to_string(&project.rows("SELECT * FROM execution"))
            .unwrap()
            .contains(SECRET)
    );
    assert!(
        !serde_json::to_string(&project.rows("SELECT * FROM attempt"))
            .unwrap()
            .contains(SECRET)
    );
    assert!(
        !serde_json::to_string(&project.rows("SELECT * FROM audit"))
            .unwrap()
            .contains(SECRET)
    );
    assert_eq!(
        machine.model.requests().len(),
        controller_requests(&machine).len()
    );
}

/// G8: a model repeatedly asks to read instead of publishing. The packaged
/// controller stops at the spec's 12-request bound, not a timeout. A fresh
/// execution in the same project publishes as its positive control.
/// Sabotage: remove the request bound, synthesize a report at terminal text,
/// or route the selected terminal frame through Pi's stdout handling.
/// Existing publication tests exercise neither internal Durable iterations
/// nor this controller's bare-answer detection. Fixture supplies all replies.
#[test]
fn a_packaged_audit_bounds_iterations_and_requires_native_publication() {
    let stop = Latch::new();
    let held_stop = stop.clone();
    let stop_requests = std::sync::atomic::AtomicUsize::new(0);
    let machine = Machine::new("g8-durable-bound", move |request| {
        assert!(
            request.has_tool("read_declared"),
            "outer Pi made a model request"
        );
        if request.prompt().contains("Stop selected controller")
            && stop_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0
        {
            return Reply::Hold(
                held_stop.clone(),
                Box::new(Reply::Tools(vec![audit_report()])),
            );
        }
        if request.prompt().contains("Endless") {
            Reply::Tools(vec![tool(
                "read_declared",
                json!({"file": "source:large.txt"}),
            )])
        } else if request.prompt().contains("Bare") {
            Reply::Text("This is not a report.".into())
        } else {
            Reply::Tools(vec![audit_report()])
        }
    });
    machine.start();
    let project = fixture(&machine);
    image(&project, "");
    for (attempt, prompt, reason, count) in [
        (1, "Endless", "audit_limit", Some(12)),
        (2, "Bare", "missing_report", None),
    ] {
        let before = controller_requests(&machine).len();
        launch(&project, attempt, prompt);
        let ticket = format!("Y-{attempt}");
        let execution = ended(&project, &ticket, prompt);
        assert!(execution["report"].is_null());
        assert_eq!(execution["result_eligible"], false);
        assert_eq!(execution["result"]["outputs"], json!({}));
        assert!(execution["result"]["outcome"].is_null());
        if let Some(count) = count {
            assert_eq!(controller_requests(&machine).len() - before, count);
        }
        assert!(transcript(&project, attempt).contains(reason));
        project.json(&["ticket", "park", &ticket]);
        project.json(&["attempt", "abandon", &ticket]);
    }
    // G8/G13: stop the selected controller while its own host-route reply is
    // held, with real box-up stdin EOF held externally during teardown.
    // Sabotage: release before controller/box disappearance, admit a successor
    // while reserved, or cancel only the wait. Bounds and outer-Pi holds do
    // not reach this boundary. /proc, pinfold and history are external oracles.
    launch(&project, 3, "Stop selected controller");
    for name in [
        "down-said-3",
        "down-release-3",
        "said-4",
        "release-4",
        "listing-said",
        "listing-release",
    ] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(machine.root.join(name))
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(machine.root.join("hold-down-3"), "hold").unwrap();
    let target_head = project.git(&["rev-parse", "HEAD"]).trim().to_owned();
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    selected_start(&project, "Stop selected controller");
    stop.wait_held();
    let before = project.json(&["attempt", "show", "Y-3"]);
    let old = before["executions"][0].clone();
    let id = old["execution"].as_i64().unwrap();
    let handle =
        project.rows(&format!("SELECT handle FROM execution WHERE id = {id}"))[0]["handle"]
            .as_str()
            .unwrap()
            .to_owned();
    let process = machine.pinfold(&["box", "exec", &handle, "--", "sh", "-c",
        r#"for p in /proc/[0-9]*/cmdline; do command=$(tr '\000' ' ' < "$p"); case "$command" in 'node/bin/node --no-addons runner.mjs '*) pid=${p#/proc/}; pid=${pid%/cmdline}; kill -0 "$pid" && echo "$pid";; esac; done"#]);
    assert!(process.status.success());
    let pid = String::from_utf8(process.stdout).unwrap();
    assert_eq!(pid.lines().count(), 1, "{pid}");
    assert_eq!(old["reserved"], true);
    assert_eq!(old["binding"]["node"], "implementer");
    assert_eq!(before["attempt"]["target"], "main");
    assert_eq!(old["base"], target_head);
    assert!(old["report"].is_null());
    assert_eq!(project.json(&["attempt", "stop", "Y-3"])["stopping"], id);
    assert_eq!(watch.said(&machine.root.join("down-said-3")), "held\n");
    assert!(
        machine
            .boxes("dev.yard.project")
            .iter()
            .any(|b| b["name"] == handle)
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT reserved, teardown FROM execution WHERE id = {id}"
        )),
        vec![json!({"reserved": 1, "teardown": 0})]
    );
    assert!(
        project
            .rows(&format!(
                "SELECT id FROM execution WHERE attempt = 3 AND id > {id}"
            ))
            .is_empty()
    );
    std::fs::write(machine.root.join("down-release-3"), "go\n").unwrap();
    let stopped = watch.until("selected controller stopped", |event| {
        event["event"] == "attention.raised" && event["execution"] == id
    });
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    let released = watch.until("selected stop checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == id
    });
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|b| b["name"] == handle)
    );
    let gone = machine.pinfold(&[
        "box",
        "exec",
        &handle,
        "--",
        "sh",
        "-c",
        &format!("kill -0 {}", pid.trim()),
    ]);
    assert!(
        gone.status.code() == Some(3),
        "pinfold must report the selected controller's box absent: {gone:?}"
    );
    let cancelled = project.json(&["attempt", "show", "Y-3"])["executions"][0].clone();
    assert_eq!(cancelled["teardown"], true);
    assert_eq!(cancelled["result_eligible"], false);
    assert!(cancelled["report"].is_null());
    assert_eq!(cancelled["result"]["outputs"], json!({}));
    assert!(cancelled["result"]["outcome"].is_null());
    assert_eq!(cancelled["binding"], old["binding"]);
    let old_lock = std::fs::OpenOptions::new()
        .read(true)
        .open(
            project
                .path
                .join(format!(".yard/local/attempts/3/effects/{id}/lock")),
        )
        .unwrap();
    old_lock.try_lock().unwrap();
    old_lock.unlock().unwrap();
    stop.release();

    // G8: the subsequent selected execution publishes, then a real host lock
    // refuses ownership despite actual box absence. Sabotage: count native
    // publication/terminal success before checked ownership, or start cleanup
    // while reserved. Generic seats do not cross Durable's native terminal
    // frame. The fixture report and same-inode host lock supply the oracles.
    launch(&project, 4, "Publish control");
    std::fs::write(machine.root.join("hold-completed-4"), "hold").unwrap();
    project.json(&["attempt", "start", "Y-3"]);
    assert_eq!(watch.said(&machine.root.join("said-4")), "held\n");
    let started = watch.find("subsequent selected execution started", |event| {
        event["event"] == "execution.started"
            && event["attempt"] == 3
            && event["execution"] != id
            && event["data"]["kind"] == "implementation"
    });
    assert!(started["seq"].as_i64().unwrap() > released["seq"].as_i64().unwrap());
    let shown = project.json(&["attempt", "show", "Y-3"]);
    let published = shown["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["execution"] == started["execution"])
        .unwrap()
        .clone();
    let current = published["execution"].as_i64().unwrap();
    let publication_box = project.rows(&format!(
        "SELECT handle FROM execution WHERE id = {current}"
    ))[0]["handle"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(published["binding"], old["binding"]);
    assert_eq!(
        shown["attempt"]["admission"],
        before["attempt"]["admission"]
    );
    assert_eq!(published["report"]["report"], expected_report());
    assert_eq!(published["base"], target_head);
    let lock_path = project
        .path
        .join(format!(".yard/local/attempts/3/effects/{current}/lock"));
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .open(&lock_path)
        .unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    std::fs::write(machine.root.join("hold-listing"), "hold").unwrap();
    std::fs::write(machine.root.join("release-4"), "go\n").unwrap();
    assert_eq!(watch.said(&machine.root.join("listing-said")), "0\n");
    // The work future dropped its lock before issuing this real listing.
    use std::os::unix::fs::MetadataExt;
    let inherited = lock.metadata().unwrap();
    let same = std::fs::metadata(&lock_path).unwrap();
    assert_eq!((inherited.dev(), inherited.ino()), (same.dev(), same.ino()));
    lock.try_lock().unwrap();
    assert!(
        !machine
            .boxes("dev.yard.project")
            .iter()
            .any(|b| b["name"] == publication_box)
    );
    std::fs::write(machine.root.join("listing-release"), "go\n").unwrap();
    let ownership = watch.until("selected publication ownership refused", |event| {
        event["event"] == "attention.raised"
            && event["execution"] == current
            && event["data"]["reason"] == "ownership"
    });
    assert_eq!(ownership["attempt"], 3);
    assert_eq!(ownership["data"]["kind"], "red");
    let retained = project.json(&["attempt", "show", "Y-3"]);
    let retained = retained["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["execution"] == current)
        .unwrap()
        .clone();
    assert_eq!(retained["reserved"], true);
    assert_eq!(retained["teardown"], false);
    assert_eq!(retained["result_eligible"], false);
    assert_eq!(retained["report"], published["report"]);
    assert_eq!(retained["result"]["infrastructure"]["outcome"], "completed");
    assert_eq!(
        retained["result"]["outputs"]["result"],
        json!({"kind": "report", "value": published["report"]})
    );
    assert!(
        project
            .rows(&format!(
                "SELECT id FROM execution WHERE attempt = 3 AND id > {current}"
            ))
            .is_empty()
    );
    // The next real listing holds the reaper after the refused checked probe.
    assert_eq!(said(&machine.root.join("listing-said")), "0\n");
    lock.unlock().unwrap();
    std::fs::remove_file(machine.root.join("hold-listing")).unwrap();
    std::fs::write(machine.root.join("listing-release"), "go\n").unwrap();
    let checked = watch.until("selected publication checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == current
    });
    let cleanup = watch.until("selected continuation cleaned", |event| {
        event["event"] == "execution.started"
            && event["attempt"] == 3
            && event["data"]["kind"] == "cleanup"
    });
    assert!(cleanup["seq"].as_i64().unwrap() > checked["seq"].as_i64().unwrap());
    let completed = project.json(&["attempt", "show", "Y-3"]);
    let control = completed["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["execution"] == current)
        .unwrap();
    assert_eq!(control["teardown"], true);
    assert_eq!(control["reserved"], false);
    assert_eq!(control["report"], published["report"]);
    assert_eq!(control["binding"], published["binding"]);
    assert_eq!(control["result"], retained["result"]);
    assert_eq!(control["result_eligible"], true, "{control}");
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE attempt = 3 AND kind = 'implementation'")
            .len(),
        2
    );
    assert_eq!(
        machine.model.requests().len(),
        controller_requests(&machine).len()
    );
}
