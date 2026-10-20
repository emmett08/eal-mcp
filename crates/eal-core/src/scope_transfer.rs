//! Reviewed conditional transfer between corresponding typed propositions.
use crate::semantic::{Argument, Program};
use crate::validation::parse_time;
use serde_json::{Value, json};

pub fn transfer_errors(program: &Program, argument: &Argument) -> Vec<String> {
    let Some(method) = program.reasoning.get(&argument.reasoning) else {
        return Vec::new();
    };
    let Some(relation) = &method.transfer else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    if method.method != "structured/1" {
        errors.push(
            "Scope transfer requires structured/1 and an explicit conditional relation".into(),
        );
    }
    if relation.source == relation.target {
        errors.push("Scope transfer must name distinct environments".into());
    }
    if !argument.assumptions.contains(&relation.assumption) {
        errors.push("The transfer assumption must be an explicit argument dependency".into());
    }
    if program
        .assumptions
        .get(&relation.assumption)
        .is_none_or(|a| a.environment != relation.target)
    {
        errors.push("The transfer assumption must be declared in the target environment".into());
    }
    if argument.binding.is_some() {
        errors.push("Scope transfer binds the source proposition through its premise, not an evidence result".into());
    }
    let target = program.claims.get(&argument.conclusion);
    if target
        .is_none_or(|claim| claim.environment != relation.target || claim.proposition.is_none())
    {
        errors.push(
            "Scope transfer requires a typed target claim in the declared target environment"
                .into(),
        );
    }
    let source = if argument.premises.len() == 1 {
        program.claims.get(&argument.premises[0])
    } else {
        None
    };
    if source
        .is_none_or(|claim| claim.environment != relation.source || claim.proposition.is_none())
    {
        errors.push(
            "Scope transfer requires exactly one typed premise in the declared source environment"
                .into(),
        );
    }
    if let (Some(a), Some(b)) = (
        source.and_then(|c| c.proposition.as_ref()),
        target.and_then(|c| c.proposition.as_ref()),
    ) {
        for key in ["subject", "quantity", "unit", "query", "result"] {
            let a = serde_json::to_value(a).unwrap_or(Value::Null);
            let b = serde_json::to_value(b).unwrap_or(Value::Null);
            if !crate::canonical::same_json(&a[key], &b[key], &program.limits) {
                errors.push(format!("Scope transfer must preserve proposition {key}"));
            }
        }
        match (
            parse_time(&a.valid_from),
            parse_time(&a.valid_until),
            parse_time(&b.valid_from),
            parse_time(&b.valid_until),
        ) {
            (Ok(a_start), Ok(a_end), Ok(b_start), Ok(b_end)) => {
                if b_start < a_start || b_end > a_end {
                    errors.push(
                        "The target interval must be contained in the source interval".into(),
                    );
                }
            }
            _ => errors.push("Scope transfer requires checked proposition intervals".into()),
        }
    }
    errors
}

pub fn computation(program: &Program, argument: &Argument) -> Value {
    let relation = program
        .reasoning
        .get(&argument.reasoning)
        .and_then(|r| r.transfer.as_ref());
    if relation.is_none() {
        return json!({"status":"unsupported","method":"structured/1","reasons":["Scope transfer requires an explicit reviewed relation"],"details":{}});
    }
    let errors = transfer_errors(program, argument);
    if !errors.is_empty() {
        return json!({"status":"unsupported","method":"structured/1","reasons":errors,"details":{}});
    }
    json!({"status":"supported","method":"structured/1","reasons":["The source proposition is conditionally transported under the explicit reviewed assumption"],"details":{"authored":true,"mechanically_proved":false},"binding":{"status":"supported","kind":"scope_transfer","reasons":["Exact proposition correspondence and the contained interval are checked"],"relation":relation,"source_claim":argument.premises.first(),"target_claim":argument.conclusion,"correspondence_checked":true,"transport_justification_verified":false,"prose_verified":false}})
}
