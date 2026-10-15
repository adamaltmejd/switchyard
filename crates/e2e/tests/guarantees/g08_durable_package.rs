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

fn checkpoint_daemon_source() -> std::path::PathBuf {
    let name = "checkpoint-debugger";
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = yard().parent().unwrap().join(name);
    let build_target = yard().parent().unwrap().join("packaged-daemon-build");
    let source = target.join("source");
    std::fs::create_dir_all(&source).unwrap();
    // The two checkpoint scenarios share one actual debugger-enabled build.
    let archive = std::process::Command::new("git")
        .current_dir(root)
        .args([
            "archive",
            "HEAD",
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
    {
        // Node's public debugger holds the real tool after its committed
        // intent. No harness replacement or binary barrier participates.
        let launch = source.join("runners/durable-audit/launch");
        let bytes = std::fs::read_to_string(&launch).unwrap();
        std::fs::write(
            launch,
            bytes.replace("--no-addons", "--no-addons --inspect=127.0.0.1:9229"),
        )
        .unwrap();
    }
    // Concurrent scenarios share dependencies. Hold clean/build/copy together:
    // Cargo's command locks do not protect the copied executable between calls.
    // Discard Yard artifacts so archived timestamps cannot reuse another seal.
    static BUILD: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _build = BUILD.lock().expect("packaged fixture build lock");
    assert!(
        std::process::Command::new(env!("CARGO"))
            .current_dir(&source)
            .args(["clean", "-p", "yard", "--target-dir"])
            .arg(&build_target)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new(env!("CARGO"))
            .current_dir(&source)
            .args(["build", "-p", "yard", "--locked", "--target-dir"])
            .arg(&build_target)
            .status()
            .unwrap()
            .success()
    );
    std::fs::create_dir_all(target.join("debug")).unwrap();
    std::fs::copy(build_target.join("debug/yard"), target.join("debug/yard")).unwrap();
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
"#, json!(prompt), json!("large.txt"))).replace("[workflows.plan]", "[workflows.plan]\nprofile = \"durable\"");
    project.reconfigure(&selected);
    project.machine.wrapper("pinfold", &format!(r#"
if [ "$1 $2" = 'image build' ]; then
    previous=''
    for arg do
        if [ "$previous" = '--context' ]; then cp -R '{root}/build-inputs' "$arg/build-inputs"; fi
        previous="$arg"
    done
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

fn controller_request_path(machine: &Machine) -> std::path::PathBuf {
    let path = machine.root.join("controller-request");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    path
}

fn assert_private_controller(project: &Project<'_>) {
    let machine = project.machine;
    for request in controller_requests(machine) {
        assert_eq!(
            request.header("authorization"),
            Some(format!("Bearer {SECRET}").as_str())
        );
        assert!(!request.body.to_string().contains(SECRET));
    }
    for table in ["execution", "attempt", "audit"] {
        assert!(
            !serde_json::to_string(&project.rows(&format!("SELECT * FROM {table}")))
                .unwrap()
                .contains(SECRET)
        );
    }
    assert_eq!(
        machine.model.requests().len(),
        controller_requests(machine).len()
    );
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

/// G8/G12/G13: a correctly sealed changed controller is refused before code
/// or model effects. The unmodified image in the same project publishes.
/// Sabotage: skip admitted hashes, load candidate code, leak host secrets,
/// grant arbitrary tools or make selected source/trusted mounts writable.
/// Fixture bytes, real boxed probes and host model requests are the oracles.
#[test]
fn a_packaged_audit_refuses_changed_assets_then_publishes_in_a_corrected_box() {
    let request_hold =
        std::sync::Arc::new(std::sync::Mutex::new(None::<(std::path::PathBuf, Latch)>));
    let model_hold = request_hold.clone();
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
            if let Some((path, latch)) = model_hold.lock().unwrap().take() {
                std::fs::write(path, "request\n").unwrap();
                return Reply::Hold(latch, Box::new(reply));
            }
            return reply;
        }
        let results = request.tool_results();
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
        Reply::Tools(vec![audit_report()])
    });
    machine.start();
    let project = fixture(&machine);
    let request_path = controller_request_path(&machine);
    // G8 admitted asset integrity: the image owns a changed controller whose
    // launcher seal is valid. Sabotage: remove runAudit's admitted hash loop.
    // Without admitted hashes the package could execute its marker and call the model.
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
    launch(&project, 1, "Changed sealed controller");
    let changed = ended(&project, "Y-1", "Changed sealed controller");
    assert_eq!(
        std::fs::read_to_string(machine.root.join("asset-1")).unwrap(),
        format!("{PREFIX}\n{original}")
    );
    assert!(
        std::fs::read_to_string(machine.root.join("fresh-1"))
            .unwrap()
            .lines()
            .any(|line| line == "package_seal=0")
    );
    assert!(
        std::fs::read_to_string(machine.root.join("probe-1"))
            .unwrap()
            .lines()
            .any(|line| line == "package_executed=1")
    );
    assert_eq!(machine.model.requests().len(), before);
    assert!(changed["report"].is_null());
    assert_eq!(changed["result_eligible"], false);
    assert_eq!(changed["result"]["outputs"], json!({}));
    assert!(changed["result"]["outcome"].is_null());
    let terminal = std::fs::read_to_string(machine.root.join("stdout-1"))
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
    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);

    image(&project, "");
    // G8: omitted effort still publishes through the real controller with
    // reasoning disabled. Sabotage: require effort in the launch schema,
    // inject null into the envelope, or enable reasoning for absent effort.
    launch(&project, 2, "Unchanged sealed controller control");
    let omitted_hold = Latch::new();
    *request_hold.lock().unwrap() = Some((request_path.clone(), omitted_hold.clone()));
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    selected_start(&project, "Unchanged sealed controller control");
    let omitted_execution = held_controller_request(&mut watch, "Y-2", &request_path);
    omitted_hold.wait_held();
    omitted_hold.release();
    watch.until("omitted-effort controller ended", |event| {
        event["event"] == "execution.ended" && event["execution"] == omitted_execution
    });
    watch.until("omitted-effort controller released", |event| {
        event["event"] == "execution.released" && event["execution"] == omitted_execution
    });
    let corrected = project.json(&["attempt", "show", "Y-2"])["executions"][0].clone();
    assert_eq!(
        std::fs::read_to_string(machine.root.join("asset-2")).unwrap(),
        original
    );
    assert!(
        std::fs::read_to_string(machine.root.join("fresh-2"))
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
    assert_private_controller(&project);
    let probe = std::fs::read_to_string(machine.root.join("probe-2")).unwrap();
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
    assert!(!transcript(&project, 2).contains(SECRET));
}

fn debugger_holds_tool(machine: &Machine, handle: &str, line: &str) -> std::process::Child {
    use std::io::{BufRead, BufReader};
    let script = r#"
import { writeSync } from 'node:fs';
const response = await fetch('http://127.0.0.1:9229/json/list');
const discovery = await response.text();
if (!response.ok || !discovery.length)
  throw new Error(`inspector_discovery: HTTP ${response.status}, ${Buffer.byteLength(discovery)} bytes`);
const targets = JSON.parse(discovery);
const socket = new WebSocket(targets[0].webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener('open', resolve, {once:true});
  socket.addEventListener('error', reject, {once:true});
});
let id = 0;
const pending = new Map();
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  if (message.id) {
    const waiter = pending.get(message.id);
    pending.delete(message.id);
    message.error ? waiter.reject(message.error) : waiter.resolve(message.result);
  }
  if (message.method === 'Debugger.paused') writeSync(1, 'held\n');
});
const call = (method, params = {}) => new Promise((resolve, reject) => {
  pending.set(++id, {resolve,reject}); socket.send(JSON.stringify({id,method,params}));
});
await call('Debugger.enable');
const source = await (await import('node:fs/promises')).readFile('/opt/yard/durable-audit/runner.mjs', 'utf8');
const lineNumber = source.split('\n').findIndex(line => line.includes(BREAKPOINT_NEEDLE));
if (lineNumber < 0) throw new Error('tool source missing');
await call('Debugger.setBreakpointByUrl', {url:'file:///opt/yard/durable-audit/runner.mjs',lineNumber});
writeSync(1, 'armed\n');
// The debugger stays attached and paused until the real box is destroyed.
"#;
    let script = script.replace("BREAKPOINT_NEEDLE", &serde_json::to_string(line).unwrap());
    let mut command = std::process::Command::new("pinfold");
    command
        .env("XDG_STATE_HOME", "/tmp/yard-e2e-state")
        .env("XDG_CONFIG_HOME", &machine.config)
        .env("XDG_CACHE_HOME", &machine.cache)
        .args(["box", "exec", handle, "--", "env"]);
    // Only this observer uses guest loopback. The controller keeps its model
    // proxy; inherited proxy routing must not intercept inspector discovery
    // or its WebSocket connection.
    for key in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "NODE_USE_ENV_PROXY",
    ] {
        command.args(["-u", key]);
    }
    let mut child = command
        .args([
            "/opt/yard/durable-audit/node/bin/node",
            "--no-addons",
            "--input-type=module",
            "-e",
            &script,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let root = machine.root.clone();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = line.unwrap();
            match line.as_str() {
                "armed" | "held" => {
                    std::fs::write(root.join(format!("debug-{line}")), format!("{line}\n")).unwrap()
                }
                _ => panic!("unexpected debugger observation: {line}"),
            }
        }
    });
    child
}

