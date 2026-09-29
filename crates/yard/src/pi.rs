//! Pi, the one harness: its launch argv, the files and environment a launch
//! needs, and the normalisation of its JSON frames.
//!
//! Measured against Pi 0.87.1 as pinfold carries it.

use crate::agent_env::AgentEnv;
use crate::config::Agent;
use crate::daemon::Machine;
use crate::harness::{
    Connection, Event, Harness, Launch, ModelRoute, Reader, Registration, Stage, Usage,
};
use serde_json::{Value, json};

/// Yard's MCP client extension. The same bytes for every execution: the
/// endpoint and bearer reach it through the box environment.
pub const EXTENSION: &str = include_str!("pi-mcp-extension.ts");
/// The extension's file name in the daemon-written input directory.
pub const EXTENSION_FILE: &str = "yard-mcp.ts";

const PI: &str = "/opt/pinfold/pi/pi";
const SESSIONS_GUEST: &str = "/yard/state/sessions";
const AGENT_GUEST: &str = "/yard/state/agent";
const HOME_GUEST: &str = "/yard/state/home";
const MCP_ENDPOINT: &str = "http://yard.mcp/mcp";
// Pi needs a non-empty key to consider the provider usable; the injecting
// route replaces the header it produces.
const KEY_PLACEHOLDER: &str = "yard-placeholder";
const SENTINEL: &str = "yard-mcp ";

// The prompt is one argv entry, so Linux's per-argument limit (32 pages)
// less the terminating NUL bounds it.
const PROMPT_MAX_BYTES: usize = 128 * 1024 - 1;
const MODEL_MAX_BYTES: usize = 256;
const SESSION_ID_MAX_BYTES: usize = 128;
// Pi's native automatic-compaction reserve: it compacts once the context
// passes `contextWindow - COMPACTION_RESERVE_TOKENS`, leaving room for the
// response. Staged as the harness's threshold so a worker's own settings do
// not carry into its next execution.
const COMPACTION_RESERVE_TOKENS: u64 = 16384;
// Yard's effort ladder is Pi's `--thinking` ladder word for word. Pi only
// warns on an unknown level and runs at the default, so configuration checks
// it at load.
pub const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

/// The one harness.
pub struct Pi;

impl Harness for Pi {
    fn name(&self) -> &'static str {
        "pi"
    }

    fn version(&self) -> &str {
        crate::harness::pinned(self.name()).unwrap_or("unknown")
    }

    fn accepts(&self, agent: &Agent) -> Result<(), String> {
        if agent.login.is_some() {
            return Err("login is only for a login harness".into());
        }
        let Some(provider) = &agent.provider else {
            return Err("provider is required".into());
        };
        if crate::harness::connection(provider).is_none() {
            return Err(format!("provider {provider:?} names no connection"));
        }
        if let Some(effort) = &agent.effort
            && !EFFORTS.contains(&effort.as_str())
        {
            return Err(format!("effort {effort:?} is unknown"));
        }
        Ok(())
    }

    fn stage(&self, st: &Stage) -> std::io::Result<()> {
        stage_state(st.state, connection(st))?;
        stage_environment(st)?;
        std::fs::create_dir_all(st.input)?;
        std::fs::write(st.input.join(EXTENSION_FILE), EXTENSION)
    }

    fn env(&self, st: &Stage) -> Vec<(String, String)> {
        env(connection(st))
    }

    fn route(&self, st: &Stage, machine: &Machine) -> Result<ModelRoute, String> {
        let connection = connection(st);
        let key = machine.vars.get(connection.key_var).ok_or_else(|| {
            format!(
                "operator.env holds no {} for connection {}",
                connection.key_var, connection.name
            )
        })?;
        Ok(ModelRoute {
            name: connection.route.to_string(),
            route: crate::r#box::Route::Inject {
                to: machine.origin(connection),
                headers: std::collections::BTreeMap::from([(
                    "Authorization".to_string(),
                    crate::r#box::Header {
                        from: connection.key_var.into(),
                        prefix: "Bearer ".into(),
                    },
                )]),
            },
            secret: Some((connection.key_var.to_string(), key.clone())),
        })
    }

    fn argv(&self, launch: &Launch) -> Result<Vec<String>, String> {
        argv(launch)
    }

    fn reader(&self) -> Box<dyn Reader> {
        Box::new(Normalizer::default())
    }
}

/// The provider connection of a provider harness. Configuration refuses a
/// provider harness with no connection, so this holds for every launch.
fn connection(st: &Stage) -> &'static Connection {
    st.connection.expect("a provider harness has a connection")
}

