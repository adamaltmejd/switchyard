//! Pi, the one harness: its launch argv, the files and environment a launch
//! needs, and the normalisation of its JSON frames.
//!
//! Measured against Pi 0.87.1 as pinfold carries it.

use serde_json::{Value, json};

/// Yard's MCP client extension. The same bytes for every execution: the
/// endpoint and bearer reach it through the box environment.
pub const EXTENSION: &str = include_str!("pi-mcp-extension.ts");
/// The extension's file name in the daemon-written input directory.
pub const EXTENSION_FILE: &str = "yard-mcp.ts";
/// Where the attempt's harness state is mounted, writable.
pub const STATE_GUEST: &str = "/yard/state";
/// Where the daemon-written input directory is mounted, read-only.
pub const INPUT_GUEST: &str = "/yard/input";
/// The route name that reaches the daemon's MCP listener.
pub const MCP_ROUTE: &str = "yard.mcp";
/// The variable the extension reads its bearer from. The caller sets it in
/// the box spec as `{"from": BEARER_VAR}`, never as a literal.
pub const BEARER_VAR: &str = "YARD_MCP_BEARER";

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
// Yard's effort ladder is Pi's `--thinking` ladder word for word. Pi only
// warns on an unknown level and runs at the default, so configuration checks
// it at load.
pub const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

/// An upstream a model request reaches through an injecting route.
#[derive(Debug)]
pub struct Connection {
    /// Pi's provider name, and the connection's name in configuration.
    pub name: &'static str,
    /// The route name the box reaches it by.
    pub route: &'static str,
    /// Where the route forwards on the host.
    pub origin: &'static str,
    /// The provider's API path under both origin and route.
    pub base_path: &'static str,
    /// The variable Pi reads the key from; the box holds a placeholder.
    pub key_var: &'static str,
}

pub const CONNECTIONS: &[Connection] = &[
    Connection {
        name: "openrouter",
        route: "openrouter.yard",
        origin: "https://openrouter.ai",
        base_path: "/api/v1",
        key_var: "OPENROUTER_API_KEY",
    },
    Connection {
        name: "opencode-go",
        route: "opencode-go.yard",
        origin: "https://opencode.ai",
        base_path: "/zen/go/v1",
        key_var: "OPENCODE_API_KEY",
    },
];

pub fn connection(name: &str) -> Option<&'static Connection> {
    CONNECTIONS.iter().find(|c| c.name == name)
}

/// One Pi run. `resume` is a session id a previous run reported.
#[derive(Debug)]
pub struct Launch<'a> {
    pub provider: &'a str,
    pub model: &'a str,
    pub effort: Option<&'a str>,
    pub resume: Option<&'a str>,
    pub prompt: &'a str,
}

/// The full argv, run with `/workspace` as its cwd and stdin on /dev/null:
/// the pinned build does not exit while its stdin is open.
///
/// Discovery of extensions, prompt templates and themes is off, so the staged
/// client is the only code that loads. Context files and skills stay on.
pub fn argv(launch: &Launch) -> Result<Vec<String>, String> {
    let Some(connection) = connection(launch.provider) else {
        return Err(format!("unknown connection {:?}", launch.provider));
    };
    check_model(launch.model)?;
    check_prompt(launch.prompt)?;
    let extension = format!("{INPUT_GUEST}/{EXTENSION_FILE}");
    let mut argv: Vec<String> = [
        PI,
        "--mode",
        "json",
        "--print",
        "--session-dir",
        SESSIONS_GUEST,
        "--no-extensions",
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
pub fn env(connection: &Connection) -> Vec<(String, String)> {
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
///
/// The worker's box mounts this directory writable, so a link in it can
/// point anywhere on the host. The write follows none: an `agent` that is
/// not a directory fails, and `models.json` is replaced, never written
/// through.
pub fn stage_state(state: &std::path::Path, connection: &Connection) -> std::io::Result<()> {
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
    match unlinkat(&agent, "models.json", UnlinkatFlags::NoRemoveDir) {
        Ok(()) | Err(nix::errno::Errno::ENOENT) => {}
        Err(errno) => return Err(errno.into()),
    }
    let file = openat(
        &agent,
        "models.json",
        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::from_bits_truncate(0o644),
    )?;
    let base_url = format!("http://{}{}", connection.route, connection.base_path);
    let models = json!({ "providers": { connection.name: { "baseUrl": base_url } } });
    std::fs::File::from(file).write_all(models.to_string().as_bytes())
}

#[derive(Debug, Clone, PartialEq)]
pub enum Registration {
    /// The tool names the server listed, in its order.
    Registered(Vec<String>),
    Refused(String),
}

/// Reads one stderr line. `None` for any line that is not the extension's.
pub fn registration(stderr_line: &str) -> Option<Registration> {
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Usage {
    /// Prompt tokens, cache reads and writes included.
    pub input: u64,
    pub output: u64,
    /// USD, as Pi prices it.
    pub cost: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Started { session_id: String },
    Finished { usage: Usage },
    Failed { message: String, usage: Usage },
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

impl Normalizer {
    pub fn frame(&mut self, line: &str) -> Option<Event> {
        let frame = serde_json::from_str::<Value>(line.trim_end_matches('\r')).ok()?;
        match frame.get("type").and_then(Value::as_str) {
            Some("session") => match frame.get("id").and_then(Value::as_str) {
                Some(id) if !id.is_empty() => Some(Event::Started {
                    session_id: id.into(),
                }),
                _ => None,
            },
            Some("message_end") => {
                self.message_end(&frame["message"]);
                None
            }
            Some("agent_settled") => Some(self.settled()),
            _ => None,
        }
    }

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
