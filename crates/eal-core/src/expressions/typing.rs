//! Static expression typing and dimensional checks against method output schemas.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use crate::{BinaryOperator, ExecutionLimits, Expression, JsonValue, UnaryOperator};

use super::{
    binary_symbol, children, evaluate_expression_with_limits, expression_budget, is_comparison,
    literal_shape, literal_value, numbers, python_repr, quantity,
};

type Types = Option<BTreeSet<String>>;
type Dimensions = BTreeMap<String, i32>;

fn set(name: &str) -> Types {
    Some(BTreeSet::from([name.to_owned()]))
}
fn contained(actual: &Types, allowed: &[&str]) -> bool {
    actual
        .as_ref()
        .is_none_or(|actual| actual.iter().all(|name| allowed.contains(&name.as_str())))
}
fn exact(actual: &Types, name: &str) -> bool {
    actual
        .as_ref()
        .is_some_and(|actual| actual.len() == 1 && actual.contains(name))
}

/// None denotes unconstrained JSON. An empty object denotes a missing schema path.
fn schema_at(schema: Option<&Value>, path: &[&str]) -> Option<Value> {
    let schema = schema?;
    if schema.get("type").and_then(Value::as_str) == Some("json") {
        return None;
    }
    if let Some(alternatives) = schema.get("anyOf").and_then(Value::as_array) {
        let choices: Option<Vec<_>> = alternatives
            .iter()
            .map(|choice| schema_at(Some(choice), path))
            .collect();
        return choices.map(|choices| json!({"anyOf": choices}));
    }
    if path.is_empty() {
        return Some(schema.clone());
    }
    if schema.get("type").and_then(Value::as_str) != Some("object") {
        return Some(json!({}));
    }
    let child = schema
        .get("properties")
        .and_then(|properties| properties.get(path[0]))
        .or_else(|| schema.get("additionalProperties"));
    if child.is_some_and(Value::is_object) {
        schema_at(child, &path[1..])
    } else {
        Some(json!({}))
    }
}

pub fn expression_errors(expression: &Expression, schema: Option<&Value>) -> Vec<String> {
    expression_errors_with_limits(expression, schema, &ExecutionLimits::default())
}

pub fn expression_errors_with_limits(
    expression: &Expression,
    schema: Option<&Value>,
    limits: &ExecutionLimits,
) -> Vec<String> {
    typed_expression_errors(expression, schema, limits, "boolean")
}

/// Validate a pure derivation without claiming a type for unknown acquired fields.
/// Concrete results must still be JSON objects at execution time.
pub fn derivation_errors_with_limits(
    expression: &Expression,
    schema: Option<&Value>,
    limits: &ExecutionLimits,
) -> Vec<String> {
    typed_expression_errors(expression, schema, limits, "object")
}

fn typed_expression_errors(
    expression: &Expression,
    schema: Option<&Value>,
    limits: &ExecutionLimits,
    expected: &str,
) -> Vec<String> {
    if let Err(message) = expression_budget(expression, limits) {
        return vec![message];
    }
    if schema.is_some_and(|schema| !finite_json(schema, limits)) {
        return vec!["Expression schema requires finite bounded JSON".into()];
    }
    let mut errors = Vec::new();
    let types = types(expression, schema, limits, &mut errors);
    if types.is_some() && !exact(&types, expected) {
        errors.push(
            if expected == "boolean" {
                "A required condition must produce a Boolean"
            } else {
                "A derivation must produce a JSON object"
            }
            .into(),
        );
    }
    if errors.is_empty() {
        dimensions(expression, schema, &mut errors);
    }
    let mut pending = vec![expression];
    let mut fields = false;
    while let Some(node) = pending.pop() {
        fields |= matches!(node, Expression::Field { .. });
        pending.extend(children(node));
    }
    if !fields && errors.is_empty() {
        let checked = evaluate_expression_with_limits(expression, &json!({}), limits);
        if !checked.known() {
            errors.push(checked.reason);
        }
    }
    errors
}

