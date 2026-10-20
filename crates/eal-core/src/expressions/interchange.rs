//! Python-model expression JSON at semantic interchange boundaries.
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::{Value, json};

use super::{binary_symbol, comparison_operator, json_to_literal, literal_value};
use crate::{BinaryOperator, ExecutionLimits, Expression, UnaryOperator};

pub fn expression_to_json(expression: &Expression) -> Result<Value, String> {
    expression_to_json_with_limits(expression, &ExecutionLimits::default())
}

pub fn expression_to_json_with_limits(
    expression: &Expression,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    super::expression_shape(expression, limits)?;
    Ok(expression_to_json_unchecked(expression))
}

fn expression_to_json_unchecked(expression: &Expression) -> Value {
    let (kind, value, arguments) = match expression {
        Expression::Literal { value } => ("literal", literal_value(value), Vec::new()),
        Expression::Field { name } => ("field", Value::String(name.clone()), Vec::new()),
        Expression::Call { name, arguments } => (
            "call",
            Value::String(name.clone()),
            arguments.iter().map(expression_to_json_unchecked).collect(),
        ),
        Expression::Unary { operator, operand } => (
            "unary",
            Value::String(
                match operator {
                    UnaryOperator::Not => "not",
                    UnaryOperator::Positive => "+",
                    UnaryOperator::Negative => "-",
                }
                .into(),
            ),
            vec![expression_to_json_unchecked(operand)],
        ),
        Expression::Binary {
            operator,
            left,
            right,
        } => (
            "binary",
            Value::String(binary_symbol(*operator).into()),
            vec![
                expression_to_json_unchecked(left),
                expression_to_json_unchecked(right),
            ],
        ),
    };
    json!({"kind":kind,"value":value,"arguments":arguments})
}

pub fn expression_from_json(value: &Value) -> Result<Expression, String> {
    fn read(value: &Value, depth: usize, nodes: &mut usize) -> Result<Expression, String> {
        *nodes += 1;
        if depth > 128 || *nodes > 10_000 {
            return Err("Expression JSON resource budget exceeded".into());
        }
        let object = value.as_object().ok_or("Expression must be an object")?;
        if object
            .keys()
            .any(|name| !matches!(name.as_str(), "kind" | "value" | "arguments"))
        {
            return Err("Expression has an undeclared field".into());
        }
        let kind = object
            .get("kind")
            .and_then(Value::as_str)
            .ok_or("Expression kind must be a string")?;
        let value = object.get("value").ok_or("Expression value is missing")?;
        let empty = Vec::new();
        let arguments = match object.get("arguments") {
            Some(value) => value
                .as_array()
                .ok_or("Expression arguments must be an array")?,
            None => &empty,
        };
        let string = || {
            value
                .as_str()
                .ok_or("Expression name/operator must be a string")
        };
        match kind {
            "literal" | "field" if !arguments.is_empty() => {
                Err(format!("A {kind} cannot have operands"))
            }
            "literal" => {
                // A literal's JSON children are not expression nodes. Check
                // their independent budget before recursively converting them.
                crate::canonical::canonical_json_bytes(value, &crate::ExecutionLimits::default())?;
                Ok(Expression::Literal {
                    value: json_to_literal(value),
                })
            }
            "field" => Ok(Expression::Field {
                name: string()?.into(),
            }),
            "unary" => {
                if arguments.len() != 1 {
                    return Err("Unary expression requires one operand".into());
                }
                let operator = match string()? {
                    "not" => UnaryOperator::Not,
                    "+" => UnaryOperator::Positive,
                    "-" => UnaryOperator::Negative,
                    _ => return Err("Unknown unary operator".into()),
                };
                Ok(Expression::Unary {
                    operator,
                    operand: Box::new(read(&arguments[0], depth + 1, nodes)?),
                })
            }
            "binary" => {
                if arguments.len() != 2 {
                    return Err("Binary expression requires two operands".into());
                }
                let symbol = string()?;
                let operator = match symbol {
                    "or" => BinaryOperator::Or,
                    "and" => BinaryOperator::And,
                    "+" => BinaryOperator::Add,
                    "-" => BinaryOperator::Subtract,
                    "*" => BinaryOperator::Multiply,
                    "/" => BinaryOperator::Divide,
                    "%" => BinaryOperator::Modulo,
                    _ => comparison_operator(symbol).ok_or("Unknown binary operator")?,
                };
                Ok(Expression::Binary {
                    operator,
                    left: Box::new(read(&arguments[0], depth + 1, nodes)?),
                    right: Box::new(read(&arguments[1], depth + 1, nodes)?),
                })
            }
            "call" => Ok(Expression::Call {
                name: string()?.into(),
                arguments: arguments
                    .iter()
                    .map(|argument| read(argument, depth + 1, nodes))
                    .collect::<Result<_, _>>()?,
            }),
            _ => Err("Unknown expression kind".into()),
        }
    }
    read(value, 1, &mut 0)
}

pub mod optional_serde {
    use super::*;
    pub fn serialize<S: Serializer>(
        expression: &Option<Expression>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match expression {
            Some(expression) => serializer.serialize_some(
                &expression_to_json(expression).map_err(serde::ser::Error::custom)?,
            ),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Expression>, D::Error> {
        Option::<Value>::deserialize(deserializer)?
            .map(|value| expression_from_json(&value).map_err(serde::de::Error::custom))
            .transpose()
    }
}
