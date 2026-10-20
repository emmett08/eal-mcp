//! Ordered static passes over composed EAL entities, without observations or IO.
use crate::ExecutionLimits;
use crate::methods::{MethodRegistry, default_registry};
use crate::semantic::{Diagnostic, Program};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

mod declarations;
mod directives;
mod graph;
mod predicates;
mod references;
mod resources;
mod time;
pub use resources::structural_value_count;
pub use time::parse_time;

/// Check finite JSON while respecting the host's explicit traversal budgets.
pub fn check_json_resources(value: &Value, limits: &ExecutionLimits) -> Result<(), String> {
    let mut pending = vec![(value, 0usize)];
    let mut nodes = 0usize;
    while let Some((item, depth)) = pending.pop() {
        nodes += 1;
        if depth > limits.json_depth || nodes > limits.json_nodes {
            return Err("JSON resource limit exceeded".into());
        }
        match item {
            Value::Array(items) => pending.extend(items.iter().map(|item| (item, depth + 1))),
            Value::Object(items) => pending.extend(items.values().map(|item| (item, depth + 1))),
            Value::Number(number) if !number.is_i64() && !number.is_u64() => {
                let text = number.to_string();
                if text.contains(['.', 'e', 'E']) && !number.as_f64().is_some_and(f64::is_finite) {
                    return Err("JSON numbers must be finite".into());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Run identity, predicate, dependency, method and typed proposition checks.
pub fn validate(program: &Program, registry: &MethodRegistry) -> Vec<Diagnostic> {
    if let Err(message) = structural_value_count(program, program.limits.structural_values) {
        return vec![Diagnostic {
            code: "resource_limit".into(),
            message,
            declaration: None,
            span: None,
            expected: None,
            actual: None,
        }];
    }
    let mut checker = Checker {
        program,
        registry,
        problems: program.lowering_diagnostics.clone(),
        references: true,
    };
    for problem in &mut checker.problems {
        if problem.span.is_none() {
            problem.span = problem
                .declaration
                .as_ref()
                .and_then(|owner| program.locations.get(owner))
                .copied();
        }
    }
    if !checker.declarations() {
        return checker.problems;
    }
    checker.predicates();
    checker.tools_and_evidence();
    checker.assumptions();
    checker.reasoning_and_claims();
    checker.arguments();
    checker.objections();
    checker.directives();
    checker.pattern_metadata();
    checker.dependencies();
    checker.problems
}

pub fn validate_default(program: &Program) -> Vec<Diagnostic> {
    validate(program, default_registry())
}

/// Infer each objection's evidence/premise scope without following attack cycles.
pub fn objection_scopes(program: &Program) -> BTreeMap<String, BTreeSet<String>> {
    program
        .objections
        .iter()
        .map(|(name, objection)| {
            let scopes = objection
                .evidence
                .iter()
                .filter_map(|id| program.evidence.get(id).map(|e| e.environment.clone()))
                .chain(
                    objection
                        .premises
                        .iter()
                        .filter_map(|id| program.claims.get(id).map(|c| c.environment.clone())),
                )
                .collect();
            (name.clone(), scopes)
        })
        .collect()
}

pub(crate) struct Checker<'a> {
    program: &'a Program,
    registry: &'a MethodRegistry,
    problems: Vec<Diagnostic>,
    references: bool,
}

impl Checker<'_> {
    fn error(&mut self, code: &str, message: impl Into<String>, owner: Option<&str>) {
        self.detail(code, message, owner, None, None);
    }
    fn detail(
        &mut self,
        code: &str,
        message: impl Into<String>,
        owner: Option<&str>,
        expected: Option<String>,
        actual: Option<String>,
    ) {
        self.problems.push(Diagnostic {
            code: code.into(),
            message: message.into(),
            declaration: owner.map(str::to_owned),
            span: owner
                .and_then(|name| self.program.locations.get(name))
                .copied(),
            expected,
            actual,
        });
    }
    fn reference(&mut self, name: &str, exists: bool, kind: &str, owner: &str) -> bool {
        if !self.references {
            return false;
        }
        if !exists {
            self.detail(
                "unknown_reference",
                format!("Unknown {kind} {}", repr(name)),
                Some(owner),
                Some(kind.into()),
                Some(name.into()),
            );
        }
        exists
    }
    fn scope(&mut self, actual: &str, expected: &str, owner: &str, dependency: &str) {
        if self.references && actual != expected {
            self.detail(
                "environment_mismatch",
                format!(
                    "{} uses environment {}; expected {}",
                    repr(dependency),
                    repr(actual),
                    repr(expected)
                ),
                Some(owner),
                Some(expected.into()),
                Some(actual.into()),
            );
        }
    }
    fn identifier(&mut self, name: &str, owner: &str) {
        if !identifier(name) {
            self.error(
                "invalid_identifier",
                format!("{} cannot be represented as an EAL identifier", repr(name)),
                Some(owner),
            );
        }
    }
}

pub(crate) fn repr(text: &str) -> String {
    crate::expressions::python_repr(&Value::String(text.into()))
}

pub(crate) fn path(text: &str, hyphen: bool) -> bool {
    !text.is_empty()
        && text.split('.').all(|part| {
            let mut bytes = part.bytes();
            bytes
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
                && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_' || hyphen && c == b'-')
        })
}

pub fn identifier(text: &str) -> bool {
    const RESERVED: &[&str] = &[
        "language",
        "environment",
        "tool",
        "version",
        "evidence",
        "kind",
        "max_age",
        "input",
        "assumption",
        "statement",
        "validate",
        "valid_from",
        "valid_until",
        "reasoning",
        "rationale",
        "backing",
        "claim",
        "argument",
        "conclusion",
        "assumptions",
        "premises",
        "objection",
        "target",
        "require",
        "true",
        "false",
        "null",
        "and",
        "or",
        "not",
    ];
    path(text, false) && text.split('.').all(|part| !RESERVED.contains(&part))
}
