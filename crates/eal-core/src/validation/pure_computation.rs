//! Admit closed mathematical arguments without manufacturing evidence observations.
use crate::{
    methods::{MethodRegistry, schema_errors_with_limits},
    semantic::{Argument, Program},
};

pub(super) fn permitted(program: &Program, argument: &Argument, registry: &MethodRegistry) -> bool {
    if argument.binding.is_some() {
        return false;
    }
    let Some(proposition) = program
        .claims
        .get(&argument.conclusion)
        .and_then(|claim| claim.proposition.as_ref())
    else {
        return false;
    };
    let Some(reasoning) = program.reasoning.get(&argument.reasoning) else {
        return false;
    };
    let mathematical = match reasoning.method.as_str() {
        "finite_probability/1" => true,
        "deductive/1" => {
            proposition.query["premises"]
                .as_array()
                .is_some_and(Vec::is_empty)
                && proposition.result.path == "entailed"
                && proposition.result.operator == "=="
                && proposition.result.expected == true
        }
        _ => false,
    };
    mathematical
        && registry.get(&reasoning.method).is_some_and(|contract| {
            schema_errors_with_limits(
                &proposition.query,
                &contract.input_schema,
                "query",
                &program.limits,
            )
            .is_empty()
        })
}
