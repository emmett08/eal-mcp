//! Exact rational accumulation around explicitly binary64 statistical operations.
use super::validation::{list, number, object, result};
use crate::expressions::numbers;
use num_bigint::{BigInt, BigUint};
use num_integer::Roots;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use serde_json::{Value, json};

pub fn fsum(values: impl IntoIterator<Item = f64>) -> Result<f64, String> {
    let mut sum = BigRational::zero();
    for value in values {
        let Some(r) = BigRational::from_float(value) else {
            return Err("Finite sum requires finite values".into());
        };
        sum += r;
    }
    sum.to_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| "Computed sum exceeds the finite numeric range".into())
}

fn numeric(value: BigRational) -> Result<Value, String> {
    if value.is_integer() {
        return serde_json::from_str(&value.to_integer().to_string()).map_err(|e| e.to_string());
    }
    let rounded = value
        .to_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| "Computed mean or contrast exceeds the finite numeric range".to_owned())?;
    if !value.is_zero() && rounded == 0.0 {
        return Err("Computed mean or contrast underflows the representable number range".into());
    }
    Ok(json!(rounded))
}

/// Round-to-odd scaled integer square root avoids forming an underflowing
/// binary64 variance. Adapted from CPython's statistics._float_sqrt_of_frac.
pub(super) fn sqrt_fraction(value: &BigRational) -> Result<f64, String> {
    if value.is_zero() {
        return Ok(0.0);
    }
    let n = value
        .numer()
        .to_biguint()
        .ok_or_else(|| "Variance cannot be negative".to_owned())?;
    let m = value
        .denom()
        .to_biguint()
        .ok_or_else(|| "Variance denominator must be positive".to_owned())?;
    let q = (n.bits() as i64 - m.bits() as i64 - 109).div_euclid(2);
    let root_odd = |n: BigUint, m: BigUint| {
        let quotient = &n / &m;
        let mut a = Roots::sqrt(&quotient);
        if &a * &a * &m != n {
            a |= BigUint::one();
        }
        a
    };
    let ratio = if q >= 0 {
        BigRational::from_integer(BigInt::from(
            root_odd(n, m << (2 * q) as usize) << q as usize,
        ))
    } else {
        BigRational::new(
            BigInt::from(root_odd(n << (-2 * q) as usize, m)),
            BigInt::one() << (-q) as usize,
        )
    };
    ratio
        .to_f64()
        .filter(|v| v.is_finite())
        .ok_or_else(|| "Computed standard deviation exceeds the finite numeric range".into())
}

pub fn causal(value: &Value) -> Result<Value, String> {
    let o = object(value, &["assignment", "treatment", "control"], "experiment")?;
    if o["assignment"] != "randomised" {
        return Err("This causal calculation requires declared randomised assignment".into());
    }
    let mut groups = Vec::new();
    for label in ["treatment", "control"] {
        let mut group = Vec::new();
        for value in list(&o[label], label, 2, 10_000)? {
            number(value, label)?;
            group.push(numbers::as_rational(value)?);
        }
        groups.push(group);
    }
    let stats = |group: &[BigRational]| -> Result<(BigRational, f64), String> {
        let mean = group.iter().cloned().sum::<BigRational>() / BigInt::from(group.len());
        let variance = group
            .iter()
            .map(|x| {
                let d = x - &mean;
                &d * &d
            })
            .sum::<BigRational>()
            / BigInt::from(group.len() - 1);
        Ok((mean, sqrt_fraction(&variance)?))
    };
    let (mt, st) = stats(&groups[0])?;
    let (mc, sc) = stats(&groups[1])?;
    let se = (st / (groups[0].len() as f64).sqrt()).hypot(sc / (groups[1].len() as f64).sqrt());
    if se == 0.0 && groups.iter().any(|g| g.iter().any(|item| item != &g[0])) {
        return Err("Computed standard error underflows the representable number range".into());
    }
    Ok(result(
        true,
        "Randomised two-group mean contrast computed; assignment and causal assumptions require evidence",
        json!({"estimate":numeric(&mt-&mc)?,"standard_error":se,"treatment_mean":numeric(mt)?,"control_mean":numeric(mc)?,"treatment_size":groups[0].len(),"control_size":groups[1].len(),"sample_size":groups[0].len()+groups[1].len(),"assumptions_verified":false}),
    ))
}
