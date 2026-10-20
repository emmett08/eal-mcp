//! Finite premise-founded argument construction with retained derivation order.
use std::collections::{BTreeMap, BTreeSet};

use eal_core::ExecutionLimits;
use indexmap::IndexSet;

use super::{SolverResult, model::*};

struct Construction<'a> {
    arguments: Arguments,
    limits: &'a ExecutionLimits,
    steps: usize,
    seen: BTreeSet<(String, Vec<usize>)>,
}

pub(super) fn construct(
    checked: &CheckedTheory,
    limits: &ExecutionLimits,
) -> SolverResult<Arguments> {
    let mut state = Construction {
        arguments: Arguments {
            values: Vec::new(),
            by_atom: BTreeMap::new(),
        },
        limits,
        steps: 0,
        seen: BTreeSet::new(),
    };
    for premise in &checked.theory.premises {
        state.add(
            &premise.atom,
            &premise.kind,
            &[],
            premise.rank.unwrap_or(1001),
            None,
            None,
        )?;
    }
    let mut rules: Vec<_> = checked.theory.rules.iter().collect();
    rules.sort_by(|left, right| left.id.cmp(&right.id));
    if let Some(atoms) = &checked.atom_order {
        for atom in atoms {
            for rule in rules
                .iter()
                .copied()
                .filter(|rule| &rule.consequent == atom)
            {
                state.apply(rule, false)?;
            }
        }
    } else {
        loop {
            let before = state.arguments.values.len();
            for rule in &rules {
                state.apply(rule, true)?;
            }
            if state.arguments.values.len() == before {
                break;
            }
        }
    }
    Ok(state.arguments)
}

impl Construction<'_> {
    fn add(
        &mut self,
        atom: &str,
        top: &str,
        children: &[usize],
        rank: u16,
        rule_id: Option<&str>,
        name: Option<&str>,
    ) -> SolverResult<()> {
        if self.arguments.values.len() >= self.limits.formal_arguments {
            return Err(crate::AspicError::Incomplete(
                "Theory exceeds the constructed-argument budget".into(),
            ));
        }
        let mut subs = IndexSet::new();
        let mut ordinary = IndexSet::new();
        let mut defeasible = IndexSet::new();
        let mut last = IndexSet::new();
        let mut strength = rank;
        if top == "ordinary" {
            ordinary.insert(atom.to_owned());
        }
        if top == "defeasible" {
            let id = rule_id.expect("checked defeasible rule").to_owned();
            defeasible.insert(id.clone());
            last.insert(id);
        }
        for &index in children {
            let child = &self.arguments.values[index];
            for &sub in child.subarguments.iter().chain([&index]) {
                subs.insert(sub);
            }
            for premise in &child.ordinary {
                ordinary.insert(premise.clone());
            }
            for rule in &child.defeasible {
                defeasible.insert(rule.clone());
            }
            if top != "defeasible" {
                for rule in &child.last {
                    last.insert(rule.clone());
                }
            }
            strength = strength.min(child.strength);
        }
        let index = self.arguments.values.len();
        self.arguments.values.push(Argument {
            conclusion: atom.to_owned(),
            top: top.to_owned(),
            subarguments: subs.into_iter().collect(),
            direct_subarguments: children.to_vec(),
            strength,
            rule_id: rule_id.map(str::to_owned),
            rule_name: name.map(str::to_owned),
            rank: (rank != 1001).then_some(rank),
            ordinary: ordinary.into_iter().collect(),
            defeasible: defeasible.into_iter().collect(),
            last: last.into_iter().collect(),
        });
        self.arguments
            .by_atom
            .entry(atom.to_owned())
            .or_default()
            .push(index);
        Ok(())
    }

    fn apply(&mut self, rule: &Rule, finite: bool) -> SolverResult<()> {
        let Some(choices): Option<Vec<Vec<usize>>> = rule
            .antecedents
            .iter()
            .map(|atom| self.arguments.by_atom.get(atom).cloned())
            .collect()
        else {
            return Ok(());
        };
        // Snapshot each antecedent's alternatives, as itertools.product does.
        let mut cursor = vec![0usize; choices.len()];
        loop {
            self.steps = self.steps.checked_add(1).ok_or_else(|| {
                crate::AspicError::Incomplete("Argument-construction search budget exceeded".into())
            })?;
            if self.steps > self.limits.formal_construction {
                return Err(crate::AspicError::Incomplete(
                    "Argument-construction search budget exceeded".into(),
                ));
            }
            let children: Vec<_> = choices
                .iter()
                .zip(&cursor)
                .map(|(items, index)| items[*index])
                .collect();
            if self.seen.insert((rule.id.clone(), children.clone()))
                && (!finite
                    || !children.iter().any(|&child| {
                        self.arguments.values[child]
                            .subarguments
                            .iter()
                            .copied()
                            .chain([child])
                            .any(|index| self.arguments.values[index].conclusion == rule.consequent)
                    }))
            {
                self.add(
                    &rule.consequent,
                    &rule.kind,
                    &children,
                    rule.rank.unwrap_or(1001),
                    Some(&rule.id),
                    rule.name.as_deref(),
                )?;
            }
            let mut position = cursor.len();
            loop {
                if position == 0 {
                    return Ok(());
                }
                position -= 1;
                cursor[position] += 1;
                if cursor[position] < choices[position].len() {
                    break;
                }
                cursor[position] = 0;
            }
        }
    }
}
