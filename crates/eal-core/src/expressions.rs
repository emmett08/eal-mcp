//! Closed, pure expression interpretation with explicit unknowns and strict scalars.
mod exact;
mod formatting;
mod functions;
mod interchange;
pub mod numbers;
mod quantity;
mod runtime;
mod selection;
mod typing;

use std::str::FromStr;

use serde::Serialize;
use serde_json::{Number, Value};

use crate::semantic::Predicate;
use crate::{BinaryOperator, ExecutionLimits, Expression, JsonValue, NumberLiteral};
pub(crate) use formatting::python_repr;
pub use formatting::{diagnostic_repr, format_expression, format_expression_with_limits};
pub use interchange::{
    expression_from_json, expression_to_json, expression_to_json_with_limits, optional_serde,
};
pub use quantity::Quantity;
pub use runtime::{evaluate_expression, evaluate_expression_with_limits};
pub use typing::{derivation_errors_with_limits, expression_errors, expression_errors_with_limits};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExpressionResult {
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Quantity>,
    pub reason: String,
    pub issue: Option<String>,
}

impl ExpressionResult {
    pub fn known(&self) -> bool {
        self.value.is_some() || self.quantity.is_some()
    }
    fn json(value: Value) -> Self {
        Self {
            value: Some(value),
            quantity: None,
            reason: String::new(),
            issue: None,
        }
    }
    fn quantity(value: Quantity) -> Self {
        Self {
            value: None,
            quantity: Some(value),
            reason: String::new(),
            issue: None,
        }
    }
    fn unknown(reason: String, issue: &str) -> Self {
        Self {
            value: None,
            quantity: None,
            reason,
            issue: Some(issue.into()),
        }
    }
}

/// Convert exact authored literals to JSON, applying binary64 rounding to decimals.
pub(crate) fn literal_value(value: &JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Boolean(value) => Value::Bool(*value),
        JsonValue::String(value) => Value::String(value.clone()),
        JsonValue::Number(value) => {
            if value.0 == "-0" {
                return Value::from(0);
            }
            if value
                .0
                .bytes()
                .any(|byte| matches!(byte, b'.' | b'e' | b'E'))
                && let Ok(number) = value.0.parse::<f64>()
                && let Some(number) = Number::from_f64(number)
            {
                return Value::Number(number);
            }
            // Nonfinite decimal lexemes remain representable for the static
            // finite-JSON check, which rejects them before semantic execution.
            Value::Number(Number::from_str(&value.0).expect("recognised EAL number literal"))
        }
        JsonValue::Array(values) => Value::Array(values.iter().map(literal_value).collect()),
        JsonValue::Object(values) => Value::Object(
            values
                .iter()
                .map(|(name, value)| (name.clone(), literal_value(value)))
                .collect(),
        ),
    }
}

/// Convert a typed literal after checking its syntax and independent JSON budgets.
pub fn try_literal_value(value: &JsonValue, limits: &ExecutionLimits) -> Result<Value, String> {
    literal_shape(value, limits)?;
    Ok(literal_value(value))
}

/// Convert finite JSON to a typed literal with bounded depth and node count.
pub fn try_json_to_literal(value: &Value, limits: &ExecutionLimits) -> Result<JsonValue, String> {
    crate::canonical::canonical_json_bytes(value, limits)?;
    Ok(json_to_literal(value))
}

