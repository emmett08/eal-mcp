//! Identity and retained-authoring consistency of composed declarations.
use super::{Checker, repr};
use std::collections::BTreeSet;

impl Checker<'_> {
    pub(super) fn declarations(&mut self) -> bool {
        let p = self.program;
        let count = p.environments.len()
            + p.tools.len()
            + p.evidence.len()
            + p.assumptions.len()
            + p.reasoning.len()
            + p.claims.len()
            + p.arguments.len()
            + p.objections.len()
            + 2 * p.patterns.len()
            + p.applications.len()
            + p.argumentation_directives.len();
        if count.max(p.declaration_count) > p.limits.declarations {
            self.error(
                "resource_limit",
                format!(
                    "At most {} declaration/body records after pattern expansion are supported",
                    p.limits.declarations
                ),
                None,
            );
            return false;
        }
        let mut seen = BTreeSet::new();
        let mut duplicates: BTreeSet<String> = p.duplicates.iter().cloned().collect();
        let mut identities = Vec::new();
        macro_rules! records {
            ($table:expr) => {
                identities.extend(
                    $table
                        .iter()
                        .map(|(key, value)| (key.as_str(), value.name.as_str(), false)),
                );
            };
        }
        records!(p.environments);
        records!(p.tools);
        records!(p.evidence);
        records!(p.assumptions);
        records!(p.reasoning);
        records!(p.claims);
        identities.extend(
            p.arguments
                .iter()
                .map(|(key, value)| (key.as_str(), value.name.as_str(), value.origin.is_some())),
        );
        records!(p.objections);
        records!(p.patterns);
        records!(p.applications);
        for (key, name, origin) in identities {
            self.identifier(key, name);
            if key != name {
                self.detail(
                    "declaration_identity",
                    "Declaration key must equal its declared name",
                    Some(key),
                    Some(key.into()),
                    Some(name.into()),
                );
            }
            if !origin && !p.generated.iter().any(|name| name == key) && !seen.insert(key) {
                duplicates.insert(key.into());
            }
        }
        for pattern in p.patterns.values() {
            for parameter in &pattern.parameters {
                self.identifier(&parameter.name, &pattern.name);
                if parameter.name.contains('.') {
                    self.error(
                        "invalid_identifier",
                        "Parameter names must be unqualified identifiers",
                        Some(&pattern.name),
                    );
                }
            }
        }
        for application in p.applications.values() {
            for binding in &application.arguments {
                self.identifier(&binding.name, &application.name);
                if binding.name.contains('.') {
                    self.error(
                        "invalid_identifier",
                        "Binding names must be unqualified identifiers",
                        Some(&application.name),
                    );
                }
            }
        }
        for name in duplicates {
            self.error(
                "duplicate_symbol",
                format!("Symbol {} is declared more than once", repr(&name)),
                Some(&name),
            );
        }
        {
            let replay = crate::composition::relower(p);
            let (different, changed) = match replay {
                Ok(fresh) => {
                    macro_rules! differs {
                        ($field:ident) => {{
                            let old = serde_json::to_value(&p.$field).ok();
                            let new = serde_json::to_value(&fresh.$field).ok();
                            old != new
                                || old.as_ref().is_none_or(|value| !finite_json(value))
                                || new.as_ref().is_none_or(|value| !finite_json(value))
                        }};
                    }
                    let mut old = p.lowering_diagnostics.clone();
                    let mut new = fresh.lowering_diagnostics.clone();
                    for diagnostic in old.iter_mut().chain(new.iter_mut()) {
                        diagnostic.span = None;
                    }
                    let changed = p
                        .arguments
                        .keys()
                        .chain(fresh.arguments.keys())
                        .find(|name| p.arguments.get(*name) != fresh.arguments.get(*name))
                        .cloned();
                    (
                        p.evidence.values().any(|value| !value.max_age.is_finite())
                            || differs!(profile)
                            || differs!(environments)
                            || differs!(tools)
                            || differs!(evidence)
                            || differs!(assumptions)
                            || differs!(reasoning)
                            || differs!(claims)
                            || differs!(arguments)
                            || differs!(objections)
                            || differs!(patterns)
                            || differs!(applications)
                            || old != new
                            || fresh.declaration_count != p.declaration_count,
                        changed,
                    )
                }
                Err(_) => (true, None),
            };
            if different {
                self.error("stale_pattern_expansion","Stored arguments or diagnostics differ from the declared patterns and applications; lower the edited program again",changed.as_deref());
            }
        }
        if p.profile
            .as_deref()
            .is_some_and(|profile| profile != "execution/1")
        {
            self.error(
                "unsupported_profile",
                "Supported execution profile is execution/1",
                None,
            );
        }
        if p.language != "EAL/3" {
            self.detail(
                "unsupported_language",
                format!("Expected EAL/3, found {}", repr(&p.language)),
                None,
                Some("EAL/3".into()),
                Some(p.language.clone()),
            );
        }
        true
    }

    pub(super) fn pattern_metadata(&mut self) {
        let p = self.program;
        for pattern in p.patterns.values() {
            if pattern.body.is_none() {
                continue;
            }
            match crate::composition::pattern_metadata(pattern, &p.limits) {
                Ok(entries) => {
                    for (name, mut local, span) in entries {
                        local.profile = p.profile.clone();
                        let mut checker = Checker {
                            program: &local,
                            registry: self.registry,
                            problems: Vec::new(),
                            references: false,
                        };
                        checker.predicates();
                        checker.tools_and_evidence();
                        checker.acquisition_bindings();
                        checker.assumptions();
                        checker.reasoning_and_claims();
                        self.problems
                            .extend(checker.problems.into_iter().map(|mut diagnostic| {
                                diagnostic.message =
                                    format!("Local {}: {}", repr(&name), diagnostic.message);
                                diagnostic.declaration = Some(pattern.name.clone());
                                diagnostic.span = Some(span);
                                diagnostic
                            }));
                    }
                }
                Err(error) => self.error("invalid_pattern", error.to_string(), Some(&pattern.name)),
            }
        }
    }
}

fn finite_json(value: &serde_json::Value) -> bool {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            serde_json::Value::Number(n)
                if n.to_string().contains(['.', 'e', 'E'])
                    && !n.as_f64().is_some_and(f64::is_finite) =>
            {
                return false;
            }
            serde_json::Value::Array(items) => pending.extend(items),
            serde_json::Value::Object(items) => pending.extend(items.values()),
            _ => {}
        }
    }
    true
}
