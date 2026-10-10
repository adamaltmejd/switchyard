//! Claude Code: its launch argv, the files a launch
//! needs, and the normalisation of its stream-json frames.
//!
//! Measured against Claude Code 2.1.283 as pinfold carries it.

use crate::config::Agent;
use crate::daemon::Machine;
use crate::harness::{
    Completeness, Event, Harness, Launch, LoginInfo, ModelRoute, Provenance, Reader, Registration,
    Stage, Usage, UsageField, UsageFields,
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

/// The Claude harness, using a subscription login or direct Anthropic API.
pub struct Claude;

impl Harness for Claude {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn login(&self) -> Option<LoginInfo> {
        Some(LoginInfo {
            name: LOGIN,
            key_var: Some(TOKEN_VAR),
        })
    }

    fn accepts(&self, agent: &Agent) -> Result<(), String> {
        match (agent.provider.as_deref(), agent.login) {
            (Some(_), Some(_)) => return Err("provider_login_conflict".into()),
            (Some("anthropic"), None) | (None, Some(true)) => {}
            (Some(provider), None) => {
                return Err(format!(
                    "provider {provider:?} is not accepted; use anthropic"
                ));
            }
            (None, _) => return Err("provider = anthropic or login = true is required".into()),
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
        // The base's skills and settings are Claude's user level; its own
        // settings ride `--settings`, which outranks them. Claude reads no
        // `.agents/skills`, so those join its own under the same names.
        st.env.stage_dir(
            st.state,
            "home/.claude/skills",
            &[".claude/skills/", ".agents/skills/"],
        )?;
        st.env.stage_file(
            st.state,
            "home/.claude/settings.json",
            ".claude/settings.json",
        )?;
        std::fs::write(st.input.join(SETTINGS_FILE), settings().to_string())
    }

    fn env(&self, st: &Stage) -> Vec<(String, String)> {
        let mut env: Vec<_> = [
            ("HOME", HOME_GUEST),
            // The same pin pinfold sets for this harness, stated by Yard so a
            // run without pinfold's merge still makes no nonessential call.
            ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
        ]
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .into();
        if let Some(connection) = st.connection {
            env.extend([
                (connection.key_var.into(), "yard-placeholder".into()),
                (
                    "ANTHROPIC_BASE_URL".into(),
                    format!("http://{}", connection.route),
                ),
            ]);
        }
        env
    }

    fn route(&self, st: &Stage, machine: &Machine) -> Result<ModelRoute, String> {
        if let Some(connection) = st.connection {
            let key = machine.vars.get(connection.key_var).ok_or_else(|| {
                format!(
                    "operator.env holds no {} for connection {}",
                    connection.key_var, connection.name
                )
            })?;
            return Ok(ModelRoute {
                name: connection.route.into(),
                route: crate::r#box::Route::Inject {
                    to: machine.origin(connection),
                    headers: std::collections::BTreeMap::from([(
                        "x-api-key".into(),
                        crate::r#box::Header {
                            from: connection.key_var.into(),
                            prefix: String::new(),
                        },
                    )]),
                },
                secret: Some((connection.key_var.into(), key.clone())),
            });
        }
        let token = machine
            .vars
            .get(TOKEN_VAR)
            .ok_or_else(|| format!("operator.env holds no {TOKEN_VAR} for the claude login"))?;
        Ok(ModelRoute {
            name: ROUTE.to_string(),
            route: crate::r#box::Route::Login {
                login: LOGIN.to_string(),
                from: Some(TOKEN_VAR.to_string()),
                to: machine.vars.get(ORIGIN_VAR).cloned(),
            },
            secret: Some((TOKEN_VAR.to_string(), token.clone())),
        })
    }

    fn argv(&self, launch: &Launch) -> Result<Vec<String>, String> {
        argv(launch)
    }

    fn reader(&self, resumed: bool) -> Box<dyn Reader> {
        Box::new(Normalizer {
            resumed,
            ..Normalizer::default()
        })
    }
}

