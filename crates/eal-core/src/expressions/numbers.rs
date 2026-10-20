//! Python-compatible exact integer arithmetic and binary64 numeric boundaries.
use std::cmp::Ordering;
use std::str::FromStr;

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use serde_json::{Number, Value};

#[derive(Clone, Debug)]
pub(crate) enum Numeric {
    Integer(BigInt),
    Float(f64),
}

impl Numeric {
    pub(crate) fn read(value: &Value) -> Result<Self, String> {
        let number = value
            .as_number()
            .ok_or("Arithmetic requires finite numbers")?;
        let text = number.to_string();
        if text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
            let value = text
                .parse::<f64>()
                .map_err(|_| "Invalid floating-point number")?;
            if !value.is_finite() {
                return Err("Arithmetic requires finite numbers".into());
            }
            Ok(Self::Float(value))
        } else {
            Ok(Self::Integer(
                BigInt::from_str(&text).map_err(|_| "Invalid integer")?,
            ))
        }
    }

    pub(crate) fn float(&self) -> Result<f64, String> {
        let value = match self {
            Self::Integer(integer) => integer
                .to_f64()
                .ok_or("int too large to convert to float")?,
            Self::Float(value) => *value,
        };
        value
            .is_finite()
            .then_some(value)
            .ok_or_else(|| "int too large to convert to float".into())
    }

    pub(crate) fn exact(&self) -> BigRational {
        match self {
            Self::Integer(value) => BigRational::from_integer(value.clone()),
            Self::Float(value) => {
                BigRational::from_float(*value).expect("finite floating-point input")
            }
        }
    }
}

pub fn is_numeric(value: &Value) -> bool {
    Numeric::read(value).is_ok()
}

/// Exact rational value, using the binary64 value for a JSON floating literal.
pub fn as_rational(value: &Value) -> Result<BigRational, String> {
    Ok(Numeric::read(value)?.exact())
}

/// Compare arbitrary integers and binary64 numbers without rounding the integer.
pub fn compare(left: &Value, right: &Value) -> Option<Ordering> {
    Some(
        Numeric::read(left)
            .ok()?
            .exact()
            .cmp(&Numeric::read(right).ok()?.exact()),
    )
}

pub fn integer(value: BigInt) -> Result<Value, String> {
    Ok(Value::Number(
        Number::from_str(&value.to_string()).map_err(|error| error.to_string())?,
    ))
}

pub fn float(value: f64) -> Result<Value, String> {
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| "Arithmetic result is not finite".into())
}

pub fn negate(value: &Value) -> Result<Value, String> {
    match Numeric::read(value)? {
        Numeric::Integer(value) => integer(-value),
        Numeric::Float(value) => float(-value),
    }
}

pub fn absolute(value: &Value) -> Result<Value, String> {
    match Numeric::read(value)? {
        Numeric::Integer(value) => integer(value.abs()),
        Numeric::Float(value) => float(value.abs()),
    }
}

/// Match the pinned CPython 3.12 sum specialization, including its integer
/// fallback boundaries and compensation only for exact floating elements.
/// Its C-long fast path is fixed at 64 bits on every Rust target, matching the
/// CPython 3.12.14 oracle profile. Adapted from CPython Python/bltinmodule.c;
/// the PSF licence is retained at ../methods/CPython-LICENSE.txt.
pub(crate) fn sum(values: &[Value]) -> Result<Value, String> {
    let mut items = values.iter();
    let mut small_total = 0_i64;
    let mut total = loop {
        let Some(item) = items.next() else {
            return integer(small_total.into());
        };
        let value = Numeric::read(item)?;
        if let Numeric::Integer(value) = &value
            && let Some(value) = value.to_i64()
            && let Some(next) = small_total.checked_add(value)
        {
            small_total = next;
            continue;
        }
        break sum_add(Numeric::Integer(small_total.into()), value)?;
    };
    if let Numeric::Float(mut high) = total {
        let mut low = 0.0;
        loop {
            let Some(item) = items.next() else {
                return sum_float(compensated_total(high, low));
            };
            let value = Numeric::read(item)?;
            match &value {
                Numeric::Float(value) => {
                    let next = high + value;
                    low += if high.abs() >= value.abs() {
                        (high - next) + value
                    } else {
                        (value - next) + high
                    };
                    high = next;
                }
                Numeric::Integer(value) if value.to_i64().is_some() => {
                    // CPython 3.12's integer fast path does not compensate.
                    high += value.to_i64().expect("checked above") as f64;
                }
                _ => {
                    total = sum_add(Numeric::Float(compensated_total(high, low)), value)?;
                    break;
                }
            }
        }
    }
    // An integer outside C long, or an overflowing integer prefix, leaves the
    // specialized loop permanently. Later floats use ordinary binary64 add.
    for item in items {
        total = sum_add(total, Numeric::read(item)?)?;
    }
    match total {
        Numeric::Integer(value) => integer(value),
        Numeric::Float(value) => sum_float(value),
    }
}

