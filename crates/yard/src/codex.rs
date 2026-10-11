//! Codex, the subscription-login harness: its launch argv, the files a
//! launch needs, and the normalisation of its JSON stream.
//!
//! Codex 0.158.0, as pinfold carries it.

use crate::config::Agent;
use crate::daemon::Machine;
use crate::harness::{
    Event, Harness, Launch, LoginInfo, ModelRoute, Reader, Registration, Stage, Usage,
};
use serde_json::Value;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const CODEX: &str = "/opt/pinfold/codex/codex";
const HOME_GUEST: &str = "/yard/state/home";
const CODEX_HOME_GUEST: &str = "/yard/state/codex";
const MCP_ENDPOINT: &str = "http://yard.mcp/mcp";
/// The server name the launch config registers under.
const MCP_SERVER: &str = "yard";
/// The login name pinfold resolves, and the route the box reaches it by.
/// Pinfold takes no `from` variable: the token comes from the host's Codex
/// login.
const LOGIN: &str = "codex";
const ROUTE: &str = "codex.yard";
/// The provider the argv names, so a staged config cannot replace it. The
/// base URL is the route's Codex backend path.
const PROVIDER: &str = "pinfold";
const PROVIDER_URL: &str = "http://codex.yard/backend-api/codex";
/// The login's origin override, as `YARD_ORIGIN_<NAME>`.
const ORIGIN_VAR: &str = "YARD_ORIGIN_CODEX";

// The prompt is one argv entry, so Linux's per-argument limit (32 pages)
// less the terminating NUL bounds it.
const PROMPT_MAX_BYTES: usize = 128 * 1024 - 1;
const MODEL_MAX_BYTES: usize = 256;
const SESSION_ID_MAX_BYTES: usize = 128;

/// Yard's effort ladder is Codex's `model_reasoning_effort` word for word.
/// An unknown level is refused at load.
pub const EFFORTS: [&str; 4] = ["minimal", "low", "medium", "high"];

/// The automatic-compaction threshold staged with the launch, in tokens. It
/// leaves room for the response inside the pinned version's default window;
/// a worker's own setting never carries into its next execution. Yard never
/// compacts a session itself.
const COMPACTION_TOKEN_LIMIT: u64 = 180_000;

/// The harness whose login token pinfold resolves from the host itself.
pub struct Codex;

impl Harness for Codex {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn login(&self) -> Option<LoginInfo> {
        Some(LoginInfo {
            name: LOGIN,
            // No `operator.env` key: pinfold resolves the host's own login.
            key_var: None,
        })
    }

    fn accepts(&self, agent: &Agent) -> Result<(), String> {
        if agent.provider.is_some() {
            return Err("provider is not accepted; set login = true".into());
        }
        if agent.login != Some(true) {
            return Err("login = true is required".into());
        }
        if let Some(effort) = &agent.effort
            && !EFFORTS.contains(&effort.as_str())
        {
            return Err(format!("effort {effort:?} is unknown"));
        }
        Ok(())
    }

    /// The base's project layer is Codex's user level, the attempt's
    /// `CODEX_HOME`: its config, guidance, skills and hooks. Yard's own keys
    /// ride the argv, which outranks the staged config.
    fn stage(&self, st: &Stage) -> std::io::Result<()> {
        // The box mounts the input dir read-only, so it must exist for the
        // mount even though Codex stages no file there.
        std::fs::create_dir_all(st.input)?;
        let env = st.env;
        env.stage_file(st.state, "codex/config.toml", ".codex/config.toml")?;
        env.stage_guidance(st.state, "codex/AGENTS.md")?;
        env.stage_dir(st.state, "codex/skills", &[".codex/skills/"])?;
        env.stage_file(st.state, "codex/hooks.json", ".codex/hooks.json")?;
        env.stage_dir(st.state, "home/.agents/skills", &[".agents/skills/"])
    }

