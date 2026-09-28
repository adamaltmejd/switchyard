//! Claude Code, the login harness: its launch argv, the files a launch
//! needs, and the normalisation of its stream-json frames.
//!
//! Measured against Claude Code 2.1.283 as pinfold carries it.

use crate::config::Agent;
use crate::daemon::Machine;
use crate::harness::{
    Event, Harness, Launch, LoginInfo, ModelRoute, Reader, Registration, Stage, Usage,
};
use serde_json::{Value, json};

const CLAUDE: &str = "/opt/pinfold/claude/claude";
const HOME_GUEST: &str = "/yard/state/home";
const MCP_ENDPOINT: &str = "http://yard.mcp/mcp";
/// The staged MCP client, read-only in the box, mounted at INPUT_GUEST.
const MCP_FILE: &str = "yard-mcp.json";
/// The staged settings, read-only in the box.
const SETTINGS_FILE: &str = "yard-claude-settings.json";
/// The server name the staged MCP client registers under.
const MCP_SERVER: &str = "yard";
/// The prefix Claude Code gives an MCP server's tools.
const MCP_PREFIX: &str = "mcp__yard__";
/// The login name pinfold resolves, and the route the box reaches it by.
const LOGIN: &str = "claude";
const ROUTE: &str = "claude.yard";
/// The `operator.env` variable holding the `claude setup-token` token.
pub const TOKEN_VAR: &str = "CLAUDE_CODE_OAUTH_TOKEN";
/// The login's origin override, as `YARD_ORIGIN_<NAME>`.
const ORIGIN_VAR: &str = "YARD_ORIGIN_CLAUDE";

// The prompt is one argv entry, so Linux's per-argument limit (32 pages)
// less the terminating NUL bounds it.
const PROMPT_MAX_BYTES: usize = 128 * 1024 - 1;
const MODEL_MAX_BYTES: usize = 256;

/// Yard's effort ladder is Claude's `--effort` word for word. An unknown
/// level is refused at load.
pub const EFFORTS: [&str; 4] = ["low", "medium", "high", "max"];

/// The one login harness.
pub struct Claude;

impl Harness for Claude {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn version(&self) -> &str {
        crate::harness::pinned(self.name()).unwrap_or("unknown")
    }

