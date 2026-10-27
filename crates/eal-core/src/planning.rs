//! Plan complete claim dependencies without collecting observations or evaluating methods.
use crate::{
    expressions::python_repr,
    methods::{self, MethodRegistry},
    semantic::Program,
    validation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Every authored declaration that may affect the target's support or defeat.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimClosure {
    pub claims: BTreeSet<String>,
    pub arguments: BTreeSet<String>,
    pub objections: BTreeSet<String>,
    pub evidence: BTreeSet<String>,
    pub assumptions: BTreeSet<String>,
    pub reasoning: BTreeSet<String>,
}
impl ClaimClosure {
    fn empty() -> Self {
        Self {
            claims: BTreeSet::new(),
            arguments: BTreeSet::new(),
            objections: BTreeSet::new(),
            evidence: BTreeSet::new(),
            assumptions: BTreeSet::new(),
            reasoning: BTreeSet::new(),
        }
    }
}

/// A declared acquisition request whose trusted executable binding is selected by the host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedCall {
    pub evidence_id: String,
    pub tool: Option<String>,
    pub tool_version: Option<String>,
    pub derived: bool,
    pub evidence_kind: String,
    pub environment: String,
}

/// Complete target closure with each evidence request retained once, in source order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePlan {
    pub claim: String,
    pub closure: ClaimClosure,
    pub calls: Vec<PlannedCall>,
}
impl EvidencePlan {
    pub fn evidence_ids(&self) -> Vec<String> {
        self.calls
            .iter()
            .map(|call| call.evidence_id.clone())
            .collect()
    }
    /// Maximum collector invocations before host-approved observation reuse.
    pub fn estimated_calls(&self) -> usize {
        self.calls.iter().filter(|call| !call.derived).count()
    }
}

/// Indexed, iterative traversal of support, attack and defence dependencies.
///
/// Construction checks the semantic precondition before indexing references.
/// Host-installed native methods use `new_with_registry` with their exact registry.
pub struct EvidencePlanner<'a> {
    program: &'a Program,
    by_conclusion: BTreeMap<String, Vec<String>>,
    by_target: BTreeMap<(String, String), Vec<String>>,
    scopes: BTreeMap<String, BTreeSet<String>>,
}
impl<'a> EvidencePlanner<'a> {
    pub fn new(program: &'a Program) -> Result<Self, String> {
        Self::new_with_registry(program, methods::default_registry())
    }
    pub fn new_with_registry(
        program: &'a Program,
        registry: &MethodRegistry,
    ) -> Result<Self, String> {
        program.limits.validate()?;
        let diagnostics = validation::validate(program, registry);
        if let Some(diagnostic) = diagnostics.first() {
            return Err(format!(
                "Evidence planning requires a validated programme: {}",
                diagnostic.message
            ));
        }
        let mut by_conclusion: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (name, argument) in &program.arguments {
            by_conclusion
                .entry(argument.conclusion.clone())
                .or_default()
                .push(name.clone());
        }
        let mut by_target: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
        for (name, objection) in &program.objections {
            by_target
                .entry((objection.target_kind.clone(), objection.target.clone()))
                .or_default()
                .push(name.clone());
        }
        Ok(Self {
            program,
            by_conclusion,
            by_target,
            scopes: validation::objection_scopes(program),
        })
    }

