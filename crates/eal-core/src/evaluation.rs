//! Pure authored support assessment with explicit observations, context and time.
use crate::{
    ExecutionLimits, canonical,
    methods::{self, MethodRegistry},
    semantic::Program,
    validation,
};
use chrono::{DateTime, SecondsFormat, Timelike, Utc};
use serde_json::{Value, json};

mod arguments;
mod evidence;
mod graph;
pub(crate) use evidence::predicate;
pub use evidence::{EvidenceVerdict, assess_environment, assess_evidence_record};

/// Select an explicitly supplied environment context without inferring a transfer.
pub fn environment_context(name: &str, context: &Value) -> Result<Value, String> {
    let object = context
        .as_object()
        .ok_or("Context and records must be mappings")?;
    if let Some(environments) = object.get("$environments") {
        let environments = environments
            .as_object()
            .filter(|o| o.values().all(Value::is_object))
            .ok_or("$environments must map environment names to JSON objects")?;
        Ok(environments.get(name).cloned().unwrap_or_else(|| json!({})))
    } else {
        Ok(context.clone())
    }
}
pub fn environment_fingerprint(
    name: &str,
    context: &Value,
    limits: &ExecutionLimits,
) -> Result<String, String> {
    canonical::canonical_digest(&json!({"environment":name,"context":context}), limits)
}

pub(crate) fn entry(status: &str, reasons: Vec<String>) -> Value {
    json!({"status":status,"reasons":reasons})
}
pub(crate) fn status<'a>(result: &'a Value, group: &str, name: &str) -> &'a str {
    result[group][name]["status"].as_str().unwrap_or("")
}
pub(crate) fn push_reason(value: &mut Value, reason: String) {
    value["reasons"]
        .as_array_mut()
        .expect("assessment reasons are an array")
        .push(Value::String(reason));
}

/// Share the language's retained Python scalar/container diagnostic convention.
pub(crate) fn py_repr(value: &Value) -> String {
    crate::expressions::python_repr(value)
}
pub(crate) fn name_repr(name: &str) -> String {
    py_repr(&Value::String(name.to_owned()))
}
pub(crate) fn general_number(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    // Decide fixed versus scientific notation after six-significant-digit rounding.
    let scientific = format!("{value:.5e}");
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("scientific float notation");
    let exponent: i32 = exponent.parse().expect("float exponent");
    if !(-4..6).contains(&exponent) {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!(
            "{mantissa}e{}{absolute:02}",
            if exponent < 0 { '-' } else { '+' },
            absolute = exponent.abs()
        )
    } else {
        let places = (5 - exponent).max(0) as usize;
        let fixed = format!("{value:.places$}");
        if fixed.contains('.') {
            fixed.trim_end_matches('0').trim_end_matches('.').into()
        } else {
            fixed
        }
    }
}

/// Assess a source-bound programme with no clock, collector or filesystem access.
///
/// Record integrity fields establish correspondence, not authentication. Scoped
/// alternatives and objections are resolved by the explicit support graph.
pub fn evaluate(
    program: &Program,
    records: &Value,
    now: &str,
    context: &Value,
    registry: &MethodRegistry,
    binding_digests: Option<&Value>,
) -> Result<Value, String> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("eal-evaluation".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || {
                evaluate_owned(program, records, now, context, registry, binding_digests)
            })
            .map_err(|error| format!("Cannot start bounded evaluation worker: {error}"))?;
        worker
            .join()
            .map_err(|_| "Evaluation worker failed".to_owned())?
    })
}

/// Use the immutable built-in registry.
pub fn evaluate_default(
    program: &Program,
    records: &Value,
    now: &str,
    context: &Value,
) -> Result<Value, String> {
    evaluate(
        program,
        records,
        now,
        context,
        methods::default_registry(),
        None,
    )
}

