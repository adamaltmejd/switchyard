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

fn packaged_daemon_source(name: &str, revision: &str, runner_suffix: &str) -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = yard().parent().unwrap().join(name);
    let source = target.join("source");
    std::fs::create_dir_all(&source).unwrap();
    // Build real runtime assets; the optional comment changes sealed bytes,
    // not the controller's launch behavior or an admission field.
    let archive = std::process::Command::new("git")
        .current_dir(root)
        .args([
            "archive",
            revision,
            "Cargo.toml",
            "Cargo.lock",
            "crates",
            "runners",
            ".agents/skills",
        ])
        .output()
        .unwrap();
    assert!(
        archive.status.success(),
        "{}",
        String::from_utf8_lossy(&archive.stderr)
    );
    let path = target.join("source.tar");
    std::fs::write(&path, archive.stdout).unwrap();
    assert!(
        std::process::Command::new("tar")
            .arg("-xf")
            .arg(path)
            .arg("-C")
            .arg(&source)
            .status()
            .unwrap()
            .success()
    );
    if !runner_suffix.is_empty() {
        let runner = source.join("runners/durable-audit/runner.mjs");
        let bytes = std::fs::read_to_string(&runner).unwrap();
        std::fs::write(runner, format!("{bytes}{runner_suffix}")).unwrap();
    }
    assert!(
        std::process::Command::new(env!("CARGO"))
            .current_dir(&source)
            .args(["build", "-p", "yard", "--locked", "--target-dir"])
            .arg(&target)
            .status()
            .unwrap()
            .success()
    );
    source
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
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
        project.path.join("trusted/reference.txt"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
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
    project.write("image-input", "PACKAGED_IMAGE_INPUT\n");
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runners/durable-audit/Dockerfile"),
    )
    .unwrap()
    .replace("\"$TARGETARCH\"", "\"$(uname -m)\"");
    project.write(
        ".yard/Dockerfile",
        &format!(
            "{source}\nCOPY image-input /admitted-image-input\nLABEL dev.yard.e2e-machine=\"{}\"\n{sabotage}",
            project.machine.root.display()
        ),
    );
    project.git(&["add", ".yard/Dockerfile", "image-input"]);
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