/// G2/G5: pause a real read_declared invocation after Durable commits its
/// unsafe intent. The debugger holds it through daemon death; a real box
/// listing and refused exec prove it cannot act before replacement starts.
/// Sabotage: resume before checked release, drop checkpoint exclusion, reuse
/// corrupt checkpoint identity, copy session usage totals, or mark the read replay-safe.
/// Existing model-reply/finished-publication holds contain no pending tool.
/// Oracles: Node debugger, boxed read-only SQLite query, pinfold, fixture tool
/// results and token constants, external marker, and execution rows/history.
fn durable_checkpoint_scenario(name: &str, corrupt: bool) {
    let initial = std::sync::Arc::new(std::sync::Mutex::new(None::<Latch>));
    let held_initial = initial.clone();
    let resumed_marker = std::sync::Arc::new(std::sync::Mutex::new(None::<std::path::PathBuf>));
    let model_marker = resumed_marker.clone();
    let continuation = std::sync::Arc::new(std::sync::Mutex::new(None::<Latch>));
    let held_continuation = continuation.clone();
    let machine = Machine::new(name, move |request| {
        let read = || {
            Reply::Tools(vec![tool(
                "read_declared",
                json!({"file":"source:large.txt"}),
            )])
        };
        if request.opens() {
            if let Some(latch) = held_initial.lock().unwrap().take() {
                return Reply::Hold(latch, Box::new(read()));
            }
            return read();
        }
        if request
            .last_tool_result()
            .unwrap()
            .1
            .contains("interrupted")
        {
            if let Some(path) = model_marker.lock().unwrap().take() {
                std::fs::write(path, "unsafe read was interrupted, not replayed\n").unwrap();
            }
            if let Some(latch) = held_continuation.lock().unwrap().take() {
                return Reply::Hold(latch, Box::new(read()));
            }
            return read();
        }
        assert!(request.last_tool_result().unwrap().1.contains("truncated"));
        Reply::Tools(vec![audit_report()])
    });
    machine.kill();
    let source = checkpoint_source();
    machine.use_binary(Some(source.parent().unwrap().join("debug/yard")));
    machine.start();
    let project = fixture(&machine);
    let launcher = std::fs::read_to_string(source.join("runners/durable-audit/launch")).unwrap();
    project.write("runners/durable-audit/launch", &launcher);
    project.git(&["add", "runners/durable-audit/launch"]);
    project.git(&[
        "commit",
        "-qm",
        "Enable the real Node debugger in the fixture image",
    ]);
    project.json(&["sync"]);
    image(&project, "");
    for path in ["debug-armed", "debug-held"] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(machine.root.join(path))
                .status()
                .unwrap()
                .success()
        );
    }
    {
        let number = 1;
        let ticket = format!("Y-{number}");
        launch(&project, number, "Checkpoint continuation");
        let latch = Latch::new();
        *initial.lock().unwrap() = Some(latch.clone());
        let marker = machine.root.join(format!("resumed-{number}"));
        *resumed_marker.lock().unwrap() = Some(marker.clone());
        let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
        selected_start(&project, "Checkpoint continuation");
        latch.wait_held();
        let old = project.rows(&format!(
            "SELECT id, handle FROM execution WHERE attempt = {number} AND kind = 'implementation'"
        ))[0]
            .clone();
        let id = old["id"].as_i64().unwrap();
        let handle = old["handle"].as_str().unwrap();
        let mut debugger =
            debugger_holds_tool(&machine, handle, "const path = allowed.get(args.file);");
        assert_eq!(
            watch.said_before_end(&machine.root.join("debug-armed"), Some(id)),
            "armed\n"
        );
        latch.release();
        assert_eq!(
            watch.said_before_end(&machine.root.join("debug-held"), Some(id)),
            "held\n"
        );
        // Usage precedes the paused tool; the held-point wait may have read it.
        watch.find("committed execution usage", |event| {
            event["event"] == "execution.usage"
                && event["execution"] == id
                && event["data"]["input"] == PROMPT_TOKENS
        });
        // Reading a database as data inside the box is the supported worker
        // boundary. No host process opens or evaluates these checkpoint bytes.
        let intent = machine.pinfold(&["box", "exec", handle, "--",
            "/opt/yard/durable-audit/node/bin/node", "--no-addons", "--input-type=module", "-e",
            r#"import {DatabaseSync} from 'node:sqlite'; const db = new DatabaseSync('/yard/state/audit-checkpoints/session.sqlite',{readOnly:true}); console.log(JSON.stringify(db.prepare('SELECT record FROM tasks').all().map(r=>JSON.parse(r.record)).filter(r=>r.state.checkpoint?.phase === 'execute'))); db.close();"#]);
        assert!(intent.status.success());
        let tasks: Value = serde_json::from_slice(&intent.stdout).unwrap();
        assert_eq!(tasks.as_array().unwrap().len(), 1);
        assert_eq!(tasks[0]["state"]["checkpoint"]["replay"], "unsafe");
        let inode = machine.pinfold(&[
            "box",
            "exec",
            handle,
            "--",
            "stat",
            "-c",
            "%i",
            "/yard/state/audit-checkpoints/session.sqlite",
        ]);
        assert!(inode.status.success());
        let refused = project.refused(&["attempt", "start", &ticket]);
        assert_eq!(refused["data"]["reason"], "checkpoint_owned");
        assert_eq!(refused["data"]["execution"], id);
        assert_eq!(refused["data"]["checkpoint"], id);
        let key = handle
            .strip_prefix("yard-")
            .unwrap()
            .rsplit_once('-')
            .unwrap()
            .0;
        let label = format!("dev.yard.project={key}");
        let boxes = machine.boxes(&label);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0]["name"], handle);
        let before = project.rows(&format!(
            "SELECT id FROM execution WHERE attempt = {number}"
        ));
        assert_eq!(before.len(), 1);
        let boxed_marker = machine.pinfold(&["box", "exec", handle, "--", "sh", "-c",
            if corrupt { "printf '%s' 'untrusted checkpoint bytes' > /yard/state/audit-checkpoints/identity.json" }
            else { "printf '%s' 'fixture checkpoint marker' > /yard/state/audit-checkpoints/fixture-marker" }]);
        assert!(boxed_marker.status.success());
        machine.kill();
        // The tool is still debugger-held at death, not a fleeting row/ref.
        let held = project.rows(&format!(
            "SELECT status, reserved, report, tokens_in FROM execution WHERE id = {id}"
        ));
        assert_eq!(held[0]["status"], "running");
        assert_eq!(held[0]["reserved"], 1);
        assert!(held[0]["report"].is_null());
        assert_eq!(held[0]["tokens_in"], PROMPT_TOKENS);
        machine.start();
        let _ = debugger.wait().unwrap();
        let dead = machine.pinfold(&[
            "box",
            "exec",
            handle,
            "--",
            "sh",
            "-c",
            "echo old-owner-acted",
        ]);
        assert!(!dead.status.success());
        assert!(machine.boxes(&label).is_empty());
        let old = project.rows(&format!("SELECT * FROM execution WHERE id = {id}"))[0].clone();
        assert_eq!(old["outcome"], "interrupted");
        assert_eq!(old["reserved"], 0);
        assert_eq!(old["teardown"], 1);
        assert_eq!(old["tokens_in"], PROMPT_TOKENS);
        assert_eq!(old["tokens_out"], COMPLETION_TOKENS);
        let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
        let resumed = Latch::new();
        if !corrupt {
            *continuation.lock().unwrap() = Some(resumed.clone());
        }
        project.json(&["attempt", "start", &ticket]);
        if !corrupt {
            resumed.wait_held();
            let replacement = project.rows(&format!("SELECT handle FROM execution WHERE attempt = {number} AND status = 'running' AND kind = 'implementation'"));
            let new_handle = replacement[0]["handle"].as_str().unwrap();
            let checkpoint = machine.pinfold(&[
                "box",
                "exec",
                new_handle,
                "--",
                "cat",
                "/yard/state/audit-checkpoints/fixture-marker",
            ]);
            assert!(checkpoint.status.success());
            assert_eq!(checkpoint.stdout, b"fixture checkpoint marker");
            let retained_inode = machine.pinfold(&[
                "box",
                "exec",
                new_handle,
                "--",
                "stat",
                "-c",
                "%i",
                "/yard/state/audit-checkpoints/session.sqlite",
            ]);
            assert!(retained_inode.status.success());
            assert_eq!(retained_inode.stdout, inode.stdout);
            assert_eq!(machine.boxes(&label).len(), 1);
            resumed.release();
        }
        if corrupt {
            let refused = watch.until("corrupt checkpoint refused", |event| {
                event["event"] == "attention.raised" && event["ticket"] == ticket
            });
            assert!(transcript(&project, number).contains("checkpoint_invalid"));
            assert!(!marker.exists());
            watch.until("corrupt checkpoint owner released", |event| {
                event["event"] == "execution.released" && event["execution"] == refused["execution"]
            });
            // A failed checkpoint is an explicit error, and supported retry
            // starts fresh instead of falling back to a second live writer.
            project.json(&["attempt", "start", &ticket]);
        }
        watch.until("continued audit completed", |event| {
            event["event"] == "attempt.ended" && event["ticket"] == ticket
        });
        let rows = project.rows(&format!("SELECT id, resumed, prepared, tokens_in, tokens_out FROM execution WHERE attempt = {number} AND kind = 'implementation' ORDER BY id"));
        let last = rows.last().unwrap();
        assert_ne!(last["id"], id);
        if !corrupt {
            assert_eq!(last["resumed"], id);
            let prepared: Value = serde_json::from_str(last["prepared"].as_str().unwrap()).unwrap();
            assert_eq!(prepared["durable"]["checkpoint"], id);
            assert_eq!(prepared["durable"]["predecessor"], id);
            assert_eq!(
                std::fs::read_to_string(marker).unwrap(),
                "unsafe read was interrupted, not replayed\n"
            );
        } else {
            assert!(last["resumed"].is_null());
            let prepared: Value = serde_json::from_str(last["prepared"].as_str().unwrap()).unwrap();
            assert_eq!(prepared["durable"]["checkpoint"], last["id"]);
            assert!(!marker.exists());
        }
        assert_eq!(last["tokens_in"], 2 * PROMPT_TOKENS);
        assert_eq!(last["tokens_out"], 2 * COMPLETION_TOKENS);
        assert_eq!(
            project.rows(&format!("SELECT * FROM execution WHERE id = {id}"))[0],
            old
        );
        watch.until("checkpoint attempt cleaned", |event| {
            event["event"] == "execution.ended"
                && event["attempt"] == number
                && event["data"]["kind"] == "cleanup"
        });
    }
}

