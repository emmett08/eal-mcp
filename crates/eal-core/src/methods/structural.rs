//! Acyclic affine intervention, scalar feature correspondence and sampled trace checks.
use super::validation::{list, name, number, numeric_compare, object, result, scalar_equal};
use crate::expressions::numbers;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub fn counterfactual(value: &Value) -> Result<Value, String> {
    let o = object(
        value,
        &["variables", "intervention", "outcome"],
        "causal_model",
    )?;
    let variables = o["variables"]
        .as_object()
        .filter(|v| (1..=32).contains(&v.len()))
        .ok_or_else(|| "variables must contain 1..32 structural equations".to_owned())?;
    let mut equations = BTreeMap::new();
    for (key, equation) in variables {
        name(&Value::String(key.clone()), "variable name")?;
        let e = object(
            equation,
            &["intercept", "coefficients", "noise"],
            "structural equation",
        )?;
        let coefficients = e["coefficients"]
            .as_object()
            .filter(|m| m.len() <= 32)
            .ok_or_else(|| "coefficients must map at most 32 variables to numbers".to_owned())?;
        if coefficients.keys().any(|p| !variables.contains_key(p)) {
            return Err("Structural equation references an unknown parent variable".into());
        }
        number(&e["intercept"], "intercept")?;
        number(&e["noise"], "noise")?;
        for c in coefficients.values() {
            number(c, "coefficient")?;
        }
        equations.insert(
            key.clone(),
            (
                e["intercept"].clone(),
                coefficients.clone(),
                e["noise"].clone(),
            ),
        );
    }
    let intervention = object(&o["intervention"], &["variable", "value"], "intervention")?;
    let changed = name(&intervention["variable"], "intervention variable")?;
    let outcome = name(&o["outcome"], "outcome")?;
    if !equations.contains_key(changed) || !equations.contains_key(outcome) {
        return Err("Intervention and outcome must name declared variables".into());
    }
    number(&intervention["value"], "intervention value")?;
    let setting = &intervention["value"];
    let mut order = Vec::<String>::new();
    let mut remaining: BTreeSet<_> = equations.keys().cloned().collect();
    while !remaining.is_empty() {
        let ready: Vec<_> = remaining
            .iter()
            .filter(|key| {
                equations[*key]
                    .1
                    .keys()
                    .all(|parent| order.contains(parent))
            })
            .cloned()
            .collect();
        if ready.is_empty() {
            return Err(
                "Structural equations must be acyclic, including zero-coefficient references"
                    .into(),
            );
        }
        for key in ready {
            remaining.remove(&key);
            order.push(key);
        }
    }
    let solve = |intervene: bool| -> Result<Map<String, Value>, String> {
        let mut values = Map::new();
        for key in &order {
            let (intercept, parents, noise) = &equations[key];
            let value = if intervene && key == changed {
                setting.clone()
            } else {
                let mut terms = vec![intercept.clone(), noise.clone()];
                for (parent, c) in parents {
                    terms.push(numbers::arithmetic("*", c, &values[parent])?);
                }
                if terms.iter().all(super::validation::is_integer) {
                    let mut sum = json!(0);
                    for term in terms {
                        sum = numbers::arithmetic("+", &sum, &term)?;
                    }
                    sum
                } else {
                    let mut floats = Vec::new();
                    for term in terms {
                        floats.push(term.as_f64().filter(|n| n.is_finite()).ok_or_else(|| {
                            "Computed structural value must be finite".to_owned()
                        })?);
                    }
                    json!(super::numerical::fsum(floats)?)
                }
            };
            number(&value, "Computed structural value")?;
            values.insert(key.clone(), value);
        }
        Ok(values)
    };
    let factual = solve(false)?;
    let counterfactual = solve(true)?;
    let difference = numbers::arithmetic("-", &counterfactual[outcome], &factual[outcome])?;
    Ok(result(
        true,
        "Intervention evaluated in an acyclic affine model with the same supplied exogenous values",
        json!({"factual":factual[outcome],"counterfactual":counterfactual[outcome],"difference":difference,"factual_values":factual,"counterfactual_values":counterfactual,"outcome":outcome,"evaluation_order":order,"assumptions_verified":false}),
    ))
}