    fn env(&self, _st: &Stage) -> Vec<(String, String)> {
        [("HOME", HOME_GUEST), ("CODEX_HOME", CODEX_HOME_GUEST)]
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .into()
    }

    fn route(&self, _st: &Stage, machine: &Machine) -> Result<ModelRoute, String> {
        Ok(ModelRoute {
            name: ROUTE.to_string(),
            route: crate::r#box::Route::Login {
                login: LOGIN.to_string(),
                from: None,
                to: machine.vars.get(ORIGIN_VAR).cloned(),
            },
            secret: None,
        })
    }

    /// The MCP server is `required`, so its connection is the registration
    /// proof; the daemon records the tools it granted with it.
    fn registration_is_connection(&self) -> bool {
        true
    }

    fn argv(&self, launch: &Launch) -> Result<Vec<String>, String> {
        argv(launch)
    }

    fn reader(&self, resumed: bool) -> Box<dyn Reader> {
        Box::new(Normalizer {
            baseline: (!resumed).then_some((Some(0), Some(0))),
            ..Default::default()
        })
    }
}

/// The full argv, run with `/workspace` as its cwd and stdin on /dev/null.
///
/// These overrides pin the required MCP server (the staged config names
/// none), the approval policy, the compaction threshold, the model provider
/// and its route, and switch off what the workspace would load. The bearer
/// reaches the box through the environment, never argv.
fn argv(launch: &Launch) -> Result<Vec<String>, String> {
    check_model(launch.model)?;
    check_prompt(launch.prompt)?;
    let mut argv: Vec<String> = [
        CODEX,
        "exec",
        "--json",
        "-s",
        "danger-full-access",
        "-m",
        launch.model,
    ]
    .map(String::from)
    .into();
    // A fresh `CODEX_HOME` has no persisted hook trust, and the staged hooks
    // are the base's, which the operator committed.
    argv.push("--dangerously-bypass-hook-trust".into());
    let mut overrides = vec![
        format!(
            "mcp_servers={{{MCP_SERVER}={{url={MCP_ENDPOINT:?},bearer_token_env_var={:?},required=true}}}}",
            crate::harness::BEARER_VAR
        ),
        "approval_policy=\"never\"".to_string(),
        // The workspace's AGENTS.md files are the candidate's; the base's
        // guidance is staged in `CODEX_HOME`.
        "project_doc_max_bytes=0".to_string(),
        format!("model_auto_compact_token_limit={COMPACTION_TOKEN_LIMIT}"),
    ];
    // A trusted `/workspace` in the staged config would activate the
    // candidate's own `.codex/config.toml`.
    // Inline table: a dotted key keeps its quotes as part of the name.
    overrides.push("projects={\"/workspace\"={trust_level=\"untrusted\"}}".into());
    // The login route is Yard's: a staged config cannot pick another provider.
    overrides.push(format!("model_provider={PROVIDER:?}"));
    overrides.push(format!(
        "model_providers.{PROVIDER}={{name={PROVIDER:?},base_url={PROVIDER_URL:?},wire_api=\"responses\",requires_openai_auth=false}}"
    ));
    // Codex discovers skills in the workspace whatever its settings say;
    // only a per-file entry turns one off. JSON strings are TOML strings.
    let skills = launch.env.workspace_skills()?;
    if !skills.is_empty() {
        let entries: Vec<String> = skills
            .iter()
            .map(|path| format!("{{path={},enabled=false}}", Value::from(path.as_str())))
            .collect();
        let config = format!("skills.config=[{}]", entries.join(","));
        // One argv entry, under the same per-argument limit as the prompt.
        if config.len() > PROMPT_MAX_BYTES {
            return Err(format!(
                "the workspace's {} skills pass the {PROMPT_MAX_BYTES} byte argument bound",
                skills.len()
            ));
        }
        overrides.push(config);
    }
    if let Some(effort) = launch.effort {
        overrides.push(format!("model_reasoning_effort={effort:?}"));
    }
    for value in overrides {
        argv.push("-c".into());
        argv.push(value);
    }
    if let Some(id) = launch.resume {
        check_session_id(id)?;
        argv.push("resume".into());
        argv.push(id.into());
    }
    // After `--`, so a prompt that looks like an option is still the prompt.
    argv.push("--".into());
    argv.push(launch.prompt.into());
    Ok(argv)
}

