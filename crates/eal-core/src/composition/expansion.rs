//! Expand checked applications while preserving identities and finite recursion.
use super::*;
impl Lowerer<'_> {
    pub(super) fn expand(&mut self, record: &Record) -> Result<()> {
        let Entity::Application(application) = self.resolved(record) else {
            return Ok(());
        };
        if self.records["argument"]
            .get(&application.name)
            .is_some_and(|r| matches!(&r.value,Entity::Argument(a)if a.origin.is_none()))
        {
            self.error(
                "generated_symbol_collision",
                "An application cannot replace an authored argument",
                Some(&application.name),
                None,
                None,
                None,
            );
            return Ok(());
        }
        if let Some(r) = self
            .records
            .get_mut("application")
            .and_then(|t| t.get_mut(&application.name))
        {
            r.value = Entity::Application(application.clone());
            r.scope.bindings.clear();
            r.scope.prefix.clear();
        }
        let Some(template) = self.records["pattern"].get(&application.pattern).cloned() else {
            self.error(
                "unknown_pattern",
                format!("Unknown pattern {}", repr(&application.pattern)),
                Some(&application.name),
                None,
                None,
                None,
            );
            return Ok(());
        };
        let Entity::Pattern(pattern) = &template.value else {
            unreachable!()
        };
        self.check_template(&template)?;
        if self
            .errors
            .iter()
            .any(|e| e.declaration.as_deref() == Some(&pattern.name))
        {
            self.error(
                "invalid_pattern",
                format!("Pattern {} has an invalid definition", repr(&pattern.name)),
                Some(&application.name),
                None,
                None,
                None,
            );
            return Ok(());
        }
        self.expanded += 1;
        if self.expanded > self.limits.applications {
            self.error(
                "resource_limit",
                "Pattern application budget exceeded",
                Some(&application.name),
                None,
                None,
                None,
            );
            return Ok(());
        }
        let parameters: BTreeMap<_, _> = pattern
            .parameters
            .iter()
            .map(|p| (p.name.clone(), p.kind.clone()))
            .collect();
        let mut bindings = Bindings::new();
        let before = self.errors.len();
        for item in &application.arguments {
            if bindings
                .insert(item.name.clone(), item.reference.clone())
                .is_some()
            {
                self.error(
                    "duplicate_pattern_argument",
                    format!("Parameter {} is supplied twice", repr(&item.name)),
                    Some(&application.name),
                    None,
                    None,
                    None,
                );
            }
            let Some(expected) = parameters.get(&item.name) else {
                self.error(
                    "unknown_pattern_argument",
                    format!("Unknown parameter {}", repr(&item.name)),
                    Some(&application.name),
                    None,
                    None,
                    None,
                );
                continue;
            };
            let many = expected.ends_with("[]");
            if many != matches!(item.reference, s::BindingValue::List(_)) {
                self.error(
                    "pattern_argument_kind",
                    format!("Parameter {} requires {expected}", repr(&item.name)),
                    Some(&application.name),
                    None,
                    None,
                    None,
                );
                continue;
            }
            let values = match &item.reference {
                s::BindingValue::Single(n) => vec![n.clone()],
                s::BindingValue::List(v) => v.clone(),
            };
            for value in values {
                self.references += 1;
                let actual = self
                    .symbols
                    .get(&value)
                    .map_or("unknown reference", String::as_str)
                    .to_owned();
                if actual != expected.trim_end_matches("[]") {
                    self.error(
                        "pattern_argument_kind",
                        format!(
                            "Parameter {} requires {expected}; {value_repr} is {actual}",
                            repr(&item.name),
                            value_repr = repr(&value)
                        ),
                        Some(&application.name),
                        None,
                        Some(expected),
                        Some(&actual),
                    );
                }
            }
        }
        for missing in parameters.keys().filter(|n| !bindings.contains_key(*n)) {
            self.error(
                "missing_pattern_argument",
                format!(
                    "Pattern {} requires {missing_repr}",
                    repr(&pattern.name),
                    missing_repr = repr(missing)
                ),
                Some(&application.name),
                None,
                None,
                None,
            );
        }
        if self.references > self.limits.expanded_references {
            self.error(
                "resource_limit",
                "Expanded reference budget exceeded",
                Some(&application.name),
                None,
                None,
                None,
            );
        }
        if self.errors.len() != before {
            return Ok(());
        }
        let measure = pattern
            .decreases
            .as_ref()
            .and_then(|name| bindings.get(name))
            .and_then(|v| {
                if let s::BindingValue::List(v) = v {
                    Some(v.len())
                } else {
                    None
                }
            });
        let ancestor = record
            .scope
            .stack
            .iter()
            .rev()
            .find(|(name, _)| name == &pattern.name);
        if let Some((_, previous)) = ancestor {
            if measure
                .zip(*previous)
                .is_none_or(|(current, previous)| current >= previous)
            {
                self.error(
                    "nonterminating_pattern",
                    "Recursive composition must strictly decrease its declared finite list",
                    Some(&application.name),
                    None,
                    None,
                    None,
                );
                return Ok(());
            }
            let caller = record
                .scope
                .pattern
                .as_ref()
                .and_then(|name| self.records["pattern"].get(name))
                .and_then(|r| {
                    if let Entity::Pattern(p) = &r.value {
                        Some(p)
                    } else {
                        None
                    }
                });
            if !caller
                .and_then(|p| p.decreases.as_ref())
                .is_some_and(|d| record.scope.guards.contains(d))
            {
                self.error(
                    "nonterminating_pattern",
                    "A recursive call must occur inside its nonempty list guard",
                    Some(&application.name),
                    None,
                    None,
                    None,
                );
                return Ok(());
            }
        }
        let mut scope = record.scope.clone();
        scope.depth += 1;
        scope.stack.push((pattern.name.clone(), measure));
        scope.bindings = bindings;
        scope.pattern = Some(pattern.name.clone());
        scope.application = Some(application.name.clone());
        scope.guards.clear();
        scope.defaults = pattern.defaults.clone();
        if let Some(body) = &pattern.body {
            scope.prefix = application.name.clone();
            scope.source_file = template.scope.source_file.clone();
            self.block(body, &scope)?;
        } else {
            let value = s::Argument {
                name: application.name.clone(),
                conclusion: pattern.conclusion.clone(),
                reasoning: pattern.reasoning.clone(),
                evidence: pattern.evidence.clone(),
                assumptions: pattern.assumptions.clone(),
                premises: pattern.premises.clone(),
                binding: pattern.binding.clone(),
                origin: Some(s::ArgumentOrigin {
                    pattern: pattern.name.clone(),
                    application: application.name.clone(),
                }),
            };
            scope.prefix.clear();
            self.register(Entity::Argument(value), &scope);
        }
        Ok(())
    }
}