/// The full argv, run with `/workspace` as its cwd and stdin on /dev/null:
/// the pinned build does not exit while its stdin is open.
///
/// Discovery of extensions, skills, context files, prompt templates and themes
/// is off; the staged client and the base's extensions and skills are passed
/// by path, and the base's guidance rides the system prompt.
fn argv(launch: &Launch) -> Result<Vec<String>, String> {
    let Some(provider) = launch.provider else {
        return Err("provider is required".into());
    };
    let Some(connection) = crate::harness::connection(provider) else {
        return Err(format!("unknown connection {provider:?}"));
    };
    check_model(launch.model)?;
    check_prompt(launch.prompt)?;
    let extension = format!("{}/{EXTENSION_FILE}", crate::harness::INPUT_GUEST);
    let mut argv: Vec<String> = [
        PI,
        "--mode",
        "json",
        "--print",
        "--session-dir",
        SESSIONS_GUEST,
        "--no-extensions",
        "--no-skills",
        "--no-context-files",
        "--no-prompt-templates",
        "--no-themes",
        "--offline",
        "--extension",
        &extension,
        "--provider",
        connection.name,
        "--model",
        launch.model,
    ]
    .map(String::from)
    .into();
    for entry in extensions(launch.env) {
        argv.extend(["-e".into(), format!("{AGENT_GUEST}/extensions/{entry}")]);
    }
    // `--skill` on a directory loads every skill below it, discovery or not.
    for (root, name) in SKILL_ROOTS {
        if launch.env.has(root) {
            argv.extend(["--skill".into(), format!("{AGENT_GUEST}/skills/{name}")]);
        }
    }
    if let Some(guidance) = launch.env.guidance() {
        argv.extend(["--append-system-prompt".into(), guidance]);
    }
    if let Some(effort) = launch.effort {
        argv.extend(["--thinking".into(), effort.into()]);
    }
    if let Some(id) = launch.resume {
        check_session_id(id)?;
        argv.extend(["--session-id".into(), id.into()]);
    }
    // After `--`, so a prompt that looks like an option is still the prompt.
    argv.extend(["--".into(), launch.prompt.into()]);
    Ok(argv)
}

const EXTENSIONS_BASE: &str = ".pi/extensions/";
/// The base's skill roots Pi loads, each staged under its own directory.
const SKILL_ROOTS: [(&str, &str); 2] = [(".pi/skills/", "pi"), (".agents/skills/", "agents")];

/// The extensions Pi would discover in the base's `.pi/extensions/`: each
/// direct `.ts` or `.js` file, and each directory's `index.ts` or `index.js`.
/// Pi refuses a directory without an entry, so none is passed.
fn extensions(env: &AgentEnv) -> Vec<String> {
    let has = |path: &str| env.get(&format!("{EXTENSIONS_BASE}{path}")).is_some();
    let mut entries: Vec<String> = env
        .under(EXTENSIONS_BASE)
        .map(|(rest, _)| rest)
        .filter(|rest| match rest.split_once('/') {
            // A directory with both entries loads its `index.ts`, as Pi does.
            Some((dir, "index.js")) => !has(&format!("{dir}/index.ts")),
            Some((_, file)) => file == "index.ts",
            None => rest.ends_with(".ts") || rest.ends_with(".js"),
        })
        .map(String::from)
        .collect();
    entries.sort();
    entries
}

/// Stage the base's extensions and skills into the agent dir, a fresh copy
/// each execution.
fn stage_environment(st: &Stage) -> std::io::Result<()> {
    st.env
        .stage_dir(st.state, "agent/extensions", EXTENSIONS_BASE)?;
    for (root, name) in SKILL_ROOTS {
        st.env
            .stage_dir(st.state, &format!("agent/skills/{name}"), root)?;
    }
    Ok(())
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
    // Pi reads a positional argument starting with `@` as a file to attach.
    if prompt.starts_with('@') {
        return Err("prompt starts with @".into());
    }
    Ok(())
}

// Pi's own rule for `--session-id`: letters, digits, `.`, `_`, `-`, starting
// and ending with a letter or digit.
fn check_session_id(id: &str) -> Result<(), String> {
    let ok = !id.is_empty()
        && id.len() <= SESSION_ID_MAX_BYTES
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id.as_bytes()[id.len() - 1].is_ascii_alphanumeric();
    if ok {
        Ok(())
    } else {
        Err(format!("session id {id:?} is not one Pi accepts"))
    }
}

/// The box env Pi needs, literal values only. The caller adds
/// `BEARER_VAR` as `{"from": BEARER_VAR}`.
fn env(connection: &Connection) -> Vec<(String, String)> {
    [
        ("PI_CODING_AGENT_DIR", AGENT_GUEST),
        ("HOME", HOME_GUEST),
        (connection.key_var, KEY_PLACEHOLDER),
        ("YARD_MCP_ENDPOINT", MCP_ENDPOINT),
        ("PI_TELEMETRY", "0"),
        ("PI_SKIP_VERSION_CHECK", "1"),
    ]
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .into()
}

