//! Exhaustive bounded propositional entailment with deterministic countermodels.
use super::validation::{list, name, object, result};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub fn deductive(value: &Value) -> Result<Value, String> {
    let o = object(value, &["premises", "conclusion"], "logical_case")?;
    let premises = list(&o["premises"], "Logical premises", 0, 128)?;
    let mut atoms = BTreeSet::new();
    let mut budget = 512;
    for formula in premises.iter().chain(std::iter::once(&o["conclusion"])) {
        check(formula, 0, &mut budget, &mut atoms)?;
    }
    if atoms.len() > 12 {
        return Err("Finite deduction supports at most 12 distinct atoms".into());
    }
    let atoms: Vec<_> = atoms.into_iter().collect();
    let valuations = 1usize << atoms.len();
    let mut satisfying = 0;
    let mut counterexample = None;
    for bits in 0..valuations {
        let assignment: BTreeMap<_, _> = atoms
            .iter()
            .enumerate()
            .map(|(i, a)| (a.clone(), bits & (1 << (atoms.len() - 1 - i)) != 0))
            .collect();
        if premises.iter().all(|f| truth(f, &assignment)) {
            satisfying += 1;
            if counterexample.is_none() && !truth(&o["conclusion"], &assignment) {
                counterexample = Some(assignment);
            }
        }
    }
    let consistent = satisfying > 0;
    let entailed = counterexample.is_none();
    let reason = if consistent && entailed {
        "Finite entailment established with satisfiable premises"
    } else if !consistent {
        "Premises are inconsistent"
    } else {
        "A counterexample refutes entailment"
    };
    Ok(result(
        consistent,
        reason,
        json!({"entailed":entailed,"consistent_premises":consistent,"counterexample":counterexample,"atoms":atoms,"valuations":valuations,"satisfying_premise_valuations":satisfying}),
    ))
}
fn check(
    value: &Value,
    depth: usize,
    budget: &mut usize,
    atoms: &mut BTreeSet<String>,
) -> Result<(), String> {
    if *budget == 0 || depth > 32 {
        return Err("Logical formulas exceed 512 total nodes or depth 32".into());
    }
    *budget -= 1;
    if value.is_string() {
        atoms.insert(name(value, "Propositional atom")?.to_owned());
        return Ok(());
    }
    let Some(o) = value.as_object().filter(|o| o.len() == 1) else {
        return Err("A formula is an atom or one not/and/or/implies object".into());
    };
    let (op, operand) = o.iter().next().expect("one operator");
    match op.as_str() {
        "not" => check(operand, depth + 1, budget, atoms)?,
        "and" | "or" | "implies" => {
            for child in list(
                operand,
                &format!("{op} operands"),
                2,
                if op == "implies" { 2 } else { 128 },
            )? {
                check(child, depth + 1, budget, atoms)?;
            }
        }
        _ => {
            return Err(format!(
                "Unsupported logical operator {}",
                crate::expressions::python_repr(&Value::String(op.clone()))
            ));
        }
    }
    Ok(())
}
fn truth(value: &Value, assignment: &BTreeMap<String, bool>) -> bool {
    if let Some(atom) = value.as_str() {
        return assignment[atom];
    }
    let (op, operand) = value
        .as_object()
        .expect("checked formula")
        .iter()
        .next()
        .expect("operator");
    match op.as_str() {
        "not" => !truth(operand, assignment),
        "and" => operand
            .as_array()
            .expect("operands")
            .iter()
            .all(|f| truth(f, assignment)),
        "or" => operand
            .as_array()
            .expect("operands")
            .iter()
            .any(|f| truth(f, assignment)),
        _ => !truth(&operand[0], assignment) || truth(&operand[1], assignment),
    }
}
