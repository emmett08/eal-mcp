//! Bounded interpretation of closed expression nodes and pure functions.
use super::{
    ExpressionResult, Quantity, binary_symbol, children, is_comparison, literal_shape,
    literal_value, numbers, python_repr,
};
use crate::{BinaryOperator, ExecutionLimits, Expression, UnaryOperator};
use serde_json::Value;
use std::cmp::Ordering;
#[derive(Clone, Debug)]
pub(super) enum RuntimeValue {
    Json(Value),
    Quantity(Quantity),
}

impl RuntimeValue {
    fn from_result(result: ExpressionResult) -> Option<Self> {
        match (result.value, result.quantity) {
            (Some(value), _) => Some(Self::Json(value)),
            (_, Some(quantity)) => Some(Self::Quantity(quantity)),
            _ => None,
        }
    }
    fn into_result(self) -> ExpressionResult {
        match self {
            Self::Json(value) => ExpressionResult::json(value),
            Self::Quantity(value) => ExpressionResult::quantity(value),
        }
    }
    fn numeric_quantity(&self) -> Result<Quantity, String> {
        match self {
            Self::Json(value) => Quantity::dimensionless(value),
            Self::Quantity(value) => Ok(value.clone()),
        }
    }
}

pub fn evaluate_expression(expression: &Expression, value: &Value) -> ExpressionResult {
    evaluate_expression_with_limits(expression, value, &ExecutionLimits::default())
}

pub fn evaluate_expression_with_limits(
    expression: &Expression,
    value: &Value,
    limits: &ExecutionLimits,
) -> ExpressionResult {
    let mut visited = 0;
    fn run(
        node: &Expression,
        value: &Value,
        limits: &ExecutionLimits,
        visited: &mut usize,
        depth: usize,
    ) -> Result<ExpressionResult, String> {
        *visited += 1;
        if *visited > limits.expression_nodes {
            return Err("Expression evaluation budget exceeded".into());
        }
        if depth > limits.expression_depth.min(128) {
            return Err("Expression depth budget exceeded".into());
        }
        match node {
            Expression::Literal { value } => {
                literal_shape(value, limits)?;
                return Ok(ExpressionResult::json(literal_value(value)));
            }
            Expression::Field { name } => {
                let mut actual = value;
                for component in name.split('.') {
                    match actual.as_object().and_then(|object| object.get(component)) {
                        Some(child) => actual = child,
                        None => {
                            return Ok(ExpressionResult::unknown(
                                format!(
                                    "Field {} is missing",
                                    python_repr(&Value::String(name.clone()))
                                ),
                                "missing_field",
                            ));
                        }
                    }
                }
                return Ok(ExpressionResult::json(actual.clone()));
            }
            _ => {}
        }
        let mut arguments = Vec::new();
        for child in children(node) {
            arguments.push(run(child, value, limits, visited, depth + 1)?);
        }
        if let Expression::Binary {
            operator: BinaryOperator::And | BinaryOperator::Or,
            ..
        } = node
        {
            if arguments
                .iter()
                .any(|argument| argument.known() && !matches!(argument.value, Some(Value::Bool(_))))
            {
                return Err("Boolean composition requires Boolean operands".into());
            }
            let decisive = matches!(
                node,
                Expression::Binary {
                    operator: BinaryOperator::Or,
                    ..
                }
            );
            if arguments
                .iter()
                .any(|argument| argument.value == Some(Value::Bool(decisive)))
            {
                return Ok(ExpressionResult::json(Value::Bool(decisive)));
            }
            if let Some(unknown) = arguments.iter().find(|argument| !argument.known()) {
                return Ok(unknown.clone());
            }
            return Ok(ExpressionResult::json(Value::Bool(!decisive)));
        }
        if let Some(unknown) = arguments.iter().find(|argument| !argument.known()) {
            return Ok(unknown.clone());
        }
        let values: Vec<_> = arguments
            .into_iter()
            .map(|argument| RuntimeValue::from_result(argument).expect("known expression argument"))
            .collect();
        let result = match node {
            Expression::Unary { operator, .. } => {
                let actual = &values[0];
                match (operator, actual) {
                    (UnaryOperator::Not, RuntimeValue::Json(Value::Bool(actual))) => {
                        RuntimeValue::Json(Value::Bool(!actual))
                    }
                    (UnaryOperator::Not, _) => return Err("Negation requires a Boolean".into()),
                    (UnaryOperator::Negative, RuntimeValue::Quantity(actual)) => {
                        RuntimeValue::Quantity(actual.negate())
                    }
                    (UnaryOperator::Positive, RuntimeValue::Quantity(actual)) => {
                        RuntimeValue::Quantity(actual.clone())
                    }
                    (_, RuntimeValue::Json(actual)) if numbers::is_numeric(actual) => {
                        RuntimeValue::Json(if *operator == UnaryOperator::Negative {
                            numbers::negate(actual)?
                        } else {
                            actual.clone()
                        })
                    }
                    _ => return Err("Unary arithmetic requires a finite number".into()),
                }
            }
            Expression::Binary { operator, .. } => {
                if is_comparison(*operator) {
                    RuntimeValue::Json(Value::Bool(compare_values(
                        *operator, &values[0], &values[1],
                    )?))
                } else {
                    arithmetic_values(binary_symbol(*operator), &values[0], &values[1])?
                }
            }
            Expression::Call { name, .. } => super::functions::call(name, values, limits)?,
            _ => unreachable!(),
        };
        Ok(result.into_result())
    }
    run(expression, value, limits, &mut visited, 1)
        .unwrap_or_else(|reason| ExpressionResult::unknown(reason, "invalid_expression"))
}

