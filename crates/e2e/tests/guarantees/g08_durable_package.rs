//! G8/G12/G13: trusted packaging, independently of daemon runner selection.

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

fn image(project: &Project<'_>, missing: bool) {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runners/durable-audit/Dockerfile"),
    )
    .unwrap()
    .replace("\"$TARGETARCH\"", "\"$(uname -m)\"");
    let sabotage = if missing {
        "RUN rm /opt/yard/durable-audit/node_modules/@earendil-works/pi-durable/dist/index.js\n"
    } else {
        ""
    };
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
    // Ordinary canonical fixture files are frozen by the read-only plan. Never
    // write daemon-owned launch inputs or replace the configured Pi command.
    project.write(
        "audit.json",
        &json!({
            "provider": "openrouter", "model": "fake-model", "prompt": prompt,
            "files": [
                {"root": "source", "path": "large.txt"},
                {"root": "source", "path": "candidate.cjs"},
                {"root": "source", "path": "candidate/.pi/extensions/candidate.js"},
                {"root": "trusted", "path": "selected.md"},
                {"root": "trusted", "path": "reference.txt"}
            ], "skill": "trusted:selected.md"
        })
        .to_string(),
    );
    project.git(&["add", "audit.json"]);
    project.git(&["commit", "-qm", "Declare trusted boxed audit inputs"]);
    project.json(&["sync"]);
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
    "$REAL" "$@"
    result=$?
    "$REAL" box exec "$handle" -- sh -c '
        env | grep -q "sk-e2e-"; echo upstream_in_env=$?
        grep -R -a -q "sk-e2e-" /workspace /yard/state /opt/yard/durable-audit; echo upstream_in_files=$?
        test -e /yard/state/sentinel; echo sentinel=$?
        echo blocked > /workspace/intrusion 2>/dev/null; echo source_write=$?
        echo blocked > /workspace/trusted/selected.md 2>/dev/null; echo trusted_write=$?
        echo scratch > /yard/state/audit-scratch/probe; echo scratch_write=$?
        echo state > /yard/state/audit-checkpoints/probe; echo checkpoint_write=$?
    ' > '{root}/probe-{attempt}'
    exit "$result"
fi
"#, root = project.machine.root.display()));
}

// Outer Pi remains real and owns MCP registration. Controller traffic is
// identified by its distinct offered tool surface, not by total route traffic.
fn outer(request: &ModelRequest) -> Option<Reply> {
    if request.has_tool("read_declared") {
        return None;
    }
    assert!(request.has_tool("yard_run_durable_audit"));
    if request.opens() {
        Some(Reply::Tools(vec![tool(
            "yard_run_durable_audit",
            json!({}),
        )]))
    } else {
        Some(Reply::Text("Audit child finished.".into()))
    }
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
    project.json(&["ticket", "new", "--title", prompt, "--workflow", "plan"]);
    watch.attention();
    let execution = project.json(&["attempt", "show", ticket])["executions"][0].clone();
    assert_eq!(execution["teardown"], true);
    execution
}

fn transcript(project: &Project<'_>, attempt: i64) -> String {
    let path = project
        .path
        .join(format!(".yard/local/attempts/{attempt}/transcripts"));
    // Transcript filenames are not asserted: only externally recorded reasons.
    fn collect(path: &Path, text: &mut String) {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                collect(&entry.unwrap().path(), text);
            }
        } else {
            text.push_str(&std::fs::read_to_string(path).unwrap());
        }
    }
    let mut text = String::new();
    collect(&path, &mut text);
    text
}