fn check_model(model: &str) -> Result<(), String> {
    if model.is_empty()
        || model.len() > MODEL_MAX_BYTES
        || model.starts_with('-')
        || model.chars().any(char::is_control)
    {
        return Err(format!("model {model:?} is not a safe identifier"));
    }
    Ok(())
}

fn check_prompt(prompt: &str) -> Result<(), String> {
    if prompt.is_empty() {
        return Err("prompt is empty".into());
    }
    if prompt.len() > PROMPT_MAX_BYTES {
        return Err(format!(
            "prompt is {} bytes; the maximum is {PROMPT_MAX_BYTES}",
            prompt.len()
        ));
    }
    if prompt.contains('\0') {
        return Err("prompt contains NUL".into());
    }
    Ok(())
}

// A thread id is a UUID-like token. Keep it to the bytes an argv entry can
// carry and that Codex accepts when resuming.
fn check_session_id(id: &str) -> Result<(), String> {
    let ok = !id.is_empty()
        && id.len() <= SESSION_ID_MAX_BYTES
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b));
    if ok {
        Ok(())
    } else {
        Err(format!("session id {id:?} is not one Codex accepts"))
    }
}

/// Normalises one Codex `exec --json` line.
///
/// `thread.started` names the thread, which is the session, and proves the
/// required MCP server connected. `turn.completed` is the outcome; a stream
/// that ends without a terminal turn has decided nothing. A panic exits 0,
/// so the exit status is never the outcome.
#[derive(Default)]
pub struct Normalizer {
    /// A thread has started, so the run is registered and no later stderr
    /// line may downgrade it.
    registered: bool,
    session: Option<String>,
    codex_home: Option<nix::dir::Dir>,
    file: Option<std::fs::File>,
    offset: u64,
    baseline: Option<(Option<u64>, Option<u64>)>,
    observed: Usage,
    terminal: bool,
}

impl Reader for Normalizer {
    fn prepare(&mut self, state: &Path, session: Option<&str>) {
        use nix::dir::Dir;
        use nix::fcntl::OFlag;
        use nix::sys::stat::Mode;
        let flags = OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC;
        self.codex_home = (|| {
            let root = Dir::open(state, flags, Mode::empty()).ok()?;
            let codex = Dir::openat(&root, "codex", flags, Mode::empty()).ok()?;
            // Hold the directory so discovery never follows a replaced path.
            Some(codex)
        })();
        self.session = session.map(str::to_owned);
        if session.is_some() {
            self.find_session();
            let mut baseline = None;
            // Bound history reads; a missing baseline never borrows a row's
            // possibly incomplete execution totals.
            for _ in 0..16 {
                let frames = self.frames();
                if frames.is_empty() {
                    break;
                }
                for frame in frames {
                    if let Some(total) = token_total(&frame) {
                        baseline = Some((total.input, total.output));
                    }
                }
            }
            if self
                .file
                .as_ref()
                .and_then(|file| file.metadata().ok())
                .is_some_and(|metadata| metadata.len() == self.offset)
            {
                self.baseline = baseline;
            }
        }
    }

    fn observe(&mut self) -> Option<Usage> {
        if self.terminal || self.baseline.is_none() {
            return None;
        }
        self.find_session();
        let previous = self.observed.clone();
        for frame in self.frames() {
            if let Some(tier) = frame
                .pointer("/payload/service_tier")
                .and_then(Value::as_str)
            {
                self.observed.fields.service_tier = Some(tier.to_owned());
            }
            if let Some(total) = token_total(&frame) {
                self.record(total);
            }
        }
        (self.observed != previous).then(|| self.observed.clone())
    }