    fn login(&self) -> Option<LoginInfo> {
        Some(LoginInfo {
            name: LOGIN,
            key_var: TOKEN_VAR,
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

    fn stage(&self, st: &Stage) -> std::io::Result<()> {
        std::fs::create_dir_all(st.input)?;
        std::fs::write(
            st.input.join(MCP_FILE),
            json!({
                "mcpServers": {
                    MCP_SERVER: {
                        "type": "http",
                        "url": MCP_ENDPOINT,
                        "headers": { "Authorization": format!("Bearer ${{{}}}", crate::harness::BEARER_VAR) },
                    }
                }
            })
            .to_string(),
        )?;
        // The worker's box mounts its state writable, so a link there is the
        // worker's; HOME is created but nothing is written through it.
        std::fs::create_dir_all(st.state.join("home"))?;
        std::fs::write(st.input.join(SETTINGS_FILE), settings().to_string())
    }

    fn env(&self, _st: &Stage) -> Vec<(String, String)> {
        [
            ("HOME", HOME_GUEST),
            // The same pin pinfold sets for this harness, stated by Yard so a
            // run without pinfold's merge still makes no nonessential call.
            ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
        ]
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .into()
    }

    fn route(&self, _st: &Stage, machine: &Machine) -> Result<ModelRoute, String> {
        let token = machine
            .vars
            .get(TOKEN_VAR)
            .ok_or_else(|| format!("operator.env holds no {TOKEN_VAR} for the claude login"))?;
        Ok(ModelRoute {
            name: ROUTE.to_string(),
            route: crate::r#box::Route::Login {
                login: LOGIN.to_string(),
                from: TOKEN_VAR.to_string(),
                to: machine.vars.get(ORIGIN_VAR).cloned(),
            },
            secret: (TOKEN_VAR.to_string(), token.clone()),
        })
    }

    fn argv(&self, launch: &Launch) -> Result<Vec<String>, String> {
        argv(launch)
    }

    fn reader(&self) -> Box<dyn Reader> {
        Box::new(Normalizer)
    }
}

/// The pinned version's automatic-compaction setting, staged so a worker's
/// own settings cannot change it.
fn settings() -> Value {
    json!({ "autoCompactEnabled": true })
}

/// The full argv, run with `/workspace` as its cwd and stdin on /dev/null.
///
/// `--strict-mcp-config` and `--setting-sources user` keep every committed
/// MCP config, setting, hook and plugin from loading; the project's memory
/// files (`CLAUDE.md`) still load from `/workspace`.
fn argv(launch: &Launch) -> Result<Vec<String>, String> {
    check_model(launch.model)?;
    check_prompt(launch.prompt)?;
    let mcp = format!("{}/{MCP_FILE}", crate::harness::INPUT_GUEST);
    let settings = format!("{}/{SETTINGS_FILE}", crate::harness::INPUT_GUEST);
    let mut argv: Vec<String> = [
        CLAUDE,
        "--print",
        "--output-format",
        "stream-json",
        "--verbose",
        "--model",
        launch.model,
        "--mcp-config",
        &mcp,
        "--strict-mcp-config",
        "--setting-sources",
        "user",
        "--settings",
        &settings,
        "--dangerously-skip-permissions",
    ]
    .map(String::from)
    .into();
    if let Some(effort) = launch.effort {
        argv.extend(["--effort".into(), effort.into()]);
    }
    if let Some(id) = launch.resume {
        argv.extend(["--resume".into(), id.into()]);
    }
    // After `--`, so a prompt that looks like an option is still the prompt.
    argv.extend(["--".into(), launch.prompt.into()]);
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

/// Normalises one Claude Code stream-json line.
///
/// The `system`/`init` frame carries the session id and the tools the
/// servers registered; the `result` frame is the outcome. A stream that ends
/// without `result` has decided nothing.
#[derive(Default)]
pub struct Normalizer;

impl Reader for Normalizer {
    fn stdout(&mut self, line: &str) -> Vec<Event> {
        let Some(frame) = serde_json::from_str::<Value>(line.trim_end_matches('\r')).ok() else {
            return Vec::new();
        };
        match frame.get("type").and_then(Value::as_str) {
            Some("system") if frame.get("subtype").and_then(Value::as_str) == Some("init") => {
                let mut events = Vec::new();
                if let Some(id) = frame
                    .get("session_id")
                    .and_then(Value::as_str)
                    .filter(|id| !id.is_empty())
                {
                    events.push(Event::Started {
                        session_id: id.into(),
                    });
                }
                events.push(Event::Registered(registration(&frame)));
                events
            }
            Some("result") => vec![result(&frame)],
            _ => Vec::new(),
        }
    }

    fn stderr(&mut self, _line: &str) -> Option<Registration> {
        None
    }
}

/// The registration proof: a session id, the Yard server connected, and the
/// tools it listed. An absent session or server list is a refusal.
fn registration(frame: &Value) -> Registration {
    if frame
        .get("session_id")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        return Registration::Refused("the init frame named no session".into());
    }
    let Some(servers) = frame.get("mcp_servers").and_then(Value::as_array) else {
        return Registration::Refused("the init frame named no MCP servers".into());
    };
    let connected = servers.iter().any(|server| {
        server.get("name").and_then(Value::as_str) == Some(MCP_SERVER)
            && server.get("status").and_then(Value::as_str) == Some("connected")
    });
    if !connected {
        return Registration::Refused("the Yard MCP server did not connect".into());
    }
    let tools: Vec<String> = frame
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|name| name.strip_prefix(MCP_PREFIX))
        .map(str::to_string)
        .collect();
    Registration::Registered(tools)
}

/// The `result` frame is the outcome; `subtype` success and no `is_error` is
/// a finished run, anything else names the failure.
fn result(frame: &Value) -> Event {
    let usage = usage(frame);
    let ok = frame.get("subtype").and_then(Value::as_str) == Some("success")
        && frame.get("is_error").and_then(Value::as_bool) != Some(true);
    if ok {
        return Event::Finished { usage };
    }
    Event::Failed {
        message: frame
            .get("result")
            .and_then(Value::as_str)
            .filter(|message| !message.is_empty())
            .unwrap_or("the run ended without a result")
            .to_string(),
        usage,
    }
}

fn usage(frame: &Value) -> Usage {
    let usage = &frame["usage"];
    let count = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0);
    Usage {
        input: count("input_tokens")
            + count("cache_read_input_tokens")
            + count("cache_creation_input_tokens"),
        output: count("output_tokens"),
        cost: frame
            .get("total_cost_usd")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
    }
}
