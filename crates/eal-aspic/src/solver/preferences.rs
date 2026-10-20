//! Strict rank and last-link preference lifting over checked arguments.
use std::collections::{BTreeMap, BTreeSet};

use eal_core::ExecutionLimits;

use super::SolverResult;
use super::model::{Argument, Priorities, Theory};

pub(super) fn priority_order(
    theory: &Theory,
    limits: &ExecutionLimits,
) -> SolverResult<Priorities> {
    if !theory.priorities.is_empty() && theory.options.preference() != "last_link_partial" {
        return Err("Partial priorities require last_link_partial preference".into());
    }
    let premise_names: BTreeSet<_> = theory
        .premises
        .iter()
        .filter(|item| item.kind == "ordinary")
        .map(|item| item.atom.as_str())
        .collect();
    let rule_names: BTreeSet<_> = theory
        .rules
        .iter()
        .filter(|item| item.kind == "defeasible")
        .map(|item| item.id.as_str())
        .collect();
    let mut order = Priorities::new();
    for item in &theory.priorities {
        let names = if item.kind == "premise" {
            &premise_names
        } else {
            &rule_names
        };
        let relation = (item.kind.clone(), item.lower.clone(), item.higher.clone());
        if !names.contains(item.lower.as_str())
            || !names.contains(item.higher.as_str())
            || item.lower == item.higher
            || !order.insert(relation)
        {
            return Err(
                "Priorities require distinct declared fallible elements of the same kind".into(),
            );
        }
    }
    loop {
        let mut extra = Priorities::new();
        for (kind, low, middle) in &order {
            for (other_kind, other_middle, high) in &order {
                if kind == other_kind && middle == other_middle {
                    let relation = (kind.clone(), low.clone(), high.clone());
                    if !order.contains(&relation) {
                        extra.insert(relation);
                    }
                }
            }
        }
        if extra.is_empty() {
            return Ok(order);
        }
        if order.len().saturating_add(extra.len()) > limits.formal_defeats {
            return Err(crate::AspicError::Incomplete(
                "Priority-closure budget exceeded".into(),
            ));
        }
        order.extend(extra);
        if order.iter().any(|(_, low, high)| low == high) {
            return Err("Strict priorities must be acyclic".into());
        }
    }
}

pub(super) struct Preference<'a> {
    policy: &'a str,
    order: &'a Priorities,
    premise_ranks: BTreeMap<&'a str, u16>,
    rule_ranks: BTreeMap<&'a str, u16>,
}

impl<'a> Preference<'a> {
    pub fn new(theory: &'a Theory, order: &'a Priorities) -> Self {
        Self {
            policy: theory.options.preference(),
            order,
            premise_ranks: theory
                .premises
                .iter()
                .map(|item| (item.atom.as_str(), item.rank.unwrap_or(1001)))
                .collect(),
            rule_ranks: theory
                .rules
                .iter()
                .map(|item| (item.id.as_str(), item.rank.unwrap_or(1001)))
                .collect(),
        }
    }

    pub fn weaker(&self, left: &Argument, right: &Argument) -> bool {
        if self.policy == "minimum_rank" {
            return left.strength < right.strength;
        }
        let (left, right, kind, ranks) = if !left.last.is_empty() || !right.last.is_empty() {
            (&left.last, &right.last, "rule", &self.rule_ranks)
        } else {
            (
                &left.ordinary,
                &right.ordinary,
                "premise",
                &self.premise_ranks,
            )
        };
        if self.policy == "last_link_rank" {
            let minimum = |items: &[String]| {
                items
                    .iter()
                    .filter_map(|item| ranks.get(item.as_str()))
                    .copied()
                    .min()
                    .unwrap_or(1001)
            };
            return minimum(left) < minimum(right);
        }
        !left.is_empty()
            && left.iter().any(|item| {
                right.iter().all(|other| {
                    self.order
                        .contains(&(kind.to_owned(), item.clone(), other.clone()))
                })
            })
    }
}