    fn stdout(&mut self, line: &str) -> Vec<Event> {
        let Some(frame) = serde_json::from_str::<Value>(line.trim_end_matches('\r')).ok() else {
            return Vec::new();
        };
        match frame.get("type").and_then(Value::as_str) {
            Some("thread.started") => {
                let Some(id) = frame
                    .get("thread_id")
                    .and_then(Value::as_str)
                    .filter(|id| !id.is_empty())
                else {
                    return vec![Event::Registered(Registration::Refused(
                        "the thread.started frame named no thread".into(),
                    ))];
                };
                self.registered = true;
                if self.session.as_deref().is_some_and(|session| session != id) {
                    self.baseline = None;
                    self.file = None;
                    self.offset = 0;
                }
                self.session = Some(id.into());
                vec![
                    Event::Started {
                        session_id: id.into(),
                    },
                    // Codex lists no tools; the daemon's MCP server records
                    // the connection and the tools it served.
                    Event::Registered(Registration::Registered(Vec::new())),
                ]
            }
            Some("turn.completed" | "turn.failed") => {
                self.observe();
                self.record(usage(&frame));
                self.terminal = true;
                if frame["type"] == "turn.completed" {
                    vec![Event::Finished {
                        usage: self.observed.clone(),
                    }]
                } else {
                    vec![Event::Failed {
                        message: message(&frame),
                        usage: self.observed.clone(),
                    }]
                }
            }
            Some("error") => vec![Event::Failed {
                message: message(&frame),
                usage: self.observed.clone(),
            }],
            _ => Vec::new(),
        }
    }

    /// A JSON error line on stderr is the refusal that ended the run before
    /// a thread started. After `thread.started` the run is registered; a
    /// later stderr line is the harness's own and never downgrades it.
    fn stderr(&mut self, line: &str) -> Option<Registration> {
        if self.registered {
            return None;
        }
        refusal(line)
    }
}

impl Normalizer {
    fn find_session(&mut self) {
        if self.file.is_some() {
            return;
        }
        let (Some(root), Some(session)) = (&self.codex_home, &self.session) else {
            return;
        };
        if check_session_id(session).is_err() {
            return;
        }
        use nix::fcntl::OFlag;
        use nix::sys::stat::Mode;
        if let Ok(mut sessions) = nix::dir::Dir::openat(
            root,
            "sessions",
            OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::empty(),
        ) {
            self.file = session_file(&mut sessions, session, 3, &mut 4096);
        }
    }

    fn frames(&mut self) -> Vec<Value> {
        let Some(file) = &mut self.file else {
            return Vec::new();
        };
        let mut bytes = Vec::new();
        if file.seek(SeekFrom::Start(self.offset)).is_err()
            || file.take(1024 * 1024).read_to_end(&mut bytes).is_err()
        {
            return Vec::new();
        }
        let Some(end) = bytes.iter().rposition(|byte| *byte == b'\n') else {
            return Vec::new();
        };
        let frames: Vec<Value> = bytes[..end]
            .split(|byte| *byte == b'\n')
            .filter_map(|line| serde_json::from_slice(line).ok())
            .collect();
        if self.offset == 0
            && !frames.first().is_some_and(|frame| {
                frame["type"] == "session_meta"
                    && frame["payload"]["id"].as_str() == self.session.as_deref()
            })
        {
            return Vec::new();
        }
        self.offset += (end + 1) as u64;
        frames
    }

