//! Typed method-input correspondence and exact-rational physical unit conversion.
use crate::ExecutionLimits;
use crate::expressions::{
    evaluate_expression, expression_errors_with_limits, format_expression, predicate_paths,
    python_repr,
};
use crate::methods::MethodRegistry;
use crate::semantic::{Predicate, Proposition};
use crate::validation::{check_json_resources, parse_time};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

pub const SCHEMA: &str = "EAL/typed-input/1";

/// Exact scale relative to the declared dimension's base unit.
pub fn unit_spec(unit: &str) -> Option<(&'static str, &'static str)> {
    Some(match unit {
        "1" => ("dimensionless", "1"),
        "Pa" => ("pressure", "1"),
        "kPa" => ("pressure", "1000"),
        "MPa" => ("pressure", "1000000"),
        "s" => ("time", "1"),
        "ms" => ("time", "0.001"),
        "us" => ("time", "0.000001"),
        "ns" => ("time", "0.000000001"),
        "min" => ("time", "60"),
        "h" => ("time", "3600"),
        "m" => ("length", "1"),
        "mm" => ("length", "0.001"),
        "cm" => ("length", "0.01"),
        "km" => ("length", "1000"),
        "kg" => ("mass", "1"),
        "g" => ("mass", "0.001"),
        "K" | "delta_degC" => ("temperature_difference", "1"),
        "m/s" => ("velocity", "1"),
        "km/h" => ("velocity", "5/18"),
        "m3/s" => ("volumetric_flow", "1"),
        "L/s" => ("volumetric_flow", "0.001"),
        _ => return None,
    })
}

pub fn quantity_dimension(quantity: &str) -> Option<&'static str> {
    Some(match quantity {
        "pressure" => "pressure",
        "time" => "time",
        "length" => "length",
        "mass" => "mass",
        "temperature_difference" => "temperature_difference",
        "velocity" => "velocity",
        "volumetric_flow" => "volumetric_flow",
        "dimensionless" | "probability" | "proposition" => "dimensionless",
        _ => return None,
    })
}

pub fn describe_bindings(registry: &MethodRegistry) -> Value {
    let quantities: BTreeMap<_, _> = [
        "pressure",
        "time",
        "length",
        "mass",
        "temperature_difference",
        "velocity",
        "volumetric_flow",
        "dimensionless",
        "probability",
        "proposition",
    ]
    .into_iter()
    .map(|q| (q, quantity_dimension(q).unwrap()))
    .collect();
    let units: BTreeMap<_, _> = [
        "1",
        "Pa",
        "kPa",
        "MPa",
        "s",
        "ms",
        "us",
        "ns",
        "min",
        "h",
        "m",
        "mm",
        "cm",
        "km",
        "kg",
        "g",
        "K",
        "delta_degC",
        "m/s",
        "km/h",
        "m3/s",
        "L/s",
    ]
    .into_iter()
    .map(|u| {
        let (dimension, base_scale) = unit_spec(u).unwrap();
        (u, json!({"dimension":dimension,"base_scale":base_scale}))
    })
    .collect();
    json!({"schema":SCHEMA,"registry_fingerprint":registry.fingerprint(),"quantities":quantities,"units":units,"methods":registry.describe(),"envelope_fields":["schema","method","subject","quantity","unit","scope","valid_from","valid_until","payload"],"interval":"half-open [valid_from, valid_until); input must contain claim interval","interpretation":"metadata correspondence; physical relevance and prose correspondence remain asserted"})
}

pub fn proposition_errors(
    proposition: &Proposition,
    method: Option<&str>,
    registry: &MethodRegistry,
) -> Vec<String> {
    proposition_errors_with_limits(proposition, method, registry, &ExecutionLimits::default())
}