/// Check external typed literals before a recursive conversion or interpretation.
pub(crate) fn literal_shape(value: &JsonValue, limits: &ExecutionLimits) -> Result<(), String> {
    let mut pending = vec![(value, 0usize)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if nodes > limits.json_nodes || depth > limits.json_depth.min(128) {
            return Err("Expression literal JSON resource limit exceeded".into());
        }
        match value {
            JsonValue::Number(value) => {
                Number::from_str(&value.0).map_err(|_| "Invalid expression number literal")?;
            }
            JsonValue::Array(values) => {
                pending.extend(values.iter().map(|value| (value, depth + 1)))
            }
            JsonValue::Object(values) => {
                let mut keys = std::collections::BTreeSet::new();
                for (name, value) in values {
                    if !keys.insert(name) {
                        return Err("Duplicate expression JSON object key".into());
                    }
                    pending.push((value, depth + 1));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn json_to_literal(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(value) => JsonValue::Boolean(*value),
        Value::Number(value) => JsonValue::Number(NumberLiteral(value.to_string())),
        Value::String(value) => JsonValue::String(value.clone()),
        Value::Array(values) => JsonValue::Array(values.iter().map(json_to_literal).collect()),
        Value::Object(values) => JsonValue::Object(
            values
                .iter()
                .map(|(name, value)| (name.clone(), json_to_literal(value)))
                .collect(),
        ),
    }
}

pub(crate) fn property_path(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|part| {
            let mut bytes = part.bytes();
            bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

pub(crate) fn make_predicate(expression: &Expression) -> Predicate {
    let mut expression = expression.clone();
    if let Expression::Binary {
        operator,
        left,
        right,
    } = &mut expression
        && is_comparison(*operator)
    {
        if let Expression::Literal {
            value: JsonValue::String(name),
        } = left.as_ref()
        {
            *left = Box::new(Expression::Field { name: name.clone() });
        }
        if let (Expression::Field { name }, Expression::Literal { value }) =
            (left.as_ref(), right.as_ref())
            && property_path(name)
        {
            return Predicate {
                path: name.clone(),
                operator: binary_symbol(*operator).into(),
                expected: literal_value(value),
                expression: None,
            };
        }
    }
    Predicate {
        path: String::new(),
        operator: "expression".into(),
        expected: Value::Null,
        expression: Some(expression),
    }
}

pub(crate) fn predicate_expression(predicate: &Predicate) -> Expression {
    predicate
        .expression
        .clone()
        .unwrap_or_else(|| Expression::Binary {
            operator: comparison_operator(&predicate.operator).unwrap_or(BinaryOperator::Equal),
            left: Box::new(Expression::Field {
                name: predicate.path.clone(),
            }),
            right: Box::new(Expression::Literal {
                value: json_to_literal(&predicate.expected),
            }),
        })
}

pub(crate) fn predicate_paths(predicate: &Predicate) -> Vec<String> {
    let expression = predicate_expression(predicate);
    let mut paths = Vec::new();
    let mut pending = vec![&expression];
    while let Some(expression) = pending.pop() {
        if let Expression::Field { name } = expression
            && !paths.contains(name)
        {
            paths.push(name.clone());
        }
        pending.extend(children(expression).into_iter().rev());
    }
    paths
}

pub(crate) fn children(expression: &Expression) -> Vec<&Expression> {
    match expression {
        Expression::Literal { .. } | Expression::Field { .. } => vec![],
        Expression::Unary { operand, .. } => vec![operand],
        Expression::Binary { left, right, .. } => vec![left, right],
        Expression::Call { arguments, .. } => arguments.iter().collect(),
    }
}

pub(crate) fn expression_budget(
    expression: &Expression,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    let mut pending = vec![(expression, 1usize)];
    let mut visited = 0;
    while let Some((node, depth)) = pending.pop() {
        visited += 1;
        if visited > limits.expression_nodes {
            return Err("Expression node budget exceeded".into());
        }
        if depth > limits.expression_depth.min(128) {
            return Err("Expression depth budget exceeded".into());
        }
        pending.extend(
            children(node)
                .into_iter()
                .rev()
                .map(|child| (child, depth + 1)),
        );
    }
    Ok(())
}

pub(crate) fn is_comparison(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
    )
}
pub(crate) fn expression_shape(
    expression: &Expression,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    expression_budget(expression, limits)?;
    let mut pending = vec![expression];
    while let Some(node) = pending.pop() {
        if let Expression::Literal { value } = node {
            literal_shape(value, limits)?;
        }
        pending.extend(children(node));
    }
    Ok(())
}

pub fn comparison_operator(symbol: &str) -> Option<BinaryOperator> {
    Some(match symbol {
        "==" => BinaryOperator::Equal,
        "!=" => BinaryOperator::NotEqual,
        "<" => BinaryOperator::Less,
        "<=" => BinaryOperator::LessEqual,
        ">" => BinaryOperator::Greater,
        ">=" => BinaryOperator::GreaterEqual,
        _ => return None,
    })
}
pub(crate) fn binary_symbol(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Or => "or",
        BinaryOperator::And => "and",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Less => "<",
        BinaryOperator::Greater => ">",
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Modulo => "%",
    }
}
