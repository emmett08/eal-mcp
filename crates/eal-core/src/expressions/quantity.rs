//! Exact rational dimensional quantities used by the closed expression language.
use std::collections::BTreeMap;

use num_rational::BigRational;
use num_traits::{Signed, Zero};
use serde::Serialize;
use serde_json::Value;

use super::numbers;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantity {
    pub amount: BigRational,
    pub dimensions: BTreeMap<String, i32>,
}

impl Serialize for Quantity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Wire<'a> {
            numerator: String,
            denominator: String,
            dimensions: &'a BTreeMap<String, i32>,
        }
        Wire {
            numerator: self.amount.numer().to_string(),
            denominator: self.amount.denom().to_string(),
            dimensions: &self.dimensions,
        }
        .serialize(serializer)
    }
}

impl Quantity {
    pub fn new(value: &Value, unit: &str) -> Result<Self, String> {
        let (dimension, scale) = crate::propositions::unit_spec(unit)
            .ok_or("quantity requires a finite number and a registered unit")?;
        let amount = numbers::decimal_rational(value)
            .map_err(|_| "quantity requires a finite number and a registered unit")?
            * numbers::parse_decimal(scale)?;
        let dimensions = match dimension {
            "dimensionless" => BTreeMap::new(),
            "velocity" => BTreeMap::from([("length".into(), 1), ("time".into(), -1)]),
            "volumetric_flow" => BTreeMap::from([("length".into(), 3), ("time".into(), -1)]),
            "pressure" => BTreeMap::from([
                ("length".into(), -1),
                ("mass".into(), 1),
                ("time".into(), -2),
            ]),
            name => BTreeMap::from([(name.to_owned(), 1)]),
        };
        Ok(Self { amount, dimensions })
    }

    pub fn dimensionless(value: &Value) -> Result<Self, String> {
        Ok(Self {
            amount: numbers::decimal_rational(value)?,
            dimensions: BTreeMap::new(),
        })
    }

    pub(super) fn arithmetic(&self, symbol: &str, other: &Self) -> Result<Self, String> {
        if matches!(symbol, "+" | "-" | "%") {
            if self.dimensions != other.dimensions {
                return Err("Quantity dimensions are incompatible".into());
            }
            let amount = match symbol {
                "+" => &self.amount + &other.amount,
                "-" => &self.amount - &other.amount,
                "%" => {
                    if other.amount.is_zero() {
                        return Err("Fraction(1, 0)".into());
                    }
                    let quotient = (&self.amount / &other.amount).floor();
                    &self.amount - quotient * &other.amount
                }
                _ => unreachable!(),
            };
            return Ok(Self {
                amount,
                dimensions: self.dimensions.clone(),
            });
        }
        let mut dimensions = self.dimensions.clone();
        for (name, power) in &other.dimensions {
            *dimensions.entry(name.clone()).or_insert(0) +=
                if symbol == "*" { *power } else { -*power };
        }
        dimensions.retain(|_, power| *power != 0);
        let amount = if symbol == "*" {
            &self.amount * &other.amount
        } else {
            if other.amount.is_zero() {
                return Err("Fraction(1, 0)".into());
            }
            &self.amount / &other.amount
        };
        Ok(Self { amount, dimensions })
    }

    pub(super) fn negate(&self) -> Self {
        Self {
            amount: -&self.amount,
            dimensions: self.dimensions.clone(),
        }
    }
    pub(super) fn absolute(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            dimensions: self.dimensions.clone(),
        }
    }
}

pub(crate) fn dimensions_for_unit(unit: &str) -> Option<BTreeMap<String, i32>> {
    Quantity::new(&serde_json::json!(1), unit)
        .ok()
        .map(|quantity| quantity.dimensions)
}

pub(crate) fn combine_dimensions(
    left: &BTreeMap<String, i32>,
    right: &BTreeMap<String, i32>,
    multiply: bool,
) -> BTreeMap<String, i32> {
    let mut result = left.clone();
    for (name, power) in right {
        *result.entry(name.clone()).or_insert(0) += if multiply { *power } else { -*power };
    }
    result.retain(|_, power| *power != 0);
    result
}