pub fn proposition_errors_with_limits(
    proposition: &Proposition,
    method: Option<&str>,
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> Vec<String> {
    let mut errors = Vec::new();
    for (key, value) in [
        ("subject", &proposition.subject),
        ("scope", &proposition.scope),
    ] {
        if value.trim().is_empty() || value.chars().count() > 4096 {
            errors.push(format!(
                "{key} must be nonblank and at most 4096 characters"
            ));
        }
    }
    if quantity_dimension(&proposition.quantity).is_none() {
        errors.push(format!(
            "Unknown quantity {}",
            python_repr(&json!(proposition.quantity))
        ));
    }
    match unit_spec(&proposition.unit) {
        None => errors.push(format!(
            "Unknown unit {}",
            python_repr(&json!(proposition.unit))
        )),
        Some((dimension, _)) => {
            if quantity_dimension(&proposition.quantity).is_some_and(|d| d != dimension) {
                errors.push(format!(
                    "Unit {} does not measure quantity {}",
                    python_repr(&json!(proposition.unit)),
                    python_repr(&json!(proposition.quantity))
                ));
            }
        }
    }
    match (
        parse_time(&proposition.valid_from),
        parse_time(&proposition.valid_until),
    ) {
        (Ok(start), Ok(end)) => {
            if start >= end {
                errors.push("Proposition valid_from must precede valid_until".into());
            }
        }
        _ => errors.push("Proposition interval requires ISO-8601 timestamps with timezones".into()),
    }
    let expected = &proposition.result.expected;
    if proposition.result.expression.is_none() && !expected.is_boolean() && !numeric(expected) {
        errors.push("A scalar proposition result must compare a finite number or boolean".into());
    }
    if expected.is_boolean() && !["==", "!="].contains(&proposition.result.operator.as_str()) {
        errors.push("Boolean proposition results support only == and !=".into());
    }
    if check_json_resources(&proposition.query, limits).is_err() {
        errors.push("Proposition query must be finite JSON".into());
    }
    if let Some(method) = method {
        let Some(contract) = registry.get(method) else {
            errors.push(format!(
                "Unknown registered method {}",
                python_repr(&json!(method))
            ));
            return errors;
        };
        errors.extend(crate::methods::schema_errors_with_limits(
            &proposition.query,
            &contract.query_schema,
            "query",
            limits,
        ));
        if let Some(expression) = &proposition.result.expression {
            errors.extend(expression_errors_with_limits(
                expression,
                Some(&contract.output_schema),
                limits,
            ));
            let paths = predicate_paths(&proposition.result);
            if paths.is_empty() {
                errors.push("A proposition result must depend on a declared method output".into());
            }
            for path in paths {
                if contract.output_type(&path).is_none() {
                    errors.push(format!(
                        "Method {} has no typed result {}",
                        python_repr(&json!(method)),
                        python_repr(&json!(path))
                    ));
                }
            }
        } else {
            match contract.output_type(&proposition.result.path) {
                None => errors.push(format!(
                    "Method {} has no typed result {}",
                    python_repr(&json!(method)),
                    python_repr(&json!(proposition.result.path))
                )),
                Some(output) => {
                    if (output == "boolean") != expected.is_boolean() {
                        errors.push(
                            "Proposition result type does not match the method output type".into(),
                        );
                    }
                }
            }
        }
        if !contract.quantities.contains(&proposition.quantity) {
            errors.push(format!(
                "Method {} does not accept quantity {}",
                python_repr(&json!(method)),
                python_repr(&json!(proposition.quantity))
            ));
        }
    }
    errors
}

pub fn prepare_binding(
    proposition: &Proposition,
    method: &str,
    evidence_id: &str,
    value: &Value,
    registry: &MethodRegistry,
) -> (Option<Value>, Value) {
    prepare_binding_with_limits(
        proposition,
        method,
        evidence_id,
        value,
        registry,
        &ExecutionLimits::default(),
    )
}

pub fn prepare_binding_with_limits(
    proposition: &Proposition,
    method: &str,
    evidence_id: &str,
    value: &Value,
    registry: &MethodRegistry,
    limits: &ExecutionLimits,
) -> (Option<Value>, Value) {
    let Some(contract) = registry.get(method) else {
        return (
            None,
            json!({"status":"unsupported","reasons":[format!("Unknown registered method {}",python_repr(&json!(method)))]}),
        );
    };
    let mut trace = json!({"evidence_id":evidence_id,"method":contract.identifier,"proposition":proposition,"method_contract":contract.describe(),"prose_verified":false,"physical_interpretation_verified":false});
    let fields: BTreeSet<_> = [
        "schema",
        "method",
        "subject",
        "quantity",
        "unit",
        "scope",
        "valid_from",
        "valid_until",
        "payload",
    ]
    .into_iter()
    .collect();
    let Some(object) = value
        .as_object()
        .filter(|o| o.keys().map(String::as_str).collect::<BTreeSet<_>>() == fields)
    else {
        trace["status"] = json!("unsupported");
        trace["reasons"] = json!(["Typed input requires exactly the declared envelope fields"]);
        return (None, trace);
    };
    let mut reasons = Vec::<String>::new();
    for (key, expected) in [
        ("schema", SCHEMA),
        ("method", contract.identifier.as_str()),
        ("subject", proposition.subject.as_str()),
        ("quantity", proposition.quantity.as_str()),
        ("scope", proposition.scope.as_str()),
    ] {
        if object.get(key).and_then(Value::as_str) != Some(expected) {
            reasons.push(format!(
                "Typed input {key} does not match the proposition or method"
            ));
        }
    }
    let unit = object.get("unit").and_then(Value::as_str);
    if unit.and_then(unit_spec).map(|(d, _)| d) != quantity_dimension(&proposition.quantity)
        || unit.and_then(unit_spec).is_none()
    {
        reasons.push("Typed input unit is incompatible with the proposition quantity".into());
    }
    if contract.exact_unit && unit != Some(proposition.unit.as_str()) {
        reasons.push("This method embeds dimensional query constants and requires the exact proposition unit".into());
    }
    let interval = object
        .get("valid_from")
        .and_then(Value::as_str)
        .zip(object.get("valid_until").and_then(Value::as_str));
    match interval.map(|(start, end)| {
        (
            parse_time(start),
            parse_time(end),
            parse_time(&proposition.valid_from),
            parse_time(&proposition.valid_until),
        )
    }) {
        Some((Ok(start), Ok(end), Ok(claim_start), Ok(claim_end))) => {
            if start >= end || start > claim_start || end < claim_end {
                reasons.push(
                    "Typed input interval does not contain the whole proposition interval".into(),
                );
            }
        }
        _ => reasons.push("Typed input interval requires valid timezone-aware timestamps".into()),
    }
    let payload = &object["payload"];
    if let Some(payload) = payload.as_object() {
        for key in &contract.query_fields {
            let expected = proposition.query.get(key).unwrap_or(&Value::Null);
            if payload
                .get(key)
                .is_none_or(|actual| !crate::canonical::same_json(actual, expected, limits))
            {
                reasons.push(format!(
                    "Typed input query field {} does not match the declared proposition query",
                    python_repr(&json!(key))
                ));
            }
        }
    } else {
        reasons.push("Typed input payload must be an object".into());
    }
    match crate::canonical::canonical_digest(payload, limits) {
        Ok(digest) => {
            trace["input_payload_digest"] = json!(digest);
            trace["formal_query"] = payload.clone();
            trace["input_unit"] = unit.map_or(Value::Null, |u| json!(u));
        }
        Err(_) => reasons.push("Typed input payload must be finite JSON".into()),
    }
    let bound = reasons.is_empty();
    trace["status"] = json!(if bound { "bound" } else { "unsupported" });
    trace["reasons"] = json!(reasons);
    (bound.then(|| payload.clone()), trace)
}

pub fn check_result(
    proposition: &Proposition,
    method: &str,
    details: &Value,
    trace: &Value,
    registry: &MethodRegistry,
) -> Value {
    if let Some(expression) = &proposition.result.expression {
        let mut converted = details.clone();
        let mut observations = serde_json::Map::new();
        for path in predicate_paths(&proposition.result) {
            let output = registry.get(method).and_then(|c| c.output_type(&path));
            let mut scalar = proposition.clone();
            scalar.result = Predicate {
                path: path.clone(),
                operator: "==".into(),
                expected: if output == Some("boolean") {
                    json!(false)
                } else {
                    json!(0)
                },
                expression: None,
            };
            let checked = check_result(&scalar, method, details, trace, registry);
            let Some(actual) = checked.get("actual") else {
                return checked;
            };
            observations.insert(path.clone(), actual.clone());
            if !set_path(&mut converted, &path, actual.clone()) {
                return unsupported(trace, "Bound method output is missing");
            }
        }
        let result = evaluate_expression(expression, &converted);
        let Some(holds) = result.value.as_ref().and_then(Value::as_bool) else {
            return unsupported(
                trace,
                if result.reason.is_empty() {
                    "Result requires a Boolean"
                } else {
                    &result.reason
                },
            );
        };
        let mut result = trace_object(trace);
        result["status"] = json!(if holds { "supported" } else { "unsupported" });
        result["holds"] = json!(holds);
        result["actual"] = Value::Object(observations);
        result["output_unit"] = json!(proposition.unit);
        let rendered = match format_expression(expression) {
            Ok(rendered) => rendered,
            Err(error) => return unsupported(trace, &error),
        };
        result["reasons"] = json!([format!(
            "{} {}",
            rendered,
            if holds { "holds" } else { "does not hold" }
        )]);
        return result;
    }
    let Some(mut actual) = get_path(details, &proposition.result.path).cloned() else {
        return unsupported(trace, "Bound method output is missing");
    };
    let Some(output) = registry
        .get(method)
        .and_then(|c| c.output_type(&proposition.result.path))
    else {
        return unsupported(
            trace,
            "Bound method output is not declared by the method contract",
        );
    };
    if output == "boolean" && !actual.is_boolean() || output != "boolean" && !numeric(&actual) {
        return unsupported(trace, "Bound method output has an incompatible type");
    }
    let mut output_unit = if output == "boolean" {
        Value::Null
    } else {
        json!("1")
    };
    if output == "basis" {
        let Some((_, input_scale)) = trace
            .get("input_unit")
            .and_then(Value::as_str)
            .and_then(unit_spec)
        else {
            return unsupported(trace, "Bound input unit is not declared");
        };
        let Some((_, output_scale)) = unit_spec(&proposition.unit) else {
            return unsupported(trace, "Bound output unit is not declared");
        };
        let ratio = decimal_ratio(input_scale).unwrap() / decimal_ratio(output_scale).unwrap();
        if !ratio.is_one() {
            let Some(exact) = numeric_decimal(&actual).map(|number| number * ratio) else {
                return unsupported(trace, "Unit conversion exceeds the finite numeric range");
            };
            let Some(number) = exact.to_f64().filter(|v| v.is_finite()) else {
                return unsupported(trace, "Unit conversion exceeds the finite numeric range");
            };
            if !exact.is_zero() && number == 0.0 {
                return unsupported(
                    trace,
                    "Unit conversion underflow would erase a nonzero result",
                );
            }
            actual = json!(number);
        }
        output_unit = json!(proposition.unit);
        if !numeric(&actual) {
            return unsupported(trace, "Converted result is not finite");
        }
    }
    let ordering = if actual.is_number() && proposition.result.expected.is_number() {
        crate::expressions::numbers::compare(&actual, &proposition.result.expected)
    } else {
        None
    };
    let holds = match proposition.result.operator.as_str() {
        "==" => ordering.map_or_else(|| actual == proposition.result.expected, |v| v.is_eq()),
        "!=" => ordering.map_or_else(|| actual != proposition.result.expected, |v| !v.is_eq()),
        "<" => ordering.is_some_and(|v| v.is_lt()),
        "<=" => ordering.is_some_and(|v| v.is_le()),
        ">" => ordering.is_some_and(|v| v.is_gt()),
        ">=" => ordering.is_some_and(|v| v.is_ge()),
        _ => false,
    };
    let mut result = trace_object(trace);
    result["status"] = json!(if holds { "supported" } else { "unsupported" });
    result["holds"] = json!(holds);
    result["actual"] = actual;
    result["output_unit"] = output_unit;
    result["reasons"] = json!([format!(
        "Bound result {} {} {} {}",
        python_repr(&json!(proposition.result.path)),
        proposition.result.operator,
        python_repr(&proposition.result.expected),
        if holds { "holds" } else { "does not hold" }
    )]);
    result
}

fn numeric(value: &Value) -> bool {
    value.as_number().is_some_and(|n| {
        !n.to_string().contains(['.', 'e', 'E']) || n.as_f64().is_some_and(f64::is_finite)
    })
}
fn trace_object(trace: &Value) -> Value {
    if trace.is_object() {
        trace.clone()
    } else {
        json!({})
    }
}
fn unsupported(trace: &Value, message: &str) -> Value {
    let mut result = trace_object(trace);
    result["status"] = json!("unsupported");
    result["reasons"] = json!([message]);
    result
}
fn get_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |value, key| value.as_object()?.get(key))
}
fn set_path(value: &mut Value, path: &str, actual: Value) -> bool {
    let mut destination = value;
    let mut parts = path.split('.').peekable();
    while let Some(key) = parts.next() {
        if parts.peek().is_none() {
            if let Some(object) = destination.as_object_mut() {
                object.insert(key.into(), actual);
                return true;
            }
            return false;
        }
        let Some(next) = destination.get_mut(key) else {
            return false;
        };
        destination = next;
    }
    false
}
fn numeric_decimal(value: &Value) -> Option<BigRational> {
    let number = value.as_number()?;
    let text = number.to_string();
    if text.contains(['.', 'e', 'E']) {
        decimal_ratio(&crate::canonical::python_float_repr(number.as_f64()?))
    } else {
        decimal_ratio(&text)
    }
}
fn decimal_ratio(text: &str) -> Option<BigRational> {
    if let Some((left, right)) = text.split_once('/') {
        return Some(BigRational::new(
            BigInt::from_str(left).ok()?,
            BigInt::from_str(right).ok()?,
        ));
    }
    let (mantissa, exponent) = if let Some(index) = text.find(['e', 'E']) {
        (&text[..index], text[index + 1..].parse::<i32>().ok()?)
    } else {
        (text, 0)
    };
    let decimals = mantissa
        .split_once('.')
        .map_or(0, |(_, part)| part.len() as i32);
    let numerator = BigInt::from_str(&mantissa.replace('.', "")).ok()?;
    let power = exponent - decimals;
    if power.unsigned_abs() > 10000 {
        return None;
    }
    if power >= 0 {
        Some(BigRational::from_integer(
            numerator * BigInt::from(10u8).pow(power as u32),
        ))
    } else {
        Some(BigRational::new(
            numerator,
            BigInt::from(10u8).pow((-power) as u32),
        ))
    }
}
