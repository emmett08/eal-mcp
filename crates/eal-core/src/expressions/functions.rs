//! Closed pure function library, separate from expression-tree interpretation.
use super::{
    Quantity, numbers, python_repr,
    runtime::{RuntimeValue, compare_values},
};
use crate::{BinaryOperator, ExecutionLimits, canonical};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn call(
    name: &str,
    values: Vec<RuntimeValue>,
    limits: &ExecutionLimits,
) -> Result<RuntimeValue, String> {
    let valid_count = match name {
        "literal" | "count" | "sum" | "abs" | "all" | "any" | "flatten" | "distinct" => {
            values.len() == 1
        }
        "quantity" | "at" | "contains" | "equal" => values.len() == 2,
        "exact_sum" | "exact_mean" | "exact_min" | "exact_max" | "exact_count" => values.len() == 1,
        "all_values" | "any_values" => values.len() == 1,
        "exact_scale" => values.len() == 5,
        "exact_compare" => values.len() == 4,
        "object" => values.len().is_multiple_of(2),
        "min" | "max" => !values.is_empty(),
        _ => {
            return Err(format!(
                "Unknown pure function {}",
                python_repr(&Value::String(name.into()))
            ));
        }
    };
    if !valid_count {
        return Err(format!("Function {name:?} has an invalid argument count"));
    }
    match name {
        "literal" => Ok(values[0].clone()),
        "quantity" => quantity(&values),
        "object" => object(values),
        "count" => count(&values[0]),
        "at" => at(&values),
        "flatten" => flatten(&values[0], limits),
        "distinct" => distinct(&values[0], limits),
        "contains" => contains(&values, limits),
        "equal" => equal(&values, limits),
        "all_values" | "any_values" => selected_booleans(name, &values[0], limits),
        "exact_sum" | "exact_mean" | "exact_min" | "exact_max" | "exact_count" => Ok(
            RuntimeValue::Json(super::exact::aggregate(name, json(&values[0])?, limits)?),
        ),
        "exact_scale" | "exact_compare" => {
            let json = values
                .iter()
                .map(|value| json(value).cloned())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(RuntimeValue::Json(if name == "exact_scale" {
                super::exact::scale(&json, limits)?
            } else {
                super::exact::compare(&json)?
            }))
        }
        "sum" => sum(&values[0]),
        "all" | "any" => boolean_array(name, &values[0]),
        "min" | "max" => extremum(name, values),
        "abs" => absolute(&values[0]),
        _ => unreachable!("closed function catalogue checked above"),
    }
}

/// Construct an explicit JSON object; duplicate names and physical quantities
/// cannot be silently coerced into untyped JSON.
fn object(values: Vec<RuntimeValue>) -> Result<RuntimeValue, String> {
    let mut fields = serde_json::Map::new();
    let mut values = values.into_iter();
    while let Some(key) = values.next() {
        let RuntimeValue::Json(Value::String(key)) = key else {
            return Err("object keys must be strings".into());
        };
        let Some(RuntimeValue::Json(value)) = values.next() else {
            return Err(
                "object values must be JSON; quantities require explicit unit fields".into(),
            );
        };
        if fields.insert(key, value).is_some() {
            return Err("object keys must be unique".into());
        }
    }
    Ok(RuntimeValue::Json(Value::Object(fields)))
}

fn quantity(values: &[RuntimeValue]) -> Result<RuntimeValue, String> {
    let (RuntimeValue::Json(value), RuntimeValue::Json(Value::String(unit))) =
        (&values[0], &values[1])
    else {
        return Err("quantity requires a finite number and a registered unit".into());
    };
    Ok(RuntimeValue::Quantity(Quantity::new(value, unit)?))
}
fn array<'a>(name: &str, value: &'a RuntimeValue) -> Result<&'a [Value], String> {
    let RuntimeValue::Json(Value::Array(items)) = value else {
        return Err(format!("{name} requires an array"));
    };
    Ok(items)
}
fn count(value: &RuntimeValue) -> Result<RuntimeValue, String> {
    Ok(RuntimeValue::Json(Value::from(
        array("count", value)?.len(),
    )))
}
fn sum(value: &RuntimeValue) -> Result<RuntimeValue, String> {
    let items = array("sum", value)?;
    if items.iter().any(|item| !numbers::is_numeric(item)) {
        return Err("sum requires finite numerical elements".into());
    }
    Ok(RuntimeValue::Json(numbers::sum(items)?))
}
fn boolean_array(name: &str, value: &RuntimeValue) -> Result<RuntimeValue, String> {
    let items = array(name, value)?;
    if items.iter().any(|item| !item.is_boolean()) {
        return Err(format!("{name} requires Boolean elements"));
    }
    Ok(RuntimeValue::Json(Value::Bool(if name == "all" {
        items.iter().all(|item| item == &Value::Bool(true))
    } else {
        items.iter().any(|item| item == &Value::Bool(true))
    })))
}
fn extremum(name: &str, values: Vec<RuntimeValue>) -> Result<RuntimeValue, String> {
    let items = if values.len() == 1 {
        if let RuntimeValue::Json(Value::Array(items)) = &values[0] {
            items
                .iter()
                .cloned()
                .map(RuntimeValue::Json)
                .collect::<Vec<_>>()
        } else {
            values
        }
    } else {
        values
    };
    if items.is_empty() {
        return Err(format!(
            "{name} requires nonempty finite numerical operands"
        ));
    }
    if items
        .iter()
        .all(|item| matches!(item, RuntimeValue::Quantity(_)))
    {
        let RuntimeValue::Quantity(first) = &items[0] else {
            unreachable!()
        };
        if items.iter().any(|item|matches!(item,RuntimeValue::Quantity(value)if value.dimensions!=first.dimensions)){return Err("Quantity dimensions are incompatible".into());}
    } else if items
        .iter()
        .any(|item| !matches!(item,RuntimeValue::Json(value)if numbers::is_numeric(value)))
    {
        return Err(format!(
            "{name} requires nonempty finite numerical operands"
        ));
    }
    let mut selected = items[0].clone();
    let operator = if name == "min" {
        BinaryOperator::Less
    } else {
        BinaryOperator::Greater
    };
    for item in &items[1..] {
        if compare_values(operator, item, &selected)? {
            selected = item.clone();
        }
    }
    Ok(selected)
}
fn absolute(value: &RuntimeValue) -> Result<RuntimeValue, String> {
    match value {
        RuntimeValue::Quantity(value) => Ok(RuntimeValue::Quantity(value.absolute())),
        RuntimeValue::Json(value) if numbers::is_numeric(value) => {
            Ok(RuntimeValue::Json(numbers::absolute(value)?))
        }
        _ => Err("abs requires a number or quantity".into()),
    }
}