#[test]
fn durable_continuations_exclusively_resume_matching_interrupted_inputs() {
    durable_checkpoint_scenario("g5-durable-resume", false);
}

#[test]
fn durable_checkpoint_corruption_refuses_before_fresh_retry() {
    durable_checkpoint_scenario("g2-durable-corrupt", true);
}

fn checkpoint_source() -> &'static std::path::PathBuf {
    static SOURCE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    SOURCE.get_or_init(checkpoint_daemon_source)
}

/// G2/G9/G13: one project refuses the audit's source write, then admits it
/// for cleanup on the sequential handoff. The real debugger holds the write
/// after fsync while its unsafe intent is pending. Host bytes, boxed SQLite,
/// Git ancestry and execution history supply the expected values.
/// Sabotage: grant audit writes, replay an unsafe write, reuse a workspace or
/// checkpoint before checked release, or let cleanup publish/land a candidate.
/// Adding undeclared retry feedback to packaged inputs also prevents continuation.
/// One machine adds an audit, implementation, interrupted cleanup, resumed
/// cleanup and repair box; the debugger-enabled build is shared with G5.
#[test]
fn selected_durable_cleanup_commits_without_replaying_an_interrupted_write() {
    use std::os::unix::fs::MetadataExt;
    use std::sync::{Arc, Mutex};
    let initial = Latch::new();
    let held_initial = initial.clone();
    let resumed = Latch::new();
    let held_resumed = resumed.clone();
    let resumed_request = Arc::new(Mutex::new(None::<std::path::PathBuf>));
    let model_resumed_request = resumed_request.clone();
    let inspection = Latch::new();
    let held_inspection = inspection.clone();
    let nested_mode = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let model_nested = nested_mode.clone();
    let nested_inspection = Latch::new();
    let held_nested = nested_inspection.clone();
    let audit_refusal = Arc::new(Mutex::new(false));
    let observed_refusal = audit_refusal.clone();
    let machine = Machine::new("g13-durable-cleanup", move |request| {
        let write = |content| {
            tool(
                "write_declared",
                json!({"file": "source:chain.txt", "content": content}),
            )
        };
        match request.body["model"].as_str().unwrap() {
            "audit-selected" => {
                if request.opens() {
                    return Reply::Tools(vec![write("first-write\n")]);
                }
                assert_eq!(
                    request.last_tool_result().unwrap().1,
                    "source_write_refused"
                );
                *observed_refusal.lock().unwrap() = true;
                Reply::Tools(vec![audit_report()])
            }
            "cleanup-selected" => {
                assert!(!request.has_tool("bash"));
                assert!(!request.has_tool("yard_attempt_approve"));
                if model_nested.load(std::sync::atomic::Ordering::SeqCst) {
                    assert!(request.opens());
                    return Reply::Hold(
                        held_nested.clone(),
                        Box::new(Reply::Tools(vec![audit_report()])),
                    );
                }
                if request.opens() {
                    return Reply::Hold(
                        held_initial.clone(),
                        Box::new(Reply::Tools(vec![write("first-write\n")])),
                    );
                }
                let (name, result) = request.last_tool_result().unwrap();
                if result.contains("interrupted") {
                    let path = model_resumed_request
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .clone();
                    std::fs::write(path, "request\n").unwrap();
                    return Reply::Hold(
                        held_resumed.clone(),
                        Box::new(Reply::Tools(vec![write("cleaned\n")])),
                    );
                }
                if name == "write_declared" {
                    assert!(
                        request
                            .tool_results()
                            .iter()
                            .any(|(_, result)| result.contains("interrupted"))
                    );
                    assert_eq!(result, "source_written");
                    Reply::Tools(vec![tool("commit_source", json!({"message": "Cleanup"}))])
                } else {
                    assert_eq!(name, "commit_source");
                    assert_eq!(result, "source_committed");
                    let path = model_resumed_request
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .with_file_name("cleanup-inspection");
                    std::fs::write(path, "request\n").unwrap();
                    Reply::Hold(
                        held_inspection.clone(),
                        Box::new(Reply::Tools(vec![audit_report()])),
                    )
                }
            }
            "implementer-selected" | "repair-selected" => {
                if request.opens() {
                    return Reply::Tools(vec![bash(
                        if request.body["model"] == "implementer-selected" {
                            if model_nested.load(std::sync::atomic::Ordering::SeqCst) {
                                r#"git init -q .git/nested-seed &&
printf 'nested\n' > .git/nested-seed/file.txt &&
printf 'file.txt filter=trip\n' > .git/nested-seed/.gitattributes &&
git -C .git/nested-seed add . && git -C .git/nested-seed commit -qm Seed &&
git -c protocol.file.allow=always submodule add /workspace/.git/nested-seed nested &&
git commit -qm 'Add nested repository' &&
git -C nested config filter.trip.clean 'touch /workspace/.git/nested-filter-ran; cat' &&
touch -t 200001010000 nested/file.txt &&
git status --porcelain && test -f .git/nested-filter-ran"#
                            } else {
                                r#"printf 'implemented\n' > chain.txt && git add chain.txt && git commit -qm Implement &&
git config core.fsmonitor 'touch /workspace/.git/fsmonitor-ran; false' &&
git config filter.trip.clean 'touch /workspace/.git/filter-ran; cat' &&
git config filter.trip.required true &&
printf 'chain.txt filter=trip\n' > .git/info/attributes &&
touch -t 200001010000 chain.txt &&
git -c core.fsmonitor=false status --porcelain && test -f .git/filter-ran &&
git status --porcelain && test -f .git/fsmonitor-ran &&
mkdir .git/alternate && git -c filter.trip.clean= -c filter.trip.smudge= -c filter.trip.process= -c filter.trip.required=false archive HEAD | tar -x -C .git/alternate"#
                            }
                        } else {
                            r#"test ! -e .git/fsmonitor-ran && test ! -e .git/filter-ran &&
git config --unset core.fsmonitor && git config --remove-section filter.trip &&
git config --unset core.worktree &&
test "$(cat chain.txt)" = cleaned && test "$(cat /yard/input/trusted/repair.md)" = trusted-repair && printf 'repaired\n' > chain.txt && git add chain.txt && git commit -qm Repair"#
                        },
                    )]);
                }
                Reply::Text("done".into())
            }
            other => panic!("unexpected selected model {other}"),
        }
    });
    let resumed_request_path = machine.root.join("cleanup-resumed");
    *resumed_request.lock().unwrap() = Some(resumed_request_path.clone());
    machine.kill();
    let source = checkpoint_source();
    machine.use_binary(Some(source.parent().unwrap().join("debug/yard")));
    machine.start();
    let project = fixture(&machine);
    project.write("chain.txt", "base\n");
    project.write(
        "implementer.md",
        "Implement the selected handoff fixture.\n",
    );
    project.write("repair.md", "trusted-repair\n");
    project.write(
        "runners/durable-audit/launch",
        &std::fs::read_to_string(source.join("runners/durable-audit/launch")).unwrap(),
    );
    project.git(&["add", "-A"]);
    project.git(&["commit", "-qm", "Selected cleanup fixture"]);
    project.json(&["sync"]);
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
    image(&project, "");
    let mut extra = String::from(
        "\n[gates.verify]\ncommand = 'test \"$(cat chain.txt)\" = repaired'\nstage = 'candidate'\nruns_in = 'host'\n",
    );
    for (role, output) in [("implementer", "candidate"), ("repair", "candidate")] {
        extra.push_str(&format!("\n[agents.{role}]\nharness = 'pi'\nprovider = 'openrouter'\nmodel = '{role}-selected'\n[profiles.{role}]\naccess = 'write-allowed'\noutcomes = ['complete']\n[profiles.{role}.outputs]\nresult = '{output}'\n[profiles.{role}.runner]\nkind = 'agent'\nagent = '{role}'\ninstructions = 'Selected {role} role.'\n"));
        extra.push_str(&format!("skills = ['{role}.md']\n"));
    }
    for (role, access) in [("audit", "read-only"), ("cleanup", "write-allowed")] {
        extra.push_str(&format!(
            r#"
[agents.{role}]
harness = "pi"
provider = "openrouter"
model = "{role}-selected"
[profiles.{role}]
access = "{access}"
outcomes = ["complete"]
[profiles.{role}.outputs]
result = "report"
[profiles.{role}.runner]
kind = "packaged"
package = "durable-audit"
agent = "{role}"
instructions = "Selected {role} role."
skill = "trusted/selected.md"
references = ["trusted/reference.txt"]
files = [{{root = "source", path = "chain.txt"}}]
"#
        ));
    }
    project.reconfigure(
        &config(&extra)
            .replace("review = [\"correctness\"]", "review = \"none\"")
            .replace(
                "[workflows.plan]",
                "[workflows.plan]\nimplementer = \"audit\"\nprofile = \"audit\"",
            ),
    );
    // A failed predecessor cannot reach a cleanup model latch. Observe its
    // terminal event first so staging/startup failures fail promptly.
    // Sabotage: fail the ordinary predecessor before cleanup can start.
    let implemented = |watch: &mut Watch, attempt: i64| {
        let event = watch.until("selected implementer ended", |event| {
            event["event"] == "execution.ended"
                && event["attempt"] == attempt
                && project.rows(&format!(
                    "SELECT node FROM execution WHERE id = {}",
                    event["execution"]
                ))[0]["node"]
                    == "implementer"
        });
        assert_eq!(event["data"]["outcome"], "candidate", "{event}");
    };
    let mut watch = project.watch(0);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Selected cleanup",
        "--workflow",
        "plan",
    ]);
    let planned = watch.event("attempt.ended", &[("outcome", "planned")]);
    watch.find("audit cleanup released", |event| {
        event["event"] == "execution.released"
            && event["attempt"] == planned["attempt"]
            && project.rows(&format!(
                "SELECT kind FROM execution WHERE id = {}",
                event["execution"]
            ))[0]["kind"]
                == "cleanup"
    });
    assert!(*audit_refusal.lock().unwrap());
    // Sabotage: let the audit mutate main, move main before approval, or
    // report landing without updating it. Canonical is bare: read its ref.
    assert_eq!(
        git(&project.canonical(), &["show", "refs/heads/main:chain.txt"]),
        "base\n"
    );
    // Ordinary Pi loads trusted base extensions even with selected skills.
    // Keep the hostile extension in the audit's history, then retire it.
    project.git(&["rm", ".pi/extensions/unselected.js"]);
    project.git(&["commit", "-qm", "Retire audit-only hostile extension"]);
    project.json(&["sync"]);
    for path in [
        "debug-armed",
        "debug-held",
        "cleanup-resumed",
        "cleanup-inspection",
    ] {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(machine.root.join(path))
                .status()
                .unwrap()
                .success()
        );
    }
    let mut operator = machine.operator();
    operator.initialize();
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1",
        "graph": super::g13_runs_where_its_row_says::sequential_handoff_graph()}),
    );
    assert!(started["error"].is_null(), "{started}");
    let attempt = started["result"]["attempt"].as_i64().unwrap();
    implemented(&mut watch, attempt);
    initial.wait_held();
    let old = project.rows(&format!("SELECT id, handle FROM execution WHERE attempt = {attempt} AND node = 'cleanup' AND status = 'running'"))[0].clone();
    let id = old["id"].as_i64().unwrap();
    let handle = old["handle"].as_str().unwrap();
    let clone = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let inode = std::fs::metadata(clone.join(".git")).unwrap().ino();
    // Apply redirection only after ordinary boxed status and publication.
    // Keep it relative so later host fetches can resolve the same repository.
    assert_eq!(
        std::fs::read_to_string(clone.join(".git/alternate/chain.txt")).unwrap(),
        "implemented\n"
    );
    git(
        &clone,
        &[
            "config",
            "--file",
            ".git/config",
            "core.worktree",
            "alternate",
        ],
    );
    // The predecessor's alternate worktree must never supply committed bytes.
    std::fs::write(clone.join(".git/alternate/chain.txt"), "substitute\n").unwrap();
    // Positive controls above execute both predecessor helpers. Sabotage:
    // inspect packaged cleanup with plain status, or leave clean filters enabled.
    for marker in ["fsmonitor-ran", "filter-ran"] {
        let path = clone.join(".git").join(marker);
        assert!(path.is_file(), "ordinary Git did not execute {marker}");
        std::fs::remove_file(path).unwrap();
    }
    let readonly = machine.pinfold(&["box", "exec", handle, "--", "sh", "-c", r#"
test "$(cat /yard/input/trusted/trusted/reference.txt)" = TRUSTED_REFERENCE_Y92 || exit 1
if printf 'changed\n' > /yard/input/trusted/trusted/reference.txt; then echo trusted=written; else echo trusted=refused; fi
if printf 'changed\n' > /yard/proof/intrusion; then echo proof=written; else echo proof=refused; fi
"#]);
    assert!(readonly.status.success());
    assert_eq!(
        String::from_utf8(readonly.stdout).unwrap(),
        "trusted=refused\nproof=refused\n"
    );
    let mut debugger = debugger_holds_tool(&machine, handle, "return text('source_written');");
    assert_eq!(
        watch.said_before_end(&machine.root.join("debug-armed"), Some(id)),
        "armed\n"
    );
    initial.release();
    assert_eq!(
        watch.said_before_end(&machine.root.join("debug-held"), Some(id)),
        "held\n"
    );
    assert_eq!(
        std::fs::read_to_string(clone.join("chain.txt")).unwrap(),
        "first-write\n"
    );
    let intent = machine.pinfold(&["box", "exec", handle, "--", "/opt/yard/durable-audit/node/bin/node", "--no-addons", "--input-type=module", "-e",
        r#"import {DatabaseSync} from 'node:sqlite'; const db = new DatabaseSync('/yard/state/audit-checkpoints/session.sqlite',{readOnly:true}); console.log(JSON.stringify(db.prepare('SELECT record FROM tasks').all().map(r=>JSON.parse(r.record)).filter(r=>r.state.checkpoint?.phase === 'execute'))); db.close();"#]);
    assert!(intent.status.success());
    let tasks: Value = serde_json::from_slice(&intent.stdout).unwrap();
    assert_eq!(tasks.as_array().unwrap().len(), 1);
    assert_eq!(tasks[0]["state"]["checkpoint"]["replay"], "unsafe");
    let refused = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-1", "workspace": "implementer"}),
    );
    assert_eq!(
        refused["error"]["data"],
        json!({"reason": "workspace_owned", "workspace": "implementer", "execution": id})
    );
    let refused = project.refused(&["attempt", "start", "Y-1"]);
    assert_eq!(refused["data"]["reason"], "checkpoint_owned");
    assert_eq!(refused["data"]["execution"], id);
    assert_eq!(refused["data"]["checkpoint"], id);
    machine.kill();
    assert_eq!(
        std::fs::read_to_string(clone.join("chain.txt")).unwrap(),
        "first-write\n"
    );
    assert_eq!(
        project.rows(&format!("SELECT reserved FROM execution WHERE id = {id}"))[0]["reserved"],
        1
    );
    machine.start();
    let _ = debugger.wait().unwrap();
    assert!(
        !machine
            .pinfold(&["box", "exec", handle, "--", "true"])
            .status
            .success()
    );
    let old = project.rows(&format!(
        "SELECT outcome, teardown, reserved FROM execution WHERE id = {id}"
    ))[0]
        .clone();
    assert_eq!(
        old,
        json!({"outcome": "interrupted", "teardown": 1, "reserved": 0})
    );
    // An outside marker makes replay observable even though replacement is
    // idempotent. The predecessor has been proved gone before this edit.
    std::fs::write(clone.join("chain.txt"), "after-release\n").unwrap();
    let mut watch = project.watch(project.json(&["status"])["seq"].as_i64().unwrap());
    project.json(&["attempt", "start", "Y-1"]);
    let replacement_started = watch.until("replacement cleanup started", |event| {
        event["event"] == "execution.started"
            && event["attempt"] == attempt
            && event["data"]["kind"] == "implementation"
    });
    let replacement_id = replacement_started["execution"].as_i64().unwrap();
    assert_eq!(
        watch.said_before_end(&resumed_request_path, Some(replacement_id)),
        "request\n"
    );
    assert_eq!(
        std::fs::read_to_string(clone.join("chain.txt")).unwrap(),
        "after-release\n"
    );
    let replacement = project.rows(&format!("SELECT id, handle, resumed, prepared FROM execution WHERE id = {replacement_id} AND attempt = {attempt} AND node = 'cleanup' AND status = 'running'"))[0].clone();
    assert_ne!(replacement["id"], id);
    assert_eq!(replacement["resumed"], id);
    let prepared: Value = serde_json::from_str(replacement["prepared"].as_str().unwrap()).unwrap();
    assert_eq!(prepared["durable"]["checkpoint"], id);
    assert_eq!(prepared["durable"]["predecessor"], id);
    assert_eq!(project.rows(&format!("SELECT id FROM execution WHERE attempt = {attempt} AND node = 'cleanup' AND reserved = 1")).len(), 1);
    resumed.release();
    assert_eq!(
        watch.said_before_end(
            &machine.root.join("cleanup-inspection"),
            Some(replacement_id)
        ),
        "request\n"
    );
    // Sabotage: omit the explicit /workspace worktree from commit or final
    // inspection. Native Git sees the substitute tree as dirty; cleanup must
    // commit the actual declared source and inspect that clean source instead.
    let redirected = machine.pinfold(&["box", "exec", replacement["handle"].as_str().unwrap(), "--",
        "/usr/bin/env", "-i", "PATH=/usr/bin:/bin", "GIT_CONFIG_NOSYSTEM=1",
        "GIT_CONFIG_GLOBAL=/dev/null", "GIT_TERMINAL_PROMPT=0", "sh", "-c", r#"
git -C /workspace -c safe.directory=/workspace -c core.hooksPath=/dev/null -c core.fsmonitor=false show HEAD:chain.txt &&
git -C /workspace -c safe.directory=/workspace -c core.hooksPath=/dev/null -c core.fsmonitor=false -c filter.trip.clean= -c filter.trip.required=false status --porcelain --untracked-files=all
"#]);
    assert!(redirected.status.success());
    assert_eq!(
        String::from_utf8(redirected.stdout).unwrap(),
        "cleaned\n M chain.txt\n"
    );
    // Force status to recheck content even after commit_source refreshed the
    // index. Without this, Git may skip the clean filter on matching stat data.
    std::fs::File::open(clone.join("chain.txt"))
        .unwrap()
        .set_times(
            std::fs::FileTimes::new()
                .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(946684800)),
        )
        .unwrap();
    inspection.release();
    let approval = watch.until("cleanup handoff approval", |event| {
        event["event"] == "attention.raised" && event["data"]["kind"] == "approval"
    });
    for marker in ["fsmonitor-ran", "filter-ran"] {
        assert!(
            !clone.join(".git").join(marker).exists(),
            "packaged cleanup executed predecessor {marker}"
        );
    }
    assert_eq!(std::fs::metadata(clone.join(".git")).unwrap().ino(), inode);
    assert_eq!(
        std::fs::read_to_string(clone.join("chain.txt")).unwrap(),
        "repaired\n"
    );
    assert_eq!(
        git(&project.canonical(), &["show", "refs/heads/main:chain.txt"]),
        "base\n"
    );
    let rows = project.rows(&format!("SELECT id, node, result, reserved, teardown FROM execution WHERE attempt = {attempt} AND kind = 'implementation' ORDER BY id"));
    assert_eq!(
        rows.iter()
            .map(|row| row["node"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["implementer", "cleanup", "cleanup", "repair"]
    );
    for row in &rows {
        assert_eq!(row["reserved"], 0);
        assert_eq!(row["teardown"], 1);
    }
    let report: Value = serde_json::from_str(rows[2]["result"].as_str().unwrap()).unwrap();
    assert_eq!(report["outputs"]["result"]["kind"], "report");
    assert_eq!(project.rows(&format!("SELECT json_extract(data, '$.candidate') AS execution FROM audit WHERE event = 'attempt.candidate' AND attempt = {attempt} ORDER BY seq")), vec![json!({"execution": rows[0]["id"]}), json!({"execution": rows[3]["id"]})]);
    let head = approval["data"]["payload"]["head"].as_str().unwrap();
    assert_eq!(
        git(&project.canonical(), &["log", "-3", "--format=%s", head])
            .lines()
            .collect::<Vec<_>>(),
        ["Repair", "Cleanup", "Implement"]
    );
    project.json(&[
        "attempt", "approve", "Y-1", "--target", "main", "--head", head,
    ]);
    watch.event("attempt.ended", &[("outcome", "landed")]);
    assert_eq!(
        git(&project.canonical(), &["show", "refs/heads/main:chain.txt"]),
        "repaired\n"
    );

    // Same-project allowed control is the completed handoff above. Sabotage:
    // remove the gitlink refusal and let status execute a nested clean filter.
    nested_mode.store(true, std::sync::atomic::Ordering::SeqCst);
    project.json(&[
        "ticket",
        "new",
        "--title",
        "Nested Git inspection",
        "--workflow",
        "plan",
    ]);
    let planned = watch.event("attempt.ended", &[("outcome", "planned")]);
    watch.find("nested planner release", |event| {
        event["event"] == "execution.released"
            && event["attempt"] == planned["attempt"]
            && project.rows(&format!(
                "SELECT kind FROM execution WHERE id = {}",
                event["execution"]
            ))[0]["kind"]
                == "cleanup"
    });
    let started = operator.call(
        "yard_attempt_start",
        json!({"project": project.path, "ticket": "Y-2",
        "graph": super::g13_runs_where_its_row_says::sequential_handoff_graph()}),
    );
    assert!(started["error"].is_null(), "{started}");
    let attempt = started["result"]["attempt"].as_i64().unwrap();
    implemented(&mut watch, attempt);
    nested_inspection.wait_held();
    let row = project.rows(&format!("SELECT id, handle FROM execution WHERE attempt = {attempt} AND node = 'cleanup' AND status = 'running'"))[0].clone();
    let clone = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/clone"));
    let marker = clone.join(".git/nested-filter-ran");
    assert!(
        marker.is_file(),
        "ordinary predecessor did not run the filter"
    );
    std::fs::remove_file(&marker).unwrap();
    let invalidate = |seconds| {
        std::fs::File::open(clone.join("nested/file.txt"))
            .unwrap()
            .set_times(
                std::fs::FileTimes::new()
                    .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds)),
            )
            .unwrap();
    };
    invalidate(978307200);
    let control = machine.pinfold(&[
        "box",
        "exec",
        row["handle"].as_str().unwrap(),
        "--",
        "/usr/bin/env",
        "-i",
        "PATH=/usr/bin:/bin",
        "GIT_CONFIG_NOSYSTEM=1",
        "GIT_CONFIG_GLOBAL=/dev/null",
        "GIT_TERMINAL_PROMPT=0",
        "/usr/bin/git",
        "-C",
        "/workspace",
        "-c",
        "safe.directory=/workspace",
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "core.fsmonitor=false",
        "status",
        "--porcelain",
        "--untracked-files=all",
    ]);
    assert!(control.status.success());
    assert!(control.stdout.is_empty(), "nested control must be clean");
    assert!(
        marker.is_file(),
        "boxed Git control did not run nested filter"
    );
    std::fs::remove_file(&marker).unwrap();
    invalidate(1009843200);
    nested_inspection.release();
    let ended = watch.find("nested inspection refusal", |event| {
        event["event"] == "execution.ended" && event["execution"] == row["id"]
    });
    assert_eq!(ended["data"]["outcome"], "refused");
    watch.find("nested inspection released", |event| {
        event["event"] == "execution.released" && event["execution"] == row["id"]
    });
    assert!(!marker.exists(), "packaged inspection ran nested filter");
    assert!(
        project
            .rows(&format!(
                "SELECT id FROM execution WHERE attempt = {attempt} AND node = 'repair'"
            ))
            .is_empty()
    );
    assert_eq!(
        git(&project.canonical(), &["show", "refs/heads/main:chain.txt"]),
        "repaired\n"
    );
}
