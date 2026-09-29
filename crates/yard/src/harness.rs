//! The harness adapter interface and the compiled-in registry.
//!
//! One adapter per harness: its launch argv, launch files, box env, model
//! route, frame normalisation and registration proof. Pi is the one harness.

use crate::r#box::Route;
use crate::config::Agent;
use crate::daemon::Machine;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

/// Where the attempt's harness state is mounted, writable.
pub const STATE_GUEST: &str = "/yard/state";
/// Where the daemon-written input directory is mounted, read-only.
pub const INPUT_GUEST: &str = "/yard/input";
/// The route name that reaches the daemon's MCP listener.
pub const MCP_ROUTE: &str = "yard.mcp";
/// The variable a staged client reads its bearer from. The caller sets it in
/// the box spec as `{"from": BEARER_VAR}`, never as a literal.
pub const BEARER_VAR: &str = "YARD_MCP_BEARER";

/// An upstream a model request reaches through an injecting route.
#[derive(Debug)]
pub struct Connection {
    /// The harness's provider name, and the connection's name in configuration.
    pub name: &'static str,
    /// The route name the box reaches it by.
    pub route: &'static str,
    /// Where the route forwards on the host.
    pub origin: &'static str,
    /// The provider's API path under both origin and route.
    pub base_path: &'static str,
    /// The variable the harness reads the key from; the box holds a placeholder.
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

/// One harness run. `resume` is a session id a previous run reported.
#[derive(Debug)]
pub struct Launch<'a> {
    /// The provider connection for an API-key harness; `None` for a login.
    pub provider: Option<&'a str>,
    pub model: &'a str,
    pub effort: Option<&'a str>,
    pub resume: Option<&'a str>,
    pub prompt: &'a str,
    /// Project guidance a harness's CLI does not load from the workspace, to
    /// append to its system prompt. `None` for a harness that loads it itself.
    pub guidance: Option<&'a str>,
}

/// Where a launch writes its files and the connection it uses.
#[derive(Debug, Clone, Copy)]
pub struct Stage<'a> {
    /// The attempt's harness state, writable in the box.
    pub state: &'a Path,
    /// The daemon-written input directory, read-only in the box.
    pub input: &'a Path,
    /// The provider connection of a `provider` agent; `None` for a login.
    pub connection: Option<&'static Connection>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Usage {
    /// Prompt tokens, cache reads and writes included.
    pub input: u64,
    pub output: u64,
    /// USD, as the harness prices it.
    pub cost: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Started {
        session_id: String,
    },
    /// The harness's registration proof, when it arrives on stdout.
    Registered(Registration),
    Finished {
        usage: Usage,
    },
    Failed {
        message: String,
        usage: Usage,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Registration {
    /// The tool names the server listed, in its order.
    Registered(Vec<String>),
    Refused(String),
}

/// A per-run reader over the harness's stdout and stderr.
pub trait Reader: Send {
    /// One stdout line, normalised. Empty for a line that decides nothing.
    fn stdout(&mut self, line: &str) -> Vec<Event>;
    /// One stderr line, if it is the registration line.
    fn stderr(&mut self, line: &str) -> Option<Registration>;
}

/// The model route and the secret `box up` reads for it.
pub struct ModelRoute {
    /// The route name in the box spec.
    pub name: String,
    /// The injecting route the box reaches.
    pub route: Route,
    /// The secret `box up` reads, `(variable, value)`.
    pub secret: (String, String),
}

/// A subscription login a harness reaches through a pinfold login route.
pub struct LoginInfo {
    /// The login's name, as pinfold knows it.
    pub name: &'static str,
    /// The `operator.env` variable holding the token.
    pub key_var: &'static str,
}

/// One harness adapter.
pub trait Harness: Send + Sync {
    /// The config token and the pinfold artifact name.
    fn name(&self) -> &'static str;
    /// The version pinfold carries for this harness.
    fn version(&self) -> &str;
    /// The login this harness uses, if it is a subscription-login harness.
    fn login(&self) -> Option<LoginInfo> {
        None
    }
    /// Whether the adapter has a control for every key the agent sets.
    fn accepts(&self, agent: &Agent) -> Result<(), String>;
    /// The project guidance a harness needs passed explicitly because its CLI
    /// loads none from the workspace. `workspace` is the host path the box
    /// mounts at `/workspace`; a harness that loads it itself returns `None`.
    fn guidance(&self, _workspace: &Path) -> Result<Option<String>, String> {
        Ok(None)
    }
    /// Write the launch files into the attempt's state and input dirs.
    fn stage(&self, st: &Stage) -> std::io::Result<()>;
    /// The literal box env. The caller adds the MCP bearer.
    fn env(&self, st: &Stage) -> Vec<(String, String)>;
    /// The model route and the secret `box up` reads for it.
    fn route(&self, st: &Stage, machine: &Machine) -> Result<ModelRoute, String>;
    /// Start or resume, the prompt as one argument.
    fn argv(&self, launch: &Launch) -> Result<Vec<String>, String>;
    /// A reader over stdout and stderr for one run.
    fn reader(&self) -> Box<dyn Reader>;
}

/// The compiled-in adapters. `name` is the config token.
pub fn adapters() -> &'static [&'static dyn Harness] {
    &[&crate::pi::Pi, &crate::claude::Claude]
}

pub fn get(name: &str) -> Option<&'static dyn Harness> {
    adapters()
        .iter()
        .copied()
        .find(|harness| harness.name() == name)
}

pub fn names() -> Vec<&'static str> {
    adapters().iter().map(|harness| harness.name()).collect()
}

/// The pins `pinfold artifacts` reported, read once per daemon.
static PINS: OnceLock<BTreeMap<String, String>> = OnceLock::new();

/// Cache the pins, refusing a set that omits any adapter. A daemon with an
/// unread pin could record `unknown` for two different versions and resume
/// across a version change, so it does not serve without one.
pub fn pin(pins: BTreeMap<String, String>) -> Result<(), String> {
    if let Some(missing) = adapters()
        .iter()
        .map(|harness| harness.name())
        .find(|name| !pins.contains_key(*name))
    {
        return Err(format!("pinfold carries no {missing}"));
    }
    let _ = PINS.set(pins);
    Ok(())
}

/// The version pinfold carries for `name`, once the daemon has read it.
pub fn pinned(name: &str) -> Option<&'static str> {
    PINS.get()?.get(name).map(String::as_str)
}