/// Stage a harness-state directory, keeping what is already there.
/// `agent/models.json` moves the provider's base URL onto its route; every
/// other fact of the provider stays Pi's built-in catalog entry.
/// `agent/settings.json` fixes automatic compaction to Pi's native reserve,
/// so no settings a worker wrote carry into its next execution.
///
/// The worker's box mounts this directory writable, so a link in it can
/// point anywhere on the host. Each write follows none: an `agent` that is
/// not a directory fails, and every staged file is replaced, never written
/// through.
fn stage_state(state: &std::path::Path, connection: &Connection) -> std::io::Result<()> {
    use nix::fcntl::{OFlag, open, openat};
    use nix::sys::stat::Mode;
    use nix::unistd::{UnlinkatFlags, unlinkat};
    use std::io::Write;
    std::fs::create_dir_all(state.join("sessions"))?;
    std::fs::create_dir_all(state.join("home"))?;
    std::fs::create_dir_all(state.join("agent"))?;
    let agent = open(
        &state.join("agent"),
        OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?;
    let replace = |name: &str, value: &Value| -> std::io::Result<()> {
        match unlinkat(&agent, name, UnlinkatFlags::NoRemoveDir) {
            Ok(()) | Err(nix::errno::Errno::ENOENT) => {}
            Err(errno) => return Err(errno.into()),
        }
        let file = openat(
            &agent,
            name,
            OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
            Mode::from_bits_truncate(0o644),
        )?;
        std::fs::File::from(file).write_all(value.to_string().as_bytes())
    };
    let base_url = format!("http://{}{}", connection.route, connection.base_path);
    replace(
        "models.json",
        &json!({ "providers": { connection.name: { "baseUrl": base_url } } }),
    )?;
    replace(
        "settings.json",
        &json!({ "compaction": { "reserveTokens": COMPACTION_RESERVE_TOKENS } }),
    )
}

/// Reads one stderr line. `None` for any line that is not the extension's.
fn registration(stderr_line: &str) -> Option<Registration> {
    let payload = stderr_line
        .trim_end_matches(['\r', '\n'])
        .strip_prefix(SENTINEL)?;
    let value: Value = serde_json::from_str(payload).ok()?;
    match value.get("registered")?.as_bool()? {
        true => {
            let tools = value.get("tools")?.as_array()?;
            let names = tools
                .iter()
                .map(|t| t.as_str().map(String::from))
                .collect::<Option<Vec<_>>>()?;
            Some(Registration::Registered(names))
        }
        false => Some(Registration::Refused(
            value.get("reason")?.as_str()?.to_string(),
        )),
    }
}

struct Final {
    stop_reason: String,
    error: Option<String>,
}

/// Normalises one Pi stdout stream, one line at a time.
///
/// The terminal frame is `agent_settled`, never `agent_end`: Pi runs more
/// passes after an `agent_end` (a retry, a compaction, a queued message). A
/// stream that ends without `agent_settled` has decided nothing.
#[derive(Default)]
pub struct Normalizer {
    last: Option<Final>,
    usage: Usage,
}

impl Reader for Normalizer {
    fn stdout(&mut self, line: &str) -> Vec<Event> {
        let Some(frame) = serde_json::from_str::<Value>(line.trim_end_matches('\r')).ok() else {
            return Vec::new();
        };
        match frame.get("type").and_then(Value::as_str) {
            Some("session") => match frame.get("id").and_then(Value::as_str) {
                Some(id) if !id.is_empty() => vec![Event::Started {
                    session_id: id.into(),
                }],
                _ => Vec::new(),
            },
            Some("message_end") => {
                self.message_end(&frame["message"]);
                Vec::new()
            }
            Some("agent_settled") => vec![self.settled()],
            _ => Vec::new(),
        }
    }

    fn stderr(&mut self, line: &str) -> Option<Registration> {
        registration(line)
    }
}

impl Normalizer {
    fn message_end(&mut self, message: &Value) {
        if message.get("role").and_then(Value::as_str) != Some("assistant") {
            return;
        }
        // Counted here only: `turn_end` repeats the same message.
        let usage = &message["usage"];
        let count = |key: &str| usage.get(key).and_then(Value::as_u64).unwrap_or(0);
        self.usage.input += count("input") + count("cacheRead") + count("cacheWrite");
        self.usage.output += count("output");
        self.usage.cost += usage
            .pointer("/cost/total")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        self.last = Some(Final {
            stop_reason: message
                .get("stopReason")
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
            error: message
                .get("errorMessage")
                .and_then(Value::as_str)
                .filter(|m| !m.is_empty())
                .map(String::from),
        });
    }

    // Settling is not succeeding: only a final message with stopReason "stop"
    // is a finished run. A refused request also exits 0.
    fn settled(&mut self) -> Event {
        let usage = self.usage.clone();
        match self.last.take() {
            Some(last) if last.stop_reason == "stop" => Event::Finished { usage },
            Some(last) => Event::Failed {
                message: last
                    .error
                    .unwrap_or_else(|| format!("settled on a {:?} message", last.stop_reason)),
                usage,
            },
            None => Event::Failed {
                message: "settled without an assistant message".into(),
                usage,
            },
        }
    }
}
