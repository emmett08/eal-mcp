//! Exact finite JSON-decimal arithmetic for deterministic evidence derivations.
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};
use serde_json::{Value, json};

pub(crate) fn rational(value: &Value) -> Result<BigRational, String> {
    if value.get("schema").and_then(Value::as_str) == Some("eal.exact-number/1") {
        let n = integer(
            value["numerator"]
                .as_str()
                .ok_or("exact numerator requires a string")?,
        )?;
        let d = integer(
            value["denominator"]
                .as_str()
                .ok_or("exact denominator requires a string")?,
        )?;
        if d.is_zero() {
            return Err("exact denominator must be nonzero".into());
        }
        return Ok(BigRational::new(n, d));
    }
    let number = value
        .as_number()
        .ok_or("numeric operation requires finite JSON numbers or exact-number records")?
        .to_string();
    let (mantissa, exponent) = match number.split_once(['e', 'E']) {
        Some((m, e)) => (
            m,
            e.parse::<i32>()
                .map_err(|_| "numeric exponent is outside the supported bound")?,
        ),
        None => (number.as_str(), 0),
    };
    if exponent.unsigned_abs() > 4096 || mantissa.len() > 4096 {
        return Err("numeric operation exceeds exact digit/exponent limits".into());
    }
    let fraction = mantissa.split_once('.').map_or(0, |(_, part)| part.len()) as i32;
    let digits = mantissa.replace('.', "");
    let mut numerator = integer(&digits)?;
    let scale = exponent - fraction;
    let denominator = if scale < 0 {
        BigInt::from(10_u8).pow(scale.unsigned_abs())
    } else {
        numerator *= BigInt::from(10_u8).pow(scale as u32);
        BigInt::one()
    };
    Ok(BigRational::new(numerator, denominator))
}

pub(crate) fn integer(value: &str) -> Result<BigInt, String> {
    if value.len() > 8192 {
        return Err("exact integer exceeds the digit limit".into());
    }
    value.parse().map_err(|_| "exact integer is invalid".into())
}

pub(crate) fn value(value: &BigRational, unit: Option<&str>) -> Value {
    let mut result = json!({"schema":"eal.exact-number/1","numerator":value.numer().to_string(),"denominator":value.denom().to_string()});
    if let Some(unit) = unit {
        result["unit"] = json!(unit);
    }
    result
}

pub(crate) fn flat_values(values: &[Value]) -> Vec<&Value> {
    if let [Value::Array(values)] = values {
        values.iter().collect()
    } else {
        values.iter().collect()
    }
}

pub(crate) fn same_unit(values: &[&Value]) -> Result<Option<String>, String> {
    let mut selected: Option<Option<String>> = None;
    for value in values {
        let unit = value.get("unit").and_then(Value::as_str).map(str::to_owned);
        if selected.as_ref().is_some_and(|previous| previous != &unit) {
            return Err("numeric inputs have incompatible or partly unspecified units".into());
        }
        selected = Some(unit);
    }
    Ok(selected.flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arbitrary_integers_and_decimal_thresholds_remain_exact() {
        let large: Value = serde_json::from_str("90071992547409931234567890").unwrap();
        let next: Value = serde_json::from_str("90071992547409931234567891").unwrap();
        assert!(rational(&large).unwrap() < rational(&next).unwrap());
        let a = rational(&json!(0.1)).unwrap() + rational(&json!(0.2)).unwrap();
        assert_eq!(a, rational(&json!(0.3)).unwrap());
        assert!(rational(&serde_json::from_str::<Value>("1e99999").unwrap()).is_err());
    }
}
