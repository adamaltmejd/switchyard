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

    fn version(&self) -> &str {
        crate::harness::pinned(self.name()).unwrap_or("unknown")
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

    fn reader(&self) -> Box<dyn Reader> {
        Box::new(Normalizer::default())
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
        overrides.push(format!("skills.config=[{}]", entries.join(",")));
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
}

impl Reader for Normalizer {
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
                vec![
                    Event::Started {
                        session_id: id.into(),
                    },
                    // Codex lists no tools; the daemon's MCP server records
                    // the connection and the tools it served.
                    Event::Registered(Registration::Registered(Vec::new())),
                ]
            }
            Some("turn.completed") => vec![Event::Finished {
                usage: usage(&frame),
            }],
            Some("turn.failed") => vec![Event::Failed {
                message: message(&frame),
                usage: usage(&frame),
            }],
            Some("error") => vec![Event::Failed {
                message: message(&frame),
                usage: Usage::default(),
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
    let count = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0);
    Usage {
        input: count("input_tokens"),
        output: count("output_tokens"),
        cost: 0.0,
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