    /// Follow every alternative derivation and every applicable objection to a fixed point.
    pub fn plan(&self, target: &str) -> Result<EvidencePlan, String> {
        if !self.program.claims.contains_key(target) {
            return Err(format!(
                "Unknown claim {}",
                python_repr(&serde_json::json!(target))
            ));
        }
        let mut closure = ClaimClosure::empty();
        let mut queue = VecDeque::new();
        let mut scheduled = BTreeSet::new();
        enqueue("claim", target, &mut queue, &mut scheduled);
        while let Some((kind, name)) = queue.pop_front() {
            // The queue contains each graph declaration at most once. Semantic
            // validation has already bounded the total declaration population.
            if scheduled.len() > self.program.limits.declarations {
                return Err("Evidence plan exceeds the declaration budget".into());
            }
            match kind {
                "claim" => {
                    closure.claims.insert(name.clone());
                    let claim = &self.program.claims[&name];
                    for argument in self.by_conclusion.get(&name).into_iter().flatten() {
                        enqueue("argument", argument, &mut queue, &mut scheduled);
                    }
                    self.attacks(
                        "claim",
                        &name,
                        &claim.environment,
                        &mut queue,
                        &mut scheduled,
                    );
                }
                "argument" => {
                    closure.arguments.insert(name.clone());
                    let argument = &self.program.arguments[&name];
                    let environment = &self.program.claims[&argument.conclusion].environment;
                    closure.evidence.extend(argument.evidence.iter().cloned());
                    closure.reasoning.insert(argument.reasoning.clone());
                    closure.evidence.extend(
                        self.program.reasoning[&argument.reasoning]
                            .backing
                            .iter()
                            .cloned(),
                    );
                    closure
                        .assumptions
                        .extend(argument.assumptions.iter().cloned());
                    closure.evidence.extend(
                        argument
                            .assumptions
                            .iter()
                            .map(|a| self.program.assumptions[a].validation.clone()),
                    );
                    for premise in &argument.premises {
                        enqueue("claim", premise, &mut queue, &mut scheduled);
                    }
                    self.attacks("argument", &name, environment, &mut queue, &mut scheduled);
                    self.attacks(
                        "reasoning",
                        &argument.reasoning,
                        environment,
                        &mut queue,
                        &mut scheduled,
                    );
                    for assumption in &argument.assumptions {
                        self.attacks(
                            "assumption",
                            assumption,
                            environment,
                            &mut queue,
                            &mut scheduled,
                        );
                    }
                }
                "objection" => {
                    closure.objections.insert(name.clone());
                    let objection = &self.program.objections[&name];
                    closure.evidence.extend(objection.evidence.iter().cloned());
                    for premise in &objection.premises {
                        enqueue("claim", premise, &mut queue, &mut scheduled);
                    }
                    let environment = self.scopes[&name]
                        .iter()
                        .next()
                        .ok_or("A checked objection requires one environment")?;
                    self.attacks("objection", &name, environment, &mut queue, &mut scheduled);
                }
                _ => unreachable!("the planner only schedules claims, arguments and objections"),
            }
        }
        let selected = self
            .program
            .evidence
            .keys()
            .filter(|name| closure.evidence.contains(*name))
            .cloned()
            .collect::<Vec<_>>();
        let ordered = crate::acquisition::dependency_order(self.program, &selected)?;
        closure.evidence.extend(ordered);
        let calls = self
            .program
            .evidence
            .iter()
            .filter(|(name, _)| closure.evidence.contains(*name))
            .map(|(name, evidence)| PlannedCall {
                evidence_id: name.clone(),
                tool: evidence.tool.clone(),
                tool_version: evidence
                    .tool
                    .as_ref()
                    .map(|tool| self.program.tools[tool].version.clone()),
                derived: evidence.derivation.is_some(),
                evidence_kind: evidence.kind.clone(),
                environment: evidence.environment.clone(),
            })
            .collect();
        Ok(EvidencePlan {
            claim: target.into(),
            closure,
            calls,
        })
    }
    fn attacks(
        &self,
        kind: &str,
        name: &str,
        environment: &str,
        queue: &mut VecDeque<(&'static str, String)>,
        scheduled: &mut BTreeSet<(&'static str, String)>,
    ) {
        for objection in self
            .by_target
            .get(&(kind.to_owned(), name.to_owned()))
            .into_iter()
            .flatten()
        {
            if self.scopes[objection].len() == 1 && self.scopes[objection].contains(environment) {
                enqueue("objection", objection, queue, scheduled);
            }
        }
    }
}
fn enqueue(
    kind: &'static str,
    name: &str,
    queue: &mut VecDeque<(&'static str, String)>,
    scheduled: &mut BTreeSet<(&'static str, String)>,
) {
    let item = (kind, name.to_owned());
    if scheduled.insert(item.clone()) {
        queue.push_back(item);
    }
}
