//! Static acquisition dependencies, distinct from support and objection relations.
use super::Checker;

impl Checker<'_> {
    pub(super) fn acquisition_bindings(&mut self) {
        let p = self.program;
        for evidence in p.evidence.values() {
            if evidence.tool.is_some() == evidence.derivation.is_some() {
                self.error(
                    "evidence_source",
                    "Evidence requires exactly one tool or derivation",
                    Some(&evidence.name),
                );
            }
            if (!evidence.bindings.is_empty() || evidence.derivation.is_some())
                && p.profile.as_deref() != Some("execution/1")
            {
                self.error(
                    "execution_profile",
                    "Input bindings require profile execution/1",
                    Some(&evidence.name),
                );
            }
            if let Some(expression) = &evidence.derivation {
                for message in
                    crate::expressions::derivation_errors_with_limits(expression, None, &p.limits)
                {
                    self.error("invalid_derivation", message, Some(&evidence.name));
                }
            }
            let mut destinations = Vec::new();
            for binding in &evidence.bindings {
                let target = pointer_tokens(&binding.target);
                let source = pointer_tokens(&binding.source);
                if target
                    .as_ref()
                    .is_some_and(|parts| parts.len() > p.limits.json_depth)
                    || source
                        .as_ref()
                        .is_some_and(|parts| parts.len() > p.limits.json_depth)
                {
                    self.error(
                        "input_binding_depth",
                        "Input binding pointer exceeds the JSON depth budget",
                        Some(&evidence.name),
                    );
                }
                if target.is_none() || source.is_none() {
                    self.error(
                        "input_binding_pointer",
                        "Input binding pointers must use RFC 6901 syntax",
                        Some(&evidence.name),
                    );
                }
                if let Some(target) = target {
                    if destinations.iter().any(|previous: &Vec<String>| {
                        target.starts_with(previous) || previous.starts_with(&target)
                    }) {
                        self.error(
                            "input_binding_conflict",
                            "Input binding destinations must not overlap",
                            Some(&evidence.name),
                        );
                    }
                    destinations.push(target);
                    if evidence.input.pointer(&binding.target).is_none() {
                        self.error(
                            "input_binding_target",
                            format!(
                                "Input binding destination {:?} must exist in the authored input",
                                binding.target
                            ),
                            Some(&evidence.name),
                        );
                    }
                }
                if self.reference(
                    &binding.evidence,
                    p.evidence.contains_key(&binding.evidence),
                    "acquired evidence",
                    &evidence.name,
                ) {
                    self.scope(
                        &p.evidence[&binding.evidence].environment,
                        &evidence.environment,
                        &evidence.name,
                        &binding.evidence,
                    );
                }
            }
        }
        if !self.references {
            return;
        }
        let all = p.evidence.keys().cloned().collect::<Vec<_>>();
        if let Err(message) = crate::acquisition::dependency_order(p, &all) {
            self.error("acquisition_dependency", message, None);
        }
    }
}

/// Parse rather than normalise a pointer: malformed escapes cannot alias keys.
fn pointer_tokens(pointer: &str) -> Option<Vec<String>> {
    if pointer.is_empty() {
        return Some(Vec::new());
    }
    let suffix = pointer.strip_prefix('/')?;
    suffix
        .split('/')
        .map(|segment| {
            let mut output = String::new();
            let mut chars = segment.chars();
            while let Some(character) = chars.next() {
                if character == '~' {
                    output.push(match chars.next()? {
                        '0' => '~',
                        '1' => '/',
                        _ => return None,
                    });
                } else {
                    output.push(character);
                }
            }
            Some(output)
        })
        .collect()
}
