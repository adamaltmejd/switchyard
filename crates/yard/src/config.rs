//! `.yard/config.toml`: one TOML document, read from canonical's target head.
//! Unknown keys are errors and every cross-reference resolves at load.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const CONFIG_PATH: &str = ".yard/config.toml";
pub const DOCKERFILE_PATH: &str = ".yard/Dockerfile";

#[derive(Debug, Clone)]
pub struct Config {
    pub max_lanes: u32,
    pub max_executions: u64,
    pub approve: Approve,
    pub target: Target,
    pub egress: Vec<String>,
    pub agents: BTreeMap<String, Agent>,
    pub workflows: BTreeMap<String, Workflow>,
    /// In declared order.
    pub gates: Vec<Gate>,
    pub review: Review,
    pub seats: BTreeMap<String, Seat>,
    pub profiles: BTreeMap<String, crate::plan::Profile>,
    pub presets: BTreeMap<String, crate::plan::Graph>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Approve {
    #[default]
    Manual,
    Auto,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(rename = "ref")]
    pub branch: String,
    #[serde(default)]
    pub protected_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub harness: String,
    /// The API-key connection; exactly one of `provider` or `login` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// A subscription login, resolved by pinfold's login route. Its presence
    /// is what counts: `login = false` is still set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login: Option<bool>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judgment_inputs: Option<JudgmentInputs>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JudgmentInputs {
    pub configuration: String,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Workflow {
    pub implementer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Seat names in panel order; empty for `none`.
    pub review: Vec<String>,
    pub instructions: String,
    pub read_only: bool,
    pub max_session_executions: u32,
    pub inactivity_timeout_minutes: u64,
    pub total_work_timeout_minutes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Candidate,
    #[default]
    Landing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RunsIn {
    #[default]
    Box,
    Host,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Gate {
    pub name: String,
    pub command: String,
    pub timeout_minutes: u64,
    pub stage: Stage,
    pub runs_in: RunsIn,
    pub env: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_inputs: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Review {
    pub max_rounds: u32,
    pub timeout_minutes: u64,
    /// 0 for P0 through 3 for P3. A finding at or above this blocks.
    pub blocking: u8,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seat {
    pub agent: String,
    #[serde(default)]
    pub instructions: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    max_lanes: Option<u32>,
    max_executions: Option<u64>,
    #[serde(default)]
    approve: Approve,
    target: Target,
    #[serde(default)]
    isolation: RawIsolation,
    #[serde(default)]
    agents: BTreeMap<String, Agent>,
    #[serde(default)]
    workflows: BTreeMap<String, RawWorkflow>,
    #[serde(default)]
    gates: toml::Table,
    #[serde(default)]
    review: RawReview,
    #[serde(default)]
    profiles: BTreeMap<String, crate::plan::Profile>,
    #[serde(default)]
    presets: BTreeMap<String, crate::plan::Graph>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawIsolation {
    #[serde(default)]
    egress: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
struct RawWorkflow {
    profile: Option<String>,
    implementer: Option<String>,
    review: Option<toml::Value>,
    instructions: Option<String>,
    access: Option<String>,
    max_session_executions: Option<u32>,
    inactivity_timeout_minutes: Option<u64>,
    total_work_timeout_minutes: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGate {
    command: String,
    timeout_minutes: Option<u64>,
    #[serde(default)]
    stage: Stage,
    #[serde(default)]
    runs_in: RunsIn,
    #[serde(default)]
    env: Vec<String>,
    closed_inputs: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawReview {
    max_rounds: Option<u32>,
    timeout_minutes: Option<u64>,
    blocking: Option<String>,
    #[serde(default)]
    seats: BTreeMap<String, Seat>,
}

/// A priority token, `P0` to `P3`, as its number.
pub fn priority(token: &str) -> Option<u8> {
    match token {
        "P0" => Some(0),
        "P1" => Some(1),
        "P2" => Some(2),
        "P3" => Some(3),
        _ => None,
    }
}

impl Config {
    pub fn parse(text: &str) -> Result<Config, String> {
        let raw: Raw = toml::from_str(text).map_err(|error| error.message().to_string())?;
        let max_lanes = raw.max_lanes.unwrap_or(1);
        if max_lanes == 0 {
            return Err("max_lanes must be at least 1".into());
        }
        let max_executions = raw.max_executions.unwrap_or(u64::from(max_lanes) + 1);
        if max_executions == 0 {
            return Err("max_executions must be at least 1".into());
        }
        if raw.target.branch.is_empty() || raw.target.branch.starts_with('-') {
            return Err(format!(
                "target.ref {:?} is not a branch",
                raw.target.branch
            ));
        }
        for (name, agent) in &raw.agents {
            if agent.judgment_inputs.as_ref().is_some_and(|inputs| {
                inputs.configuration.trim().is_empty()
                    || inputs.model.trim().is_empty()
                    || agent.login.is_some()
            }) {
                return Err(format!(
                    "agents.{name}.judgment_inputs requires configuration and model identities without login"
                ));
            }
            let Some(harness) = crate::harness::get(&agent.harness) else {
                return Err(format!(
                    "agents.{name}.harness {:?} is unknown; Yard knows {}",
                    agent.harness,
                    crate::harness::names().join(", ")
                ));
            };
            harness
                .accepts(agent)
                .map_err(|error| format!("agents.{name}.{error}"))?;
        }
        let mut seats = BTreeMap::new();
        for (name, seat) in raw.review.seats {
            if !raw.agents.contains_key(&seat.agent) {
                return Err(format!(
                    "review.seats.{name}.agent names unknown agent {:?}",
                    seat.agent
                ));
            }
            seats.insert(name, seat);
        }
        let blocking = raw.review.blocking.as_deref().unwrap_or("P1");
        let blocking = priority(blocking)
            .ok_or_else(|| format!("review.blocking {blocking:?} is not P0 to P3"))?;
        let base = raw
            .workflows
            .get("default")
            .cloned()
            .ok_or("workflows.default is required")?;
        let mut workflows = BTreeMap::new();
        for (name, workflow) in &raw.workflows {
            let pick = |own: &Option<String>, inherited: &Option<String>| {
                own.clone().or_else(|| inherited.clone())
            };
            let implementer = pick(&workflow.implementer, &base.implementer)
                .ok_or_else(|| format!("workflows.{name}.implementer is required"))?;
            if !raw.agents.contains_key(&implementer) {
                return Err(format!(
                    "workflows.{name}.implementer names unknown agent {implementer:?}"
                ));
            }
            let review = match workflow.review.clone().or_else(|| base.review.clone()) {
                None => Vec::new(),
                Some(toml::Value::String(none)) if none == "none" => Vec::new(),
                Some(toml::Value::Array(names)) => {
                    let mut panel = Vec::new();
                    for value in names {
                        let seat = value
                            .as_str()
                            .ok_or_else(|| format!("workflows.{name}.review holds a non-string"))?;
                        if !seats.contains_key(seat) {
                            return Err(format!(
                                "workflows.{name}.review names unknown seat {seat:?}"
                            ));
                        }
                        panel.push(seat.to_string());
                    }
                    panel
                }
                Some(_) => {
                    return Err(format!(
                        "workflows.{name}.review is a list of seats or \"none\""
                    ));
                }
            };
            let read_only = match pick(&workflow.access, &base.access).as_deref() {
                None | Some("write") => false,
                Some("read-only") => true,
                Some(other) => {
                    return Err(format!(
                        "workflows.{name}.access {other:?} is write or read-only"
                    ));
                }
            };
            workflows.insert(
                name.clone(),
                Workflow {
                    implementer,
                    profile: workflow.profile.clone(),
                    review,
                    instructions: pick(&workflow.instructions, &base.instructions)
                        .unwrap_or_default(),
                    read_only,
                    max_session_executions: workflow
                        .max_session_executions
                        .or(base.max_session_executions)
                        .unwrap_or(5)
                        .max(1),
                    inactivity_timeout_minutes: workflow
                        .inactivity_timeout_minutes
                        .or(base.inactivity_timeout_minutes)
                        .unwrap_or(30),
                    total_work_timeout_minutes: workflow
                        .total_work_timeout_minutes
                        .or(base.total_work_timeout_minutes)
                        .unwrap_or(240),
                },
            );
        }
        let mut gates = Vec::new();
        for (name, value) in raw.gates {
            let gate: RawGate = value
                .try_into()
                .map_err(|error: toml::de::Error| format!("gates.{name}: {}", error.message()))?;
            for variable in &gate.env {
                if !valid_env_name(variable) {
                    return Err(format!("gates.{name}.env names {variable:?}"));
                }
            }
            if gate
                .closed_inputs
                .as_ref()
                .is_some_and(|identity| identity.trim().is_empty())
            {
                return Err(format!(
                    "gates.{name}.closed_inputs must name declared runtime configuration"
                ));
            }
            gates.push(Gate {
                name,
                command: gate.command,
                timeout_minutes: gate.timeout_minutes.unwrap_or(30),
                stage: gate.stage,
                runs_in: gate.runs_in,
                env: gate.env,
                closed_inputs: gate.closed_inputs,
            });
        }
        let config = Config {
            max_lanes,
            max_executions,
            approve: raw.approve,
            target: raw.target,
            egress: raw.isolation.egress,
            agents: raw.agents,
            workflows,
            gates,
            review: Review {
                max_rounds: raw.review.max_rounds.unwrap_or(3).max(1),
                timeout_minutes: raw.review.timeout_minutes.unwrap_or(30),
                blocking,
            },
            seats,
            profiles: raw.profiles,
            presets: raw.presets,
        };
        for (name, profile) in &config.profiles {
            crate::plan::Plan {
                classifiers: BTreeMap::new(),
                inputs: BTreeMap::new(),
                nodes: BTreeMap::from([(name.clone(), profile.node())]),
            }
            .validate(&config)
            .map_err(|error| format!("profiles.{name}: {error}"))?;
        }
        for (name, graph) in &config.presets {
            config
                .resolve_graph(graph)
                .map_err(|error| format!("presets.{name}: {error}"))?;
        }
        for (name, workflow) in &config.workflows {
            if workflow.profile.is_some() {
                let plan = config.workflow_plan(name)?;
                plan.validate(&config)
                    .map_err(|error| format!("workflows.{name}.profile: {error}"))?;
                if !workflow.read_only
                    || !workflow.review.is_empty()
                    || !matches!(plan.packaged(), Some(crate::plan::Runner::Packaged { agent, .. }) if agent == &workflow.implementer)
                {
                    return Err(format!(
                        "workflows.{name}.profile requires a single packaged audit"
                    ));
                }
            }
        }
        Ok(config)
    }

    /// The gate digest: the Dockerfile and every gate as declared.
    pub fn gate_digest(&self, dockerfile: &[u8]) -> String {
        let gates = serde_json::to_vec(&self.gates).expect("gates serialize");
        digest(&[b"gates", dockerfile, &gates])
    }

    /// The review digest for one workflow: its panel, each seat's agent
    /// settings and instructions, and `review.blocking`.
    pub fn review_digest(&self, workflow: &Workflow) -> String {
        let panel: Vec<_> = workflow
            .review
            .iter()
            .map(|seat| {
                let seat_config = &self.seats[seat];
                serde_json::json!({
                    "seat": seat,
                    "agent": self.agents[&seat_config.agent],
                    "instructions": seat_config.instructions,
                })
            })
            .collect();
        let panel = serde_json::to_vec(&panel).expect("panel serializes");
        digest(&[b"review", &panel, &[self.review.blocking]])
    }

    pub fn workflow(&self, name: &str) -> Result<&Workflow, String> {
        self.workflows
            .get(name)
            .ok_or_else(|| format!("workflow {name:?} is not in the configuration"))
    }

    /// Whether `path` falls under a protected path. A path ending in `/`
    /// protects everything below it.
    pub fn is_protected(&self, path: &str) -> bool {
        self.target.protected_paths.iter().any(|protected| {
            if protected.ends_with('/') {
                path.starts_with(protected.as_str())
            } else {
                path == protected || path.starts_with(&format!("{protected}/"))
            }
        })
    }
}

pub(crate) fn digest(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hex(&hasher.finalize())
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The scaffold `yard init` writes.
pub const SCAFFOLD_CONFIG: &str = include_str!("scaffold/config.toml");

pub const SCAFFOLD_DOCKERFILE: &str = "FROM pinfold/profile-default:latest\n";
