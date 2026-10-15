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
    #[serde(rename = "finding-selection")]
    FindingSelection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Runner {
    Agent {
        agent: String,
        instructions: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        skills: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        references: Vec<String>,
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

impl Runner {
    pub fn selected_paths(&self) -> Option<(&[String], &[String])> {
        match self {
            Self::Agent {
                skills, references, ..
            } => Some((skills, references)),
            Self::Packaged {
                skill, references, ..
            } => Some((std::slice::from_ref(skill), references)),
            Self::Command { .. } => None,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            workspace: None,
            operator_failure: false,
            advisory: false,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub operator_failure: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub advisory: bool,
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
    pub workspace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_failure: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory: Option<bool>,
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
            advisory: self
                .advisory
                .or_else(|| base.map(|n| n.advisory))
                .unwrap_or(false),
            workspace: self
                .workspace
                .clone()
                .or_else(|| base.and_then(|n| n.workspace.clone())),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Continuation {
    Original,
    Fixer {
        profile: String,
    },
    Retry,
    Operator,
    Accept {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        authorization: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Classifier {
    pub runner: Runner,
    pub outcomes: BTreeMap<String, Continuation>,
    pub repeat: u32,
}

impl Classifier {
    pub fn node(&self) -> Node {
        Profile {
            runner: self.runner.clone(),
            access: Access::ReadOnly,
            outputs: BTreeMap::from([("result".into(), Output::Report)]),
            outcomes: self.outcomes.keys().cloned().collect(),
        }
        .node()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub classifiers: BTreeMap<String, Classifier>,
    #[serde(default)]
    pub inputs: BTreeMap<String, Output>,
    pub nodes: BTreeMap<String, NodeSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub classifiers: BTreeMap<String, Classifier>,
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
                && let Some(selected) =
                    plan.ordinary_selection(&ordinary, self.review.max_rounds, &self.profiles)
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
            classifiers: graph.classifiers.clone(),
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
                classifiers: BTreeMap::new(),
                inputs: BTreeMap::new(),
                nodes: BTreeMap::from([("implementer".into(), node)]),
            });
        }
        let mut chain = vec![(
            "implementer".to_string(),
            Node {
                workspace: None,
                operator_failure: false,
                advisory: false,
                runner: Runner::Agent {
                    agent: workflow.implementer.clone(),
                    instructions: workflow.instructions.clone(),
                    skills: Vec::new(),
                    references: Vec::new(),
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
                        let seat = self.seats.get(seat).ok_or_else(|| {
                            format!("workflows.{name}.review names unknown seat {seat:?}")
                        })?;
                        chain.push((
                            format!("review-{index}"),
                            Node {
                                workspace: None,
                                operator_failure: false,
                                advisory: false,
                                runner: Runner::Agent {
                                    agent: seat.agent.clone(),
                                    instructions: seat.instructions.clone(),
                                    skills: Vec::new(),
                                    references: Vec::new(),
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
                            workspace: None,
                            operator_failure: false,
                            advisory: false,
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
            classifiers: BTreeMap::new(),
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
                        (plan.packaged().is_some() && self.workflow(name).is_ok_and(|workflow| {
                            workflow.read_only
                                && matches!(
                                    &plan.nodes["implementer"].runner,
                                    Runner::Packaged { agent, .. } if agent == &workflow.implementer
                                )
                        })) || plan.handoff_selection(&ordinary, self)
                    } else {
                        plan.ordinary_selection(&ordinary, self.review.max_rounds, &self.profiles)
                            .is_some()
                            || plan.read_only_selection(&ordinary, self)
                            || plan.finding_selection(&ordinary, self)
                            || plan.writer_selection(self, name)
                            || plan.handoff_selection(&ordinary, self)
                    }
                })
            })
            .map(str::to_string)
    }
}

impl Plan {
    pub fn packaged_cleanup(&self) -> Option<&Runner> {
        self.nodes
            .get("cleanup")
            .filter(|node| self.handoff() && matches!(node.runner, Runner::Packaged { .. }))
            .map(|node| &node.runner)
    }

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
        } else if self.writers() {
            "writer-graph"
        } else if self.concurrent() {
            "read-only-graph"
        } else {
            "ordinary-linear"
        }
    }
    pub fn admitted(admission: &Value) -> Result<Self, serde_json::Error> {
        let mut plan: Self = serde_json::from_value(admission["plan"].clone())?;
        if admission["runtime"] != "writer-graph"
            && admission["runtime"] != "read-only-graph"
            && admission["runtime"] != "ordinary-linear"
            && admission["runtime"] != "packaged-audit"
            && !plan.inputs.contains_key("feedback")
        {
            plan.ordinary_routes(admission["review"]["max_rounds"].as_u64().unwrap_or(3) as u32);
        }
        Ok(plan)
    }

    pub fn concurrent(&self) -> bool {
        if self.handoff() {
            return false;
        }
        self.writers()
            || self.nodes.values().any(|node| {
                node.routes
                    .get("complete")
                    .is_some_and(|routes| routes.len() > 1)
                    || node.inputs.iter().any(|(name, input)| {
                        name != "panel" && input.node.is_some() && input.kind == Output::Verdict
                    })
            })
            || self.nodes.iter().any(|(name, node)| {
                name.starts_with("candidate-gate-") && node.access == Access::ReadOnly
            })
    }

    /// Complete edges define a DAG. Required inputs must come from an ancestor.
    pub fn complete_order(&self) -> Option<Vec<&str>> {
        let mut order = Vec::new();
        let mut reached = if self.writers() {
            self.nodes
                .iter()
                .filter(|(_, node)| {
                    node.access == Access::WriteAllowed
                        && node.outputs.get("result") == Some(&Output::Candidate)
                        && node.inputs.is_empty()
                })
                .map(|(name, _)| name.as_str())
                .collect()
        } else {
            BTreeSet::from(["implementer"])
        };
        while order.len() < self.nodes.len() {
            let next = self.nodes.keys().map(String::as_str).find(|name| {
                reached.contains(name)
                    && !order.contains(name)
                    && self.nodes.iter().all(|(source, node)| {
                        !node
                            .routes
                            .get("complete")
                            .is_some_and(|routes| routes.iter().any(|route| route.to == *name))
                            || order.contains(&source.as_str())
                    })
            })?;
            order.push(next);
            for route in self.nodes[next]
                .routes
                .get("complete")
                .into_iter()
                .flatten()
            {
                if route.repeat.is_some() {
                    return None;
                }
                reached.insert(route.to.as_str());
            }
        }
        Some(order)
    }

    /// Independent writers are roots; integration consumes their candidates.
    /// A selector may route an existing publication into native verification.
    pub fn writers(&self) -> bool {
        !self.handoff()
            && self
                .nodes
                .values()
                .filter(|node| {
                    node.access == Access::WriteAllowed
                        && node.outputs.get("result") == Some(&Output::Candidate)
                })
                .count()
                > 1
    }

    /// Only the admitted three-role path may name a shared workspace.
    pub fn handoff(&self) -> bool {
        self.nodes.values().any(|node| node.workspace.is_some())
    }

    pub fn candidate_node(&self) -> &str {
        if self.handoff() {
            "repair"
        } else {
            "implementer"
        }
    }

    pub fn cleanup_audits(&self) -> impl Iterator<Item = (&String, &Node)> {
        self.nodes
            .iter()
            .filter(|(name, _)| self.handoff() && name.starts_with("cleanup-audit-"))
    }

    fn handoff_selection(&self, ordinary: &Self, config: &Config) -> bool {
        if !self.handoff() || ordinary.nodes["implementer"].access != Access::WriteAllowed {
            return false;
        }
        let mut reduced = self.clone();
        let audits: Vec<_> = self
            .cleanup_audits()
            .map(|(name, _)| name.clone())
            .collect();
        for name in &audits {
            let node = &self.nodes[name];
            let current = Input {
                node: None,
                output: "candidate".into(),
                kind: Output::Candidate,
            };
            let original = Input {
                node: Some("implementer".into()),
                output: "result".into(),
                kind: Output::Candidate,
            };
            if node.workspace.is_some()
                || node.operator_failure
                || node.advisory
                || node.access != Access::ReadOnly
                || node.outputs != BTreeMap::from([("result".into(), Output::Report)])
                || node.outcomes != ["complete"]
                || !node.routes.is_empty()
                || node.requires != ["candidate"]
                || node.inputs.len() != 1
                || !node
                    .inputs
                    .get("candidate")
                    .is_some_and(|input| input == &current || input == &original)
                || !config.profiles.values().any(|profile| {
                    profile.runner == node.runner
                        && profile.access == node.access
                        && profile.outputs == node.outputs
                        && profile.outcomes == node.outcomes
                })
                || !matches!(node.runner, Runner::Agent { .. } | Runner::Packaged { .. })
            {
                return false;
            }
            reduced.nodes.remove(name);
        }
        if !audits.is_empty() {
            if self.inputs.get("candidate") != Some(&Output::Candidate) {
                return false;
            }
            reduced.inputs.remove("candidate");
        }
        for (name, output, source) in [
            ("implementer", Output::Candidate, None),
            (
                "cleanup",
                Output::Report,
                Some(("candidate", "implementer", Output::Candidate)),
            ),
            (
                "repair",
                Output::Candidate,
                Some(("cleanup", "cleanup", Output::Report)),
            ),
        ] {
            let Some(node) = self.nodes.get(name) else {
                return false;
            };
            let mut inputs = BTreeMap::new();
            if name == "repair" {
                inputs.insert(
                    "feedback".into(),
                    Input {
                        node: None,
                        output: "feedback".into(),
                        kind: Output::Verdict,
                    },
                );
            }
            let requires = if let Some((input, source, kind)) = source {
                inputs.insert(
                    input.into(),
                    Input {
                        node: Some(source.into()),
                        output: "result".into(),
                        kind,
                    },
                );
                vec![input.to_string()]
            } else {
                Vec::new()
            };
            if node.workspace.as_deref() != Some("implementer")
                || node.operator_failure
                || node.access != Access::WriteAllowed
                || node.outputs != BTreeMap::from([("result".into(), output)])
                || node.inputs != inputs
                || node.requires != requires
                || node.outcomes != ["complete"]
                || !(matches!(node.runner, Runner::Agent { .. })
                    || (name == "cleanup" && matches!(node.runner, Runner::Packaged { .. })))
                || !config.profiles.values().any(|profile| {
                    profile.runner == node.runner
                        && profile.access == node.access
                        && profile.outputs == node.outputs
                        && profile.outcomes == node.outcomes
                })
            {
                return false;
            }
            let mut routes = node.routes.clone();
            if name == "implementer"
                && let Some(complete) = routes.get_mut("complete")
            {
                for audit in &audits {
                    let expected = Route {
                        to: audit.clone(),
                        repeat: None,
                    };
                    if complete.iter().filter(|route| **route == expected).count() != 1 {
                        return false;
                    }
                    complete.retain(|route| route != &expected);
                }
            }
            if name != "repair"
                && routes
                    != BTreeMap::from([(
                        "complete".into(),
                        vec![Route {
                            to: if name == "implementer" {
                                "cleanup"
                            } else {
                                "repair"
                            }
                            .into(),
                            repeat: None,
                        }],
                    )])
            {
                return false;
            }
        }
        if self.nodes.iter().any(|(name, node)| {
            !matches!(name.as_str(), "implementer" | "cleanup" | "repair")
                && node.workspace.is_some()
        }) {
            return false;
        }
        if self.nodes.iter().any(|(name, node)| {
            !matches!(name.as_str(), "implementer" | "cleanup" | "repair")
                && !audits.contains(name)
                && (node.inputs.values().any(|input| {
                    input.kind == Output::Candidate
                        && input.node.is_some()
                        && input.node.as_deref() != Some("repair")
                }) || node
                    .routes
                    .values()
                    .flatten()
                    .any(|route| route.to == "implementer" || route.to == "cleanup"))
        }) {
            return false;
        }
        let mut implementer = ordinary.nodes["implementer"].clone();
        implementer.routes = self.nodes["repair"].routes.clone();
        reduced.nodes.insert("implementer".into(), implementer);
        reduced.nodes.remove("cleanup");
        reduced.nodes.remove("repair");
        for node in reduced.nodes.values_mut() {
            for input in node.inputs.values_mut() {
                if input.node.as_deref() == Some("repair") {
                    input.node = Some("implementer".into());
                }
            }
            for route in node.routes.values_mut().flatten() {
                if route.to == "repair" {
                    route.to = "implementer".into();
                }
            }
        }
        reduced
            .ordinary_selection(ordinary, config.review.max_rounds, &config.profiles)
            .is_some()
    }

    pub fn selector(&self) -> Option<&Node> {
        self.nodes
            .get("selector")
            .filter(|node| node.outputs.get("result") == Some(&Output::Selection))
    }

    pub fn finding_selector(&self) -> Option<&Node> {
        self.nodes
            .get("selector")
            .filter(|node| node.outputs.get("result") == Some(&Output::FindingSelection))
    }

    fn finding_selection(&self, ordinary: &Self, config: &Config) -> bool {
        let Some(picker) = self.finding_selector() else {
            return false;
        };
        if picker.access != Access::ReadOnly
            || picker.operator_failure
            || picker.advisory
            || picker.outputs != BTreeMap::from([("result".into(), Output::FindingSelection)])
            || picker.outcomes != ["complete"]
            || !picker.routes.is_empty()
            || !matches!(picker.runner, Runner::Agent { .. })
            || !config.profiles.values().any(|profile| {
                profile.runner == picker.runner
                    && profile.access == picker.access
                    && profile.outputs == picker.outputs
                    && profile.outcomes == picker.outcomes
            })
            || picker.requires.len() != picker.inputs.len()
            || picker
                .inputs
                .keys()
                .any(|name| !picker.requires.contains(name))
            || picker.inputs.get("candidate")
                != ordinary
                    .nodes
                    .get("review-0")
                    .and_then(|node| node.inputs.get("candidate"))
        {
            return false;
        }
        let incoming: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.routes.get("complete").is_some_and(|routes| {
                    routes
                        .iter()
                        .any(|route| route.to == "selector" && route.repeat.is_none())
                })
            })
            .map(|(name, _)| name.as_str())
            .collect();
        let [join] = incoming.as_slice() else {
            return false;
        };
        if !join.starts_with("review-")
            || self.nodes[*join]
                .inputs
                .values()
                .filter(|input| {
                    input.kind == Output::Verdict
                        && input.node.as_ref().is_some_and(|name| {
                            matches!(self.nodes[name].runner, Runner::Agent { .. })
                        })
                })
                .count()
                < 2
        {
            return false;
        }
        let mut ancestors = BTreeSet::from([*join]);
        let Some(order) = self.complete_order() else {
            return false;
        };
        for name in order.iter().rev() {
            if self.nodes[*name]
                .routes
                .get("complete")
                .is_some_and(|routes| {
                    routes
                        .iter()
                        .any(|route| ancestors.contains(route.to.as_str()))
                })
            {
                ancestors.insert(*name);
            }
        }
        let mut sources = BTreeSet::new();
        if !picker
            .inputs
            .values()
            .any(|input| input.node.as_deref() == Some(*join))
            || picker.inputs.iter().any(|(name, input)| {
                name != "candidate"
                    && (input.kind != Output::Verdict
                        || input.output != "result"
                        || input.node.as_deref().is_none_or(|source| {
                            !ancestors.contains(source)
                                || !source.starts_with("review-")
                                || !sources.insert(source)
                        }))
            })
        {
            return false;
        }
        let mut base = self.clone();
        base.nodes.remove("selector");
        for (name, node) in &mut base.nodes {
            if node.advisory {
                if !name.starts_with("review-")
                    || node.operator_failure
                    || !matches!(node.runner, Runner::Agent { .. })
                    || node.outputs.get("result") != Some(&Output::Verdict)
                {
                    return false;
                }
                node.advisory = false;
            }
        }
        for node in base.nodes.values_mut() {
            if let Some(routes) = node.routes.get_mut("complete") {
                routes.retain(|r| r.to != "selector");
            }
            node.routes.retain(|_, routes| !routes.is_empty());
        }
        base.read_only_selection(ordinary, config)
    }

    pub fn integration(&self) -> Option<(&str, &Node)> {
        if !self.writers() {
            return None;
        }
        self.nodes.iter().find_map(|(name, node)| {
            (node.access == Access::WriteAllowed
                && node.outputs.get("result") == Some(&Output::Candidate)
                && !node.inputs.is_empty())
            .then_some((name.as_str(), node))
        })
    }

    fn writer_selection(&self, config: &Config, workflow: &str) -> bool {
        if self.handoff() {
            return false;
        }
        if self.selector().is_some() {
            return self.selected_writer_graph(config, workflow);
        }
        self.candidate_writer_graph(config, workflow)
    }

    fn selected_writer_graph(&self, config: &Config, workflow: &str) -> bool {
        let Some(selector) = self.selector() else {
            return false;
        };
        let Ok(ordinary) = config.workflow_plan(workflow) else {
            return false;
        };
        if selector.access != Access::ReadOnly
            || selector.operator_failure
            || selector.outputs != BTreeMap::from([("result".into(), Output::Selection)])
            || selector.outcomes != ["complete"]
            || !config.profiles.values().any(|profile| {
                profile.runner == selector.runner
                    && profile.access == selector.access
                    && profile.outputs == selector.outputs
                    && profile.outcomes == selector.outcomes
            })
            || !matches!(selector.runner, Runner::Agent { .. })
            || selector.inputs.len() != selector.requires.len()
            || selector
                .inputs
                .keys()
                .any(|name| !selector.requires.contains(name))
            || !self
                .inputs
                .iter()
                .all(|(name, kind)| name == "merged" && *kind == Output::Candidate)
        {
            return false;
        }
        let mut roots = self.clone();
        roots.inputs.clear();
        roots.nodes.retain(|_, node| {
            matches!(node.runner, Runner::Agent { .. })
                && matches!(
                    node.outputs.get("result"),
                    Some(Output::Candidate | Output::Verdict)
                )
        });
        for node in roots.nodes.values_mut() {
            node.routes
                .get_mut("complete")
                .into_iter()
                .for_each(|routes| routes.retain(|r| r.to != "selector"));
            node.routes.retain(|_, routes| !routes.is_empty());
        }
        if !roots.candidate_writer_graph(config, workflow) {
            return false;
        }
        for (name, node) in &roots.nodes {
            if roots.integration().is_some_and(|(_, integration)| {
                node.access == Access::WriteAllowed
                    && integration
                        .inputs
                        .values()
                        .any(|input| input.node.as_deref() == Some(name))
            }) {
                if selector
                    .inputs
                    .values()
                    .any(|input| input.node.as_deref() == Some(name))
                {
                    return false;
                }
                continue;
            }
            let kind = if node.access == Access::WriteAllowed {
                Output::Candidate
            } else {
                Output::Verdict
            };
            if selector
                .inputs
                .values()
                .filter(|input| {
                    input.node.as_deref() == Some(name)
                        && input.output == "result"
                        && input.kind == kind
                })
                .count()
                != 1
                || !self.nodes[name]
                    .routes
                    .get("complete")
                    .is_some_and(|routes| {
                        routes
                            .iter()
                            .any(|r| r.to == "selector" && r.repeat.is_none())
                    })
            {
                return false;
            }
        }
        let consumed = roots.integration().map_or(0, |(_, node)| node.inputs.len());
        if selector.inputs.len() + consumed != roots.nodes.len() {
            return false;
        }
        for (name, node) in &self.nodes {
            if name == "selector" || roots.nodes.contains_key(name) {
                continue;
            }
            let Some(defined) = ordinary.nodes.get(name) else {
                return false;
            };
            let mut defined = defined.clone();
            if defined.inputs.contains_key("merged") && node.operator_failure {
                defined.outcomes.push("failed".into());
            }
            if !matches!(node.runner, Runner::Command { .. })
                || node.runner != defined.runner
                || node.access != defined.access
                || node.outputs != defined.outputs
                || node.outcomes != defined.outcomes
            {
                return false;
            }
            let landing = defined.inputs.contains_key("merged");
            let reference = Input {
                node: (!landing).then(|| "selector".into()),
                output: if landing { "merged" } else { "result" }.into(),
                kind: if landing {
                    Output::Candidate
                } else {
                    Output::Selection
                },
            };
            if node.inputs
                != BTreeMap::from([(
                    if landing { "merged" } else { "candidate" }.into(),
                    reference,
                )])
                || node.requires != [if landing { "merged" } else { "candidate" }]
            {
                return false;
            }
        }
        let mut visited = BTreeSet::new();
        let mut current = "selector";
        let mut landing = false;
        loop {
            if !visited.insert(current) {
                return false;
            }
            let node = &self.nodes[current];
            for (outcome, routes) in &node.routes {
                if outcome != "complete"
                    && (outcome != "failed"
                        || routes.len() != 1
                        || routes[0].to != "implementer"
                        || routes[0].repeat != Some(config.review.max_rounds))
                {
                    return false;
                }
            }
            match node.routes.get("complete").map(Vec::as_slice) {
                None | Some([]) => break,
                Some([route]) if route.repeat.is_none() => {
                    if roots.nodes.contains_key(&route.to) || route.to == "selector" {
                        return false;
                    }
                    if self.nodes[&route.to].inputs.contains_key("merged") {
                        landing = true;
                    } else if landing {
                        return false;
                    }
                    current = &route.to;
                }
                _ => return false,
            }
        }
        visited.len() + roots.nodes.len() == self.nodes.len() && self.complete_order().is_some()
    }

    fn candidate_writer_graph(&self, config: &Config, workflow: &str) -> bool {
        if config
            .workflow(workflow)
            .map_or(true, |workflow| workflow.read_only)
            || !self.writers()
            || !self.inputs.is_empty()
            || self
                .nodes
                .get("implementer")
                .is_none_or(|node| node.access != Access::WriteAllowed)
        {
            return false;
        }
        if self
            .nodes
            .values()
            .filter(|node| node.access == Access::WriteAllowed && !node.inputs.is_empty())
            .count()
            > 1
        {
            return false;
        }
        for (name, node) in &self.nodes {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
                || !matches!(node.runner, Runner::Agent { .. })
            {
                return false;
            }
            if node.access == Access::WriteAllowed {
                if node.operator_failure
                    || node.inputs.len() != node.requires.len()
                    || node
                        .inputs
                        .keys()
                        .any(|input| !node.requires.contains(input))
                    || (!node.inputs.is_empty()
                        && (name == "implementer"
                            || node.inputs.len() < 2
                            || node.inputs.values().any(|input| {
                                input.kind != Output::Candidate
                                    || input.output != "result"
                                    || input
                                        .node
                                        .as_ref()
                                        .and_then(|source| self.nodes.get(source))
                                        .is_none_or(|source| {
                                            source.access != Access::WriteAllowed
                                                || !source.inputs.is_empty()
                                        })
                            })
                            || node
                                .inputs
                                .values()
                                .map(|input| &input.node)
                                .collect::<BTreeSet<_>>()
                                .len()
                                != node.inputs.len()))
                    || node.outputs != BTreeMap::from([("result".into(), Output::Candidate)])
                    || node.outcomes != ["complete"]
                    || node.routes.keys().any(|outcome| outcome != "complete")
                    || !config.profiles.values().any(|profile| {
                        profile.runner == node.runner
                            && profile.access == node.access
                            && profile.outputs == node.outputs
                            && profile.outcomes == node.outcomes
                    })
                {
                    return false;
                }
                for input in node.inputs.values() {
                    if !self.nodes[input.node.as_ref().expect("validated source")]
                        .routes
                        .get("complete")
                        .is_some_and(|routes| {
                            routes
                                .iter()
                                .any(|route| route.to == *name && route.repeat.is_none())
                        })
                    {
                        return false;
                    }
                }
                for route in node.routes.values().flatten() {
                    if route.repeat.is_some()
                        || self.nodes.get(&route.to).is_none_or(|reader| {
                            if reader.access == Access::WriteAllowed {
                                !reader
                                    .inputs
                                    .values()
                                    .any(|input| input.node.as_deref() == Some(name))
                            } else {
                                reader
                                    .inputs
                                    .get("candidate")
                                    .is_none_or(|input| input.node.as_deref() != Some(name))
                            }
                        })
                    {
                        return false;
                    }
                }
            } else {
                let Some(index) = name
                    .strip_prefix("review-")
                    .and_then(|i| i.parse::<usize>().ok())
                else {
                    return false;
                };
                if config
                    .workflow(workflow)
                    .map_or(true, |workflow| index >= workflow.review.len())
                    || !node.routes.is_empty()
                    || node.inputs.len() != 1
                    || node.requires != ["candidate"]
                    || node.outputs != BTreeMap::from([("result".into(), Output::Verdict)])
                    || node.outcomes != ["complete", "blocked"]
                {
                    return false;
                }
                let Some(input) = node.inputs.get("candidate") else {
                    return false;
                };
                let Some(source) = input
                    .node
                    .as_ref()
                    .and_then(|source| self.nodes.get(source))
                else {
                    return false;
                };
                if input.kind != Output::Candidate
                    || input.output != "result"
                    || source.access != Access::WriteAllowed
                    || !source
                        .routes
                        .get("complete")
                        .is_some_and(|routes| routes.iter().any(|route| route.to == *name))
                {
                    return false;
                }
            }
        }
        self.complete_order().is_some()
    }

    fn read_only_selection(&self, ordinary: &Self, config: &Config) -> bool {
        if self.handoff()
            || !self.concurrent()
            || self.inputs != ordinary.inputs
            || !self.nodes.contains_key("implementer")
        {
            return false;
        }
        let Some(order) = self.complete_order() else {
            return false;
        };
        let mut landing = false;
        for (position, name) in order.iter().enumerate() {
            let node = &self.nodes[*name];
            let Some(defined) = ordinary.nodes.get(*name) else {
                return false;
            };
            let mut expected = defined.clone();
            expected.routes.remove("complete");
            let mut provided = node.clone();
            provided.routes.remove("complete");
            if *name == "implementer" || name.starts_with("landing-gate-") {
                if name.starts_with("landing-gate-") {
                    let incoming: Vec<_> =
                        self.nodes
                            .iter()
                            .filter(|(_, source)| {
                                source.routes.get("complete").is_some_and(|routes| {
                                    routes.iter().any(|route| route.to == *name)
                                })
                            })
                            .map(|(source, _)| source.as_str())
                            .collect();
                    if incoming.len() != 1 || (landing && incoming[0] != order[position - 1]) {
                        return false;
                    }
                    if node.routes.get("complete").is_some_and(|r| r.len() > 1) {
                        return false;
                    }
                    landing = true;
                    expected.operator_failure = node.operator_failure;
                }
                if provided != expected {
                    return false;
                }
                continue;
            }
            if landing || node.access != Access::ReadOnly {
                return false;
            }
            if let Runner::Command { gate } = &node.runner {
                if !config
                    .gates
                    .iter()
                    .any(|g| g.name == *gate && g.runs_in == crate::config::RunsIn::Box)
                {
                    return false;
                }
            } else if node.runner != expected.runner {
                if !config.profiles.values().any(|profile| {
                    profile.runner == node.runner
                        && profile.access == Access::ReadOnly
                        && profile.outputs == expected.outputs
                        && profile.outcomes.iter().any(|o| o == "complete")
                        && profile
                            .outcomes
                            .iter()
                            .all(|o| o == "complete" || o == "blocked")
                }) {
                    return false;
                }
                expected.runner = node.runner.clone();
            }
            expected.access = Access::ReadOnly;
            expected.operator_failure = node.operator_failure;
            expected.inputs.remove("panel");
            expected.requires = vec!["candidate".into()];
            let outcome = if name.starts_with("review-") {
                "blocked"
            } else {
                "failed"
            };
            expected.outcomes = vec!["complete".into(), outcome.into()];
            expected.routes = BTreeMap::from([(
                outcome.into(),
                vec![Route {
                    to: "implementer".into(),
                    repeat: Some(if outcome == "blocked" {
                        config.review.max_rounds
                    } else {
                        1
                    }),
                }],
            )]);
            if node.inputs.get("candidate") != expected.inputs.get("candidate")
                || node.requires.len() != node.inputs.len()
                || node
                    .inputs
                    .keys()
                    .any(|input| !node.requires.contains(input))
            {
                return false;
            }
            for (input, reference) in &node.inputs {
                if input == "candidate" {
                    continue;
                }
                let Some(source) = reference.node.as_deref() else {
                    return false;
                };
                if reference.output != "result"
                    || reference.kind != Output::Verdict
                    || !order[..position].contains(&source)
                    || source.starts_with("landing-gate-")
                    || source == "implementer"
                {
                    return false;
                }
                let mut ancestors = BTreeSet::from([*name]);
                for predecessor in order[..position].iter().rev() {
                    if self.nodes[*predecessor]
                        .routes
                        .get("complete")
                        .is_some_and(|routes| {
                            routes
                                .iter()
                                .any(|route| ancestors.contains(route.to.as_str()))
                        })
                    {
                        ancestors.insert(*predecessor);
                    }
                }
                if !ancestors.contains(source) {
                    return false;
                }
            }
            let incoming: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, predecessor)| {
                    predecessor
                        .routes
                        .get("complete")
                        .is_some_and(|routes| routes.iter().any(|route| route.to == *name))
                })
                .map(|(source, _)| source.as_str())
                .collect();
            if incoming.len() > 1
                && incoming.iter().any(|source| {
                    !node
                        .inputs
                        .values()
                        .any(|input| input.node.as_deref() == Some(source))
                })
            {
                return false;
            }
            expected.inputs = node.inputs.clone();
            expected.requires = node.requires.clone();
            if provided != expected {
                return false;
            }
        }
        true
    }

    /// Retained definitions and repair bounds stay ordinary. Panel links follow
    /// retained seats, even when their original predecessor is omitted.
    fn ordinary_selection(
        &self,
        ordinary: &Self,
        max_rounds: u32,
        profiles: &BTreeMap<String, Profile>,
    ) -> Option<Self> {
        let mut ordinary = ordinary.clone();
        for (name, node) in &self.nodes {
            let defined = ordinary.nodes.get_mut(name)?;
            if name == "implementer" && node.runner != defined.runner {
                let same_agent = matches!((&node.runner, &defined.runner),
                    (Runner::Agent { agent, .. }, Runner::Agent { agent: previous, .. }) if agent == previous);
                if !same_agent
                    || !profiles.values().any(|profile| {
                        profile.runner == node.runner
                            && profile.access == defined.access
                            && profile.outputs == defined.outputs
                            && profile.outcomes == defined.outcomes
                    })
                {
                    return None;
                }
                defined.runner = node.runner.clone();
            }
            if name.starts_with("review-") && node.runner != defined.runner {
                if !profiles.values().any(|profile| {
                    profile.runner == node.runner
                        && matches!(profile.runner, Runner::Agent { .. })
                        && profile.access == Access::ReadOnly
                        && profile.outputs == defined.outputs
                        && profile.outcomes.iter().any(|outcome| outcome == "complete")
                        && profile
                            .outcomes
                            .iter()
                            .all(|outcome| matches!(outcome.as_str(), "complete" | "blocked"))
                }) {
                    return None;
                }
                defined.runner = node.runner.clone();
            }
        }
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
                && !(name.starts_with("candidate-gate-")
                    || name.starts_with("review-")
                    || name.starts_with("landing-gate-"))
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
            // A profile declares reviewer capabilities. Its seat position
            // determines which ordinary outcomes and bounded routes run.
            if name.starts_with("review-")
                && profiles.values().any(|profile| {
                    profile.runner == node.runner
                        && profile.access == Access::ReadOnly
                        && profile.outputs == node.outputs
                        && profile.outcomes == node.outcomes
                        && profile.outcomes.iter().any(|outcome| outcome == "complete")
                        && profile
                            .outcomes
                            .iter()
                            .all(|outcome| matches!(outcome.as_str(), "complete" | "blocked"))
                })
            {
                provided.outcomes = retained.outcomes.clone();
            }
            if provided != defined && provided != retained {
                return None;
            }
        }
        Some(selected)
    }

    /// Upgrade the ordinary descriptive chain in memory. Persisted admission
    /// and historical execution evidence remain untouched.
    pub fn ordinary_routes(&mut self, max_rounds: u32) {
        for (name, node) in &mut self.nodes {
            if name.starts_with("landing-gate-")
                && node.operator_failure
                && !node.outcomes.iter().any(|outcome| outcome == "failed")
            {
                node.outcomes.push("failed".into());
            }
        }
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
        for (name, classifier) in &self.classifiers {
            let path = format!("classifiers.{name}");
            let source = self
                .nodes
                .get(name)
                .ok_or_else(|| Invalid::new(&path, "unresolved_reference"))?;
            if source.outputs.get("result") != Some(&Output::Verdict)
                || source.advisory
                || classifier.repeat == 0
                || (name.starts_with("landing-") && !source.operator_failure)
                || (self.selector().is_some() && !matches!(source.runner, Runner::Command { .. }))
                || !(name.starts_with("candidate-gate-")
                    || name.starts_with("landing-gate-")
                    || name
                        .strip_prefix("review-")
                        .is_some_and(|index| index.parse::<usize>().is_ok()))
                || classifier.outcomes.is_empty()
                || classifier
                    .outcomes
                    .keys()
                    .any(|name| name.trim().is_empty())
            {
                return Err(Invalid::new(&path, "invalid_input"));
            }
            classifier.node().validate_runner(config, &path)?;
            match &classifier.runner {
                Runner::Command { gate } => {
                    let gate = config
                        .gates
                        .iter()
                        .find(|entry| entry.name == *gate)
                        .expect("validated gate");
                    if gate.runs_in != crate::config::RunsIn::Box || !gate.env.is_empty() {
                        return Err(Invalid::new(&path, "invalid_input"));
                    }
                }
                Runner::Agent {
                    skills, references, ..
                } if skills.is_empty() && references.is_empty() => {}
                _ => return Err(Invalid::new(&path, "invalid_input")),
            }
            for action in classifier.outcomes.values() {
                if let Continuation::Accept {
                    authorization: Some(text),
                } = action
                    && (text.trim().is_empty() || text.len() > 8000)
                {
                    return Err(Invalid::new(&path, "invalid_input"));
                }
                if let Continuation::Fixer { profile } = action {
                    let profile = config
                        .profiles
                        .get(profile)
                        .ok_or_else(|| Invalid::new(&path, "unresolved_reference"))?;
                    if profile.access != Access::WriteAllowed
                        || profile.outputs != BTreeMap::from([("result".into(), Output::Candidate)])
                        || profile.outcomes != ["complete"]
                        || !matches!(&profile.runner, Runner::Agent { skills, references, .. }
                            if skills.is_empty() && references.is_empty())
                    {
                        return Err(Invalid::new(&path, "invalid_input"));
                    }
                }
            }
        }
        for (name, node) in &self.nodes {
            if name.is_empty() {
                return Err(Invalid::new("nodes", "empty_name"));
            }
            if name.starts_with("classifier:") {
                return Err(Invalid::new(format!("nodes.{name}"), "invalid_input"));
            }
            node.validate_runner(config, &format!("nodes.{name}"))?;
            if node.outputs.is_empty() || node.outputs.keys().any(String::is_empty) {
                return Err(Invalid::new(format!("nodes.{name}.outputs"), "required"));
            }
            for (input, reference) in &node.inputs {
                let path = format!("nodes.{name}.inputs.{input}");
                if input == "feedback"
                    && reference.node.is_some()
                    && node.access == Access::WriteAllowed
                    && node.outputs.get("result") == Some(&Output::Candidate)
                {
                    return Err(Invalid::new(path, "invalid_input"));
                }
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
        if let Some((name, _)) = self.nodes.iter().find(|(_, node)| node.advisory)
            && !std::iter::once("default")
                .chain(config.workflows.keys().map(String::as_str))
                .any(|workflow| {
                    config
                        .workflow_plan(workflow)
                        .is_ok_and(|ordinary| self.finding_selection(&ordinary, config))
                })
        {
            return Err(Invalid::new(
                format!("nodes.{name}.advisory"),
                "invalid_input",
            ));
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
            Runner::Agent {
                agent,
                skills,
                references,
                ..
            } => {
                if !config.agents.contains_key(agent) {
                    return Err(Invalid::new(
                        format!("{path}.runner.agent"),
                        "unresolved_reference",
                    ));
                }
                let paths: Vec<_> = skills.iter().chain(references).collect();
                if paths.len() > 512
                    || paths.iter().any(|path| !plain_path(path))
                    || skills.iter().any(|path| !path.ends_with(".md"))
                    || paths.iter().collect::<BTreeSet<_>>().len() != paths.len()
                {
                    return Err(Invalid::new(format!("{path}.runner"), "invalid_input"));
                }
                Ok(())
            }
            Runner::Command { gate } if !config.gates.iter().any(|g| &g.name == gate) => Err(
                Invalid::new(format!("{path}.runner.gate"), "unresolved_reference"),
            ),
            _ => Ok(()),
        }
    }
}
