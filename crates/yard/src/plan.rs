//! Shared typed resolver. Plans select existing definitions, never worker grants.
use crate::config::{Config, Stage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn is_false(value: &bool) -> bool {
    !value
}

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
    Agent {
        agent: String,
        instructions: String,
    },
    Command {
        gate: String,
    },
    Packaged {
        package: String,
        agent: String,
        instructions: String,
        skill: String,
        #[serde(default)]
        references: Vec<String>,
        files: Vec<AuditFile>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditFile {
    pub root: String,
    pub path: String,
}

pub fn plain_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.contains('\0')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
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
            operator_failure: false,
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
    #[serde(default, skip_serializing_if = "is_false")]
    pub operator_failure: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_failure: Option<bool>,
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
            operator_failure: self
                .operator_failure
                .or_else(|| base.map(|n| n.operator_failure))
                .unwrap_or(false),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        self.normalize_ordinary(&mut plan);
        plan.validate(self)?;
        Ok(plan)
    }

    fn normalize_ordinary(&self, plan: &mut Plan) {
        for name in std::iter::once("default").chain(self.workflows.keys().map(String::as_str)) {
            if let Ok(ordinary) = self.workflow_plan(name)
                && let Some(selected) = plan.ordinary_selection(&ordinary, self.review.max_rounds)
            {
                *plan = selected;
                break;
            }
        }
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
        let mut plan = Plan {
            inputs: graph.inputs.clone(),
            nodes,
        };
        self.normalize_ordinary(&mut plan);
        plan.validate(self)?;
        Ok(plan)
    }

    /// Ordinary routes; the queue supplies the merged landing input.
    pub fn workflow_plan(&self, name: &str) -> Result<Plan, String> {
        let workflow = self.workflow(name)?;
        if let Some(profile) = &workflow.profile {
            let node = self
                .profiles
                .get(profile)
                .ok_or_else(|| {
                    format!("workflows.{name}.profile names unknown profile {profile:?}")
                })?
                .node();
            return Ok(Plan {
                inputs: BTreeMap::new(),
                nodes: BTreeMap::from([("implementer".into(), node)]),
            });
        }
        let mut chain = vec![(
            "implementer".to_string(),
            Node {
                operator_failure: false,
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
                                operator_failure: false,
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
                            operator_failure: false,
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
        let mut plan = Plan {
            inputs: if !workflow.read_only && !self.gates.is_empty() {
                BTreeMap::from([("merged".into(), Output::Candidate)])
            } else {
                BTreeMap::new()
            },
            nodes: chain.into_iter().collect(),
        };
        plan.ordinary_routes(self.review.max_rounds);
        Ok(plan)
    }

    pub fn runnable_workflow(&self, plan: &Plan, selected: Option<&str>) -> Option<String> {
        let selected = selected.unwrap_or("default");
        std::iter::once(selected)
            .chain(self.workflows.keys().map(String::as_str))
            .find(|name| {
                self.workflow_plan(name).is_ok_and(|ordinary| {
                    if plan
                        .nodes
                        .values()
                        .any(|node| matches!(node.runner, Runner::Packaged { .. }))
                    {
                        plan.packaged().is_some() && self.workflow(name).is_ok_and(|workflow| {
                            workflow.read_only
                                && matches!(
                                    &plan.nodes["implementer"].runner,
                                    Runner::Packaged { agent, .. } if agent == &workflow.implementer
                                )
                        })
                    } else {
                        plan.ordinary_selection(&ordinary, self.review.max_rounds)
                            .is_some()
                    }
                })
            })
            .map(str::to_string)
    }
}

impl Plan {
    pub fn packaged(&self) -> Option<&Runner> {
        let node = self.nodes.get("implementer")?;
        (self.inputs.is_empty()
            && self.nodes.len() == 1
            && !node.operator_failure
            && node.access == Access::ReadOnly
            && node.inputs.is_empty()
            && node.requires.is_empty()
            && node.routes.is_empty()
            && node.outputs == BTreeMap::from([("result".into(), Output::Report)])
            && node.outcomes == ["complete"]
            && matches!(node.runner, Runner::Packaged { .. }))
        .then_some(&node.runner)
    }

    pub fn runtime(&self) -> &'static str {
        if self.packaged().is_some() {
            "packaged-audit"
        } else {
            "ordinary-linear"
        }
    }
    pub fn admitted(admission: &Value) -> Result<Self, serde_json::Error> {
        let mut plan: Self = serde_json::from_value(admission["plan"].clone())?;
        if admission["runtime"] != "ordinary-linear"
            && admission["runtime"] != "packaged-audit"
            && !plan.inputs.contains_key("feedback")
        {
            plan.ordinary_routes(admission["review"]["max_rounds"].as_u64().unwrap_or(3) as u32);
        }
        Ok(plan)
    }

    /// Retained definitions and repair bounds stay ordinary. Panel links follow
    /// retained seats, even when their original predecessor is omitted.
    fn ordinary_selection(&self, ordinary: &Self, max_rounds: u32) -> Option<Self> {
        if self
            .inputs
            .iter()
            .any(|(name, kind)| ordinary.inputs.get(name) != Some(kind))
        {
            return None;
        }
        let mut visited = BTreeSet::new();
        let mut current = "implementer";
        let mut phase = 0;
        let mut previous_review = None;
        loop {
            if !visited.insert(current) {
                return None;
            }
            let node = self.nodes.get(current)?;
            let next_phase = if current == "implementer" {
                0
            } else if current.starts_with("candidate-gate-") {
                1
            } else if let Some(index) = current.strip_prefix("review-") {
                let index = index.parse::<usize>().ok()?;
                if previous_review.is_some_and(|previous| index <= previous) {
                    return None;
                }
                previous_review = Some(index);
                2
            } else if current.starts_with("landing-gate-") {
                3
            } else {
                return None;
            };
            if next_phase < phase {
                return None;
            }
            phase = next_phase;
            match node.routes.get("complete").map(Vec::as_slice) {
                None | Some([]) => break,
                Some([route]) if route.repeat.is_none() => current = &route.to,
                _ => return None,
            }
        }
        if visited.len() != self.nodes.len() {
            return None;
        }
        let mut selected = self.clone();
        for (name, node) in &mut selected.nodes {
            let defined = ordinary.nodes.get(name)?;
            let complete = node.routes.get("complete").cloned();
            let operator_failure = node.operator_failure;
            if operator_failure
                && !(name.starts_with("candidate-gate-") || name.starts_with("review-"))
            {
                return None;
            }
            *node = defined.clone();
            node.operator_failure = operator_failure;
            node.routes.remove("complete");
            if let Some(routes) = complete {
                node.routes.insert("complete".into(), routes);
            }
            if name.starts_with("review-") {
                node.inputs.remove("panel");
                node.requires = vec!["candidate".into()];
                node.outcomes.retain(|outcome| outcome != "blocked");
                node.routes.remove("blocked");
            }
        }
        selected.ordinary_routes(max_rounds);
        for (name, node) in &self.nodes {
            let mut provided = node.clone();
            provided.routes.remove("complete");
            let mut defined = ordinary.nodes[name].clone();
            defined.operator_failure = node.operator_failure;
            defined.routes.remove("complete");
            let mut retained = selected.nodes[name].clone();
            retained.routes.remove("complete");
            if provided != defined && provided != retained {
                return None;
            }
        }
        Some(selected)
    }

    /// Upgrade the ordinary descriptive chain in memory. Persisted admission
    /// and historical execution evidence remain untouched.
    pub fn ordinary_routes(&mut self, max_rounds: u32) {
        if self
            .nodes
            .get("implementer")
            .and_then(|node| node.outputs.get("result"))
            != Some(&Output::Candidate)
        {
            return;
        }
        self.inputs.insert("feedback".into(), Output::Verdict);
        self.nodes
            .get_mut("implementer")
            .expect("implementer")
            .inputs
            .insert(
                "feedback".into(),
                Input {
                    node: None,
                    output: "feedback".into(),
                    kind: Output::Verdict,
                },
            );
        let mut current = "implementer".to_string();
        let mut previous_review = None;
        while let Some(next) = self.nodes[&current]
            .routes
            .get("complete")
            .and_then(|routes| routes.first())
            .map(|route| route.to.clone())
        {
            current = next;
            if current.starts_with("landing-gate-") {
                break;
            }
            let next_review = self.nodes[&current]
                .routes
                .get("complete")
                .and_then(|routes| routes.first())
                .is_some_and(|route| route.to.starts_with("review-"));
            let node = self.nodes.get_mut(&current).expect("route destination");
            let outcome = if current.starts_with("review-") {
                if let Some(previous) = previous_review {
                    node.inputs.insert(
                        "panel".into(),
                        Input {
                            node: Some(previous),
                            output: "result".into(),
                            kind: Output::Verdict,
                        },
                    );
                    node.requires = vec!["candidate".into(), "panel".into()];
                }
                previous_review = Some(current.clone());
                if next_review {
                    continue;
                }
                "blocked"
            } else {
                "failed"
            };
            if !node.outcomes.iter().any(|name| name == outcome) {
                node.outcomes.push(outcome.into());
            }
            node.routes.insert(
                outcome.into(),
                vec![Route {
                    to: "implementer".into(),
                    repeat: Some(if outcome == "blocked" { max_rounds } else { 1 }),
                }],
            );
        }
    }

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
            Runner::Packaged {
                package,
                agent,
                instructions,
                skill,
                references,
                files,
            } => {
                if package != "durable-audit" {
                    return Err(Invalid::new(
                        format!("{path}.runner.package"),
                        "unresolved_reference",
                    ));
                }
                if !config
                    .agents
                    .get(agent)
                    .is_some_and(|agent| agent.harness == "pi")
                {
                    return Err(Invalid::new(
                        format!("{path}.runner.agent"),
                        "unresolved_reference",
                    ));
                }
                if !plain_path(skill)
                    || !skill.ends_with(".md")
                    || references.len() >= 512
                    || references.iter().any(|path| !plain_path(path))
                    || references.iter().collect::<BTreeSet<_>>().len() != references.len()
                    || references.contains(skill)
                    || instructions.len() > 32768
                    || files.len() + references.len() + 1 > 512
                    || files.iter().any(|file| {
                        !["source", "proof"].contains(&file.root.as_str())
                            || file.path.is_empty()
                            || file.path.len() > 1024
                            || file.path.contains('\0')
                    })
                    || files
                        .iter()
                        .map(|file| (&file.root, &file.path))
                        .collect::<BTreeSet<_>>()
                        .len()
                        != files.len()
                {
                    return Err(Invalid::new(format!("{path}.runner"), "invalid_input"));
                }
                Ok(())
            }
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