/// Select a zero-based integer index; missing positions are errors, never null data.
fn at(values: &[RuntimeValue]) -> Result<RuntimeValue, String> {
    let items = array("at", &values[0])?;
    let RuntimeValue::Json(index) = &values[1] else {
        return Err("at requires a nonnegative integer index".into());
    };
    let index = index
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or("at requires a nonnegative integer index")?;
    items
        .get(index)
        .cloned()
        .map(RuntimeValue::Json)
        .ok_or_else(|| "at index is outside the array".into())
}

/// Flatten exactly one level, preserving non-array members without coercion.
fn flatten(value: &RuntimeValue, limits: &ExecutionLimits) -> Result<RuntimeValue, String> {
    let items = array("flatten", value)?;
    let mut output = Vec::new();
    for item in items {
        let additional = item.as_array().map_or(1, Vec::len);
        if output.len().saturating_add(additional) > limits.json_nodes {
            return Err("flatten exceeds the JSON node budget".into());
        }
        match item {
            Value::Array(nested) => output.extend(nested.iter().cloned()),
            scalar => output.push(scalar.clone()),
        }
    }
    let output = Value::Array(output);
    canonical::canonical_json_bytes(&output, limits)?;
    Ok(RuntimeValue::Json(output))
}

/// Exact typed JSON equality: object key order is irrelevant, numeric kinds are not.
fn distinct(value: &RuntimeValue, limits: &ExecutionLimits) -> Result<RuntimeValue, String> {
    let RuntimeValue::Json(value) = value else {
        return Err("distinct requires an array".into());
    };
    let items = value.as_array().ok_or("distinct requires an array")?;
    canonical::canonical_json_bytes(value, limits)?;
    let mut seen = BTreeSet::new();
    for item in items {
        if !seen.insert(canonical::canonical_json_bytes(item, limits)?) {
            return Ok(RuntimeValue::Json(Value::Bool(false)));
        }
    }
    Ok(RuntimeValue::Json(Value::Bool(true)))
}

fn contains(values: &[RuntimeValue], limits: &ExecutionLimits) -> Result<RuntimeValue, String> {
    let items = array("contains", &values[0])?;
    let RuntimeValue::Json(needle) = &values[1] else {
        return Err("contains requires a JSON search value".into());
    };
    if let RuntimeValue::Json(value) = &values[0] {
        canonical::canonical_json_bytes(value, limits)?;
    }
    let needle = canonical::canonical_json_bytes(needle, limits)?;
    for item in items {
        if canonical::canonical_json_bytes(item, limits)? == needle {
            return Ok(RuntimeValue::Json(Value::Bool(true)));
        }
    }
    Ok(RuntimeValue::Json(Value::Bool(false)))
}

fn json(value: &RuntimeValue) -> Result<&Value, String> {
    match value {
        RuntimeValue::Json(value) => Ok(value),
        RuntimeValue::Quantity(_) => {
            Err("This function requires JSON values, not implicit quantities".into())
        }
    }
}
fn equal(values: &[RuntimeValue], limits: &ExecutionLimits) -> Result<RuntimeValue, String> {
    let left = canonical::canonical_json_bytes(json(&values[0])?, limits)?;
    let right = canonical::canonical_json_bytes(json(&values[1])?, limits)?;
    Ok(RuntimeValue::Json(Value::Bool(left == right)))
}

fn selected_booleans(
    name: &str,
    value: &RuntimeValue,
    limits: &ExecutionLimits,
) -> Result<RuntimeValue, String> {
    let value = json(value)?;
    canonical::canonical_json_bytes(value, limits)?;
    let values = super::selection::values(value)?;
    if values.iter().any(|value| !value.is_boolean()) {
        return Err("Boolean evaluation requires Boolean measurements".into());
    }
    Ok(RuntimeValue::Json(Value::Bool(if name == "all_values" {
        values.iter().all(|value| value == &Value::Bool(true))
    } else {
        values.iter().any(|value| value == &Value::Bool(true))
    })))
}
