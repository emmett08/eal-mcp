//! Composed EAL/3 entities, independent of parsing contexts and execution adapters.
use crate::{ExecutionLimits, Expression};
use eal_parser::SourceSpan;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Predicate {
    pub path: String,
    pub operator: String,
    pub expected: Value,
    #[serde(with = "crate::expressions::optional_serde")]
    pub expression: Option<Expression>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub predicates: Vec<Predicate>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub name: String,
    pub tool: String,
    pub kind: String,
    pub environment: String,
    pub max_age: f64,
    pub input: Value,
    pub predicates: Vec<Predicate>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assumption {
    pub name: String,
    pub statement: String,
    pub environment: String,
    pub validation: String,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScopeTransfer {
    pub source: String,
    pub target: String,
    pub assumption: String,
    pub review: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reasoning {
    pub name: String,
    pub method: String,
    pub rationale: String,
    pub backing: Vec<String>,
    pub predicates: Vec<Predicate>,
    pub transfer: Option<ScopeTransfer>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proposition {
    pub subject: String,
    pub quantity: String,
    pub unit: String,
    pub scope: String,
    pub valid_from: String,
    pub valid_until: String,
    pub result: Predicate,
    pub query: Value,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub name: String,
    pub statement: String,
    pub environment: String,
    pub proposition: Option<Proposition>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentOrigin {
    pub pattern: String,
    pub application: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    pub conclusion: String,
    pub reasoning: String,
    pub evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub premises: Vec<String>,
    pub binding: Option<String>,
    pub origin: Option<ArgumentOrigin>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternParameter {
    pub name: String,
    pub kind: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub parameters: Vec<PatternParameter>,
    pub conclusion: String,
    pub reasoning: String,
    pub evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub premises: Vec<String>,
    pub binding: Option<String>,
    pub body: Option<Vec<crate::Declaration>>,
    pub decreases: Option<String>,
    #[serde(skip)]
    pub defaults: BTreeMap<String, crate::Field>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BindingValue {
    Single(String),
    List(Vec<String>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternBinding {
    pub name: String,
    pub reference: BindingValue,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub pattern: String,
    pub arguments: Vec<PatternBinding>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objection {
    pub name: String,
    pub target_kind: String,
    pub target: String,
    pub evidence: Vec<String>,
    pub premises: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentationDirective {
    pub kind: String,
    pub name: String,
    pub other: Option<String>,
    pub rank: Option<i64>,
    pub review: String,
    #[serde(with = "crate::semantic_spans::required")]
    pub span: SourceSpan,
    pub source_file: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub declaration: Option<String>,
    #[serde(with = "crate::semantic_spans::optional")]
    pub span: Option<SourceSpan>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub language: String,
    pub source_digest: String,
    pub environments: IndexMap<String, Environment>,
    pub tools: IndexMap<String, Tool>,
    pub evidence: IndexMap<String, Evidence>,
    pub assumptions: IndexMap<String, Assumption>,
    pub reasoning: IndexMap<String, Reasoning>,
    pub claims: IndexMap<String, Claim>,
    pub arguments: IndexMap<String, Argument>,
    pub objections: IndexMap<String, Objection>,
    pub patterns: IndexMap<String, Pattern>,
    pub applications: IndexMap<String, Application>,
    pub argumentation_directives: Vec<ArgumentationDirective>,
    pub duplicates: Vec<String>,
    pub declaration_count: usize,
    #[serde(with = "crate::semantic_spans::locations")]
    pub locations: BTreeMap<String, SourceSpan>,
    pub lowering_diagnostics: Vec<Diagnostic>,
    pub authored: Option<crate::Program>,
    pub generated: Vec<String>,
    pub imports: BTreeMap<String, String>,
    pub limits: ExecutionLimits,
    pub source_files: BTreeMap<String, String>,
    #[serde(skip)]
    pub import_sources: BTreeMap<String, String>,
    #[serde(skip)]
    pub root_origin: String,
}

impl Program {
    pub fn empty(language: String, source_digest: String, limits: ExecutionLimits) -> Self {
        Self {
            language,
            source_digest,
            limits,
            environments: IndexMap::new(),
            tools: IndexMap::new(),
            evidence: IndexMap::new(),
            assumptions: IndexMap::new(),
            reasoning: IndexMap::new(),
            claims: IndexMap::new(),
            arguments: IndexMap::new(),
            objections: IndexMap::new(),
            patterns: IndexMap::new(),
            applications: IndexMap::new(),
            argumentation_directives: Vec::new(),
            duplicates: Vec::new(),
            declaration_count: 0,
            locations: BTreeMap::new(),
            lowering_diagnostics: Vec::new(),
            authored: None,
            generated: Vec::new(),
            imports: BTreeMap::new(),
            source_files: BTreeMap::new(),
            import_sources: BTreeMap::new(),
            root_origin: String::new(),
        }
    }
}
