//! Stable Python-compatible expression and diagnostic text.
use super::{binary_symbol, literal_value, property_path};
use crate::{ExecutionLimits, Expression, UnaryOperator};
use serde_json::Value;
pub(crate) fn python_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(number) => {
            let text = number.to_string();
            if text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
                crate::canonical::python_float_repr(text.parse::<f64>().unwrap_or(f64::NAN))
            } else {
                text
            }
        }
        Value::String(value) => {
            let quote = if value.contains('\'') && !value.contains('"') {
                '"'
            } else {
                '\''
            };
            let mut out = String::new();
            out.push(quote);
            for character in value.chars() {
                match character {
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    value if value == quote => {
                        out.push('\\');
                        out.push(value);
                    }
                    value if !python_printable(value) => {
                        use std::fmt::Write;
                        if (value as u32) <= 0xff {
                            write!(out, "\\x{:02x}", value as u32).unwrap();
                        } else if (value as u32) <= 0xffff {
                            write!(out, "\\u{:04x}", value as u32).unwrap();
                        } else {
                            write!(out, "\\U{:08x}", value as u32).unwrap();
                        }
                    }
                    value => out.push(value),
                }
            }
            out.push(quote);
            out
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(python_repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(items) => format!(
            "{{{}}}",
            items
                .iter()
                .map(|(name, value)| format!(
                    "{}: {}",
                    python_repr(&Value::String(name.clone())),
                    python_repr(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn json_encode(value: &Value) -> String {
    match value {
        Value::Number(number) => {
            let text = number.to_string();
            if text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
                crate::canonical::python_float_repr(text.parse::<f64>().unwrap_or(f64::NAN))
            } else {
                text
            }
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(json_encode).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(items) => format!(
            "{{{}}}",
            items
                .iter()
                .map(|(name, value)| format!(
                    "{}: {}",
                    serde_json::to_string(name).unwrap(),
                    json_encode(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => serde_json::to_string(value).expect("JSON scalar serialisation"),
    }
}

pub fn format_expression(expression: &Expression) -> Result<String, String> {
    format_expression_with_limits(expression, &ExecutionLimits::default())
}

pub fn format_expression_with_limits(
    expression: &Expression,
    limits: &ExecutionLimits,
) -> Result<String, String> {
    super::expression_shape(expression, limits)?;
    let mut pending = vec![expression];
    while let Some(node) = pending.pop() {
        if let Expression::Literal { value } = node {
            crate::canonical::canonical_json_bytes(&literal_value(value), limits)?;
        }
        pending.extend(super::children(node));
    }
    Ok(format_expression_unchecked(expression))
}

fn format_expression_unchecked(expression: &Expression) -> String {
    const RESERVED: &[&str] = &[
        "language",
        "environment",
        "tool",
        "version",
        "evidence",
        "kind",
        "max_age",
        "input",
        "assumption",
        "statement",
        "validate",
        "valid_from",
        "valid_until",
        "reasoning",
        "rationale",
        "backing",
        "claim",
        "argument",
        "conclusion",
        "assumptions",
        "premises",
        "objection",
        "target",
        "require",
        "true",
        "false",
        "null",
        "and",
        "or",
        "not",
    ];
    match expression {
        Expression::Literal { value } => json_encode(&literal_value(value)),
        Expression::Field { name }
            if property_path(name) && name.split('.').all(|part| !RESERVED.contains(&part)) =>
        {
            name.clone()
        }
        Expression::Field { name } => format!("field({})", serde_json::to_string(name).unwrap()),
        Expression::Call { name, arguments } => format!(
            "{name}({})",
            arguments
                .iter()
                .map(format_expression_unchecked)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Unary { operator, operand } => format!(
            "({}{})",
            match operator {
                UnaryOperator::Not => "not ",
                UnaryOperator::Positive => "+",
                UnaryOperator::Negative => "-",
            },
            format_expression_unchecked(operand)
        ),
        Expression::Binary {
            operator,
            left,
            right,
        } => format!(
            "({} {} {})",
            format_expression_unchecked(left),
            binary_symbol(*operator),
            format_expression_unchecked(right)
        ),
    }
}

fn python_printable(value: char) -> bool {
    use unicode_general_category::{GeneralCategory as G, get_general_category};
    value == ' '
        || !matches!(
            get_general_category(value),
            G::Control
                | G::Format
                | G::PrivateUse
                | G::Surrogate
                | G::Unassigned
                | G::LineSeparator
                | G::ParagraphSeparator
                | G::SpaceSeparator
        )
}
