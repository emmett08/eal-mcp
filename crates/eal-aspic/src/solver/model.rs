//! Checked, finite theory data and constructed argument identities.
use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Theory {
    pub premises: Vec<Premise>,
    pub rules: Vec<Rule>,
    pub contraries: Vec<Contrary>,
    pub goal: String,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub priorities: Vec<Priority>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Premise {
    pub atom: String,
    pub kind: String,
    pub rank: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Rule {
    pub id: String,
    pub kind: String,
    pub antecedents: Vec<String>,
    pub consequent: String,
    pub name: Option<String>,
    pub rank: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Contrary {
    pub attacker: String,
    pub target: String,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct Options {
    pub semantics: Option<String>,
    pub query_mode: Option<String>,
    pub preference: Option<String>,
}

impl Options {
    pub fn semantics(&self) -> &str {
        self.semantics.as_deref().unwrap_or("grounded")
    }
    pub fn query_mode(&self) -> &str {
        self.query_mode.as_deref().unwrap_or("sceptical")
    }
    pub fn preference(&self) -> &str {
        self.preference.as_deref().unwrap_or("minimum_rank")
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Priority {
    pub kind: String,
    pub higher: String,
    pub lower: String,
}

pub(super) type Contraries = BTreeSet<(String, String)>;
pub(super) type Priorities = BTreeSet<(String, String, String)>;
pub(super) type Edges = BTreeSet<(usize, usize)>;

pub(super) struct CheckedTheory {
    pub theory: Theory,
    pub contraries: Contraries,
    pub atom_order: Option<Vec<String>>,
    pub priorities: Priorities,
}

#[derive(Clone, Debug)]
pub(super) struct Argument {
    pub conclusion: String,
    pub top: String,
    pub subarguments: Vec<usize>,
    pub direct_subarguments: Vec<usize>,
    pub strength: u16,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub rank: Option<u16>,
    pub ordinary: Vec<String>,
    pub defeasible: Vec<String>,
    pub last: Vec<String>,
}

pub(super) struct Arguments {
    pub values: Vec<Argument>,
    pub by_atom: BTreeMap<String, Vec<usize>>,
}

pub(super) fn argument_id(index: usize) -> String {
    format!("A{index}")
}
