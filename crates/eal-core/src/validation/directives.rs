//! Reviewed argumentation target kinds, priorities and formal consistency.
use super::{Checker, repr};
use crate::semantic::{ArgumentationDirective, Diagnostic};
use std::collections::{BTreeMap, BTreeSet};

impl Checker<'_> {
    pub(super) fn directives(&mut self) {
        let p = self.program;
        let mut kinds: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        macro_rules! table {
            ($table:expr,$kind:expr) => {
                for name in $table.keys() {
                    kinds.entry(name.clone()).or_default().insert($kind.into());
                }
            };
        }
        table!(p.environments, "environment");
        table!(p.tools, "tool");
        table!(p.evidence, "evidence");
        table!(p.assumptions, "assumption");
        table!(p.reasoning, "reasoning");
        table!(p.claims, "claim");
        table!(p.arguments, "argument");
        table!(p.objections, "objection");
        table!(p.patterns, "pattern");
        for name in p.applications.keys() {
            if !p.arguments.contains_key(name) {
                kinds
                    .entry(name.clone())
                    .or_default()
                    .insert("application".into());
            }
        }
        let ranked: BTreeSet<_> = p
            .argumentation_directives
            .iter()
            .filter(|d| d.kind == "rank")
            .map(|d| d.name.as_str())
            .collect();
        let strict: BTreeSet<_> = p
            .argumentation_directives
            .iter()
            .filter(|d| d.kind == "strict")
            .map(|d| d.name.as_str())
            .collect();
        let mut seen = BTreeSet::new();
        for directive in &p.argumentation_directives {
            let (valid, allowed): (_, BTreeSet<String>) = match directive.kind.as_str() {
                "strict" => (
                    directive.other.is_none() && directive.rank.is_none(),
                    BTreeSet::from(["argument".into()]),
                ),
                "rank" => (
                    directive.other.is_none()
                        && directive.rank.is_some_and(|r| (0..=1000).contains(&r)),
                    BTreeSet::from(["evidence".into(), "assumption".into(), "argument".into()]),
                ),
                "contrary" => (
                    directive.rank.is_none() && directive.other.is_some(),
                    BTreeSet::from(["claim".into()]),
                ),
                "prefer" => (
                    directive.rank.is_none() && directive.other.is_some(),
                    BTreeSet::from(["evidence".into(), "assumption".into(), "argument".into()]),
                ),
                _ => (false, BTreeSet::new()),
            };
            let key = (
                directive.kind.clone(),
                directive.name.clone(),
                if matches!(directive.kind.as_str(), "contrary" | "prefer") {
                    directive.other.clone()
                } else {
                    None
                },
            );
            if !valid {
                self.formal_error("invalid_formal_directive","Invalid argumentation directive or rank; ranks must be integers from 0 through 1000",directive,None,None);
            }
            if directive.review.trim().is_empty() {
                self.formal_error(
                    "missing_formal_review",
                    "An argumentation directive requires a nonempty review reference",
                    directive,
                    None,
                    None,
                );
            }
            if !seen.insert(key) {
                self.formal_error(
                    "duplicate_formal_directive",
                    "An argumentation directive is declared more than once",
                    directive,
                    None,
                    None,
                );
            }
            if !allowed.is_empty() {
                self.formal_target(&directive.name, &allowed, directive, &kinds);
            }
            if directive.kind == "contrary" {
                if directive.other.as_deref() == Some(&directive.name) {
                    self.formal_error(
                        "invalid_formal_contrary",
                        "A claim cannot be contrary to itself",
                        directive,
                        None,
                        None,
                    );
                }
                if let Some(other) = &directive.other {
                    self.formal_target(other, &BTreeSet::from(["claim".into()]), directive, &kinds);
                    if let (Some(first), Some(second)) =
                        (p.claims.get(&directive.name), p.claims.get(other))
                        && first.environment != second.environment
                    {
                        self.formal_error(
                            "formal_environment_mismatch",
                            "Contrary claims require the same EAL environment",
                            directive,
                            Some(first.environment.clone()),
                            Some(second.environment.clone()),
                        );
                    }
                }
            }
            if directive.kind == "strict" && ranked.contains(directive.name.as_str()) {
                self.formal_error(
                    "formal_strict_rank",
                    "A strict rule cannot have a defeasible rank",
                    directive,
                    None,
                    None,
                );
            }
            if directive.kind == "prefer"
                && let Some(other) = &directive.other
            {
                self.formal_target(other, &allowed, directive, &kinds);
                let evidence = BTreeSet::from(["evidence".into()]);
                if directive.name == *other
                    || (kinds.get(&directive.name) == Some(&evidence))
                        != (kinds.get(other) == Some(&evidence))
                {
                    self.formal_error(
                        "invalid_formal_preference",
                        "Priorities compare distinct evidence premises or distinct fallible rules",
                        directive,
                        None,
                        None,
                    );
                }
                if strict.contains(directive.name.as_str()) || strict.contains(other.as_str()) {
                    self.formal_error(
                        "invalid_formal_preference",
                        "A strict rule cannot carry fallible priority",
                        directive,
                        None,
                        None,
                    );
                }
            }
        }
        let priorities: Vec<_> = p
            .argumentation_directives
            .iter()
            .filter(|d| d.kind == "prefer")
            .collect();
        let mut graph: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for directive in &priorities {
            if let Some(other) = &directive.other {
                graph.entry(&directive.name).or_default().push(other);
            }
        }
        for directive in priorities {
            let Some(other) = &directive.other else {
                continue;
            };
            let mut pending = vec![other.as_str()];
            let mut visited = BTreeSet::new();
            while let Some(node) = pending.pop() {
                if node == directive.name {
                    self.formal_error(
                        "cyclic_formal_preference",
                        "Strict priorities must be acyclic",
                        directive,
                        None,
                        None,
                    );
                    break;
                }
                if visited.insert(node)
                    && let Some(next) = graph.get(node)
                {
                    pending.extend(next);
                }
            }
        }
    }

    fn formal_error(
        &mut self,
        code: &str,
        message: impl Into<String>,
        directive: &ArgumentationDirective,
        expected: Option<String>,
        actual: Option<String>,
    ) {
        self.problems.push(Diagnostic {
            code: code.into(),
            message: message.into(),
            declaration: Some(directive.name.clone()),
            span: Some(directive.span),
            expected,
            actual,
        });
    }
    fn formal_target(
        &mut self,
        name: &str,
        allowed: &BTreeSet<String>,
        directive: &ArgumentationDirective,
        kinds: &BTreeMap<String, BTreeSet<String>>,
    ) {
        let empty = BTreeSet::new();
        let found = kinds.get(name).unwrap_or(&empty);
        if found.is_empty() {
            self.formal_error(
                "unknown_formal_reference",
                format!("Unknown argumentation target {}", repr(name)),
                directive,
                Some(allowed.iter().cloned().collect::<Vec<_>>().join(" | ")),
                Some(name.into()),
            );
        } else if found.len() != 1 || self.program.duplicates.iter().any(|n| n == name) {
            self.formal_error(
                "ambiguous_formal_reference",
                format!("Argumentation target {} is not unique", repr(name)),
                directive,
                Some("unique symbol".into()),
                Some(found.iter().cloned().collect::<Vec<_>>().join(" | ")),
            );
        } else if !found.is_subset(allowed) {
            self.formal_error(
                "formal_target_kind",
                format!(
                    "Argumentation target {} has the wrong declaration kind",
                    repr(name)
                ),
                directive,
                Some(allowed.iter().cloned().collect::<Vec<_>>().join(" | ")),
                found.iter().next().cloned(),
            );
        }
    }
}