/// G8 pre-call packaging and G12/G13 isolation: remove a sealed Durable
/// entrypoint, then repair the image in the SAME project. The controller owns
/// no box; real Pi invokes its trusted child in Yard's box. The fixture supplies
/// report text, selected skill/reference, key, hostile executable/extension
/// and an oversized source. Expected 32768-byte read comes from the spec.
/// Sabotage: skip identity checks, load CodingTools or candidate code, ignore
/// declared files/output bound, omit selected skill, persist the bearer/key,
/// count report characters instead of bytes, or bypass the injecting route.
/// Existing Pi report tests prove publication,
/// not this packaged runtime, its preflight, or its bounded read surface.
#[test]
fn a_packaged_audit_refuses_missing_assets_then_publishes_in_a_corrected_box() {
    let machine = Machine::new("g8-durable-package", |request| {
        if let Some(reply) = outer(request) {
            return reply;
        }
        assert!(request.context().contains("TRUSTED_SELECTED_SKILL_Y92"));
        assert!(!request.has_tool("bash"));
        if request.opens() {
            return Reply::Tools(vec![
                tool("read_declared", json!({"file": "source:large.txt"})),
                tool("read_declared", json!({"file": "trusted:reference.txt"})),
                tool("read_declared", json!({"file": "source:candidate.cjs"})),
                tool(
                    "read_declared",
                    json!({"file": "source:candidate/.pi/extensions/candidate.js"}),
                ),
                tool("read_declared", json!({"file": "source:../operator.env"})),
                tool("bash", json!({"command": "node /workspace/candidate.cjs"})),
            ]);
        }
        let results = request.tool_results();
        if results
            .iter()
            .any(|(name, _)| name == "yard_publish_report")
        {
            assert!(results.iter().any(|(_, s)| s == "invalid_report"));
            return Reply::Tools(vec![publish_report()]);
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
    image(&project, true);
    launch(&project, 1, "Packaged audit");
    let refused = ended(&project, "Y-1", "Missing packaged asset");
    assert!(refused["report"].is_null());
    assert!(controller_requests(&machine).is_empty());
    assert!(
        machine
            .model
            .requests()
            .iter()
            .any(|r| r.has_tool("yard_run_durable_audit"))
    );
    let log = transcript(&project, 1);
    assert!(log.contains("runner_unavailable"));
    assert!(log.contains("pi-durable/dist/index.js"));
    project.json(&["ticket", "park", "Y-1"]);
    project.json(&["attempt", "abandon", "Y-1"]);

    image(&project, false);
    launch(&project, 2, "Corrected packaged audit");
    let completed = ended(&project, "Y-2", "Corrected packaged asset");
    assert_eq!(
        completed["report"]["report"],
        json!({"summary": "Fixture audit", "findings": []})
    );
    let requests = controller_requests(&machine);
    for request in requests {
        assert_eq!(
            request.header("authorization"),
            Some(format!("Bearer {SECRET}").as_str())
        );
        assert!(!request.body.to_string().contains(SECRET));
    }
    let probe = std::fs::read_to_string(machine.root.join("probe-2")).unwrap();
    for token in [
        "upstream_in_env=1",
        "upstream_in_files=1",
        "sentinel=1",
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
    assert!(
        !serde_json::to_string(&project.rows("SELECT report FROM execution"))
            .unwrap()
            .contains(SECRET)
    );
}

/// G8: a model repeatedly asks to read instead of publishing. The packaged
/// controller stops at the spec's 12-request bound, not a timeout. A fresh
/// execution in the same project publishes as its positive control.
/// Sabotage: remove the request bound or synthesize a report at terminal text.
/// Existing publication tests exercise neither internal Durable iterations
/// nor this controller's bare-answer detection. Fixture supplies all replies.
#[test]
fn a_packaged_audit_bounds_iterations_and_requires_native_publication() {
    let machine = Machine::new("g8-durable-bound", |request| {
        if let Some(reply) = outer(request) {
            return reply;
        }
        if request.prompt().contains("Endless") {
            Reply::Tools(vec![tool(
                "read_declared",
                json!({"file": "source:large.txt"}),
            )])
        } else if request.prompt().contains("Bare") {
            Reply::Text("This is not a report.".into())
        } else {
            Reply::Tools(vec![publish_report()])
        }
    });
    machine.start();
    let project = fixture(&machine);
    image(&project, false);
    for (attempt, prompt, reason, count) in [
        (1, "Endless", "audit_limit", Some(12)),
        (2, "Bare", "missing_report", None),
    ] {
        let before = controller_requests(&machine).len();
        launch(&project, attempt, prompt);
        let ticket = format!("Y-{attempt}");
        let execution = ended(&project, &ticket, prompt);
        assert!(execution["report"].is_null());
        if let Some(count) = count {
            assert_eq!(controller_requests(&machine).len() - before, count);
        }
        assert!(transcript(&project, attempt).contains(reason));
        project.json(&["ticket", "park", &ticket]);
        project.json(&["attempt", "abandon", &ticket]);
    }
    launch(&project, 3, "Publish control");
    let control = ended(&project, "Y-3", "Publish control");
    assert_eq!(
        control["report"]["report"],
        json!({"summary": "Fixture audit", "findings": []})
    );
}