/// Yard's own settings, the highest layer: automatic compaction, so a
/// worker's settings cannot change it.
fn settings() -> Value {
    json!({ "autoCompactEnabled": true })
}

/// The full argv, run with `/workspace` as its cwd and stdin on /dev/null.
///
/// `--strict-mcp-config` and `--setting-sources user` keep every
/// workspace MCP config, setting, hook and plugin from loading. The base's
/// guidance reaches the model on the system prompt; Claude's own
/// project-memory loading is off with the setting sources.
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
    if let Some(guidance) = launch.env.guidance() {
        argv.extend(["--append-system-prompt".into(), guidance]);
    }
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
pub struct Normalizer {
    messages: std::collections::BTreeMap<String, Usage>,
    resumed: bool,
}

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
            Some("assistant")
                if frame.get("isApiErrorMessage").and_then(Value::as_bool) != Some(true) =>
            {
                let message = &frame["message"];
                let Some(id) = message.get("id").and_then(Value::as_str) else {
                    return Vec::new();
                };
                // Content blocks can repeat a message id. Replace its observation,
                // rather than counting the same API response twice.
                self.messages.insert(id.into(), usage(message, false));
                vec![Event::Usage {
                    usage: self.observed(),
                }]
            }
            Some("result") => vec![result(&frame, self.observed(), self.resumed)],
            _ => Vec::new(),
        }
    }

    fn stderr(&mut self, _line: &str) -> Option<Registration> {
        None
    }
}

impl Normalizer {
    fn observed(&self) -> Usage {
        let mut observed = Usage::default();
        for message in self.messages.values() {
            if let Some(input) = message.input {
                observed.input = Some(observed.input.unwrap_or(0) + input);
                observed.fields.input = field(Completeness::Partial);
            }
            if let Some(output) = message.output {
                observed.output = Some(observed.output.unwrap_or(0) + output);
                observed.fields.output = field(Completeness::Partial);
            }
        }
        observed
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
fn result(frame: &Value, observed: Usage, resumed: bool) -> Event {
    let ok = frame.get("subtype").and_then(Value::as_str) == Some("success")
        && frame.get("is_error").and_then(Value::as_bool) != Some(true);
    let mut usage = usage(frame, ok);
    // The pin restores session cost, but result tokens cover only this run.
    // No execution-local baseline is reported on resume. A missing earlier
    // cost or interrupted run cannot be priced by subtracting stored values.
    if resumed {
        usage.cost = None;
        usage.fields.cost = UsageField::default();
    }
    if usage.input.is_none() {
        usage.input = observed.input;
        usage.fields.input = observed.fields.input;
    }
    if usage.output.is_none() {
        usage.output = observed.output;
        usage.fields.output = observed.fields.output;
    }
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

// Result token totals cover this invocation, including on resume. They replace
// streamed observations. Result cost is session-cumulative, handled in result.
fn usage(frame: &Value, complete: bool) -> Usage {
    let values = &frame["usage"];
    let counts = [
        "input_tokens",
        "cache_read_input_tokens",
        "cache_creation_input_tokens",
    ]
    .map(|key| values.get(key).and_then(Value::as_u64));
    let input = counts.iter().flatten().copied().reduce(|a, b| a + b);
    let output = values.get("output_tokens").and_then(Value::as_u64);
    let cost = frame
        .get("total_cost_usd")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value >= 0.0);
    let completeness = |present: bool, covered: bool| {
        if !present {
            UsageField::default()
        } else {
            field(if complete && covered {
                Completeness::Complete
            } else {
                Completeness::Partial
            })
        }
    };
    Usage {
        input,
        output,
        cost,
        fields: UsageFields {
            input: completeness(input.is_some(), counts.iter().all(Option::is_some)),
            output: completeness(output.is_some(), true),
            cost: completeness(cost.is_some(), true),
        },
    }
}

fn field(completeness: Completeness) -> UsageField {
    UsageField {
        completeness,
        provenance: Provenance::Harness,
    }
}