fn evaluate_owned(
    program: &Program,
    records: &Value,
    now: &str,
    context: &Value,
    registry: &MethodRegistry,
    binding_digests: Option<&Value>,
) -> Result<Value, String> {
    let instant = validation::parse_time(now)?;
    if !context.is_object() || !records.is_object() {
        return Err("Context and records must be mappings".into());
    }
    if binding_digests.is_some_and(|v| !v.is_object()) {
        return Err(
            "binding_digests must map evidence IDs to current operator configurations".into(),
        );
    }
    if records.as_object().unwrap().len() > program.limits.declarations {
        return Err("Evidence records exceed the host declaration budget".into());
    }
    let diagnostics = validation::validate(program, registry);
    let timestamp = instant.to_rfc3339_opts(
        if instant.nanosecond() == 0 {
            SecondsFormat::Secs
        } else {
            SecondsFormat::Micros
        },
        true,
    );
    let mut result = json!({"language":program.language,"source_digest":program.source_digest,"assessed_at":timestamp,"context_fingerprint":canonical::canonical_digest(context,&program.limits)?,"method_registry_fingerprint":registry.fingerprint(),"valid":diagnostics.is_empty(),"complete":diagnostics.is_empty(),"diagnostics":diagnostics,"environments":{},"evidence":{},"assumptions":{},"reasoning":{},"objections":{},"arguments":{},"claims":{}});
    if !result["valid"].as_bool().unwrap() {
        return Ok(result);
    }
    for (name, environment) in &program.environments {
        result["environments"][name] = assess_environment(environment, context)?;
    }
    for (name, evidence) in &program.evidence {
        result["evidence"][name] = assess_evidence_record(
            program,
            name,
            records.get(name),
            instant,
            context,
            status(&result, "environments", &evidence.environment) == "matched",
            binding_digests.and_then(|v| v.get(name)),
            binding_digests.is_some(),
        )?
        .entry;
    }
    assess_local_declarations(program, instant, &mut result)?;
    let sources = arguments::assess_source_claims(program, records, &mut result, registry)?;
    result["complete"] = json!(!result["arguments"].as_object().unwrap().values().any(
        |v| v["reasoning_result"]["status"] == methods::MethodResultStatus::Incomplete.as_str()
    ));
    let composed = graph::construct(program, &mut result, &sources).and_then(|graph| {
        let grounded = crate::dialectic::solve_composed(
            &graph.nodes,
            &graph.claims,
            &json!(graph.attacks),
            &program.limits,
        )?;
        Ok((graph, grounded))
    });
    match composed {
        Ok((graph, grounded)) => graph::project(program, &mut result, &sources, &graph, grounded),
        Err(error) => {
            result["valid"] = json!(false);
            result["complete"] = json!(false);
            result["diagnostics"].as_array_mut().unwrap().push(json!({"code":"resource_limit","message":error,"declaration":null,"span":null,"expected":null,"actual":null}));
            for group in ["arguments", "claims", "objections"] {
                result[group] = json!({});
            }
        }
    }
    Ok(result)
}

fn assess_local_declarations(
    program: &Program,
    instant: DateTime<Utc>,
    result: &mut Value,
) -> Result<(), String> {
    for (name, assumption) in &program.assumptions {
        let starts = assumption
            .valid_from
            .as_deref()
            .map(validation::parse_time)
            .transpose()?;
        let ends = assumption
            .valid_until
            .as_deref()
            .map(validation::parse_time)
            .transpose()?;
        let not_started = starts.is_some_and(|start| instant < start);
        let expired = ends.is_some_and(|end| instant >= end);
        let time_status = if not_started {
            "not_started"
        } else if expired {
            "expired"
        } else {
            "within_interval"
        };
        let mut reasons = vec![];
        let status = if status(result, "environments", &assumption.environment) != "matched" {
            reasons.push(format!(
                "Environment {} does not match supplied context",
                name_repr(&assumption.environment)
            ));
            "out_of_scope"
        } else {
            if not_started {
                reasons.push("The assumption's validity interval has not begun".into());
            }
            if expired {
                reasons.push("The assumption's validity interval has ended".into());
            }
            if status(result, "evidence", &assumption.validation) != "available" {
                reasons.push(format!(
                    "Validation evidence {} is unavailable",
                    name_repr(&assumption.validation)
                ));
            }
            if reasons.is_empty() {
                reasons.push(format!(
                    "Validation evidence {} holds within the declared interval",
                    name_repr(&assumption.validation)
                ));
                "supported"
            } else {
                "unsupported"
            }
        };
        result["assumptions"][name] = json!({"status":status,"reasons":reasons,"validation":assumption.validation,"time_status":time_status,"valid_from":assumption.valid_from,"valid_until":assumption.valid_until});
    }
    for (name, reasoning) in &program.reasoning {
        let mut reasons: Vec<_> = reasoning
            .backing
            .iter()
            .filter(|e| status(result, "evidence", e) != "available")
            .map(|e| {
                format!(
                    "Method {}: backing evidence {} is unavailable",
                    reasoning.method,
                    name_repr(e)
                )
            })
            .collect();
        let state = if reasons.is_empty() {
            reasons.push(format!("Method {} and backing are locally available; acceptance is assessed per application",reasoning.method));
            "supported"
        } else {
            "unsupported"
        };
        result["reasoning"][name] = json!({"status":state,"reasons":reasons,"rationale":reasoning.rationale,"backing":reasoning.backing,"method":reasoning.method});
    }
    Ok(())
}

pub(crate) fn proposition_value(proposition: Option<&crate::semantic::Proposition>) -> Value {
    proposition
        .map(|p| serde_json::to_value(p).expect("checked proposition serialises"))
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::general_number;
    #[test]
    fn observation_age_uses_six_significant_digit_python_diagnostics() {
        for (number, expected) in [
            (61.0, "61"),
            (1_000_000.0, "1e+06"),
            (999_999.5, "1e+06"),
            (1e-5, "1e-05"),
            (1.23456789, "1.23457"),
            (1e20, "1e+20"),
            (-0.0, "-0"),
            (9.999995e-5, "0.0001"),
        ] {
            assert_eq!(general_number(number), expected);
        }
    }
}