    fn record(&mut self, total: Usage) {
        if total.fields.service_tier.is_some() {
            self.observed.fields.service_tier = total.fields.service_tier.clone();
        }
        let Some((input, output)) = self.baseline else {
            return;
        };
        for (value, baseline, observed, field, reported) in [
            (
                total.input,
                input,
                &mut self.observed.input,
                &mut self.observed.fields.input,
                &total.fields.input,
            ),
            (
                total.output,
                output,
                &mut self.observed.output,
                &mut self.observed.fields.output,
                &total.fields.output,
            ),
        ] {
            if let Some(local) = value
                .zip(baseline)
                .and_then(|(value, baseline)| value.checked_sub(baseline))
            {
                *observed = Some(local);
                *field = crate::harness::UsageField {
                    completeness: reported.completeness.clone(),
                    provenance: crate::harness::Provenance::Harness,
                };
            }
        }
    }
}

/// Open every worker-controlled component relative to a held directory.
/// No links, devices or unbounded traversal can redirect a read onto the host.
fn session_file(
    dir: &mut nix::dir::Dir,
    session: &str,
    depth: usize,
    budget: &mut usize,
) -> Option<std::fs::File> {
    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::{Mode, SFlag, fstat};
    let names: Vec<_> = dir
        .iter()
        .take(*budget)
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_owned())
        .collect();
    *budget = budget.saturating_sub(names.len());
    for name in names {
        let bytes = name.to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        let flags = OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC | OFlag::O_NONBLOCK;
        if depth > 0 {
            if let Ok(mut child) = nix::dir::Dir::openat(
                &*dir,
                name.as_c_str(),
                flags | OFlag::O_DIRECTORY,
                Mode::empty(),
            ) && let Some(file) = session_file(&mut child, session, depth - 1, budget)
            {
                return Some(file);
            }
        } else if bytes.starts_with(b"rollout-")
            && bytes.ends_with(format!("-{session}.jsonl").as_bytes())
        {
            let fd = openat(&*dir, name.as_c_str(), flags, Mode::empty()).ok()?;
            if SFlag::from_bits_truncate(fstat(&fd).ok()?.st_mode) == SFlag::S_IFREG {
                return Some(std::fs::File::from(fd));
            }
        }
    }
    None
}

fn token_total(frame: &Value) -> Option<Usage> {
    if frame["type"] != "event_msg" || frame["payload"]["type"] != "token_count" {
        return None;
    }
    let total = frame.pointer("/payload/info/total_token_usage")?;
    Some(usage(&serde_json::json!({"type": "usage", "usage": total})))
}

fn refusal(line: &str) -> Option<Registration> {
    let value: Value = serde_json::from_str(line.trim()).ok()?;
    if value.get("type").and_then(Value::as_str) != Some("error") {
        return None;
    }
    Some(Registration::Refused(message(&value)))
}

/// Codex's usage. The Responses API's `input_tokens` and `output_tokens` are
/// already totals: `cached_input_tokens` and `reasoning_output_tokens` are
/// their subsets, so adding them would double-count. Cost is not reported.
fn usage(frame: &Value) -> Usage {
    let usage = &frame["usage"];
    let input = usage.get("input_tokens").and_then(Value::as_u64);
    let output = usage.get("output_tokens").and_then(Value::as_u64);
    let field = |value: Option<u64>| crate::harness::UsageField {
        completeness: if value.is_some() {
            if frame["type"] == "turn.completed" {
                crate::harness::Completeness::Complete
            } else {
                crate::harness::Completeness::Partial
            }
        } else {
            crate::harness::Completeness::Unknown
        },
        provenance: if value.is_some() {
            crate::harness::Provenance::Harness
        } else {
            crate::harness::Provenance::Unreported
        },
    };
    Usage {
        input,
        output,
        cost: None,
        fields: crate::harness::UsageFields {
            input: field(input),
            output: field(output),
            cost: Default::default(),
            service_tier: frame
                .get("service_tier")
                .and_then(Value::as_str)
                .map(str::to_owned),
        },
    }
}

fn message(frame: &Value) -> String {
    frame
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| frame.get("message").and_then(Value::as_str))
        .filter(|message| !message.is_empty())
        .unwrap_or("the run ended without a result")
        .to_string()
}