fn held_controller_request(watch: &mut Watch, ticket: &str, path: &Path) -> i64 {
    let started = watch.until("admitted planner started", |event| {
        event["event"] == "execution.started"
            && event["ticket"] == ticket
            && event["data"]["kind"] == "implementation"
    });
    let execution = started["execution"].as_i64().unwrap();
    assert_eq!(watch.said_before_end(path, Some(execution)), "request\n");
    execution
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
/// G6: omit separate input commits or executable bits: committed host Git
/// expectations fail. Build from another context or stage another blob: box
/// image bytes and host staged bytes fail. Sync and quiescent reopen must
/// preserve the original admission and completed report, without backfill.
/// Existing Pi report tests prove publication,
/// not this packaged runtime, its preflight, or its bounded read surface.
/// Declared candidate/../large.txt normalizes to a real fixture file, so
/// bypassing the declared '..' component guard reaches model requests.
/// Sabotage: remove that guard. The undeclared traversal tool argument below
/// exercises a different mechanism: allowlist lookup, not path validation.
/// G13: a real pinfold exec wrapper holds after Pi and its child have published
/// and exited, while their box still exists and the daemon awaits exec end.
/// Revise there; keep the checkpoint while the replacement controller's model
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
    let request_hold =
        std::sync::Arc::new(std::sync::Mutex::new(None::<(std::path::PathBuf, Latch)>));
    let model_hold = request_hold.clone();
    let second = Latch::new();
    let second_held = second.clone();
    let third = Latch::new();
    let third_held = third.clone();
    let fourth = Latch::new();
    let fourth_held = fourth.clone();
    let phase = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let model_phase = phase.clone();
    let assert_context = |request: &ModelRequest, staged: bool| {
        assert!(
            request.has_tool("read_declared"),
            "outer Pi made a model request"
        );
        assert!(request.context().contains(if staged {
            "ADOPTED_SELECTED_SKILL_Y141"
        } else {
            "TRUSTED_SELECTED_SKILL_Y92"
        }));
        if !staged {
            assert!(!request.context().contains("ADOPTED_SELECTED_SKILL_Y141"));
        }
        for marker in [
            "UNSELECTED_GUIDANCE",
            "UNSELECTED_SKILL",
            "CANDIDATE_SKILL",
            "CANDIDATE_LAUNCH",
        ] {
            assert!(!request.context().contains(marker));
        }
        assert!(!request.has_tool("bash"));
    };
    let machine = Machine::new("g8-durable-package", move |request| {
        let phase = model_phase.load(std::sync::atomic::Ordering::SeqCst);
        let staged = matches!(phase, 2 | 3);
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
            if let Some((path, latch)) = model_hold.lock().unwrap().take() {
                std::fs::write(path, "request\n").unwrap();
                return Reply::Hold(latch, Box::new(reply));
            }
            // Host phases choose holds before inspecting any tested input.
            let latch = match phase {
                1 => Some(&second_held),
                2 => Some(&third_held),
                3 => Some(&fourth_held),
                0 => None,
                _ => panic!("unexpected packaged fixture phase"),
            };
            if let Some(latch) = latch {
                return Reply::Hold(latch.clone(), Box::new(reply));
            }
            assert_context(request, staged);
            return reply;
        }
        assert_context(request, staged);
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
        assert!(results.iter().any(|(_, s)| s.contains(if staged {
            "ADOPTED_REFERENCE_Y141"
        } else {
            "TRUSTED_REFERENCE_Y92"
        })));
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
    project.json(&["ticket", "park", "Y-2"]);
    project.json(&["attempt", "abandon", "Y-2"]);

    launch(&project, 3, "Corrected packaged audit");
    let initial_configuration =
        std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
    project.reconfigure(&initial_configuration.replace(
        "model = \"fake-model\"",
        "model = \"fake-model\"\neffort = \"medium\"",
    ));
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
        {"path": "trusted/reference.txt", "blob": project.git(&["rev-parse", "HEAD:trusted/reference.txt"]).trim(), "executable": true}
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
    // G8: a valid controller reaches the host model before its execution ends.
    // Sabotage: require an unsupported launch field or reject valid launch data.
    // Observe the request itself and terminal history, not refusal prose or a
    // latch deadline. The fixture holds its reply until this observer releases it.
    let request_path = machine.root.join("controller-request");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&request_path)
            .status()
            .unwrap()
            .success()
    );
    let valid_hold = Latch::new();
    *request_hold.lock().unwrap() = Some((request_path.clone(), valid_hold.clone()));
    selected_start(&project, "Corrected packaged asset");
    held_controller_request(&mut watch, "Y-3", &request_path);
    valid_hold.wait_held();
    valid_hold.release();
    assert_eq!(watch.said(&machine.root.join("said-3")), "held\n");
    let initial_request = controller_requests(&machine)
        .into_iter()
        .find(ModelRequest::opens)
        .unwrap();
    assert_eq!(initial_request.body["model"], "fake-model");
    assert_eq!(initial_request.body["reasoning"]["effort"], "medium");
    assert!(
        initial_request
            .prompt()
            .starts_with("Corrected packaged audit\n")
    );
    assert_context(&initial_request, false);
    let held = project.json(&["attempt", "show", "Y-3"]);
    // G8: fallback must not hide missing admission launch-contract metadata.
    // Sabotage: omit package_launch in admit(); the spec supplies this token.
    assert_eq!(held["attempt"]["admission"]["package_launch"], "effort");
    let source = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let inputs = &held["attempt"]["admission"]["inputs"];
    for key in ["definition_commit", "staging_commit", "image_commit"] {
        assert_eq!(inputs[key], source);
    }
    for (key, suffix) in [
        ("config_blob", ":.yard/config.toml"),
        ("dockerfile_blob", ":.yard/Dockerfile"),
        ("build_context_tree", "^{tree}"),
    ] {
        assert_eq!(
            inputs[key],
            project
                .git(&["rev-parse", &format!("{source}{suffix}")])
                .trim()
        );
    }
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
    let image_bytes = machine.pinfold(&[
        "box",
        "exec",
        owner["handle"].as_str().unwrap(),
        "--",
        "cat",
        "/admitted-image-input",
    ]);
    assert!(image_bytes.status.success());
    assert_eq!(
        image_bytes.stdout,
        project
            .git(&["show", &format!("{source}:image-input")])
            .as_bytes()
    );
    for path in ["trusted/selected.md", "trusted/reference.txt"] {
        assert_eq!(
            std::fs::read(project.path.join(format!(
                ".yard/local/attempts/3/audits/{old}/input/trusted/{path}"
            )))
            .unwrap(),
            project
                .git(&["show", &format!("{source}:{path}")])
                .as_bytes()
        );
    }
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
    // G2/G6/G9: independent source adoption while a published predecessor
    // still owns its box. Sabotage: ignore a selected commit, adopt omitted
    // sources, read Attempt.implementer, drop CAS, replace package identity,
    // release the predecessor early, or omit audit.json's handoff. Host model
    // requests and Git blobs supply expectations. Ordinary controls cannot
    // catch the packaged controller's distinct launch and publication path.
    // Original controller settings, instructions or stale staged skill bytes
    // fail host request assertions after fixture-phase holds. Instructions
    // must open the prompt; their appearance in the handoff cannot qualify.
    // Dropping effort or launching predecessor effort with current model and
    // handoff fails the independently held provider-wire effort assertion.
    // Omitting the SDK's extended-level map clamps max and fails it too.
    let original_rows = project.rows(&format!("SELECT * FROM execution WHERE id = {old}"));
    let configuration = std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
    project.write(
        ".yard/config.toml",
        &configuration
            .replace(
                "model = \"fake-model\"",
                "model = \"packaged-adopted-model\"",
            )
            .replace("effort = \"medium\"", "effort = \"max\"")
            .replace("Corrected packaged audit", "Fresh replacement audit"),
    );
    project.git(&["add", ".yard/config.toml"]);
    project.git(&["commit", "-qm", "Select packaged controller settings"]);
    project.json(&["sync"]);
    let definition = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    assert_eq!(
        project.json(&["attempt", "show", "Y-3"])["attempt"]["plan_revision"],
        1
    );
    let adopted = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-3", "plan_revision": 1,
            "definition_commit": definition
        }),
    );
    assert_eq!(adopted["result"]["plan_revision"], 2, "{adopted}");
    // Sabotage: drop the recorded launch contract during definition adoption.
    assert_eq!(adopted["result"]["plan"]["package_launch"], "effort");
    assert_eq!(
        adopted["result"]["plan"]["inputs"]["staging_commit"],
        source
    );
    assert_eq!(adopted["result"]["plan"]["inputs"]["image_commit"], source);
    assert_eq!(
        adopted["result"]["plan"]["inputs"]["package"],
        inputs["package"]
    );
    let revisions = project.rows("SELECT * FROM plan_revision ORDER BY attempt, revision");
    let audit = project.rows("SELECT * FROM audit WHERE attempt = 3 ORDER BY seq");
    let stale = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-3", "plan_revision": 1,
            "definition_commit": definition
        }),
    );
    assert_eq!(stale["error"]["code"], "stale");
    assert_eq!(
        stale["error"]["data"],
        json!({"identity": "plan_revision", "expected": 1, "current": 2})
    );
    assert_eq!(
        project.rows("SELECT * FROM plan_revision ORDER BY attempt, revision"),
        revisions
    );
    assert_eq!(
        project.rows("SELECT * FROM audit WHERE attempt = 3 ORDER BY seq"),
        audit
    );
    assert_eq!(
        project.rows(&format!("SELECT * FROM execution WHERE id = {old}")),
        original_rows
    );
    project.write("trusted/selected.md", "ADOPTED_SELECTED_SKILL_Y141\n");
    project.write("trusted/reference.txt", "ADOPTED_REFERENCE_Y141\n");
    project.write("image-input", "LATER_IMAGE_INPUT\n");
    project.write("candidate/trusted/selected.md", "LATER_CANDIDATE_SKILL\n");
    project.git(&["add", "-A"]);
    project.git(&[
        "commit",
        "-qm",
        "Change canonical and candidate instructions after admission",
    ]);
    project.json(&["sync"]);
    let staging = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    phase.store(1, std::sync::atomic::Ordering::SeqCst);
    std::fs::write(machine.root.join("release-3"), "go\n").unwrap();
    second.wait_held();
    let settings_request = controller_requests(&machine).last().unwrap().clone();
    assert_eq!(settings_request.body["model"], "packaged-adopted-model");
    assert_eq!(settings_request.body["reasoning"]["effort"], "max");
    assert_context(&settings_request, false);
    assert!(
        settings_request
            .prompt()
            .starts_with("Fresh replacement audit\n")
    );
    assert!(
        controller_requests(&machine)
            .last()
            .unwrap()
            .prompt()
            .contains("packaged-adopted-model")
    );
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
    let settings_owner = project.rows("SELECT id, handle, model FROM execution WHERE attempt = 3 AND plan_revision = 2 AND kind = 'implementation'")[0].clone();
    assert_eq!(settings_owner["model"], "packaged-adopted-model");
    for path in ["trusted/selected.md", "trusted/reference.txt"] {
        assert_eq!(
            std::fs::read(project.path.join(format!(
                ".yard/local/attempts/3/audits/{}/input/trusted/{path}",
                settings_owner["id"]
            )))
            .unwrap(),
            project
                .git(&["show", &format!("{source}:{path}")])
                .as_bytes()
        );
    }
    let settings_image = machine.pinfold(&[
        "box",
        "exec",
        settings_owner["handle"].as_str().unwrap(),
        "--",
        "cat",
        "/admitted-image-input",
    ]);
    assert!(settings_image.status.success());
    assert_eq!(
        settings_image.stdout,
        project
            .git(&["show", &format!("{source}:image-input")])
            .as_bytes()
    );
    std::fs::write(machine.root.join("hold-completed-3"), "hold").unwrap();
    second.release();
    assert_eq!(watch.said(&machine.root.join("said-3")), "held\n");
    let staged = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-3", "plan_revision": 2,
            "staging_commit": staging
        }),
    );
    assert_eq!(staged["result"]["plan_revision"], 3, "{staged}");
    assert_eq!(staged["result"]["plan"]["package_launch"], "effort");
    phase.store(2, std::sync::atomic::Ordering::SeqCst);
    std::fs::write(machine.root.join("release-3"), "go\n").unwrap();
    third.wait_held();
    let staged_request = controller_requests(&machine).last().unwrap().clone();
    assert_eq!(staged_request.body["model"], "packaged-adopted-model");
    assert_eq!(staged_request.body["reasoning"]["effort"], "max");
    assert_context(&staged_request, true);
    assert!(
        staged_request
            .prompt()
            .starts_with("Fresh replacement audit\n")
    );
    let staged_request = staged_request.prompt();
    for commit in [&source, &staging] {
        let blob = project.git(&["rev-parse", &format!("{commit}:trusted/selected.md")]);
        assert!(staged_request.contains(blob.trim()), "{staged_request}");
    }
    let observe = |revision: i64, staged_commit: &str, image_commit: &str| {
        let row = project.rows(&format!(
            "SELECT id, handle FROM execution WHERE attempt = 3 AND plan_revision = {revision} AND kind = 'implementation'"
        ))[0].clone();
        let shown = project.json(&["attempt", "show", "Y-3"]);
        assert_eq!(shown["attempt"]["plan_revision"], revision);
        let recorded = &shown["attempt"]["plan"]["inputs"];
        assert_eq!(recorded["definition_commit"], definition);
        assert_eq!(recorded["staging_commit"], staged_commit);
        assert_eq!(recorded["image_commit"], image_commit);
        for path in ["trusted/selected.md", "trusted/reference.txt"] {
            assert_eq!(
                std::fs::read(project.path.join(format!(
                    ".yard/local/attempts/3/audits/{}/input/trusted/{path}",
                    row["id"]
                )))
                .unwrap(),
                project
                    .git(&["show", &format!("{staged_commit}:{path}")])
                    .as_bytes()
            );
        }
        let bytes = machine.pinfold(&[
            "box",
            "exec",
            row["handle"].as_str().unwrap(),
            "--",
            "cat",
            "/admitted-image-input",
        ]);
        assert!(bytes.status.success());
        assert_eq!(
            bytes.stdout,
            project
                .git(&["show", &format!("{image_commit}:image-input")])
                .as_bytes()
        );
        for (key, suffix) in [
            ("dockerfile_blob", ":.yard/Dockerfile"),
            ("build_context_tree", "^{tree}"),
        ] {
            assert_eq!(
                recorded[key],
                project
                    .git(&["rev-parse", &format!("{image_commit}{suffix}")])
                    .trim()
            );
        }
    };
    // Settings-only replacement consumes the original staged/image inputs.
    // Its startup rows remain immutable after independent staging adoption.
    let settings_row = project.rows("SELECT plan_revision, inputs, result FROM execution WHERE attempt = 3 AND plan_revision = 2 AND kind = 'implementation'");
    assert_eq!(settings_row[0]["plan_revision"], 2);
    observe(3, &staging, &source);
    let predecessor = project.rows(&format!(
        "SELECT plan_revision, inputs, report, result FROM execution WHERE id = {old}"
    ));
    assert_eq!(predecessor[0]["inputs"], original_rows[0]["inputs"]);
    assert_eq!(predecessor[0]["report"], original_rows[0]["report"]);
    project.write("image-input", "ADOPTED_IMAGE_INPUT_Y141\n");
    project.git(&["add", "image-input"]);
    project.git(&["commit", "-qm", "Select independent packaged image context"]);
    project.json(&["sync"]);
    let image_commit = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    std::fs::write(machine.root.join("hold-completed-3"), "hold").unwrap();
    third.release();
    assert_eq!(watch.said(&machine.root.join("said-3")), "held\n");
    let image_adopted = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-3", "plan_revision": 3,
            "image_commit": image_commit
        }),
    );
    assert_eq!(
        image_adopted["result"]["plan_revision"], 4,
        "{image_adopted}"
    );
    assert_eq!(image_adopted["result"]["plan"]["package_launch"], "effort");
    phase.store(3, std::sync::atomic::Ordering::SeqCst);
    std::fs::write(machine.root.join("release-3"), "go\n").unwrap();
    fourth.wait_held();
    let image_request = controller_requests(&machine).last().unwrap().clone();
    assert_eq!(image_request.body["model"], "packaged-adopted-model");
    assert_eq!(image_request.body["reasoning"]["effort"], "max");
    assert_context(&image_request, true);
    assert!(
        image_request
            .prompt()
            .starts_with("Fresh replacement audit\n")
    );
    let image_request = image_request.prompt();
    for commit in [&source, &image_commit] {
        let tree = project.git(&["rev-parse", &format!("{commit}^{{tree}}")]);
        assert!(image_request.contains(tree.trim()), "{image_request}");
    }
    observe(4, &staging, &image_commit);
    assert_eq!(project.rows("SELECT plan_revision, inputs, result FROM execution WHERE attempt = 3 AND plan_revision = 2 AND kind = 'implementation'"), settings_row);
    assert_eq!(
        project.rows(&format!(
            "SELECT plan_revision, inputs, report, result FROM execution WHERE id = {old}"
        )),
        predecessor
    );
    let shown = project.json(&["attempt", "show", "Y-3"]);
    assert_eq!(shown["executions"][0]["result_eligible"], false);
    assert_eq!(
        shown["attempt"]["plan"]["inputs"]["package"],
        inputs["package"]
    );
    fourth.release();
    watch.until("replacement audit ended", |event| {
        event["event"] == "attempt.ended" && event["ticket"] == "Y-3"
    });
    let cleanup = watch.until("audit state cleaned", |event| {
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
        .find(|row| row["kind"] == "implementation" && row["plan_revision"] == 4)
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
    watch.until("audit cleanup released", |event| {
        event["event"] == "execution.released" && event["execution"] == cleanup["execution"]
    });
    let evidence = project.json(&["attempt", "show", "Y-3"]);
    assert!(
        project
            .rows("SELECT id FROM execution WHERE status = 'running' OR reserved = 1")
            .is_empty()
    );
    assert_eq!(
        evidence["attempt"]["admission"],
        held["attempt"]["admission"]
    );
    machine.kill();
    machine.start();
    let reopened = project.json(&["attempt", "show", "Y-3"]);
    assert_eq!(
        reopened["attempt"]["admission"],
        held["attempt"]["admission"]
    );
    assert_eq!(reopened["executions"], evidence["executions"]);
    phase.store(0, std::sync::atomic::Ordering::SeqCst);

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
        {"path": "trusted/reference.txt", "blob": project.git(&["rev-parse", "HEAD:trusted/reference.txt"]).trim(), "executable": true}
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
    // G8: omitted effort still publishes through the real controller with
    // reasoning disabled. Sabotage: require effort in the launch schema,
    // inject null into the envelope, or enable reasoning for absent effort.
    launch(&project, 6, "Unchanged sealed controller control");
    let omitted_hold = Latch::new();
    *request_hold.lock().unwrap() = Some((request_path.clone(), omitted_hold.clone()));
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    selected_start(&project, "Unchanged sealed controller control");
    let omitted_execution = held_controller_request(&mut watch, "Y-8", &request_path);
    omitted_hold.wait_held();
    omitted_hold.release();
    watch.until("omitted-effort controller ended", |event| {
        event["event"] == "execution.ended" && event["execution"] == omitted_execution
    });
    watch.until("omitted-effort controller released", |event| {
        event["event"] == "execution.released" && event["execution"] == omitted_execution
    });
    let corrected = project.json(&["attempt", "show", "Y-8"])["executions"][0].clone();
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
    let omitted_request = controller_requests(&machine)
        .into_iter()
        .rev()
        .find(ModelRequest::opens)
        .unwrap();
    assert!(omitted_request.body.get("reasoning").is_none());
    assert!(omitted_request.body.get("reasoning_effort").is_none());
    // Cleanup terminal events name the attempt, not the ticket. Sabotage:
    // omit this attempt's cleanup completion; another attempt must not satisfy it.
    watch.until("corrected controller cleanup ended", |event| {
        event["event"] == "execution.ended"
            && event["attempt"] == corrected["attempt"]
            && event["data"]["kind"] == "cleanup"
    });

    // G8: a real old admission records medium, but its sealed controller has
    // no effort capability. Sabotage: add effort solely because settings name
    // it, relabel the old package, or require today's controller hashes.
    // The old daemon admits and stops a held real controller. Only after checked
    // release does the new daemon start its successor from that frozen admission.
    // Current-package controls cannot catch this six-field protocol failure.
    let old_source =
        packaged_daemon_source("pre-effort", "559f730724a89222a4e57868b4770a2a4da65485", "");
    machine.kill();
    machine.use_binary(Some(old_source.parent().unwrap().join("debug/yard")));
    machine.start();
    let old_runner =
        std::fs::read_to_string(old_source.join("runners/durable-audit/runner.mjs")).unwrap();
    project.write("runners/durable-audit/runner.mjs", &old_runner);
    project.git(&["add", "runners/durable-audit/runner.mjs"]);
    project.git(&["commit", "-qm", "Freeze supplied six-field controller"]);
    project.json(&["sync"]);
    launch(&project, 7, "Frozen old controller");
    let old_config = std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
    project.reconfigure(&old_config.replace(
        "model = \"fake-model\"",
        "model = \"fake-model\"\neffort = \"medium\"",
    ));
    let old_hold = Latch::new();
    *request_hold.lock().unwrap() = Some((request_path.clone(), old_hold.clone()));
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    selected_start(&project, "Old controller with declared effort");
    let old_execution = held_controller_request(&mut watch, "Y-9", &request_path);
    old_hold.wait_held();
    let old_admission = project.json(&["attempt", "show", "Y-9"])["attempt"]["admission"].clone();
    assert_eq!(
        old_admission["agents"]["worker"]["settings"]["effort"],
        "medium"
    );
    assert_eq!(
        std::fs::read_to_string(machine.root.join("asset-7")).unwrap(),
        old_runner
    );
    project.json(&["attempt", "stop", "Y-9"]);
    watch.until("old controller stopped", |event| {
        event["event"] == "execution.ended" && event["execution"] == old_execution
    });
    watch.until("old controller checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == old_execution
    });
    old_hold.release();
    let old_rows = project.rows(&format!(
        "SELECT * FROM execution WHERE id = {old_execution}"
    ));
    machine.kill();
    machine.use_binary(None);
    machine.start();
    let successor_hold = Latch::new();
    *request_hold.lock().unwrap() = Some((request_path.clone(), successor_hold.clone()));
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-9"]);
    let successor = held_controller_request(&mut watch, "Y-9", &request_path);
    successor_hold.wait_held();
    let old_request = controller_requests(&machine).last().unwrap().clone();
    assert_eq!(old_request.body["model"], "fake-model");
    assert!(old_request.body.get("reasoning").is_none());
    assert!(old_request.body.get("reasoning_effort").is_none());

    // G2/G8: a changed explicit effort cannot be recorded and silently omitted
    // by this six-field controller. Sabotage: remove the adoption capability
    // guard while keeping the launch guard. The original medium startup above
    // still succeeds, so it cannot detect this new unsupported control request.
    // Committed max/omitted fixture settings supply expectations. Hold the real
    // successor request throughout refusal so its ownership cannot race writes.
    let historical_config =
        std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
    project.reconfigure(&historical_config.replace("effort = \"medium\"", "effort = \"max\""));
    let unsupported_definition = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let attempt = project.json(&["attempt", "show", "Y-9"])["attempt"]["attempt"]
        .as_i64()
        .unwrap();
    let recorded_rows = || {
        [
            ("attempt", "id"),
            ("plan_revision", "attempt"),
            ("execution", "attempt"),
            ("attention", "attempt"),
            ("approval", "attempt"),
            ("check", "execution"),
            ("audit", "attempt"),
        ]
        .map(|(table, key)| {
            let predicate = if table == "check" {
                format!("{key} IN (SELECT id FROM execution WHERE attempt = {attempt})")
            } else {
                format!("{key} = {attempt}")
            };
            project.rows(&format!(
                "SELECT * FROM \"{table}\" WHERE {predicate} ORDER BY rowid"
            ))
        })
    };
    let before_refusal = recorded_rows();
    let mut operator = machine.operator();
    operator.initialize();
    let unsupported = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-9", "plan_revision": 1,
            "definition_commit": unsupported_definition
        }),
    );
    assert_eq!(unsupported["error"]["code"], "refused");
    assert_eq!(
        unsupported["error"]["data"],
        json!({
            "reason": "unsupported_effort", "identity": "effort", "expected": "max", "current": "medium"
        })
    );
    assert_eq!(recorded_rows(), before_refusal);
    let frozen_successor = project.rows(&format!(
        "SELECT plan_revision, inputs, effort FROM execution WHERE id = {successor}"
    ));

    // Omission is a supported adjustment in this same project. Checked stop
    // and explicit start retain the old frozen startup and publish with the
    // adopted contract, without promising an effort control the package lacks.
    project.reconfigure(&historical_config.replace("\neffort = \"medium\"", ""));
    let supported_definition = project.git(&["rev-parse", "HEAD"]).trim().to_string();
    let supported = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-9", "plan_revision": 1,
            "definition_commit": supported_definition
        }),
    );
    assert_eq!(supported["result"]["plan_revision"], 2, "{supported}");
    project.json(&["attempt", "stop", "Y-9"]);
    watch.until("historical-effort successor stopped", |event| {
        event["event"] == "execution.ended" && event["execution"] == successor
    });
    watch.until("historical-effort successor released", |event| {
        event["event"] == "execution.released" && event["execution"] == successor
    });
    successor_hold.release();
    let successor_rows = project.rows(&format!("SELECT * FROM execution WHERE id = {successor}"));
    let omitted_old_hold = Latch::new();
    *request_hold.lock().unwrap() = Some((request_path.clone(), omitted_old_hold.clone()));
    project.json(&["attempt", "start", "Y-9"]);
    let productive = held_controller_request(&mut watch, "Y-9", &request_path);
    omitted_old_hold.wait_held();
    let omitted_old_request = controller_requests(&machine).last().unwrap().clone();
    assert_eq!(omitted_old_request.body["model"], "fake-model");
    assert!(omitted_old_request.body.get("reasoning").is_none());
    assert!(omitted_old_request.body.get("reasoning_effort").is_none());
    omitted_old_hold.release();
    watch.until("old package successor ended", |event| {
        event["event"] == "execution.ended" && event["execution"] == productive
    });
    watch.until("old package successor released", |event| {
        event["event"] == "execution.released" && event["execution"] == productive
    });
    let preserved = project.json(&["attempt", "show", "Y-9"]);
    assert_eq!(preserved["attempt"]["admission"], old_admission);
    let published = preserved["executions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["execution"] == productive)
        .unwrap();
    assert_eq!(published["report"]["report"], expected_report());
    assert_eq!(published["result_eligible"], true);
    assert_eq!(
        project.rows(&format!(
            "SELECT plan_revision, inputs, effort FROM execution WHERE id = {successor}"
        )),
        frozen_successor
    );
    assert_eq!(published["plan_revision"], 2);
    assert_eq!(
        preserved["attempt"]["plan"]["agents"]["worker"]["settings"]["effort"],
        Value::Null
    );
    assert_eq!(
        project.rows(&format!("SELECT * FROM execution WHERE id = {successor}")),
        successor_rows
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM execution WHERE id = {old_execution}"
        )),
        old_rows
    );
    watch.until("older omission control cleanup ended", |event| {
        event["event"] == "execution.ended"
            && event["attempt"] == attempt
            && event["data"]["kind"] == "cleanup"
    });

    // G8: independently exercise both capability paths across real daemon
    // changes. Sabotage: remove the metadata arm (different compiled package),
    // or remove the exact-package arm (real b9e374 admission without metadata).
    // Same-package new admissions mask either fault. Actual archived source,
    // a fixture comment compiled into a real sealed asset, and boxed asset
    // reads establish the package relation without forged store identities.
    const CAPABILITY_VARIANT: &str = "\n// Y136 recorded-launch capability fixture asset.\n";
    for (attempt_number, ticket, source, contract) in [
        (
            8,
            "Y-10",
            packaged_daemon_source("effort-metadata", "HEAD", CAPABILITY_VARIANT),
            Some("effort"),
        ),
        (
            9,
            "Y-11",
            packaged_daemon_source(
                "effort-fallback",
                "b9e374f59b2141edc3d2666f07796805c865764b",
                "",
            ),
            None,
        ),
    ] {
        let source_runner =
            std::fs::read_to_string(source.join("runners/durable-audit/runner.mjs")).unwrap();
        assert_eq!(
            source_runner,
            format!(
                "{original}{}",
                if contract.is_some() {
                    CAPABILITY_VARIANT
                } else {
                    ""
                }
            )
        );
        for asset in ["launch", "package-lock.json"] {
            assert_eq!(
                std::fs::read(source.join("runners/durable-audit").join(asset)).unwrap(),
                std::fs::read(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../runners/durable-audit")
                        .join(asset)
                )
                .unwrap()
            );
        }
        machine.kill();
        machine.use_binary(Some(source.parent().unwrap().join("debug/yard")));
        machine.start();
        project.write("runners/durable-audit/runner.mjs", &source_runner);
        project.git(&["add", "runners/durable-audit/runner.mjs"]);
        project.git(&[
            "commit",
            "-qm",
            "Select actual capability qualification runtime",
        ]);
        project.json(&["sync"]);
        launch(&project, attempt_number, "Preserved capable controller");
        let configuration =
            std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
        project.reconfigure(&configuration.replace(
            "model = \"fake-model\"",
            "model = \"fake-model\"\neffort = \"medium\"",
        ));
        let admitting_hold = Latch::new();
        *request_hold.lock().unwrap() = Some((request_path.clone(), admitting_hold.clone()));
        let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
        selected_start(&project, "Preserved capable controller");
        let admitting_execution = held_controller_request(&mut watch, ticket, &request_path);
        admitting_hold.wait_held();
        let admission = project.json(&["attempt", "show", ticket])["attempt"]["admission"].clone();
        assert_eq!(
            admission.get("package_launch").and_then(Value::as_str),
            contract
        );
        assert_eq!(
            std::fs::read_to_string(machine.root.join(format!("asset-{attempt_number}"))).unwrap(),
            source_runner
        );
        assert_eq!(
            controller_requests(&machine).last().unwrap().body["reasoning"]["effort"],
            "medium"
        );
        project.json(&["attempt", "stop", ticket]);
        watch.until("capable admitting execution stopped", |event| {
            event["event"] == "execution.ended" && event["execution"] == admitting_execution
        });
        watch.until("capable admitting execution released", |event| {
            event["event"] == "execution.released" && event["execution"] == admitting_execution
        });
        admitting_hold.release();
        let historical = project.rows(&format!(
            "SELECT * FROM execution WHERE id = {admitting_execution}"
        ));
        machine.kill();
        machine.use_binary(None);
        machine.start();
        let mut operator = machine.operator();
        operator.initialize();
        let receiving_selection = operator.call(
            "yard_plan_validate",
            json!({
                "project": project.path, "workflow": "plan",
                "overrides": {"implementer": {"profile": "durable"}}
            }),
        );
        assert_eq!(receiving_selection["result"]["shape"], "packaged-audit");
        assert_eq!(
            receiving_selection["result"]["bindings"]["selected_inputs"]["package"]
                == admission["inputs"]["package"],
            contract.is_none()
        );
        let receiving_hold = Latch::new();
        *request_hold.lock().unwrap() = Some((request_path.clone(), receiving_hold.clone()));
        let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
        project.json(&["attempt", "start", ticket]);
        let receiving_execution = held_controller_request(&mut watch, ticket, &request_path);
        receiving_hold.wait_held();
        assert_eq!(
            std::fs::read_to_string(machine.root.join(format!("asset-{attempt_number}"))).unwrap(),
            source_runner
        );
        // This provider-wire observation cannot be satisfied by the other arm.
        assert_eq!(
            controller_requests(&machine).last().unwrap().body["reasoning"]["effort"],
            "medium"
        );
        let frozen = project.rows(&format!(
            "SELECT plan_revision, inputs, effort FROM execution WHERE id = {receiving_execution}"
        ));
        let configuration =
            std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
        project.reconfigure(&configuration.replace("effort = \"medium\"", "effort = \"max\""));
        let definition = project.git(&["rev-parse", "HEAD"]).trim().to_string();
        let adopted = operator.call("yard_attempt_revise", json!({
            "project": project.path, "ticket": ticket, "plan_revision": 1, "definition_commit": definition
        }));
        assert_eq!(adopted["result"]["plan_revision"], 2, "{adopted}");
        assert_eq!(
            adopted["result"]["plan"]
                .get("package_launch")
                .and_then(Value::as_str),
            contract
        );
        project.json(&["attempt", "stop", ticket]);
        watch.until("capable receiving execution stopped", |event| {
            event["event"] == "execution.ended" && event["execution"] == receiving_execution
        });
        watch.until("capable receiving execution released", |event| {
            event["event"] == "execution.released" && event["execution"] == receiving_execution
        });
        receiving_hold.release();
        let publishing_hold = Latch::new();
        *request_hold.lock().unwrap() = Some((request_path.clone(), publishing_hold.clone()));
        project.json(&["attempt", "start", ticket]);
        let publishing_execution = held_controller_request(&mut watch, ticket, &request_path);
        publishing_hold.wait_held();
        assert_eq!(
            controller_requests(&machine).last().unwrap().body["reasoning"]["effort"],
            "max"
        );
        publishing_hold.release();
        watch.until("capable publication ended", |event| {
            event["event"] == "execution.ended" && event["execution"] == publishing_execution
        });
        watch.until("capable publication released", |event| {
            event["event"] == "execution.released" && event["execution"] == publishing_execution
        });
        let preserved = project.json(&["attempt", "show", ticket]);
        assert_eq!(preserved["attempt"]["admission"], admission);
        let publication = preserved["executions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["execution"] == publishing_execution)
            .unwrap();
        assert_eq!(publication["report"]["report"], expected_report());
        assert_eq!(publication["result_eligible"], true);
        assert_eq!(
            project.rows(&format!(
                "SELECT * FROM execution WHERE id = {admitting_execution}"
            )),
            historical
        );
        assert_eq!(project.rows(&format!("SELECT plan_revision, inputs, effort FROM execution WHERE id = {receiving_execution}")), frozen);
        watch.until("capable controller cleanup ended", |event| {
            event["event"] == "execution.ended"
                && event["attempt"] == publication["attempt"]
                && event["data"]["kind"] == "cleanup"
        });
    }
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
    // G4/G5/G8: restoring release's completed-only qualification leaves the
    // stopped owner's exact attention open after checked release. The later
    // publication is the same-project control; it cannot diagnose this item.
    // Adoption at this held boundary must preserve the old revision, result
    // and both questions. Sabotage: resolve ownership during revise or clear
    // stop during revise/release. Git and fixture instructions supply the
    // successor's expected definitions; running-adoption tests have no stop.
    // Latest-admission qualification and invented missing-revision proof
    // need independent historical controls: both admissions here are packaged.
    launch(&project, 3, "Stop selected controller");
    for name in [
        "down-said-3",
        "down-release-3",
        "lock-said-3",
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
    // G8: record the same launch contract before held stopped-owner adoption.
    assert_eq!(before["attempt"]["admission"]["package_launch"], "effort");
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
    let old_lock_path = project
        .path
        .join(format!(".yard/local/attempts/3/effects/{id}/lock"));
    let old_lock = std::fs::OpenOptions::new()
        .read(true)
        .open(&old_lock_path)
        .unwrap();
    assert!(matches!(
        old_lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    std::fs::write(machine.root.join("hold-listing"), "hold").unwrap();
    std::fs::write(machine.root.join("down-release-3"), "go\n").unwrap();
    let stopped = watch.until("selected controller stopped", |event| {
        event["event"] == "attention.raised" && event["execution"] == id
    });
    assert_eq!(stopped["data"]["kind"], "stopped");
    assert_eq!(stopped["data"]["reason"], "unchanged");
    assert_eq!(watch.said(&machine.root.join("listing-said")), "0\n");
    // A listing reply does not order the stopped streaming child's exit.
    // Keep the probe held while a native blocking lock takes ownership only
    // after every inherited descriptor releases this same inode.
    use std::os::unix::fs::MetadataExt;
    let inherited = old_lock.metadata().unwrap();
    let same = std::fs::metadata(&old_lock_path).unwrap();
    assert_eq!((inherited.dev(), inherited.ino()), (same.dev(), same.ino()));
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
    let lock_said = machine.root.join("lock-said-3");
    let owner = std::thread::spawn(move || {
        old_lock.lock().unwrap();
        std::fs::write(lock_said, "held\n").unwrap();
        old_lock
    });
    assert_eq!(watch.said(&machine.root.join("lock-said-3")), "held\n");
    let old_lock = owner.join().unwrap();
    let same = std::fs::metadata(&old_lock_path).unwrap();
    assert_eq!((inherited.dev(), inherited.ino()), (same.dev(), same.ino()));
    std::fs::write(machine.root.join("listing-release"), "go\n").unwrap();
    let refused = watch.until("selected stop ownership refused", |event| {
        event["event"] == "attention.raised"
            && event["execution"] == id
            && event["data"]["reason"] == "ownership"
    });
    assert_eq!(refused["data"]["kind"], "red");
    assert_eq!(
        project.rows(&format!(
            "SELECT reserved, teardown FROM execution WHERE id = {id}"
        )),
        vec![json!({"reserved": 1, "teardown": 0})]
    );
    // Hold the next checked probe until the verified same-inode owner leaves.
    assert_eq!(said(&machine.root.join("listing-said")), "0\n");
    let frozen_rows = project.rows(&format!("SELECT * FROM execution WHERE id = {id}"));
    let ownership = project.rows(&format!(
        "SELECT * FROM attention WHERE execution = {id} AND reason = 'ownership'"
    ));
    let stopped_item = project.rows(&format!(
        "SELECT * FROM attention WHERE execution = {id} AND kind = 'stopped'"
    ));
    assert_eq!(ownership.len(), 1);
    assert_eq!(ownership[0]["state"], "open");
    assert_eq!(stopped_item.len(), 1);
    assert_eq!(stopped_item[0]["state"], "open");
    let frozen_execution = project.json(&["attempt", "show", "Y-3"])["executions"][0].clone();
    assert_eq!(frozen_execution["plan_revision"], 1);
    let configuration = std::fs::read_to_string(project.path.join(".yard/config.toml")).unwrap();
    project.reconfigure(
        &configuration.replace("Stop selected controller", "Adopted stopped audit Y138"),
    );
    let definition = project.git(&["rev-parse", "HEAD"]).trim().to_owned();
    let mut operator = machine.operator();
    operator.initialize();
    let adopted = operator.call(
        "yard_attempt_revise",
        json!({
            "project": project.path, "ticket": "Y-3", "plan_revision": 1,
            "definition_commit": definition
        }),
    );
    assert_eq!(adopted["result"]["plan_revision"], 2, "{adopted}");
    assert_eq!(
        adopted["result"]["plan"]["inputs"]["definition_commit"],
        definition
    );
    // Sabotage: drop package_launch while adopting for a stopped owner.
    assert_eq!(adopted["result"]["plan"]["package_launch"], "effort");
    assert_eq!(
        project.rows(&format!("SELECT * FROM execution WHERE id = {id}")),
        frozen_rows
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM attention WHERE execution = {id} AND reason = 'ownership'"
        )),
        ownership
    );
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM attention WHERE execution = {id} AND kind = 'stopped'"
        )),
        stopped_item
    );
    old_lock.unlock().unwrap();
    old_lock.try_lock().unwrap();
    old_lock.unlock().unwrap();
    std::fs::remove_file(machine.root.join("hold-listing")).unwrap();
    std::fs::write(machine.root.join("listing-release"), "go\n").unwrap();
    let released = watch.until("selected stop checked release", |event| {
        event["event"] == "execution.released" && event["execution"] == id
    });
    // Observe this owner directly before a successor FIFO can time out.
    assert_eq!(
        project.rows(&format!(
            "SELECT state, resolution FROM attention WHERE execution = {id} AND reason = 'ownership'"
        )),
        vec![json!({"state": "resolved", "resolution": "reconciled"})]
    );
    let cancelled = project.json(&["attempt", "show", "Y-3"])["executions"][0].clone();
    assert_eq!(cancelled["teardown"], true);
    assert_eq!(cancelled["result_eligible"], false);
    assert!(cancelled["report"].is_null());
    assert_eq!(cancelled["result"]["outputs"], json!({}));
    assert!(cancelled["result"]["outcome"].is_null());
    assert_eq!(cancelled["binding"], old["binding"]);
    assert_eq!(cancelled["plan_revision"], 1);
    assert_eq!(cancelled["result"], frozen_execution["result"]);
    assert_eq!(cancelled["reserved"], false);
    assert_eq!(cancelled["outcome"], "unchanged");
    assert_eq!(
        project.rows(&format!(
            "SELECT * FROM attention WHERE execution = {id} AND kind = 'stopped'"
        )),
        stopped_item
    );
    assert_eq!(
        project
            .rows("SELECT id FROM execution WHERE attempt = 3")
            .len(),
        1
    );
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
    assert_eq!(published["plan_revision"], 2);
    let request = controller_requests(&machine).last().unwrap().clone();
    assert!(request.prompt().starts_with("Adopted stopped audit Y138\n"));
    assert_eq!(
        shown["attempt"]["plan"]["inputs"]["definition_commit"],
        definition
    );
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
