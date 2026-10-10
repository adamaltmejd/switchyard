//! Typed inspection only. No plan here admits work or grants worker tools.
use crate::config::{Config, Stage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize)]
pub struct Invalid {
    pub input: String,
    pub reason: &'static str,
}

impl Invalid {
    fn new(input: impl Into<String>, reason: &'static str) -> Self {
        Self {
            input: input.into(),
            reason,
        }
    }
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.input, self.reason)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    ReadOnly,
    WriteAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Output {
    Candidate,
    Report,
    Verdict,
    Selection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Runner {
    Agent { agent: String, instructions: String },
    Command { gate: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub runner: Runner,
    pub access: Access,
    pub outputs: BTreeMap<String, Output>,
    pub outcomes: Vec<String>,
}

impl Profile {
    pub fn node(&self) -> Node {
        Node {
            runner: self.runner.clone(),
            access: self.access.clone(),
            outputs: self.outputs.clone(),
            outcomes: self.outcomes.clone(),
            inputs: BTreeMap::new(),
            routes: BTreeMap::new(),
            requires: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Absent for a declared external input.
    pub node: Option<String>,
    pub output: String,
    pub kind: Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub to: String,
    /// Only these edges may close a cycle.
    pub repeat: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub runner: Runner,
    pub access: Access,
    #[serde(default)]
    pub inputs: BTreeMap<String, Input>,
    pub outputs: BTreeMap<String, Output>,
    pub outcomes: Vec<String>,
    #[serde(default)]
    pub routes: BTreeMap<String, Vec<Route>>,
    /// Input names whose results must all be present at a join.
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSpec {
    pub profile: Option<String>,
    pub runner: Option<Runner>,
    pub access: Option<Access>,
    pub inputs: Option<BTreeMap<String, Input>>,
    pub outputs: Option<BTreeMap<String, Output>>,
    pub outcomes: Option<Vec<String>>,
    pub routes: Option<BTreeMap<String, Vec<Route>>>,
    pub requires: Option<Vec<String>>,
}

impl NodeSpec {
    fn resolve(&self, base: Option<&Node>, path: &str) -> Result<Node, Invalid> {
        macro_rules! pick {
            ($field:ident) => {
                self.$field
                    .clone()
                    .or_else(|| base.map(|n| n.$field.clone()))
                    .ok_or_else(|| {
                        Invalid::new(format!("{path}.{}", stringify!($field)), "required")
                    })?
            };
        }
        Ok(Node {
            runner: pick!(runner),
            access: pick!(access),
            outputs: pick!(outputs),
            outcomes: pick!(outcomes),
            inputs: self
                .inputs
                .clone()
                .or_else(|| base.map(|n| n.inputs.clone()))
                .unwrap_or_default(),
            routes: self
                .routes
                .clone()
                .or_else(|| base.map(|n| n.routes.clone()))
                .unwrap_or_default(),
            requires: self
                .requires
                .clone()
                .or_else(|| base.map(|n| n.requires.clone()))
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(default)]
    pub inputs: BTreeMap<String, Output>,
    pub nodes: BTreeMap<String, NodeSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub inputs: BTreeMap<String, Output>,
    pub nodes: BTreeMap<String, Node>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(rename = "project")]
    pub _project: String,
    pub target: Option<String>,
    pub preset: Option<String>,
    pub workflow: Option<String>,
    pub graph: Option<Graph>,
    #[serde(default)]
    pub overrides: BTreeMap<String, NodeSpec>,
}

impl Config {
    pub fn resolve_plan(&self, request: &Request) -> Result<Plan, Invalid> {
        let count = usize::from(request.graph.is_some())
            + usize::from(request.preset.is_some())
            + usize::from(request.workflow.is_some());
        if count > 1 {
            return Err(Invalid::new("graph,preset,workflow", "exclusive_selection"));
        }
        let mut plan = if let Some(graph) = &request.graph {
            self.resolve_graph(graph)?
        } else if let Some(name) = &request.preset {
            self.resolve_graph(
                self.presets
                    .get(name)
                    .ok_or_else(|| Invalid::new("preset", "unresolved_reference"))?,
            )?
        } else {
            self.workflow_plan(request.workflow.as_deref().unwrap_or("default"))
                .map_err(|_| Invalid::new("workflow", "unresolved_reference"))?
        };
        for (name, patch) in &request.overrides {
            let old = plan
                .nodes
                .get(name)
                .ok_or_else(|| Invalid::new(format!("overrides.{name}"), "unresolved_reference"))?;
            let mut base = old.clone();
            if let Some(profile) = &patch.profile {
                let profile = self.profiles.get(profile).ok_or_else(|| {
                    Invalid::new(format!("overrides.{name}.profile"), "unresolved_reference")
                })?;
                base.runner = profile.runner.clone();
                base.access = profile.access.clone();
                base.outputs = profile.outputs.clone();
                base.outcomes = profile.outcomes.clone();
            }
            let node = patch.resolve(Some(&base), &format!("overrides.{name}"))?;
            plan.nodes.insert(name.clone(), node);
        }
        plan.validate(self)?;
        Ok(plan)
    }

    pub fn resolve_graph(&self, graph: &Graph) -> Result<Plan, Invalid> {
        let mut nodes = BTreeMap::new();
        for (name, spec) in &graph.nodes {
            let base = spec
                .profile
                .as_ref()
                .map(|profile| {
                    self.profiles.get(profile).ok_or_else(|| {
                        Invalid::new(format!("nodes.{name}.profile"), "unresolved_reference")
                    })
                })
                .transpose()?
                .map(Profile::node);
            nodes.insert(
                name.clone(),
                spec.resolve(base.as_ref(), &format!("nodes.{name}"))?,
            );
        }
        let plan = Plan {
            inputs: graph.inputs.clone(),
            nodes,
        };
        plan.validate(self)?;
        Ok(plan)
    }

    /// A descriptive chain of the existing worker, gates and panel. It does
    /// not replace admission, repair, approval or landing policy.
    pub fn workflow_plan(&self, name: &str) -> Result<Plan, String> {
        let workflow = self.workflow(name)?;
        let mut chain = vec![(
            "implementer".to_string(),
            Node {
                runner: Runner::Agent {
                    agent: workflow.implementer.clone(),
                    instructions: workflow.instructions.clone(),
                },
                access: if workflow.read_only {
                    Access::ReadOnly
                } else {
                    Access::WriteAllowed
                },
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([(
                    "result".into(),
                    if workflow.read_only {
                        Output::Report
                    } else {
                        Output::Candidate
                    },
                )]),
                outcomes: vec!["complete".into()],
                routes: BTreeMap::new(),
                requires: Vec::new(),
            },
        )];
        if !workflow.read_only {
            for stage in [Stage::Candidate, Stage::Landing] {
                if stage == Stage::Landing {
                    for (index, seat) in workflow.review.iter().enumerate() {
                        let seat = &self.seats[seat];
                        chain.push((
                            format!("review-{index}"),
                            Node {
                                runner: Runner::Agent {
                                    agent: seat.agent.clone(),
                                    instructions: seat.instructions.clone(),
                                },
                                access: Access::ReadOnly,
                                inputs: BTreeMap::new(),
                                outputs: BTreeMap::from([("result".into(), Output::Verdict)]),
                                outcomes: vec!["complete".into()],
                                routes: BTreeMap::new(),
                                requires: Vec::new(),
                            },
                        ));
                    }
                }
                for gate in self
                    .gates
                    .iter()
                    .filter(|g| stage == Stage::Landing || g.stage == Stage::Candidate)
                {
                    chain.push((
                        format!(
                            "{}-gate-{}",
                            if stage == Stage::Candidate {
                                "candidate"
                            } else {
                                "landing"
                            },
                            gate.name
                        ),
                        Node {
                            runner: Runner::Command {
                                gate: gate.name.clone(),
                            },
                            access: Access::WriteAllowed,
                            inputs: if stage == Stage::Landing {
                                BTreeMap::from([(
                                    "merged".into(),
                                    Input {
                                        node: None,
                                        output: "merged".into(),
                                        kind: Output::Candidate,
                                    },
                                )])
                            } else {
                                BTreeMap::new()
                            },
                            outputs: BTreeMap::from([("result".into(), Output::Verdict)]),
                            outcomes: vec!["complete".into()],
                            routes: BTreeMap::new(),
                            requires: if stage == Stage::Landing {
                                vec!["merged".into()]
                            } else {
                                Vec::new()
                            },
                        },
                    ));
                }
            }
            for (_, node) in chain.iter_mut().skip(1) {
                if !node.inputs.is_empty() {
                    continue;
                }
                node.inputs.insert(
                    "candidate".into(),
                    Input {
                        node: Some("implementer".into()),
                        output: "result".into(),
                        kind: Output::Candidate,
                    },
                );
                node.requires.push("candidate".into());
            }
        }
        for index in 0..chain.len().saturating_sub(1) {
            let to = chain[index + 1].0.clone();
            chain[index]
                .1
                .routes
                .insert("complete".into(), vec![Route { to, repeat: None }]);
        }
        Ok(Plan {
            inputs: if !workflow.read_only && !self.gates.is_empty() {
                BTreeMap::from([("merged".into(), Output::Candidate)])
            } else {
                BTreeMap::new()
            },
            nodes: chain.into_iter().collect(),
        })
    }

    pub fn runnable_workflow(&self, plan: &Plan, selected: Option<&str>) -> Option<String> {
        let selected = selected.unwrap_or("default");
        if self
            .workflow_plan(selected)
            .is_ok_and(|legacy| legacy == *plan)
        {
            return Some(selected.to_string());
        }
        self.workflows
            .keys()
            .find(|name| self.workflow_plan(name).is_ok_and(|legacy| legacy == *plan))
            .cloned()
    }
}

impl Plan {
    pub fn unsupported_shape(&self) -> &'static str {
        if self
            .nodes
            .values()
            .flat_map(|n| n.routes.values().flatten())
            .any(|r| r.repeat.is_some())
        {
            "bounded-repeat"
        } else if self
            .nodes
            .values()
            .any(|n| n.routes.values().any(|r| r.len() > 1))
        {
            "fan-out"
        } else if self.nodes.values().any(|n| n.requires.len() > 1) {
            "required-result-join"
        } else {
            "custom-graph"
        }
    }

    pub fn validate(&self, config: &Config) -> Result<(), Invalid> {
        if self.nodes.is_empty() {
            return Err(Invalid::new("nodes", "required"));
        }
        if self.inputs.keys().any(String::is_empty) {
            return Err(Invalid::new("inputs", "empty_name"));
        }
        for (name, node) in &self.nodes {
            if name.is_empty() {
                return Err(Invalid::new("nodes", "empty_name"));
            }
            node.validate_runner(config, &format!("nodes.{name}"))?;
            if node.outputs.is_empty() || node.outputs.keys().any(String::is_empty) {
                return Err(Invalid::new(format!("nodes.{name}.outputs"), "required"));
            }
            for (input, reference) in &node.inputs {
                let path = format!("nodes.{name}.inputs.{input}");
                let outputs = match &reference.node {
                    Some(source) => {
                        &self
                            .nodes
                            .get(source)
                            .ok_or_else(|| {
                                Invalid::new(format!("{path}.node"), "unresolved_reference")
                            })?
                            .outputs
                    }
                    None => &self.inputs,
                };
                let kind = outputs.get(&reference.output).ok_or_else(|| {
                    Invalid::new(format!("{path}.output"), "unresolved_reference")
                })?;
                if kind != &reference.kind {
                    return Err(Invalid::new(format!("{path}.kind"), "output_type_mismatch"));
                }
            }
            let mut required = BTreeSet::new();
            for input in &node.requires {
                if !node.inputs.contains_key(input) || !required.insert(input) {
                    return Err(Invalid::new(
                        format!("nodes.{name}.requires.{input}"),
                        "unresolved_or_duplicate_result",
                    ));
                }
            }
            let mut outcomes = BTreeSet::new();
            for outcome in &node.outcomes {
                if outcome.is_empty() || !outcomes.insert(outcome) {
                    return Err(Invalid::new(
                        format!("nodes.{name}.outcomes"),
                        "empty_or_duplicate_outcome",
                    ));
                }
            }
            if outcomes.is_empty() {
                return Err(Invalid::new(format!("nodes.{name}.outcomes"), "required"));
            }
            for (outcome, routes) in &node.routes {
                if !outcomes.contains(outcome) {
                    return Err(Invalid::new(
                        format!("nodes.{name}.routes.{outcome}"),
                        "undeclared_outcome",
                    ));
                }
                for (index, route) in routes.iter().enumerate() {
                    if !self.nodes.contains_key(&route.to) {
                        return Err(Invalid::new(
                            format!("nodes.{name}.routes.{outcome}.{index}.to"),
                            "unresolved_reference",
                        ));
                    }
                    if route.repeat == Some(0) {
                        return Err(Invalid::new(
                            format!("nodes.{name}.routes.{outcome}.{index}.repeat"),
                            "unbounded_repeat",
                        ));
                    }
                }
            }
        }
        // Removing explicitly bounded repeat edges must leave a DAG.
        let mut pending: BTreeSet<_> = self.nodes.keys().cloned().collect();
        while !pending.is_empty() {
            let ends: Vec<_> = pending
                .iter()
                .filter(|target| {
                    !pending.iter().any(|source| {
                        self.nodes[source]
                            .routes
                            .values()
                            .flatten()
                            .any(|r| r.repeat.is_none() && &r.to == *target)
                    }) || !self.nodes[*target]
                        .routes
                        .values()
                        .flatten()
                        .any(|r| r.repeat.is_none() && pending.contains(&r.to))
                })
                .cloned()
                .collect();
            if ends.is_empty() {
                let mut source = pending.first().expect("nonempty");
                let mut seen = BTreeSet::new();
                loop {
                    seen.insert(source);
                    let (outcome, index, route) = self.nodes[source]
                        .routes
                        .iter()
                        .flat_map(|(o, rs)| rs.iter().enumerate().map(move |(i, r)| (o, i, r)))
                        .find(|(_, _, r)| r.repeat.is_none() && pending.contains(&r.to))
                        .expect("each remaining node has an outgoing edge");
                    if seen.contains(&route.to) {
                        return Err(Invalid::new(
                            format!("nodes.{source}.routes.{outcome}.{index}.repeat"),
                            "unbounded_repeat",
                        ));
                    }
                    source = &route.to;
                }
            }
            for end in ends {
                pending.remove(&end);
            }
        }
        Ok(())
    }
}

impl Node {
    pub fn validate_runner(&self, config: &Config, path: &str) -> Result<(), Invalid> {
        match &self.runner {
            Runner::Agent { agent, .. } if !config.agents.contains_key(agent) => Err(Invalid::new(
                format!("{path}.runner.agent"),
                "unresolved_reference",
            )),
            Runner::Command { gate } if !config.gates.iter().any(|g| &g.name == gate) => Err(
                Invalid::new(format!("{path}.runner.gate"), "unresolved_reference"),
            ),
            _ => Ok(()),
        }
    }
}
