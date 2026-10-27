//! Bounded decimal-rational derivations with explicit exact-number records and units.
use super::numbers;
use crate::{ExecutionLimits, canonical};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;
use serde_json::{Value, json};

const MAX_DIGITS: usize = 8192;
const MAX_DECIMAL: usize = 4096;

pub(super) fn aggregate(
    name: &str,
    value: &Value,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    canonical::canonical_json_bytes(value, limits)?;
    let values = super::selection::values(value)?;
    if name == "exact_count" {
        return encode(
            &BigRational::from_integer(values.len().into()),
            Some("1"),
            limits,
        );
    }
    if values.is_empty() {
        return Err("Exact numeric aggregation requires a nonempty sample".into());
    }
    let selected_unit = unit(&values[0])?;
    let mut total = rational(&values[0])?;
    for value in &values[1..] {
        if unit(value)? != selected_unit {
            return Err(
                "Exact numeric inputs have incompatible or partly unspecified units".into(),
            );
        }
        let next = rational(value)?;
        total = match name {
            "exact_sum" | "exact_mean" => total + next,
            "exact_min" => total.min(next),
            "exact_max" => total.max(next),
            _ => return Err("Unknown exact aggregation".into()),
        };
        check_size(&total)?;
    }
    if name == "exact_mean" {
        total /= BigRational::from_integer(values.len().into());
    }
    encode(&total, selected_unit, limits)
}

pub(super) fn scale(values: &[Value], limits: &ExecutionLimits) -> Result<Value, String> {
    let numerator = integer(
        values[1]
            .as_str()
            .ok_or("Scale numerator requires an integer string")?,
    )?;
    let denominator = integer(
        values[2]
            .as_str()
            .ok_or("Scale denominator requires an integer string")?,
    )?;
    if denominator.is_zero() {
        return Err("Scale denominator must be nonzero".into());
    }
    let from = nonblank(&values[3], "source unit")?;
    let to = nonblank(&values[4], "target unit")?;
    if unit(&values[0])?.is_some_and(|unit| unit != from) {
        return Err("Scaling requires a matching explicit input unit".into());
    }
    encode(
        &(rational(&values[0])? * BigRational::new(numerator, denominator)),
        Some(to),
        limits,
    )
}

pub(super) fn compare(values: &[Value]) -> Result<Value, String> {
    let operator = values[2]
        .as_str()
        .ok_or("Exact comparison requires an operator string")?;
    let expected_unit = nonblank(&values[3], "comparison unit")?;
    for value in &values[..2] {
        if unit(value)?.is_some_and(|unit| unit != expected_unit) {
            return Err("Exact comparison units differ; supply an explicit conversion".into());
        }
    }
    let left = rational(&values[0])?;
    let right = rational(&values[1])?;
    Ok(json!(match operator {
        "<" => left < right,
        "<=" => left <= right,
        "==" => left == right,
        "!=" => left != right,
        ">=" => left >= right,
        ">" => left > right,
        _ => return Err("Unknown exact comparison operator".into()),
    }))
}

fn rational(value: &Value) -> Result<BigRational, String> {
    if value.get("schema").and_then(Value::as_str) == Some("eal.exact-number/1") {
        let fields = value.as_object().expect("schema field requires object");
        if fields
            .keys()
            .any(|key| !["schema", "numerator", "denominator", "unit"].contains(&key.as_str()))
        {
            return Err("Exact number contains an undeclared field".into());
        }
        let numerator = integer(
            value["numerator"]
                .as_str()
                .ok_or("Exact numerator requires an integer string")?,
        )?;
        let denominator = integer(
            value["denominator"]
                .as_str()
                .ok_or("Exact denominator requires an integer string")?,
        )?;
        if denominator.is_zero() {
            return Err("Exact denominator must be nonzero".into());
        }
        unit(value)?;
        return Ok(BigRational::new(numerator, denominator));
    }
    let number = value
        .as_number()
        .ok_or("Exact arithmetic requires JSON numbers or exact-number records")?
        .to_string();
    let (mantissa, exponent) = match number.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (
            mantissa,
            exponent
                .parse::<i32>()
                .map_err(|_| "Exact decimal exponent is invalid")?,
        ),
        None => (number.as_str(), 0),
    };
    if mantissa.len() > MAX_DECIMAL || exponent.unsigned_abs() > MAX_DECIMAL as u32 {
        return Err("Exact decimal exceeds the digit or exponent limit".into());
    }
    numbers::parse_decimal(&number)
}

fn integer(text: &str) -> Result<BigInt, String> {
    if text.len() > MAX_DIGITS {
        return Err("Exact integer exceeds the digit limit".into());
    }
    text.parse().map_err(|_| "Exact integer is invalid".into())
}
fn nonblank<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|value| !value.trim().is_empty() && value.len() <= MAX_DECIMAL)
        .ok_or_else(|| format!("Exact {name} requires a nonblank bounded string"))
}
fn unit(value: &Value) -> Result<Option<&str>, String> {
    value
        .get("unit")
        .map(|unit| nonblank(unit, "number unit"))
        .transpose()
}
fn check_size(value: &BigRational) -> Result<(), String> {
    if value.numer().to_string().len() > MAX_DIGITS || value.denom().to_string().len() > MAX_DIGITS
    {
        return Err("Exact result exceeds the digit limit".into());
    }
    Ok(())
}
fn encode(
    value: &BigRational,
    unit: Option<&str>,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    check_size(value)?;
    let mut record = json!({"schema":"eal.exact-number/1","numerator":value.numer().to_string(),"denominator":value.denom().to_string()});
    if let Some(unit) = unit {
        record["unit"] = json!(unit);
    }
    canonical::canonical_json_bytes(&record, limits)?;
    Ok(record)
}
