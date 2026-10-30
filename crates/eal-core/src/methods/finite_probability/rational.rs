//! Parse and bound exact probability scalars and encode their public results.
use crate::methods::MethodError;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;
use serde_json::{Value, json};

const MAX_INPUT_CHARACTERS: usize = 128;
const MAX_RESULT_BITS: u64 = 8192;

pub(super) fn parse(value: &Value) -> Result<BigRational, MethodError> {
    let text = value.as_str().filter(|text| !text.is_empty() && text.len() <= MAX_INPUT_CHARACTERS)
        .ok_or("Exact probabilities and thresholds require rational strings of at most 128 ASCII characters")?;
    let (numerator, denominator) = text.split_once('/').unwrap_or((text, "1"));
    if !integer_text(numerator) || !integer_text(denominator) {
        return Err("Exact values require signed integer or numerator/denominator strings".into());
    }
    let numerator: BigInt = numerator.parse().map_err(|_| "Invalid exact numerator")?;
    let denominator: BigInt = denominator
        .parse()
        .map_err(|_| "Invalid exact denominator")?;
    if denominator <= BigInt::zero() {
        return Err("Exact denominators must be positive".into());
    }
    checked(BigRational::new(numerator, denominator))
}

fn integer_text(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

pub(super) fn checked(value: BigRational) -> Result<BigRational, MethodError> {
    if value.numer().bits() > MAX_RESULT_BITS || value.denom().bits() > MAX_RESULT_BITS {
        return Err(MethodError::Incomplete(
            "Exact finite-probability intermediate exceeds 8192 numerator or denominator bits"
                .into(),
        ));
    }
    Ok(value)
}

pub(super) fn encode(value: &BigRational) -> Value {
    json!({"schema":"eal.exact-number/1","numerator":value.numer().to_string(),
        "denominator":value.denom().to_string(),"unit":"1"})
}