fn finite_json(value: &Value, limits: &ExecutionLimits) -> bool {
    let mut pending = vec![(value, 0usize)];
    let mut count = 0;
    while let Some((value, depth)) = pending.pop() {
        count += 1;
        if depth > limits.json_depth.min(128) || count > limits.json_nodes {
            return false;
        }
        match value {
            Value::Number(_) if !numbers::is_numeric(value) => return false,
            Value::Array(items) => pending.extend(items.iter().map(|child| (child, depth + 1))),
            Value::Object(items) => pending.extend(items.values().map(|child| (child, depth + 1))),
            _ => {}
        }
    }
    true
}

fn types(
    node: &Expression,
    schema: Option<&Value>,
    limits: &ExecutionLimits,
    errors: &mut Vec<String>,
) -> Types {
    match node {
        Expression::Literal { value } => {
            if literal_shape(value, limits).is_err() {
                errors.push("An expression literal requires finite JSON".into());
                return Some(BTreeSet::new());
            }
            let value = literal_value(value);
            if !finite_json(&value, limits) {
                errors.push("An expression literal requires finite JSON".into());
            }
            set(match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            })
        }
        Expression::Field { name } => {
            if name.is_empty() || name.split('.').any(str::is_empty) {
                errors.push("A field requires a nonempty literal property path".into());
            }
            let spec = schema_at(schema, &name.split('.').collect::<Vec<_>>())?;
            let choices = if let Some(choices) = spec.get("anyOf").and_then(Value::as_array) {
                choices.iter().collect::<Vec<_>>()
            } else {
                vec![&spec]
            };
            let result: BTreeSet<_> = choices
                .iter()
                .filter_map(|choice| choice.get("type").and_then(Value::as_str))
                .map(|name| if name == "integer" { "number" } else { name }.to_owned())
                .collect();
            if result.is_empty() {
                errors.push(format!(
                    "No declared output exists at {}",
                    python_repr(&Value::String(name.clone()))
                ));
            }
            Some(result)
        }
        Expression::Unary { operator, operand } => {
            let actual = types(operand, schema, limits, errors);
            let (symbol, allowed) = match operator {
                UnaryOperator::Not => ("not", vec!["boolean"]),
                UnaryOperator::Positive => ("+", vec!["number", "quantity"]),
                UnaryOperator::Negative => ("-", vec!["number", "quantity"]),
            };
            if !contained(&actual, &allowed) {
                errors.push(format!(
                    "Operator {} requires {}",
                    python_repr(&Value::String(symbol.into())),
                    allowed.join(" or ")
                ));
            }
            if *operator == UnaryOperator::Not {
                set("boolean")
            } else {
                actual
            }
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let left = types(left, schema, limits, errors);
            let right = types(right, schema, limits, errors);
            if matches!(operator, BinaryOperator::And | BinaryOperator::Or) {
                if [left, right]
                    .iter()
                    .any(|actual| actual.is_some() && !exact(actual, "boolean"))
                {
                    errors.push(format!(
                        "Operator {} requires Boolean operands",
                        python_repr(&Value::String(binary_symbol(*operator).into()))
                    ));
                }
                return set("boolean");
            }
            if is_comparison(*operator) {
                if let (Some(left), Some(right)) = (&left, &right)
                    && left.is_disjoint(right)
                {
                    errors.push("Comparison operands have incompatible types".into());
                }
                let allowed =
                    if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                        vec!["number", "string", "boolean", "null", "quantity"]
                    } else {
                        vec!["number", "string", "quantity"]
                    };
                if [&left, &right]
                    .iter()
                    .any(|actual| !contained(actual, &allowed))
                {
                    errors.push("Comparison requires compatible scalar operands".into());
                }
                return set("boolean");
            }
            if [&left, &right]
                .iter()
                .any(|actual| !contained(actual, &["number", "quantity"]))
            {
                errors.push("Arithmetic requires finite numbers or quantities".into());
            }
            set(if exact(&left, "quantity") || exact(&right, "quantity") {
                "quantity"
            } else {
                "number"
            })
        }
        Expression::Call { name, arguments } => {
            if name == "field" {
                errors.push("A field lookup must use the canonical field expression node".into());
                return Some(BTreeSet::new());
            }
            let count_ok = match name.as_str() {
                "literal" | "count" | "sum" | "all" | "any" | "abs" | "flatten" | "distinct" => {
                    arguments.len() == 1
                }
                "quantity" | "at" | "contains" | "equal" => arguments.len() == 2,
                "exact_sum" | "exact_mean" | "exact_min" | "exact_max" | "exact_count" => {
                    arguments.len() == 1
                }
                "all_values" | "any_values" => arguments.len() == 1,
                "exact_scale" => arguments.len() == 5,
                "exact_compare" => arguments.len() == 4,
                "object" => arguments.len().is_multiple_of(2),
                "min" | "max" => !arguments.is_empty(),
                _ => {
                    errors.push(format!(
                        "Unknown pure expression function {}",
                        python_repr(&Value::String(name.clone()))
                    ));
                    return Some(BTreeSet::new());
                }
            };
            if !count_ok {
                errors.push(format!(
                    "Function {} has an invalid argument count",
                    python_repr(&Value::String(name.clone()))
                ));
                return Some(BTreeSet::new());
            }
            let actual: Vec<_> = arguments
                .iter()
                .map(|argument| types(argument, schema, limits, errors))
                .collect();
            if name == "all_values" || name == "any_values" {
                if actual[0].is_some() && !exact(&actual[0], "array") {
                    errors.push("Boolean evaluation requires an array of selected inputs".into());
                }
                return set("boolean");
            }
            if name.starts_with("exact_") {
                if [
                    "exact_sum",
                    "exact_mean",
                    "exact_min",
                    "exact_max",
                    "exact_count",
                ]
                .contains(&name.as_str())
                    && actual[0].is_some()
                    && !exact(&actual[0], "array")
                {
                    errors.push("Exact aggregation requires an array of selected inputs".into());
                }
                let text_start = if name == "exact_scale" {
                    Some(1)
                } else if name == "exact_compare" {
                    Some(2)
                } else {
                    None
                };
                if let Some(start) = text_start {
                    for value in &actual[start..] {
                        if value.is_some() && !exact(value, "string") {
                            errors
                                .push("Exact factors, units and operators must be strings".into());
                        }
                    }
                }
                return set(if name == "exact_compare" {
                    "boolean"
                } else {
                    "object"
                });
            }
            if name == "equal" {
                if actual.iter().any(|value| exact(value, "quantity")) {
                    errors.push("equal requires JSON values, not implicit quantities".into());
                }
                return set("boolean");
            }
            if name == "object" {
                let mut keys = BTreeSet::new();
                for (index, pair) in arguments.chunks_exact(2).enumerate() {
                    if actual[index * 2].is_some() && !exact(&actual[index * 2], "string") {
                        errors.push("object keys must be strings".into());
                    }
                    if exact(&actual[index * 2 + 1], "quantity") {
                        errors.push(
                            "object values must be JSON; quantities require explicit unit fields"
                                .into(),
                        );
                    }
                    if let Expression::Literal {
                        value: JsonValue::String(key),
                    } = &pair[0]
                        && !keys.insert(key)
                    {
                        errors.push("object keys must be unique".into());
                    }
                }
                return set("object");
            }
            if name == "quantity" {
                if actual[0].is_some() && !exact(&actual[0], "number") {
                    errors.push("quantity requires a finite number".into());
                }
                if let Expression::Literal {
                    value: JsonValue::String(unit),
                } = &arguments[1]
                {
                    if crate::propositions::unit_spec(unit).is_none() {
                        errors.push(format!(
                            "Unknown unit {}",
                            python_repr(&Value::String(unit.clone()))
                        ));
                    }
                } else {
                    errors.push("quantity requires a literal unit".into());
                }
                return set("quantity");
            }
            if matches!(
                name.as_str(),
                "count" | "sum" | "all" | "any" | "at" | "flatten" | "contains" | "distinct"
            ) && actual[0].is_some()
                && !exact(&actual[0], "array")
            {
                errors.push(format!("{name} requires an array"));
            }
            if name == "at" && actual[1].is_some() && !exact(&actual[1], "number") {
                errors.push("at requires a nonnegative integer index".into());
            }
            if name == "contains" && exact(&actual[1], "quantity") {
                errors.push("contains requires a JSON search value".into());
            }
            match name.as_str() {
                "all" | "any" | "contains" | "distinct" => set("boolean"),
                "flatten" => set("array"),
                "at" => None,
                "count" | "sum" => set("number"),
                "min" | "max" | "abs" => {
                    if actual
                        .iter()
                        .any(|actual| !contained(actual, &["number", "quantity", "array"]))
                    {
                        errors.push(format!("{name} requires numeric operands"));
                    }
                    if actual.iter().any(Option::is_none) {
                        None
                    } else {
                        set(if actual.iter().any(|actual| exact(actual, "quantity")) {
                            "quantity"
                        } else {
                            "number"
                        })
                    }
                }
                _ => actual[0].clone(),
            }
        }
    }
}

