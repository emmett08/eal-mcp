//! Observation-bound EAL/3 to the explicit finite ASPIC profile.
use crate::AspicError;
use eal_core::{
    ExecutionLimits, canonical, composition, evaluation, methods, semantic as s, validation,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

mod observations;
mod policies;
mod routes;
mod snapshot;
mod translation;
pub use snapshot::CompiledTheory;

pub const PROFILE: &str = "EAL/3-compiled-aspic/5";
const RANK: i64 = 500;

/// All authority and snapshot inputs are supplied by the caller. Compilation
/// performs no collection, clock lookup or filesystem access.
pub struct CompileOptions<'a> {
    pub goal: &'a str,
    pub now: &'a str,
    pub context: &'a Value,
    pub registry: Option<&'a methods::MethodRegistry>,
    pub binding_digests: Option<&'a Value>,
    pub limits: ExecutionLimits,
    pub resolver: Option<&'a composition::SourceResolver<'a>>,
    pub origin: &'a str,
    pub semantics: &'a str,
    pub query_mode: &'a str,
    pub preference: Option<&'a str>,
}
impl<'a> CompileOptions<'a> {
    pub fn new(goal: &'a str, now: &'a str, context: &'a Value) -> Self {
        Self {
            goal,
            now,
            context,
            registry: None,
            binding_digests: None,
            limits: ExecutionLimits::default(),
            resolver: None,
            origin: "",
            semantics: "grounded",
            query_mode: "sceptical",
            preference: None,
        }
    }
}

pub fn compile_eal_aspic(
    source: &str,
    records: &Value,
    options: CompileOptions<'_>,
) -> Result<CompiledTheory, AspicError> {
    options.limits.validate()?;
    validation::check_json_resources(records, &options.limits)?;
    validation::check_json_resources(options.context, &options.limits)?;
    if let Some(bindings) = options.binding_digests {
        validation::check_json_resources(bindings, &options.limits)?;
    }
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("eal-aspic-compiler".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || compile(source, records, options))
            .map_err(|error| format!("Cannot start bounded ASPIC compiler: {error}"))?
            .join()
            .map_err(|_| "Bounded ASPIC compiler failed".to_owned())?
    })
}

fn symbol(kind: &str, name: &str) -> String {
    format!(
        "{kind}{:.20}",
        format!("{:x}", Sha256::digest(name.as_bytes()))
    )
}
fn sorted<'a>(keys: impl Iterator<Item = &'a String>) -> Vec<&'a str> {
    let mut names: Vec<_> = keys.map(String::as_str).collect();
    names.sort();
    names
}
fn unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|v| seen.insert(v.clone()))
        .collect()
}
fn location(program: &s::Program, name: &str) -> Value {
    program.locations.get(name).map_or(Value::Null,|span|json!({"line":span.start.line,"column":span.start.column,"end_line":span.end.line,"end_column":span.end.column}))
}
fn backend(limits: &ExecutionLimits) -> Result<Value, String> {
    let contract = crate::registry::aspic_contract();
    Ok(
        json!({"method":contract.identifier,"contract_digest":canonical::canonical_digest(&contract.describe(),limits)?,"module_digest":crate::registry::module_identity()}),
    )
}
fn snapshot_digest(mapping: &Value, limits: &ExecutionLimits) -> Result<String, String> {
    let mut value = json!({"evidence_record_digests":{}});
    for key in [
        "source_digest",
        "assessed_at",
        "context_fingerprint",
        "method_registry_fingerprint",
        "backend",
        "assessment_digest",
        "theory_digest",
        "execution_limits",
    ] {
        value[key] = mapping
            .get(key)
            .ok_or_else(|| format!("Missing snapshot field {key}"))?
            .clone();
    }
    for (name, evidence) in mapping["evidence"]
        .as_object()
        .ok_or("Invalid evidence source map")?
    {
        value["evidence_record_digests"][name] = evidence
            .get("record_digest")
            .ok_or("Missing evidence record digest")?
            .clone();
    }
    canonical::canonical_digest(&value, limits)
}
fn repr(name: &str, limits: &ExecutionLimits) -> Result<String, String> {
    eal_core::expressions::diagnostic_repr(&Value::String(name.into()), limits)
}

fn compile(
    source: &str,
    records: &Value,
    options: CompileOptions<'_>,
) -> Result<CompiledTheory, AspicError> {
    let program = composition::compile(
        source,
        options.resolver,
        options.origin,
        options.limits.clone(),
    )
    .map_err(|error| error.to_string())?;
    let limits = &program.limits;
    if program.evidence.len() > limits.formal_premises
        || program.arguments.len() + program.assumptions.len() + program.objections.len()
            > limits.formal_rules
    {
        return Err(AspicError::Incomplete(
            "Compiled profile exceeds the host premise or rule budget".into(),
        ));
    }
    let assessment = evaluation::evaluate(
        &program,
        records,
        options.now,
        options.context,
        options
            .registry
            .unwrap_or_else(|| methods::default_registry()),
        options.binding_digests,
    )?;
    if assessment["valid"] != true {
        let message = format!(
            "EAL programme is invalid: {}",
            eal_core::expressions::diagnostic_repr(&assessment["diagnostics"], limits)?
        );
        return Err(
            if assessment["diagnostics"]
                .as_array()
                .is_some_and(|diagnostics| {
                    diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic["code"] == "resource_limit")
                })
            {
                AspicError::Incomplete(message)
            } else {
                AspicError::Invalid(message)
            },
        );
    }
    if assessment["complete"] == false {
        return Err(AspicError::Incomplete(
            "Local EAL assessment is incomplete; no partial formal snapshot is compiled".into(),
        ));
    }
    if !program.claims.contains_key(options.goal) {
        return Err(format!("Unknown EAL goal claim {}", repr(options.goal, limits)?).into());
    }
    if program
        .arguments
        .values()
        .any(|a| program.reasoning[&a.reasoning].method == "argumentation/aspic/2")
    {
        return Err("Compile authored EAL routes, not an argumentation/aspic/2 theory".into());
    }
    let (theory, mut mapping) = translation::translate(&program, records, &assessment, &options)?;
    crate::solver::validate_theory(&theory, limits).map_err(|error| match error {
        AspicError::Incomplete(message) => AspicError::Incomplete(format!(
            "Compiled theory exceeds its bounded profile: {message}"
        )),
        AspicError::Invalid(message) => AspicError::Invalid(format!(
            "Compiled theory exceeds its bounded profile: {message}"
        )),
    })?;
    mapping["assessment_digest"] = json!(canonical::canonical_digest(&assessment, limits)?);
    mapping["theory_digest"] = json!(canonical::canonical_digest(&theory, limits)?);
    mapping["snapshot_digest"] = json!(snapshot_digest(&mapping, limits)?);
    CompiledTheory::new(theory, mapping, assessment, program.limits)
}
