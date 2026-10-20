//! Rebuild compact direct-IR patterns using their separately declared scalar contract.
use super::*;

pub(super) fn lower_direct(program: &s::Program) -> Result<s::Program> {
    let mut result = program.clone();
    result
        .arguments
        .retain(|_, argument| argument.origin.is_none());
    result.lowering_diagnostics.clear();
    let mut errors = Vec::new();
    let mut invalid = BTreeSet::new();
    let mut error =
        |code: &str, message: String, owner: &str, expected: Option<&str>, actual: Option<&str>| {
            errors.push(s::Diagnostic {
                code: code.into(),
                message,
                declaration: Some(owner.into()),
                span: program.locations.get(owner).copied(),
                expected: expected.map(str::to_owned),
                actual: actual.map(str::to_owned),
            })
        };
    for pattern in program.patterns.values() {
        let mut bad = program.duplicates.contains(&pattern.name);
        let mut parameters = BTreeMap::new();
        if pattern.evidence.is_empty()
            && pattern.assumptions.is_empty()
            && pattern.premises.is_empty()
        {
            error(
                "empty_pattern",
                "An argument pattern requires evidence, assumptions or premise claims".into(),
                &pattern.name,
                None,
                None,
            );
            bad = true;
        }
        for parameter in &pattern.parameters {
            if parameters
                .insert(parameter.name.clone(), parameter.kind.clone())
                .is_some()
            {
                error(
                    "duplicate_pattern_parameter",
                    format!(
                        "Parameter {} is declared more than once",
                        repr(&parameter.name)
                    ),
                    &pattern.name,
                    None,
                    None,
                );
                bad = true;
            }
            if !matches!(
                parameter.kind.as_str(),
                "claim" | "reasoning" | "evidence" | "assumption"
            ) {
                error(
                    "invalid_pattern_parameter_kind",
                    format!(
                        "Parameter {} requires claim, reasoning, evidence or assumption",
                        repr(&parameter.name)
                    ),
                    &pattern.name,
                    Some("claim | reasoning | evidence | assumption"),
                    Some(&parameter.kind),
                );
                bad = true;
            }
        }
        let mut refs = vec![
            (&pattern.conclusion, "claim", "conclusion"),
            (&pattern.reasoning, "reasoning", "reasoning"),
        ];
        refs.extend(pattern.evidence.iter().map(|r| (r, "evidence", "evidence")));
        refs.extend(
            pattern
                .assumptions
                .iter()
                .map(|r| (r, "assumption", "assumptions")),
        );
        refs.extend(pattern.premises.iter().map(|r| (r, "claim", "premises")));
        if let Some(binding) = &pattern.binding {
            refs.push((binding, "evidence", "binding"));
        }
        for (reference, kind, clause) in refs {
            match parameters.get(reference) {
                None => {
                    error(
                        "unbound_pattern_reference",
                        format!(
                            "{clause} reference {} must name a declared parameter; global capture is forbidden",
                            repr(reference)
                        ),
                        &pattern.name,
                        Some(kind),
                        Some("undeclared parameter"),
                    );
                    bad = true;
                }
                Some(actual) if actual != kind => {
                    error(
                        "pattern_reference_kind",
                        format!(
                            "{clause} parameter {} has kind {}; expected {}",
                            repr(reference),
                            repr(actual),
                            repr(kind)
                        ),
                        &pattern.name,
                        Some(kind),
                        Some(actual),
                    );
                    bad = true;
                }
                _ => {}
            }
        }
        if bad {
            invalid.insert(pattern.name.clone());
        }
    }
    let mut kinds = BTreeMap::new();
    for value in entities(program) {
        kinds.insert(value.name().to_owned(), value.kind().to_owned());
    }
    let mut authored = BTreeSet::new();
    for value in entities(program) {
        if !matches!(value, Entity::Application(_) | Entity::Argument(_)) {
            authored.insert(value.name().to_owned());
        }
    }
    authored.extend(result.arguments.keys().cloned());
    let old_count = program.arguments.len() - result.arguments.len();
    if program.applications.len() > program.limits.applications {
        if let Some((name, _)) = program.applications.first() {
            error(
                "resource_limit",
                format!(
                    "At most {} pattern applications are supported",
                    program.limits.applications
                ),
                name,
                None,
                None,
            );
        }
        result.lowering_diagnostics = errors;
        result.declaration_count = program.declaration_count.saturating_sub(old_count);
        return Ok(result);
    }
    let mut references = 0;
    for application in program.applications.values() {
        let Some(pattern) = program.patterns.get(&application.pattern) else {
            error(
                "unknown_pattern",
                format!("Unknown pattern {}", repr(&application.pattern)),
                &application.name,
                Some("pattern"),
                Some(
                    kinds
                        .get(&application.pattern)
                        .map_or("unknown reference", String::as_str),
                ),
            );
            continue;
        };
        let mut bad = false;
        if authored.contains(&application.name) || program.duplicates.contains(&application.name) {
            error(
                "generated_symbol_collision",
                format!(
                    "Generated argument {} collides with another declaration",
                    repr(&application.name)
                ),
                &application.name,
                None,
                None,
            );
            bad = true;
        }
        if invalid.contains(&pattern.name) {
            error(
                "invalid_pattern",
                format!("Pattern {} has an invalid definition", repr(&pattern.name)),
                &application.name,
                None,
                None,
            );
            continue;
        }
        let parameters: BTreeMap<_, _> = pattern
            .parameters
            .iter()
            .map(|p| (&p.name, &p.kind))
            .collect();
        let mut bindings: BTreeMap<String, String> = BTreeMap::new();
        for binding in &application.arguments {
            let value = match &binding.reference {
                s::BindingValue::Single(v) => v.clone(),
                s::BindingValue::List(_) => String::new(),
            };
            if bindings
                .insert(binding.name.clone(), value.clone())
                .is_some()
            {
                error(
                    "duplicate_pattern_argument",
                    format!(
                        "Argument {} is supplied more than once",
                        repr(&binding.name)
                    ),
                    &application.name,
                    None,
                    None,
                );
                bad = true;
            }
            if let Some(expected) = parameters.get(&binding.name) {
                let actual = kinds
                    .get(&value)
                    .map_or("unknown reference", String::as_str);
                if actual != expected.as_str() {
                    error(
                        "pattern_argument_kind",
                        format!(
                            "Argument {} must reference {expected}; {} is {actual}",
                            repr(&binding.name),
                            repr(&value)
                        ),
                        &application.name,
                        Some(expected),
                        Some(actual),
                    );
                    bad = true;
                }
            } else {
                error(
                    "unknown_pattern_argument",
                    format!(
                        "Pattern {} has no parameter {}",
                        repr(&pattern.name),
                        repr(&binding.name)
                    ),
                    &application.name,
                    None,
                    None,
                );
                bad = true;
            }
        }
        for (missing, expected) in parameters
            .iter()
            .filter(|(name, _)| !bindings.contains_key(name.as_str()))
        {
            error(
                "missing_pattern_argument",
                format!(
                    "Pattern {} requires argument {}",
                    repr(&pattern.name),
                    repr(missing)
                ),
                &application.name,
                Some(expected),
                Some("missing"),
            );
            bad = true;
        }
        if bad {
            continue;
        }
        let count = 2
            + pattern.evidence.len()
            + pattern.assumptions.len()
            + pattern.premises.len()
            + usize::from(pattern.binding.is_some());
        if references + count > program.limits.expanded_references {
            error(
                "resource_limit",
                format!(
                    "Pattern expansion permits at most {} references",
                    program.limits.expanded_references
                ),
                &application.name,
                None,
                None,
            );
            break;
        }
        references += count;
        let refs = |values: &[String]| values.iter().map(|name| bindings[name].clone()).collect();
        result.arguments.insert(
            application.name.clone(),
            s::Argument {
                name: application.name.clone(),
                conclusion: bindings[&pattern.conclusion].clone(),
                reasoning: bindings[&pattern.reasoning].clone(),
                evidence: refs(&pattern.evidence),
                assumptions: refs(&pattern.assumptions),
                premises: refs(&pattern.premises),
                binding: pattern.binding.as_ref().map(|name| bindings[name].clone()),
                origin: Some(s::ArgumentOrigin {
                    pattern: pattern.name.clone(),
                    application: application.name.clone(),
                }),
            },
        );
    }
    let new_count = result
        .arguments
        .values()
        .filter(|argument| argument.origin.is_some())
        .count();
    result.declaration_count = program.declaration_count.saturating_sub(old_count) + new_count;
    result.lowering_diagnostics = errors;
    Ok(result)
}
