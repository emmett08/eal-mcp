//! Adapt bounded JSON and source inputs to the opt-in ASPIC library.
use crate::input::Workspace;
use eal_aspic::{AspicError, CompileOptions};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};

fn result(value: Result<Value, AspicError>) -> Result<Value, String> {
    match value {
        Ok(value) => Ok(value),
        Err(AspicError::Invalid(reason)) => Err(reason),
        Err(AspicError::Incomplete(reason)) => Ok(json!({
            "status":"incomplete", "complete":false, "method":eal_aspic::METHOD,
            "reasons":[reason], "details":{}
        })),
    }
}

pub fn json_command(
    workspace: &Workspace,
    name: &str,
    command: &str,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    let payload = eal_core::json::parse(&workspace.source(name, limits.record_bytes)?, limits)?;
    if command == "export-aspic" {
        return eal_aspic::export_aspic_view(&payload, limits);
    }
    result(
        eal_aspic::solve_aspic(&payload, limits)
            .map(|formal| json!({"theory":payload["theory"], "formal":formal})),
    )
}

pub fn compile(
    source: &str,
    records: &Value,
    options: CompileOptions<'_>,
) -> Result<Value, String> {
    result(
        eal_aspic::compile_eal_aspic(source, records, options)
            .and_then(|compiled| compiled.to_dict()),
    )
}