fn dimensions(
    node: &Expression,
    schema: Option<&Value>,
    errors: &mut Vec<String>,
) -> Option<Dimensions> {
    if let Expression::Literal { value } = node {
        return numbers::is_numeric(&literal_value(value)).then(BTreeMap::new);
    }
    if let Expression::Field { name } = node {
        let spec = schema_at(schema, &name.split('.').collect::<Vec<_>>())?;
        return matches!(
            spec.get("type").and_then(Value::as_str),
            Some("number" | "integer")
        )
        .then(BTreeMap::new);
    }
    if let Expression::Call { name, arguments } = node
        && name == "quantity"
        && let Some(Expression::Literal {
            value: JsonValue::String(unit),
        }) = arguments.get(1)
    {
        return quantity::dimensions_for_unit(unit);
    }
    let child: Vec<_> = children(node)
        .iter()
        .map(|child| dimensions(child, schema, errors))
        .collect();
    match node {
        Expression::Unary {
            operator: UnaryOperator::Positive | UnaryOperator::Negative,
            ..
        } => child[0].clone(),
        Expression::Call { name, .. } if matches!(name.as_str(), "abs" | "literal") => {
            child[0].clone()
        }
        Expression::Call { name, .. } if matches!(name.as_str(), "count" | "sum") => {
            Some(BTreeMap::new())
        }
        Expression::Call { name, .. } if matches!(name.as_str(), "min" | "max") => {
            let known: Vec<_> = child.iter().flatten().collect();
            if let Some(first) = known.first() {
                if known.iter().any(|dimension| dimension != first) {
                    errors.push("Quantity dimensions are incompatible".into());
                }
                Some((*first).clone())
            } else {
                None
            }
        }
        Expression::Binary { operator, .. }
            if is_comparison(*operator)
                || matches!(
                    operator,
                    BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::Modulo
                ) =>
        {
            let left = child[0].as_ref()?;
            let right = child[1].as_ref()?;
            if is_comparison(*operator)
                || matches!(
                    operator,
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Modulo
                )
            {
                if left != right {
                    errors.push("Quantity dimensions are incompatible".into());
                }
                if is_comparison(*operator) {
                    None
                } else {
                    Some(left.clone())
                }
            } else {
                Some(quantity::combine_dimensions(
                    left,
                    right,
                    *operator == BinaryOperator::Multiply,
                ))
            }
        }
        _ => None,
    }
}
