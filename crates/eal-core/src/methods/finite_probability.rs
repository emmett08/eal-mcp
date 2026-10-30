//! Exact finite Bayesian inference with inspectable mappings and explicit undefined conditioning.
mod model;
mod rational;

use super::validation::result;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use serde_json::{Value, json};

pub(super) const MAX_ASSIGNMENTS: usize = 65_536;

pub(super) fn assess(value: &Value) -> Result<Value, String> {
    match compute(value) {
        Ok(value) => Ok(value),
        Err(super::MethodError::Invalid(error)) => Err(error),
        Err(super::MethodError::Incomplete(error)) => {
            Ok(json!({"status":"incomplete","reasons":[error],"details":{}}))
        }
    }
}

fn compute(value: &Value) -> Result<Value, super::MethodError> {
    let model = model::parse(value)?;
    let mut coordinates = vec![0usize; model.nodes.len()];
    let mut normalizer = BigRational::zero();
    let mut numerator = BigRational::zero();
    let mut matching = 0usize;
    for ordinal in 0..model.assignments {
        let mut remainder = ordinal;
        // Supplied node order determines enumeration only, never graph meaning.
        // The last supplied node varies fastest.
        for (coordinate, node) in coordinates.iter_mut().zip(&model.nodes).rev() {
            *coordinate = remainder % node.states.len();
            remainder /= node.states.len();
        }
        if model
            .evidence
            .iter()
            .any(|(node, state)| coordinates[*node] != *state)
        {
            continue;
        }
        matching += 1;
        let mut joint = BigRational::one();
        for (index, node) in model.nodes.iter().enumerate() {
            let row = node.parents.iter().fold(0usize, |row, parent| {
                row * model.nodes[*parent].states.len() + coordinates[*parent]
            });
            let offset = row * node.states.len() + coordinates[index];
            joint = rational::checked(joint * &node.probabilities[offset])?;
        }
        normalizer = rational::checked(normalizer + &joint)?;
        if coordinates[model.target.0] == model.target.1 {
            numerator = rational::checked(numerator + joint)?;
        }
    }
    let defined = !normalizer.is_zero();
    let mut details = json!({
        "defined":defined,"satisfies":false,
        "normalizer":normalizer.to_f64().ok_or("Normalizer cannot be represented as a finite display number")?,
        "normalizer_exact":rational::encode(&normalizer),"numerator_exact":rational::encode(&numerator),
        "assignments":model.assignments,"matching_assignments":matching,
        "comparison":{"operator":model.operator,"threshold_exact":rational::encode(&model.threshold)},
        "mapping":{
            "node_order":model.nodes.iter().map(|node|node.name.clone()).collect::<Vec<_>>(),
            "nodes":model.nodes.iter().map(|node|json!({"name":node.name,"states":node.states,
                "parents":node.parents.iter().map(|index|model.nodes[*index].name.clone()).collect::<Vec<_>>() })).collect::<Vec<_>>(),
            "cpd_layout":"parents in declared order; last parent varies fastest; child state varies fastest within each parent configuration",
            "evidence":value["evidence"],"target":value["target"]
        }
    });
    if defined {
        let posterior = rational::checked(numerator / normalizer)?;
        details["satisfies"] = json!(match model.operator.as_str() {
            "<" => posterior < model.threshold,
            "<=" => posterior <= model.threshold,
            "==" => posterior == model.threshold,
            "!=" => posterior != model.threshold,
            ">=" => posterior >= model.threshold,
            ">" => posterior > model.threshold,
            _ => unreachable!("validated comparison"),
        });
        details["posterior"] = json!(
            posterior
                .to_f64()
                .ok_or("Posterior cannot be represented as a finite display number")?
        );
        details["posterior_exact"] = rational::encode(&posterior);
    }
    Ok(result(
        true,
        if defined {
            "Complete exact finite-model conditioning; model correspondence and empirical adequacy remain separate obligations"
        } else {
            "Complete finite calculation found zero-probability evidence; the conditional probability is undefined and no posterior is returned"
        },
        details,
    ))
}
