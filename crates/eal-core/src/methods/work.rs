//! Deterministic admission charges for the bounded native algorithms.
//!
//! A work unit represents a scalar operation, collection visit or scalar lookup.
//! Elementary numerical/library operations count as one unit: this is a bound
//! on the declared algorithm's structural work, not CPU time or bit complexity.
//! Schema/JSON validation runs first under its independent size/depth budgets.
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn required(mode: &str, payload: &Value) -> usize {
    let mut pending = vec![payload];
    let mut nodes = 0usize;
    let mut atoms = BTreeSet::new();
    while let Some(value) = pending.pop() {
        nodes = nodes.saturating_add(1);
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => pending.extend(values.values()),
            Value::String(value) if mode == "deductive" => {
                atoms.insert(value);
            }
            _ => {}
        }
    }
    let linear = nodes.saturating_add(1).saturating_mul(64);
    match mode {
        // Full truth-tree visits and assignment construction for every valuation.
        // The algorithm admits at most twelve atoms; malformed larger formulae
        // retain their normal validation error rather than overflowing a shift.
        "deductive" => linear.saturating_add(
            (1usize << atoms.len().min(12))
                .saturating_mul(nodes.saturating_add(atoms.len()))
                .saturating_mul(atoms.len().saturating_add(1)),
        ),
        // AS241 evaluates a fixed number of polynomial terms and scalar operations.
        "inductive" => linear.saturating_add(256),
        // Map insertion/lookup, normalisation and posterior scans are bounded by
        // n squared even when every comparison visits a different tree branch.
        "abductive" => {
            let n = payload["candidates"].as_array().map_or(0, Vec::len);
            linear.saturating_add(n.saturating_mul(n).saturating_mul(16))
        }
        "counterfactual" => {
            let variables = payload["variables"].as_object();
            let n = variables.map_or(0, serde_json::Map::len);
            let edges = variables.map_or(0, |variables| {
                variables.values().fold(0usize, |sum, equation| {
                    sum.saturating_add(
                        equation["coefficients"]
                            .as_object()
                            .map_or(0, serde_json::Map::len),
                    )
                })
            });
            // At most n rounds; each dependency check scans at most n preceding
            // variables. Two affine solves and result construction fit the linear term.
            linear.saturating_add(
                n.saturating_add(1)
                    .saturating_mul(n.saturating_add(1))
                    .saturating_mul(edges.saturating_add(1))
                    .saturating_mul(8),
            )
        }
        "analogical" => {
            let n = payload["relevant_features"].as_array().map_or(0, Vec::len);
            linear.saturating_add(n.saturating_mul(n).saturating_mul(16))
        }
        // Mean/variance contrast and ordered sampled traces use linear scans.
        _ => linear,
    }
}
