//! Closed pure function library, separate from expression-tree interpretation.
use super::{
    Quantity, numbers, python_repr,
    runtime::{RuntimeValue, compare_values},
};
use crate::BinaryOperator;
use serde_json::Value;

pub(super) fn call(name: &str, values: Vec<RuntimeValue>) -> Result<RuntimeValue, String> {
    let valid_count = match name {
        "literal" | "count" | "sum" | "abs" | "all" | "any" => values.len() == 1,
        "quantity" => values.len() == 2,
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
        "count" => count(&values[0]),
        "sum" => sum(&values[0]),
        "all" | "any" => boolean_array(name, &values[0]),
        "min" | "max" => extremum(name, values),
        "abs" => absolute(&values[0]),
        _ => unreachable!("closed function catalogue checked above"),
    }
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