fn compensated_total(high: f64, low: f64) -> f64 {
    if low != 0.0 && low.is_finite() {
        high + low
    } else {
        high
    }
}

fn sum_float(value: f64) -> Result<Value, String> {
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| "Function result is not finite".into())
}

fn sum_add(left: Numeric, right: Numeric) -> Result<Numeric, String> {
    Ok(match (left, right) {
        (Numeric::Integer(left), Numeric::Integer(right)) => Numeric::Integer(left + right),
        (Numeric::Integer(left), Numeric::Float(right)) => {
            Numeric::Float(Numeric::Integer(left).float()? + right)
        }
        (Numeric::Float(left), Numeric::Integer(right)) => {
            Numeric::Float(left + Numeric::Integer(right).float()?)
        }
        (Numeric::Float(left), Numeric::Float(right)) => Numeric::Float(left + right),
    })
}

/// Arithmetic retains Python integer precision and Python's floor remainder.
pub fn arithmetic(symbol: &str, left: &Value, right: &Value) -> Result<Value, String> {
    let left = Numeric::read(left)?;
    let right = Numeric::read(right)?;
    if let (Numeric::Integer(a), Numeric::Integer(b)) = (&left, &right) {
        return match symbol {
            "+" => integer(a + b),
            "-" => integer(a - b),
            "*" => integer(a * b),
            "%" => {
                if b.is_zero() {
                    return Err("integer modulo by zero".into());
                }
                integer(a.mod_floor(b))
            }
            "/" => {
                if b.is_zero() {
                    return Err("division by zero".into());
                }
                if a.is_zero() {
                    return float(if b.is_negative() { -0.0 } else { 0.0 });
                }
                let result = BigRational::new(a.clone(), b.clone())
                    .to_f64()
                    .ok_or("integer division result too large for a float")?;
                if !result.is_finite() {
                    return Err("integer division result too large for a float".into());
                }
                float(result)
            }
            _ => Err("Unknown arithmetic operator".into()),
        };
    }
    let a = left.float()?;
    let b = right.float()?;
    let result = match symbol {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => {
            if b == 0.0 {
                return Err("float division by zero".into());
            }
            a / b
        }
        "%" => {
            if b == 0.0 {
                return Err("float modulo".into());
            }
            let mut remainder = a % b;
            if remainder != 0.0 {
                if remainder.is_sign_negative() != b.is_sign_negative() {
                    remainder += b;
                }
            } else {
                remainder = 0.0_f64.copysign(b);
            }
            remainder
        }
        _ => return Err("Unknown arithmetic operator".into()),
    };
    float(result)
}

/// Decimal rational for quantity construction: Fraction(str(value)) in Python.
pub fn decimal_rational(value: &Value) -> Result<BigRational, String> {
    let text = match Numeric::read(value)? {
        Numeric::Integer(value) => value.to_string(),
        Numeric::Float(value) => crate::canonical::python_float_repr(value),
    };
    parse_decimal(&text)
}

pub fn parse_decimal(text: &str) -> Result<BigRational, String> {
    if let Some((numerator, denominator)) = text.split_once('/') {
        let numerator = BigInt::from_str(numerator).map_err(|_| "Invalid rational")?;
        let denominator = BigInt::from_str(denominator).map_err(|_| "Invalid rational")?;
        if denominator.is_zero() {
            return Err("division by zero".into());
        }
        return Ok(BigRational::new(numerator, denominator));
    }
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (
            mantissa,
            exponent
                .parse::<i32>()
                .map_err(|_| "Invalid decimal exponent")?,
        ),
        None => (text, 0),
    };
    let decimals = mantissa
        .split_once('.')
        .map(|(_, tail)| tail.len())
        .unwrap_or(0);
    let digits = mantissa.replace('.', "");
    let integer = BigInt::from_str(&digits).map_err(|_| "Invalid decimal number")?;
    let power = exponent
        .checked_sub(i32::try_from(decimals).map_err(|_| "Decimal is too long")?)
        .ok_or("Decimal exponent exceeds the resource limit")?;
    if power.unsigned_abs() > 1_000_000 {
        return Err("Decimal exponent exceeds the resource limit".into());
    }
    let factor = BigInt::from(10_u8).pow(power.unsigned_abs());
    Ok(if power >= 0 {
        BigRational::from_integer(integer * factor)
    } else {
        BigRational::new(integer, factor)
    })
}