fn arithmetic_values(
    symbol: &str,
    left: &RuntimeValue,
    right: &RuntimeValue,
) -> Result<RuntimeValue, String> {
    if matches!(left, RuntimeValue::Quantity(_)) || matches!(right, RuntimeValue::Quantity(_)) {
        let left = left
            .numeric_quantity()
            .map_err(|_| "Quantity arithmetic requires numerical operands")?;
        let right = right
            .numeric_quantity()
            .map_err(|_| "Quantity arithmetic requires numerical operands")?;
        Ok(RuntimeValue::Quantity(left.arithmetic(symbol, &right)?))
    } else {
        let (RuntimeValue::Json(left), RuntimeValue::Json(right)) = (left, right) else {
            unreachable!()
        };
        Ok(RuntimeValue::Json(numbers::arithmetic(
            symbol, left, right,
        )?))
    }
}

pub(super) fn compare_values(
    operator: BinaryOperator,
    left: &RuntimeValue,
    right: &RuntimeValue,
) -> Result<bool, String> {
    let order = match (left, right) {
        (RuntimeValue::Quantity(left), RuntimeValue::Quantity(right))
            if left.dimensions == right.dimensions =>
        {
            left.amount.cmp(&right.amount)
        }
        (RuntimeValue::Quantity(_), _) | (_, RuntimeValue::Quantity(_)) => {
            return Err("Compared quantities have incompatible dimensions".into());
        }
        (RuntimeValue::Json(left), RuntimeValue::Json(right)) => {
            if left.is_number() && right.is_number() {
                numbers::compare(left, right).ok_or("Compared numbers must be finite")?
            } else {
                if std::mem::discriminant(left) != std::mem::discriminant(right) {
                    return Err("Compared values have incompatible types".into());
                }
                match (left, right) {
                    (Value::String(left), Value::String(right)) => left.cmp(right),
                    (Value::Bool(left), Value::Bool(right))
                        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual) =>
                    {
                        left.cmp(right)
                    }
                    (Value::Null, Value::Null)
                        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual) =>
                    {
                        Ordering::Equal
                    }
                    (Value::Array(_), _) | (Value::Object(_), _) => {
                        return Err("Comparison requires scalar values".into());
                    }
                    _ => {
                        return Err(
                            "Ordered comparison requires finite numbers, strings or quantities"
                                .into(),
                        );
                    }
                }
            }
        }
    };
    Ok(match operator {
        BinaryOperator::Equal => order == Ordering::Equal,
        BinaryOperator::NotEqual => order != Ordering::Equal,
        BinaryOperator::Less => order == Ordering::Less,
        BinaryOperator::LessEqual => order != Ordering::Greater,
        BinaryOperator::Greater => order == Ordering::Greater,
        BinaryOperator::GreaterEqual => order != Ordering::Less,
        _ => return Err("Unknown comparison operator".into()),
    })
}
