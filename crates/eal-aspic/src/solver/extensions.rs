//! Exact bounded admissible/stable extension enumeration with arbitrary masks.
use std::cmp::Reverse;
use std::collections::BTreeSet;

use eal_core::ExecutionLimits;
use num_bigint::BigUint;
use num_traits::{One, Zero};

use super::{SolverResult, model::Edges};

pub(super) fn extensions(
    count: usize,
    edges: &Edges,
    grounded: &BTreeSet<usize>,
    rejected: &BTreeSet<usize>,
    semantics: &str,
    limits: &ExecutionLimits,
) -> SolverResult<Vec<BTreeSet<usize>>> {
    let mut incoming = vec![BigUint::zero(); count];
    let mut outgoing = vec![BigUint::zero(); count];
    for &(attacker, target) in edges {
        outgoing[attacker] |= BigUint::one() << target;
        incoming[target] |= BigUint::one() << attacker;
    }
    let fixed = grounded.iter().fold(BigUint::zero(), |mask, index| {
        mask | (BigUint::one() << index)
    });
    let undecided: Vec<_> = (0..count)
        .filter(|index| !grounded.contains(index) && !rejected.contains(index))
        .collect();
    let universe = (BigUint::one() << count) - BigUint::one();
    let mut candidates = Vec::new();
    let mut steps = 0usize;
    let mut pending = vec![(0usize, fixed)];
    while let Some((cursor, selected)) = pending.pop() {
        consume(&mut steps, limits, "Extension enumeration budget exceeded")?;
        let mut attacked = BigUint::zero();
        let mut attackers = BigUint::zero();
        for index in 0..count {
            if selected.bit(index as u64) {
                attacked |= &outgoing[index];
                attackers |= &incoming[index];
            }
        }
        if !(&attacked & &selected).is_zero() {
            continue;
        }
        if cursor < undecided.len() {
            let included = &selected | (BigUint::one() << undecided[cursor]);
            pending.push((cursor + 1, selected));
            pending.push((cursor + 1, included));
            continue;
        }
        if (&attackers & &attacked) != attackers {
            continue;
        }
        if semantics != "stable" || (&selected | &attacked) == universe {
            candidates.push(selected);
        }
    }
    if semantics == "preferred" {
        let mut maximal: Vec<BigUint> = Vec::new();
        candidates.sort_by_key(|candidate| {
            Reverse(
                candidate
                    .iter_u64_digits()
                    .map(u64::count_ones)
                    .sum::<u32>(),
            )
        });
        for candidate in candidates {
            let mut subset = false;
            for larger in &maximal {
                consume(
                    &mut steps,
                    limits,
                    "Maximal-extension search budget exceeded",
                )?;
                if (&candidate & larger) == candidate {
                    subset = true;
                    break;
                }
            }
            if !subset {
                maximal.push(candidate);
            }
        }
        candidates = maximal;
    }
    candidates.sort();
    Ok(candidates
        .into_iter()
        .map(|mask| (0..count).filter(|index| mask.bit(*index as u64)).collect())
        .collect())
}

fn consume(steps: &mut usize, limits: &ExecutionLimits, message: &str) -> SolverResult<()> {
    *steps = steps
        .checked_add(1)
        .ok_or_else(|| crate::AspicError::Incomplete(message.into()))?;
    if *steps > limits.extension_search {
        return Err(crate::AspicError::Incomplete(message.into()));
    }
    Ok(())
}