pub fn analogical(value: &Value) -> Result<Value, String> {
    let o = object(value, &["relevant_features", "source", "target"], "analogy")?;
    let features: Vec<_> = list(&o["relevant_features"], "relevant_features", 1, 256)?
        .iter()
        .map(|x| name(x, "feature").map(str::to_owned))
        .collect::<Result<_, _>>()?;
    if features.iter().collect::<BTreeSet<_>>().len() != features.len() {
        return Err("Relevant feature names must be unique".into());
    }
    let (Some(source), Some(target)) = (o["source"].as_object(), o["target"].as_object()) else {
        return Err("Source and target must be feature maps".into());
    };
    let mut missing: Vec<_> = features
        .iter()
        .filter(|key| !source.contains_key(*key) || !target.contains_key(*key))
        .cloned()
        .collect();
    missing.sort();
    if !missing.is_empty() {
        return Ok(result(
            false,
            "The relevant-feature mapping is incomplete",
            json!({"complete":false,"missing":missing,"match_fraction":null,"mismatches":[],"assumptions_verified":false}),
        ));
    }
    let mut mismatches = Vec::new();
    for key in &features {
        if !scalar_equal(&source[key], &target[key])? {
            mismatches.push(key.clone());
        }
    }
    Ok(result(
        true,
        "Declared features compared exactly; feature relevance and conclusion transfer remain authored reasoning",
        json!({"complete":true,"feature_count":features.len(),"matched":features.len()-mismatches.len(),"match_fraction":(features.len()-mismatches.len()) as f64/features.len() as f64,"mismatches":mismatches,"missing":[],"interpretation":"feature_correspondence","assumptions_verified":false}),
    ))
}

pub fn temporal(value: &Value) -> Result<Value, String> {
    let o = object(
        value,
        &["start", "end", "max_gap", "events", "property", "semantics"],
        "trace",
    )?;
    if o["semantics"] != "sampled" {
        return Err("Temporal mode supports only explicitly sampled semantics".into());
    }
    for key in ["start", "end", "max_gap"] {
        number(&o[key], key)?;
    }
    let (start, end, max_gap) = (&o["start"], &o["end"], &o["max_gap"]);
    if numeric_compare(start, end)?.is_gt() || !numeric_compare(max_gap, &json!(0))?.is_gt() {
        return Err("Trace requires start <= end and max_gap > 0".into());
    }
    let property = object(&o["property"], &["operator", "value"], "trace property")?;
    let op = property["operator"]
        .as_str()
        .filter(|op| ["lt", "le", "eq", "ne", "ge", "gt"].contains(op))
        .ok_or_else(|| "Trace property operator must be lt/le/eq/ne/ge/gt".to_owned())?;
    let threshold = &property["value"];
    number(threshold, "property value")?;
    let events = list(&o["events"], "events", 1, 10_000)?;
    let mut times: Vec<Value> = Vec::new();
    let mut failures = Vec::new();
    for event in events {
        let e = object(event, &["time", "value"], "trace event")?;
        let time = &e["time"];
        number(time, "event time")?;
        number(&e["value"], "event value")?;
        if numeric_compare(start, time)?.is_gt()
            || numeric_compare(time, end)?.is_gt()
            || times
                .last()
                .is_some_and(|previous| numeric_compare(time, previous).is_ok_and(|o| !o.is_gt()))
        {
            return Err(
                "Events must be strictly time-ordered and within the declared interval".into(),
            );
        }
        let order = numeric_compare(&e["value"], threshold)?;
        let holds = match op {
            "lt" => order.is_lt(),
            "le" => !order.is_gt(),
            "eq" => order.is_eq(),
            "ne" => !order.is_eq(),
            "ge" => !order.is_lt(),
            _ => order.is_gt(),
        };
        times.push(time.clone());
        if !holds {
            failures.push(time.clone());
        }
    }
    let mut largest_gap = json!(0.0);
    for pair in times.windows(2) {
        let gap = numbers::arithmetic("-", &pair[1], &pair[0])?;
        if numeric_compare(&gap, &largest_gap)?.is_gt() {
            largest_gap = gap;
        }
    }
    let coverage = numeric_compare(&times[0], start)?.is_eq()
        && numeric_compare(times.last().expect("nonempty events"), end)?.is_eq()
        && !numeric_compare(&largest_gap, max_gap)?.is_gt();
    let holds = failures.is_empty();
    let reason = if coverage && holds {
        "The sampled property holds throughout the complete declared sampling contract"
    } else if !coverage {
        "The sampling contract has a gap or missing endpoint"
    } else {
        "The sampled trace contains a property violation"
    };
    Ok(result(
        coverage,
        reason,
        json!({"holds":holds,"coverage":coverage,"sample_size":events.len(),"largest_gap":largest_gap,"violation_count":failures.len(),"violation_times":failures,"semantics":"sampled","continuous_truth_established":false}),
    ))
}
